//! Logger implementation
use std::{fs, io};
use std::fs::File;
use std::io::{BufWriter, stderr, stdout, Write};
use std::path::PathBuf;
use std::sync::Mutex;

use colored::Colorize;
use log::{error, Log, Metadata, Record};
use time::{format_description, OffsetDateTime};
use time::format_description::FormatItem;
use time::format_description::well_known::{Iso8601, Rfc2822, Rfc3339};

use crate::{Config, config};
use crate::config::OutputDirName;

pub struct Logger {
    output_lock: Mutex<Option<Output>>,
    config: Config,
    custom_time_format: Option<Vec<FormatItem<'static>>>,
}

enum OutputContainer {
    File(File),
    Buffered(BufWriter<File>)
}

struct Output {
    container: OutputContainer,
    file_size: u64,
}

impl Write for OutputContainer {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            OutputContainer::File(file) => file.write(buf),
            OutputContainer::Buffered(buffer) => buffer.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            OutputContainer::File(file) => file.flush(),
            OutputContainer::Buffered(buffer) => buffer.flush(),
        }
    }
}

impl Logger {
    /// Creates a new `Logger` object.
    ///
    /// # Arguments
    /// * `config`: A `Config` instance that specifies the settings for the logger.
    ///
    /// # Panics
    /// This function will panic if it fails to create the log directory or the log file.
    /// This can happen if the user does not have the required permissions.
    pub fn new(config: Config) -> Logger {
        let output_handle = match config.logging_mode {
            config::LoggingMode::File | config::LoggingMode::FileAndConsole => {
                Some(Logger::output_file_setup(&config))
            },
            _ => None,
        };
        
        let custom_time_format = match &config.datetime_format {
            config::DateTimeFormat::Custom(format_str) => {
                Some(format_description::parse_borrowed::<1>(format_str).unwrap_or_else(|err| {
                    panic!("Invalid custom time format description: {:?}", err);
                }))
            },
            _ => None,
        };

        Logger {
            output_lock: Mutex::new(output_handle),
            config,
            custom_time_format
        }
    }

    fn output_file_setup(config: &Config) -> Output {
        let log_path = Logger::get_log_pathbuf(config);

        fs::create_dir_all(log_path.parent().unwrap_or_else(|| {
            panic!("Failed to get parent directory for log path: {:?}", log_path)
        })).unwrap_or_else(|err| {
            panic!("Failed to create log directory at {:?}: {}", log_path, err)
        });

        let file = File::options()
            .append(true)
            .create(true)
            .open(&log_path)
            .unwrap_or_else(|err| {
                panic!("Failed to open log file at {:?}: {}", log_path, err)
            });

        let output_container = if config.logging_strategy == config::LoggingStrategy::Asynchronous {
            OutputContainer::Buffered(BufWriter::new(file))
        } else {
            OutputContainer::File(file)
        };

        Output {
            container: output_container,
            file_size: 0,
        }
    }

    /// Writes a message to the log file and/or the console.
    pub fn write(&self, message: &str, log_level: &log::Level) {
        if self.config.logging_mode == config::LoggingMode::FileAndConsole || self.config.logging_mode == config::LoggingMode::Console {
            let colored_message = if self.config.use_term_color == config::UseTermColor::True {
                match log_level {
                    log::Level::Error => message.red(),
                    log::Level::Warn => message.yellow(),
                    log::Level::Info => message.green(),
                    log::Level::Debug => message.blue(),
                    log::Level::Trace => message.magenta(),
                }
            } else {
                message.normal()
            };

            match self.config.console_mode {
                config::ConsoleMode::Stdout => write!(stdout(), "{}", colored_message).unwrap_or_else(|err| {
                    error!("Failed to write to stdout {:?}", err);
                }),
                config::ConsoleMode::Stderr => write!(stderr(), "{}", colored_message).unwrap_or_else(|_| ()),
                config::ConsoleMode::Mixed => {
                    match log_level {
                        log::Level::Error => write!(stderr(), "{}", colored_message).unwrap_or_else(|err| {
                            error!("Failed to write to stdout {:?}", err);
                        }),
                        _ => write!(stdout(), "{}", colored_message).unwrap_or_else(|_| ()),
                    }
                }
            }
        }
        
        let mut is_split = false;
        if let Some(output_handle) = self.output_lock.lock().unwrap_or_else(|err| {
            error!("A thread panicked while holding the log file lock, using into_inner {:?}", err);
            err.into_inner()
        }).as_mut() {
            if let Err(e) = output_handle.container.write(message.as_bytes()) {
                error!("Failed to write to log file {:?}", e);
            }

            if self.config.split_log_files != config::SplitLogFiles::False {
                output_handle.file_size += message.len() as u64;

                if let config::SplitLogFiles::True(max_size) = self.config.split_log_files {
                    if output_handle.file_size >= max_size {
                        is_split = true;
                    }
                }
            }
        }

        // This needs to happen after the lock has been dropped, or we get into a deadlock
        if is_split {
            self.split_output_file();
        }
    }

