use log::LevelFilter;

#[derive(PartialEq)]
pub enum LoggingMode {
    File,
    Console,
    FileAndConsole,
}

#[derive(PartialEq)]
pub enum ConsoleMode {
    Stdout,
    Stderr,
    StdoutAndStderr,
}

pub enum OutputFileName {
    AutoGenerate,
    Custom(&'static str),
}

pub enum OutputDirName {
    CurrentDir,
    CustomDir(&'static str),
}

pub enum TimeZone {
    Local,
    UTC,
}

pub enum DateTimeFormat {
    ISO8601,
    RFC3339,
    RFC2822,
    Custom(&'static str),
}

#[derive(PartialEq)]
pub enum UseTermColor {
    True,
    False,
}

pub struct Config {
    pub out_file_name: OutputFileName,
    pub out_dir_name: OutputDirName,
    pub min_log_level: LevelFilter,
    pub timezone: TimeZone,
    pub logging_mode: LoggingMode,
    pub console_mode: ConsoleMode,
    pub datetime_format: DateTimeFormat,
    pub use_term_color: UseTermColor,
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
        }
    }
}