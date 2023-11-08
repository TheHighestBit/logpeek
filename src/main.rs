use log::warn;
use logpeek::config::{Config, OutputDirName};
use logpeek::init;

fn main() {
    let config = Config { out_dir_name: OutputDirName::CustomDir("logs") , ..Default::default() };
    init(config).unwrap();

    warn!("TESTING!");
}
