//! `logpeek` is a logger implementation for the `log` crate, which focuses on reliability and simplicity.
//! It is meant to integrate seamlessly with logpeek-server (coming soon).
//!
//! It provides a `Config` struct for configuring the logger.
//! The logger can be initialized with the `init` function.
//!
//! # Examples
//! ```
//! use logpeek;
//! use log::error;
//!
//! // See the documentation for the config module for more options
//! let config = logpeek::config::Config {
//!     out_dir_name: logpeek::config::OutputDirName::CustomDir("logs"),
//!     logging_mode: logpeek::config::LoggingMode::FileAndConsole,
//!     datetime_format: logpeek::config::DateTimeFormat::Custom("[hour]:[minute]:[second]:[subsecond]"),
//!    ..Default::default()
//! };
//!
//! logpeek::init(config).unwrap(); // For the default config use logpeek::init(Default::default()).unwrap();
//!
//! error!("This is a test error!");
//! ```
use log::SetLoggerError;

use config::Config;
use logger::Logger;

pub mod config;
mod logger;

/// Initializes the logger by setting it as the global boxed logger for the `log` crate.
///
/// # Arguments
/// * `config`: A `Config` instance that specifies the settings for the logger. Use `Default::default()` for the default settings.
///
/// # Returns
///
/// * `Ok(())` if the logger was successfully initialized.
/// * `Err(SetLoggerError)` if the logger failed to initialize for any reason.
///
/// # Panics
///
/// This function will panic if it fails to create the log directory or the log file.
/// This can happen if the user does not have the required permissions.
pub fn init(config: Config) -> Result<(), SetLoggerError> {
    log::set_max_level(config.min_log_level);

    log::set_boxed_logger(Box::new(Logger::new(config)))?;
    Ok(())
}