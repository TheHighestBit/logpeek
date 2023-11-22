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
///
/// # Examples
///
/// ```
/// use logpeek::config::{Config, OutputDirName, LoggingMode, DateTimeFormat, ConsoleMode};
/// use logpeek::Logger;
///
/// let config = Config {
///     out_dir_name: OutputDirName::CustomDir("logs"),
///     console_mode: ConsoleMode::Mixed,
///     logging_mode: LoggingMode::FileAndConsole,
///     datetime_format: DateTimeFormat::Custom("[hour]:[minute]:[second]:[subsecond]"),
///     ..Default::default()
/// };
///
/// let logger = Logger::new(config);
/// ```
pub fn init(config: Config) -> Result<(), SetLoggerError> {
    log::set_max_level(config.min_log_level);

    log::set_boxed_logger(Box::new(Logger::new(config)))?;
    Ok(())
}