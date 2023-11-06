use log::LevelFilter;

pub struct Config {
    pub out_file: &'static str,
    pub log_level: LevelFilter,
}

impl Config {
    pub fn new(out_file_name: &'static str, log_level: LevelFilter) -> Config {
        Config { out_file: out_file_name, log_level }
    }
}