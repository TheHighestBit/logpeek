use std::fs::File;
use std::io::Write;
use std::sync::Mutex;

use log::{Log, Metadata, Record, SetLoggerError};

use crate::config::Config;

pub mod config;

pub struct Logger {
    file_handle: Mutex<File>,
    config: Config,
}

impl Logger {
    pub fn new(config: Config) -> Logger {
        Logger {
            file_handle: Mutex::new(File::options()
                .append(true)
                .create(true)
                .open(config.out_file)
                .expect("Failed to open file!")),
            config
        }
    }

    pub fn write(&self, message: &str) {
        let mut file_mutex = self.file_handle.lock().unwrap();

        file_mutex.write_all(message.as_bytes()).expect("Failed to write to file!");
        file_mutex.flush().expect("Failed to write to disk!");
    }
}

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let message = format!("{} - {}\n", record.level(), record.args());

            self.write(&message);
        }
    }

    fn flush(&self) {
        // No need to flush stdout
    }
}

pub fn init(config: Config) -> Result<(), SetLoggerError> {
    log::set_max_level(config.log_level);

    log::set_boxed_logger(Box::new(Logger::new(config)))?;
    Ok(())
}