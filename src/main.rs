use log::*;
use logpeek::config::{Config, OutputDirName, LoggingMode, DateTimeFormat, ConsoleMode};
use logpeek::init;

//TODO finalize error handling, the logger should never cause a panic under any circumstances
//TODO verify correct behaviour under multithreaded context
//TODO add tests
//TODO add documentation

fn main() {
    let config = Config { 
        out_dir_name: OutputDirName::CustomDir("logs"), 
        logging_mode: LoggingMode::FileAndConsole,
        datetime_format: DateTimeFormat::Custom("[hour]:[minute]:[second]"),
        console_mode: ConsoleMode::StdoutAndStderr,
        ..Default::default() 
    };

    init(config).unwrap();
    timeit(100);
}

fn timeit(num_of_iters: u32) {
    let start_time = std::time::Instant::now();

    for _ in 0..num_of_iters {
        warn!("TESTING!");
    }
    println!("Time elapsed: {:?}", start_time.elapsed());
}
