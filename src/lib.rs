use log::SetLoggerError;

use config::Config;
use logger::Logger;

pub mod config;
mod logger;

pub fn init(config: Config) -> Result<(), SetLoggerError> {
    log::set_max_level(config.min_log_level);

    log::set_boxed_logger(Box::new(Logger::new(config)))?;
    Ok(())
}