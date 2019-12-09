mod args;
mod irust;
// uncomment next line to enable logging
// mod log;
mod options;
mod utils;

use crate::args::handle_args;
use irust::IRust;

fn main() {
    let options = options::Options::new().unwrap_or_default();
    let _ = handle_args();
    let mut irust = IRust::new(options);
    irust.run().expect("IRust Out");
}
