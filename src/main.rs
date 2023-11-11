use log::*;
use logpeek::config::{Config, OutputDirName, LoggingMode};
use logpeek::init;

fn main() {
    let config = Config { 
        out_dir_name: OutputDirName::CustomDir("logs"), 
        logging_mode: LoggingMode::Console, 
        ..Default::default() 
    };

    init(config).unwrap();

    error!("TESTING!");
}