    pub fn flush(&self) {
        if let Some(output_handle) = self.output_lock.lock().unwrap_or_else(|err| {
            error!("A thread panicked while holding the log file lock, using into_inner {:?}", err);
            err.into_inner()
        }).as_mut() {
            if let Err(e) = output_handle.container.flush() {
                error!("Failed to flush log file {:?}", e);
            }
        };

        match self.config.console_mode {
            config::ConsoleMode::Stdout => stdout().flush().ok(),
            config::ConsoleMode::Stderr => stderr().flush().ok(),
            config::ConsoleMode::Mixed => {
                stdout().flush().ok();
                stderr().flush().ok()
            }
        };
    }

    fn split_output_file(&self) {
        // If two files are created within a second of each other, they will have the same name and logs will be appended to the first.
        if let Some(output_handle) = self.output_lock.lock().unwrap_or_else(|err| {
            error!("A thread panicked while holding the log file lock, using into_inner {:?}", err);
            err.into_inner()
        }).as_mut() {
            *output_handle = Logger::output_file_setup(&self.config);
        }
    }

    /// Returns the current time as a string in the format specified in the config.
    pub fn get_current_time(&self) -> Result<String, ()> {
        let dt: OffsetDateTime = match self.config.timezone {
            config::TimeZone::Local => OffsetDateTime::now_local().map_err(|_| {
                error!("Failed to get local time");
            })?,
            config::TimeZone::UTC => OffsetDateTime::now_utc(),
        };

        match &self.config.datetime_format {
            config::DateTimeFormat::ISO8601 => dt.format(&Iso8601::DEFAULT),
            config::DateTimeFormat::RFC3339 => dt.format(&Rfc3339),
            config::DateTimeFormat::RFC2822 => dt.format(&Rfc2822),
            config::DateTimeFormat::Custom(_) => dt.format(self.custom_time_format.as_ref().expect("This shouldn't happen! Custom time format not parsed!")),
        }.map_err(|_| {
            error!("Failed to format date");
        })
    }

    /// Constructs the path for the log file.
    fn get_log_pathbuf(config: &Config) -> PathBuf {
        let mut out_path = match &config.out_dir_name {
            OutputDirName::CurrentDir => PathBuf::from("."),
            OutputDirName::Custom(custom_dir) => PathBuf::from(custom_dir),
        };

        out_path.push(match &config.out_file_name {
            config::OutputFileName::AutoGenerate => Logger::generate_log_name().unwrap_or_else(|_| String::from("default.log")),
            config::OutputFileName::Custom(name) => name.to_string(),
        });

        out_path
    }

    /// Generates a log file name based on the current date and time (UTC).
    fn generate_log_name() -> Result<String, ()> {
        let format = format_description::parse(
            "[year]_[month]_[day]_[hour]_[minute]_[second].log",
        ).map_err(|err| {
            error!("This shouldn't happen! Failed to parse date format: {:?}", err);
        })?;

        OffsetDateTime::now_utc().format(&format).map_err(|err| {
            error!("This shouldn't happen! Failed to format date: {:?}", err);
        })
    }
}

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.config.min_log_level
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let message = if self.get_current_time().is_ok() {
                format!("{} {} {} - {}\n", self.get_current_time().unwrap(), record.level(), record.target(), record.args())
            } else {
                format!("{} {} - {}\n", record.level(), record.target(), record.args())
            };


            self.write(&message, &record.level());
        }
    }

    fn flush(&self) {
        self.flush();
    }
}

#[cfg(test)]
mod tests {
    use std::io::BufRead;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::{io, panic, thread};
    use log::*;
    use crate::config::{LoggingMode, OutputFileName};
    use crate::init;
    use super::*;

    // Cleans up the log file after the test is done.
    struct FileCleaner {
        file_name: String,
    }

    impl Drop for FileCleaner {
        fn drop(&mut self) {
            fs::remove_file(self.file_name.clone()).unwrap();
        }
    }

    fn setup(config: Config) -> Logger {
        Logger::new(config)
    }

