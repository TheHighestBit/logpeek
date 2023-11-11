use log::*;
use logpeek::config::{Config, OutputDirName, LoggingMode, DateTimeFormat};
use logpeek::init;

fn main() {
    let config = Config { 
        out_dir_name: OutputDirName::CustomDir("logs"), 
        logging_mode: LoggingMode::Console,
        datetime_format: DateTimeFormat::Custom("[hour]:[minute]:[second]"), 
        ..Default::default() 
    };

    init(config).unwrap();

    error!("TESTING!");
}
