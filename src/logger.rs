use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;
use log::{Log, Metadata, Record};
use time::{format_description, OffsetDateTime};
use time::format_description::well_known::{Iso8601, Rfc2822, Rfc3339};
use crate::{Config, config};
use crate::config::OutputDirName;

pub struct Logger {
    file_handle: Mutex<File>,
    config: Config,
}

impl Logger {
    pub fn new(config: Config) -> Logger {
        let out_path = Logger::get_log_pathbuf(&config);

        fs::create_dir_all(out_path.parent().unwrap()).expect("Failed to create parent directories");

        println!("Logging to: {}", out_path.display());

        let file_handle = File::options()
            .append(true)
            .create(true)
            .open(&out_path)
            .expect("Failed to open file!");

        Logger {
            file_handle: Mutex::new(file_handle),
            config
        }
    }

    pub fn write(&self, message: &str) {
        let mut file_mutex = self.file_handle.lock().unwrap();

        file_mutex.write_all(message.as_bytes()).expect("Failed to write to file!");
    }

    pub fn flush(&self) {
        let mut file_mutex = self.file_handle.lock().unwrap();

        file_mutex.flush().expect("Failed to write to disk!");
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
        //This functions is called once per execution so this is fine performance wise
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
            let message = format!("{} - {} - {}\n", self.get_current_time(), record.level(), record.args());

            self.write(&message);
        }
    }

    fn flush(&self) {
        self.flush();
    }
}