    // Logs all log levels into a file and verifies that they were written correctly.
    #[test]
    fn test_all_levels() {
        let log_file_name = String::from("test_all_levels.log");
        let _file_cleaner = FileCleaner { file_name: log_file_name.clone() };

        let logger = setup(Config {
            min_log_level: LevelFilter::Trace,
            out_file_name: OutputFileName::Custom(log_file_name.clone()),
            out_dir_name: OutputDirName::CurrentDir,
            logging_mode: LoggingMode::File,
            ..Default::default()
        });

        logger.write("TRACE test\n", &Level::Trace);
        logger.write("DEBUG test\n", &Level::Debug);
        logger.write("INFO test\n", &Level::Info);
        logger.write("WARN test\n", &Level::Warn);
        logger.write("ERROR test\n", &Level::Error);

        let file_handle = File::open(log_file_name).unwrap();
        let reader = io::BufReader::new(file_handle);

        let lines= reader.lines()
            .map(|line| line.unwrap())
            .collect::<Vec<_>>();

        assert!(lines.get(0).unwrap().contains("TRACE"));
        assert!(lines.get(1).unwrap().contains("DEBUG"));
        assert!(lines.get(2).unwrap().contains("INFO"));
        assert!(lines.get(3).unwrap().contains("WARN"));
        assert!(lines.get(4).unwrap().contains("ERROR"));
    }

    // Logs all log levels into a file via the macros from the `log` crate and verifies that they were written correctly.
    #[test]
    fn test_all_macros() {
        let log_file_name = String::from("test_all_macros.log");
        let _file_cleaner = FileCleaner { file_name: log_file_name.clone() };

        init(Config {
            min_log_level: LevelFilter::Trace,
            out_file_name: OutputFileName::Custom(log_file_name.clone()),
            out_dir_name: OutputDirName::CurrentDir,
            logging_mode: LoggingMode::File,
            ..Default::default()
        }).unwrap();

        trace!("trace test");
        debug!("debug test");
        info!("info test");
        warn!("warn test");
        error!("error test");

        let file_handle = File::open(log_file_name).unwrap();
        let reader = io::BufReader::new(file_handle);

        let lines= reader.lines()
            .map(|line| line.unwrap())
            .collect::<Vec<_>>();

        assert!(lines.get(0).unwrap().contains("TRACE"));
        assert!(lines.get(1).unwrap().contains("DEBUG"));
        assert!(lines.get(2).unwrap().contains("INFO"));
        assert!(lines.get(3).unwrap().contains("WARN"));
        assert!(lines.get(4).unwrap().contains("ERROR"));
    }

    // Tests logging from multiple threads simultaneously into a single file.
    #[test]
    fn test_logging_multithreaded() {
        let log_file_name = String::from("test_multithreaded.log");
        let _file_cleaner = FileCleaner { file_name: log_file_name.clone() };

        let logger = Arc::new(setup(Config {
            min_log_level: LevelFilter::Trace,
            out_file_name: OutputFileName::Custom(log_file_name.clone()),
            out_dir_name: OutputDirName::CurrentDir,
            logging_mode: LoggingMode::File,
            ..Default::default()
        }));
        let mut threads = vec![];

        for i in 0..5 {
            let logger = Arc::clone(&logger);
            threads.push(thread::spawn(move || {
                for _ in 0..100 {
                    let message = format!("TESTING from thread {}!\n", i);
                    logger.write(&message, &Level::Error)
                }

                logger.flush();
            }));
        }

        for thread in threads {
            thread.join().unwrap();
        }

        let file_handle = File::open(log_file_name).unwrap();
        let reader = io::BufReader::new(file_handle);

        let lines= reader.lines()
            .map(|line| line.unwrap())
            .collect::<Vec<_>>();

        assert_eq!(lines.len(), 500);
    }

    // Verifies that the logging operation is atomic and entries are written even in case of a panic.
    #[test]
    fn test_panic_logging() {
        let log_file_name = String::from("test_panic.log");
        let _file_cleaner = FileCleaner { file_name: log_file_name.clone() };

        let logger = Arc::new(setup(Config {
            min_log_level: LevelFilter::Trace,
            out_file_name: OutputFileName::Custom(log_file_name.clone()),
            out_dir_name: OutputDirName::CurrentDir,
            logging_mode: LoggingMode::File,
            ..Default::default()
        }));

        let panic_handled = Arc::new(AtomicBool::new(false));
        let panic_handled_clone = Arc::clone(&panic_handled);

        let _thread = thread::spawn(move || {
            let result = panic::catch_unwind(|| {
                logger.write("LAST LOG BEFORE PANIC!\n", &Level::Error);
                panic!("Testing panic!");
            });

            panic_handled_clone.store(true, Ordering::SeqCst);

            result
        });

        while !panic_handled.load(Ordering::SeqCst) {
            thread::yield_now();
        }

        let file_handle = File::open(log_file_name).expect("Failed to open log file");
        let reader = io::BufReader::new(file_handle);

        let lines = reader.lines()
            .map(|line| line.expect("Failed to read line"))
            .collect::<Vec<_>>();

        assert!(lines.get(0).unwrap().contains("LAST LOG BEFORE PANIC!"));
    }
}