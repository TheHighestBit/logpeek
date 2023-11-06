use log::{LevelFilter, warn};
use logpeek::config::Config;
use logpeek::init;

fn main() {
    let config = Config::new("test_log.log", LevelFilter::Warn);
    init(config).unwrap();

    warn!("TESTING!");
}
