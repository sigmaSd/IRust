mod args;
mod irust;
// uncomment next line to enable logging
// mod log;
mod utils;

use crate::args::handle_args;
use irust::IRust;

fn main() {
    let _ = handle_args();

    let mut irust = IRust::new();
    irust.run().expect("IRust Out");
}
