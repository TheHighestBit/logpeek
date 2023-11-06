use log::LevelFilter;

pub enum OutputFileName {
    AutoGenerate,
    Custom(&'static str),
}

pub enum OutputDirName {
    CurrentDir,
    CustomDir(&'static str),
}

pub enum TimeZone { //TODO finalize this implementation (probably more timezones)
    Local,
    UTC,
}

pub struct Config {
    pub out_file_name: OutputFileName,
    pub out_dir_name: OutputDirName,
    pub min_log_level: LevelFilter,
    pub timezone: TimeZone,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            out_file_name: OutputFileName::AutoGenerate,
            out_dir_name: OutputDirName::CurrentDir,
            min_log_level: LevelFilter::Info,
            timezone: TimeZone::Local,
        }
    }
}