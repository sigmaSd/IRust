mod args;
mod irust;
// uncomment next line to enable logging
// mod log;
mod opt_deps;
mod options;
mod utils;

use crate::args::handle_args;
use crate::opt_deps::warn_about_opt_deb;
use irust::IRust;

fn main() {
    let mut options = options::Options::new().unwrap_or_default();
    let _ = handle_args();
    warn_about_opt_deb(&mut options);
    let mut irust = IRust::new(options);
    irust.run().expect("IRust Out");
}
