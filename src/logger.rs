use std::fs::File;
use std::io::Write;
use std::sync::Mutex;
use log::{Log, Metadata, Record};
use crate::{Config, config};
use crate::config::OutputDirName;

pub struct Logger {
    file_handle: Mutex<File>,
    config: Config,
}

impl Logger {
    pub fn new(config: Config) -> Logger {
        //TODO this doesn't work
        if let OutputDirName::CustomDir(dir) = config.out_dir_name {
            std::env::set_current_dir(dir).expect("Failed to set current directory!");
        }

        Logger {
            file_handle: Mutex::new(File::options()
                .append(true)
                .create(true)
                .open(match config.out_file_name { //This needs to be improved for sure
                    config::OutputFileName::AutoGenerate => "logpeek.log",
                    config::OutputFileName::Custom(name) => name,
                })
                .expect("Failed to open file!")),
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
        self.flush();
    }
}