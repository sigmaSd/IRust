mod args;
mod irust;
// uncomment next line to enable logging
// mod log;
mod dependencies;
mod utils;
use crate::irust::IRust;
use dependencies::{check_required_deps, warn_about_opt_deps};

use crate::args::handle_args;
use crossterm::style::Colorize;
use std::process::exit;

fn main() {
    if !check_required_deps() {
        exit(1);
    }

    let mut irust = IRust::default();

    let exit_flag = handle_args(&mut irust);
    if exit_flag {
        drop(irust);
        exit(0);
    }

    warn_about_opt_deps(&mut irust);

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
