use log::{LevelFilter, Log, Metadata, Record, SetLoggerError};

mod config;

pub struct Logger;

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        println!("This is coming from logpeek! {}", record.args());
    }

    fn flush(&self) {
        // No need to flush stdout
    }
}

pub fn init(level: LevelFilter) -> Result<(), SetLoggerError> {
    log::set_boxed_logger(Box::new(Logger))?;
    log::set_max_level(level);
    Ok(())
}