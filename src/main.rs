use std::thread::sleep;
use std::time::Duration;
use helpers::timeit;
use log::*;
use logpeek::config::{Config, OutputDirName, LoggingMode, ConsoleMode, SplitLogFiles};
use logpeek::init;
use crate::helpers::timeit_multithreaded;

fn main() {
    let config = Config {
        out_dir_name: OutputDirName::CustomDir("logs"),
        console_mode: ConsoleMode::Mixed,
        logging_mode: LoggingMode::File,
        split_log_files: SplitLogFiles::True(5_000_0),
        ..Default::default() 
    };

    init(config).unwrap();
    
    for _ in 0..10000 {
        error!("TESTING!");
    }
}

mod helpers {
    use super::*;
    pub fn timeit(num_of_iters: u32) {
        let start_time = std::time::Instant::now();

        for _ in 0..num_of_iters {
            error!("TESTING!");
        }

        println!("Time elapsed: {:?}", start_time.elapsed());
    }

    pub fn timeit_multithreaded(num_of_threads: u32, num_of_iters: u32) {
        let start_time = std::time::Instant::now();

        let mut threads = vec![];

        for i in 0..num_of_threads {
            threads.push(std::thread::spawn(move || {
                for _ in 0..num_of_iters {
                    error!("TESTING from thread {}!", i);
                }
            }));
        }

        for thread in threads {
            thread.join().unwrap();
        }

        logger().flush();

        println!("Time elapsed: {:?}", start_time.elapsed());
    }
}
