use std::fs;
use std::fs::File;
use std::io::Write;
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

                fs::create_dir_all(log_path.parent().unwrap()).expect("Failed to create log directory!");

                let file = File::options()
                .append(true)
                .create(true)
                .open(&log_path)
                .expect("Failed to open file!");

                Some(file)

            },
            _ => None,
        };

        Logger {
            output_lock: Mutex::new(file_handle),
            config
        }
    }

    pub fn write(&self, message: &str) {
        let mut output_mutex = self.output_lock.lock().unwrap();

        if self.config.logging_mode == config::LoggingMode::FileAndConsole || self.config.logging_mode == config::LoggingMode::Console {
            print!("{}", message);
        }

        if let Some(file) = output_mutex.as_mut() {
            file.write_all(message.as_bytes()).expect("Failed to write to file!");
        }
    }

    pub fn flush(&self) {
        if self.config.logging_mode == config::LoggingMode::FileAndConsole || self.config.logging_mode == config::LoggingMode::File {
            let mut output_mutex = self.output_lock.lock().unwrap();

            if let Some(file) = output_mutex.as_mut() {
                file.flush().expect("Failed to write to file!");
            }
        };
    }

    pub fn get_current_time(&self) -> String {
        let dt: OffsetDateTime = match self.config.timezone {
            config::TimeZone::Local => OffsetDateTime::now_local().expect("Failed to get local time"),
            config::TimeZone::UTC => OffsetDateTime::now_utc(),
        };

        match self.config.datetime_format {
            config::DateTimeFormat::ISO8601 => dt.format(&Iso8601::DEFAULT).expect("Failed to format date"),
            config::DateTimeFormat::RFC3339 => dt.format(&Rfc3339).expect("Failed to format date"),
            config::DateTimeFormat::RFC2822 => dt.format(&Rfc2822).expect("Failed to format date"),
            config::DateTimeFormat::Custom(format_str) => {
                match format_description::parse_borrowed::<1>(&format_str) {
                    Ok(format_description) => dt.format(&format_description)
                        .unwrap_or_else(|_| "Failed to format date with custom format".to_string()),
                    Err(_) => "Invalid custom format string".to_string(),
                }
            }
        }
    }

    fn get_log_pathbuf(config: &Config) -> PathBuf {
        let mut out_path = match &config.out_dir_name {
            OutputDirName::CurrentDir => PathBuf::from("."),
            OutputDirName::CustomDir(custom_dir) => PathBuf::from(custom_dir),
        };

        out_path.push(match &config.out_file_name {
            config::OutputFileName::AutoGenerate => Logger::generate_log_name(),
            config::OutputFileName::Custom(name) => name.to_string(),
        });

        out_path
    }

    //Generates the logfile name automatically, always UTC (for now)
    fn generate_log_name() -> String {
        let format = format_description::parse(
            "[year]_[month]_[day]_[hour]_[minute]_[second].log",
        ).expect("Invalid time format description");

        OffsetDateTime::now_utc().format(&format).expect("Failed to format date")
    }
}

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.config.min_log_level
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let message = format!("{} {} {} - {}\n", self.get_current_time(), record.level(), record.target(), record.args());

            if self.config.use_term_color == config::UseTermColor::True {
                let color = match record.level() {
                    log::Level::Error => "red",
                    log::Level::Warn => "yellow",
                    log::Level::Info => "green",
                    log::Level::Debug => "blue",
                    log::Level::Trace => "magenta",
                };

                self.write(&message.color(color).to_string());
            } else {
                self.write(&message);
            };
        }
    }

    fn flush(&self) {
        self.flush();
    }
}