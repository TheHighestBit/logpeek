use log::warn;
use logpeek::config::Config;
use logpeek::init;

fn main() {
    let config = Config{ ..Default::default() };
    init(config).unwrap();

    warn!("TESTING!");
}
