use log::LevelFilter;

pub enum LoggingMode {
    File,
    Console,
    FileAndConsole,
}

pub enum OutputFileName {
    AutoGenerate,
    Custom(&'static str),
}

pub enum OutputDirName {
    CurrentDir,
    CustomDir(&'static str),
}

//Can only use UTC and Local unless we add another dependency
pub enum TimeZone {
    Local,
    UTC,
}

//TODO Add a custom format options as well
pub enum DateTimeFormat {
    ISO8601,
    RFC3339,
    RFC2822,
}

pub struct Config {
    pub out_file_name: OutputFileName,
    pub out_dir_name: OutputDirName,
    pub min_log_level: LevelFilter,
    pub timezone: TimeZone,
    pub logging_mode: LoggingMode,
    pub datetime_format: DateTimeFormat,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            out_file_name: OutputFileName::AutoGenerate,
            out_dir_name: OutputDirName::CurrentDir,
            min_log_level: LevelFilter::Info,
            timezone: TimeZone::Local,
            logging_mode: LoggingMode::Console,
            datetime_format: DateTimeFormat::ISO8601,
        }
    }
}