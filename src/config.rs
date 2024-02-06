//! Module containing the `Config` struct and its associated enums.
use log::LevelFilter;


/// Where the logs are written to
/// Defaults to `Console`
#[derive(PartialEq)]
pub enum LoggingMode {
    /// Logs are written to a file
    File,
    /// Logs are written to the console (stdout or stderr)
    Console,
    /// Logs are written to both a file and the console (stdout or stderr)
    FileAndConsole,
}


/// The output stream for the console. Only applies if `LoggingMode` is `Console` or `FileAndConsole`.
/// Defaults to `Stdout`
#[derive(PartialEq)]
pub enum ConsoleMode {
    /// Logs are written to stdout
    Stdout,
    /// Logs are written to stderr
    Stderr,
    /// Only log entries with a level of `Error` or higher are written to stderr. All other log entries are written to stdout.
    Mixed,
}

/// The name of the log file. Only applies if `LoggingMode` is `File` or `FileAndConsole`.
/// Defaults to `AutoGenerate`
pub enum OutputFileName {
    /// The log file name is automatically generated based on the current date and time (UTC).
    AutoGenerate,
    /// The log file name is specified by the user.
    Custom(String),
}

/// The name of the directory where the log file is written to. Only applies if `LoggingMode` is `File` or `FileAndConsole`.
/// Defaults to `CurrentDir`
pub enum OutputDirName {
    /// The log file is written to the current directory.
    CurrentDir,
    /// The log file is written to a directory specified by the user.
    CustomDir(String),
}

/// The time zone used for the log entries.
/// Defaults to `Local`
pub enum TimeZone {
    /// Local system time.
    Local,
    UTC,
}

/// The format of the date and time in the log entries.
/// Defaults to `ISO8601`
pub enum DateTimeFormat {
    ISO8601,
    RFC3339,
    RFC2822,
    /// Refer to `<https://time-rs.github.io/book/api/format-description.html#components>` (ver 1) for a list of valid format components.
    Custom(String),
}

/// Whether to use ANSI escape codes to color the log entries in the terminal.
/// Defaults to `True`
#[derive(PartialEq)]
pub enum UseTermColor {
    True,
    False,
}

/// Whether to write log entries synchronously (io-blocking) or asynchronously.
/// When using asynchronous logging, make sure to call log::logger().flush() before exiting the program.
/// Defaults to `Synchronous`
#[derive(PartialEq)]
pub enum LoggingStrategy {
    Synchronous,
    Asynchronous
}

/// 'Config' struct that contains the configuration options for the logger.
/// Use `Default::default()` for the default settings.
pub struct Config {
    pub out_file_name: OutputFileName,
    pub out_dir_name: OutputDirName,
    pub min_log_level: LevelFilter,
    pub timezone: TimeZone,
    pub logging_mode: LoggingMode,
    pub console_mode: ConsoleMode,
    pub datetime_format: DateTimeFormat,
    pub use_term_color: UseTermColor,
    pub logging_strategy: LoggingStrategy,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            out_file_name: OutputFileName::AutoGenerate,
            out_dir_name: OutputDirName::CurrentDir,
            min_log_level: LevelFilter::Info,
            timezone: TimeZone::Local,
            logging_mode: LoggingMode::Console,
            console_mode: ConsoleMode::Stdout,
            datetime_format: DateTimeFormat::ISO8601,
            use_term_color: UseTermColor::True,
            logging_strategy: LoggingStrategy::Synchronous,
        }
    }
}