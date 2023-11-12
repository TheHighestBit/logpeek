use log::*;
use logpeek::config::{Config, OutputDirName, LoggingMode, DateTimeFormat};
use logpeek::init;

//TODO Make sure the .flush() calls in Logger work as expected. Especially in case of a panic.
//TODO finalize error handling, the logger should never cause a panic under any circumstances
//TODO verify correct behaviour under multithreaded context
//TODO compare with BufWriter, maybe reliability for performance is a good tradeoff?
//TODO add tests
//TODO add documentation

fn main() {
    let config = Config { 
        out_dir_name: OutputDirName::CustomDir("logs"), 
        logging_mode: LoggingMode::FileAndConsole,
        datetime_format: DateTimeFormat::Custom("[hour]:[minute]:[second]"), 
        ..Default::default() 
    };

    init(config).unwrap();

    for _ in 0..1000 {
        error!("TESTING!");
    }
}
