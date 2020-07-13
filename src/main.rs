mod args;
mod irust;
// uncomment next line to enable logging
// mod log;
mod dependencies;
mod utils;
use dependencies::{check_required_deps, warn_about_opt_deps};

use crate::args::handle_args;
use irust::IRust;

fn main() {
    check_required_deps();

    let mut irust = IRust::new();
    let _ = handle_args();
    warn_about_opt_deps(&mut irust);

    irust.run().expect("IRust Out");
}
