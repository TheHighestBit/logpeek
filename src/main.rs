use log::*;
use logpeek::config::{Config, OutputDirName, LoggingMode, DateTimeFormat, ConsoleMode};
use logpeek::init;

//TODO verify correct behaviour under multithreaded context
//TODO verify correct target names
//TODO add cfg check for ansi support
//TODO add tests
//TODO add documentation

fn main() {
    let config = Config {
        out_dir_name: OutputDirName::CustomDir("logs"),
        console_mode: ConsoleMode::Mixed,
        logging_mode: LoggingMode::FileAndConsole,
        datetime_format: DateTimeFormat::Custom("[hour]:[minute]:[second]:[subsecond]"),
        ..Default::default() 
    };

    init(config).unwrap();
    timeit(5);
}

fn timeit(num_of_iters: u32) {
    let start_time = std::time::Instant::now();

    for _ in 0..num_of_iters {
        error!("TESTING!");
    }
    println!("Time elapsed: {:?}", start_time.elapsed());
}
