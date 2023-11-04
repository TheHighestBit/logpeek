use log::{LevelFilter, warn};
use logpeek::init;

fn main() {
    init(LevelFilter::Warn).unwrap();

    warn!("TESTING!");
}
