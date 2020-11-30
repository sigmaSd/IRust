mod args;
mod irust;
// uncomment next line to enable logging
// mod log;
mod dependencies;
mod utils;
use crate::irust::options::Options;
use crate::irust::IRust;
use dependencies::{check_required_deps, warn_about_opt_deps};

use crate::args::handle_args;
use crossterm::style::Colorize;
use std::process::exit;

fn main() {
    let mut options = Options::new().unwrap_or_default();

    handle_args(&mut options);
    if !check_required_deps() {
        exit(1);
    }
    warn_about_opt_deps(&mut options);

    let mut irust = IRust::new(options);
    let err = if let Err(e) = irust.run() {
        Some(e)
    } else {
        None
    };

    // now IRust has been dropped we can safely print to stderr
    if let Some(err) = err {
        eprintln!("{}", format!("\r\nIRust exited with error: {}", err).red());
    }
}
