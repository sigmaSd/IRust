use simplelog::*;
use std::fs::File;

mod args;
mod irust;
mod utils;

use crate::args::handle_args;
use irust::IRust;

fn main() {
    WriteLogger::init(
        LevelFilter::Info,
        Config::default(),
        File::create("irust.log").unwrap(),
    )
    .unwrap();
    let _ = handle_args();

    let mut irust = IRust::new();
    irust.run().expect("IRust Out");
}
