use std::fmt::Display;
use std::fs;
use std::fs::File;
use std::io::{stderr, stdout, Write};
use std::path::PathBuf;
use std::sync::Mutex;

use colored::Colorize;
use log::{Log, Metadata, Record};
use time::{format_description, OffsetDateTime};
use time::format_description::well_known::{Iso8601, Rfc2822, Rfc3339};

use crate::{Config, config};
use crate::config::OutputDirName;

pub struct Logger {
    output_lock: Mutex<Option<File>>,
    config: Config,
}

impl Logger {
    pub fn new(config: Config) -> Logger {
        let file_handle = match config.logging_mode {
            config::LoggingMode::File | config::LoggingMode::FileAndConsole => {
                let log_path = Logger::get_log_pathbuf(&config);

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

                Some(file)
            },
            _ => None,
        };

        Logger {
            output_lock: Mutex::new(file_handle),
            config
        }
    }

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
                    Logger::log_critical(format!("Failed to write to stdout {:?}", err));
                }),
                config::ConsoleMode::Stderr => write!(stderr(), "{}", colored_message).unwrap_or_else(|_| ()),
                config::ConsoleMode::Mixed => {
                    match log_level {
                        log::Level::Error => write!(stdout(), "{}", colored_message).unwrap_or_else(|err| {
                            Logger::log_critical(format!("Failed to write to stdout {:?}", err));
                        }),
                        _ => write!(stderr(), "{}", colored_message).unwrap_or_else(|_| ()),
                    }
                }
            }
        }

        if let Some(file) = self.output_lock.lock().unwrap_or_else(|err| {
            Logger::log_critical(format!("A thread panicked while holding the log file lock, using into_inner {:?}", err));
            err.into_inner()
        }).as_mut() {
            file.write_all(message.as_bytes()).unwrap_or_else(|err| {
                Logger::log_critical(format!("Failed to write to log file {:?}", err));
            });
        }
    }

    pub fn get_current_time(&self) -> Result<String, ()> {
        let dt: OffsetDateTime = match self.config.timezone {
            config::TimeZone::Local => OffsetDateTime::now_local().map_err(|_| {
                Logger::log_critical("Failed to get local time");
            })?,
            config::TimeZone::UTC => OffsetDateTime::now_utc(),
        };

        let format_result = match self.config.datetime_format {
            config::DateTimeFormat::ISO8601 => dt.format(&Iso8601::DEFAULT),
            config::DateTimeFormat::RFC3339 => dt.format(&Rfc3339),
            config::DateTimeFormat::RFC2822 => dt.format(&Rfc2822),
            config::DateTimeFormat::Custom(format_str) => dt.format(&format_description::parse_borrowed::<1>(format_str).map_err(|_| {
                Logger::log_critical("Invalid custom time format description");
            })?),
        };

        format_result.map_err(|_| {
            Logger::log_critical("Failed to format date");
        })
    }

    fn get_log_pathbuf(config: &Config) -> PathBuf {
        let mut out_path = match &config.out_dir_name {
            OutputDirName::CurrentDir => PathBuf::from("."),
            OutputDirName::CustomDir(custom_dir) => PathBuf::from(custom_dir),
        };

        out_path.push(match &config.out_file_name {
            config::OutputFileName::AutoGenerate => Logger::generate_log_name().unwrap_or_else(|_| String::from("default.log")),
            config::OutputFileName::Custom(name) => name.to_string(),
        });

        out_path
    }

    fn generate_log_name() -> Result<String, ()> {
        let format = format_description::parse(
            "[year]_[month]_[day]_[hour]_[minute]_[second].log",
        ).map_err(|err| {
            Logger::log_critical(format!("This shouldn't happen! Failed to parse date format: {:?}", err));
        })?;

        OffsetDateTime::now_utc().format(&format).map_err(|err| {
            Logger::log_critical(format!("This shouldn't happen! Failed to format date: {:?}", err));
        })
    }

    fn log_critical<T>(message: T)
    where T: Display {
        let message = format!("{} {} - {}\n", "CRITICAL", "logpeek", message).bright_red().bold();

        let _ = write!(stderr(), "{}", message);
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
        match self.config.console_mode {
            config::ConsoleMode::Stdout => stdout().flush().ok(),
            config::ConsoleMode::Stderr => stderr().flush().ok(),
            config::ConsoleMode::Mixed => {
                stdout().flush().ok();
                stderr().flush().ok()
            }
        };
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

    struct FileCleaner {
        file_name: &'static str,
    }

    impl Drop for FileCleaner {
        fn drop(&mut self) {
            fs::remove_file(self.file_name).unwrap();
        }
    }

    fn setup(config: Config) -> Logger {
        Logger::new(config)
    }

    #[test]
    fn test_all_levels() {
        let log_file_name = "test_all_levels.log";
        let _file_cleaner = FileCleaner { file_name: log_file_name };

        let logger = setup(Config {
            min_log_level: LevelFilter::Trace,
            out_file_name: OutputFileName::Custom(log_file_name),
            logging_mode: LoggingMode::File,
            ..Default::default()
        });

        logger.write("TRACE test\n", &Level::Trace);
        logger.write("DEBUG test\n", &Level::Debug);
        logger.write("INFO test\n", &Level::Info);
        logger.write("WARN test\n", &Level::Warn);
        logger.write("ERROR test\n", &Level::Error);

        let file_handle = File::open(log_file_name).unwrap();
        let reader = std::io::BufReader::new(file_handle);

        let lines= reader.lines()
            .map(|line| line.unwrap())
            .collect::<Vec<_>>();

        assert!(lines.get(0).unwrap().contains("TRACE"));
        assert!(lines.get(1).unwrap().contains("DEBUG"));
        assert!(lines.get(2).unwrap().contains("INFO"));
        assert!(lines.get(3).unwrap().contains("WARN"));
        assert!(lines.get(4).unwrap().contains("ERROR"));
    }

    #[test]
    fn test_all_macros() {
        let log_file_name = "test_all_macros.log";
        let _file_cleaner = FileCleaner { file_name: log_file_name };

        init(Config {
            min_log_level: LevelFilter::Trace,
            out_file_name: OutputFileName::Custom(log_file_name),
            logging_mode: LoggingMode::File,
            ..Default::default()
        }).unwrap();

        trace!("trace test");
        debug!("debug test");
        info!("info test");
        warn!("warn test");
        error!("error test");

        let file_handle = File::open(log_file_name).unwrap();
        let reader = std::io::BufReader::new(file_handle);

        let lines= reader.lines()
            .map(|line| line.unwrap())
            .collect::<Vec<_>>();

        assert!(lines.get(0).unwrap().contains("TRACE"));
        assert!(lines.get(1).unwrap().contains("DEBUG"));
        assert!(lines.get(2).unwrap().contains("INFO"));
        assert!(lines.get(3).unwrap().contains("WARN"));
        assert!(lines.get(4).unwrap().contains("ERROR"));
    }

    #[test]
    fn test_logging_multithreaded() {
        let log_file_name = "test_multithreaded.log";
        let _file_cleaner = FileCleaner { file_name: log_file_name };

        let logger = Arc::new(setup(Config {
            min_log_level: LevelFilter::Trace,
            out_file_name: OutputFileName::Custom(log_file_name),
            logging_mode: LoggingMode::File,
            ..Default::default()
        }));
        let mut threads = vec![];

        for i in 0..5 {
            let logger = Arc::clone(&logger);
            threads.push(std::thread::spawn(move || {
                for _ in 0..100 {
                    let message = format!("TESTING from thread {}!\n", i);
                    logger.write(&message, &Level::Error)
                }
            }));
        }

        for thread in threads {
            thread.join().unwrap();
        }

        let file_handle = File::open(log_file_name).unwrap();
        let reader = std::io::BufReader::new(file_handle);

        let lines= reader.lines()
            .map(|line| line.unwrap())
            .collect::<Vec<_>>();

        assert_eq!(lines.len(), 500);
    }

    #[test]
    fn test_panic_logging() {
        let log_file_name = "test_panic.log";
        let _file_cleaner = FileCleaner { file_name: log_file_name };

        let logger = Arc::new(setup(Config {
            min_log_level: LevelFilter::Trace,
            out_file_name: OutputFileName::Custom(log_file_name),
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