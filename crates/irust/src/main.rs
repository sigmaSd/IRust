mod args;
mod irust;
// uncomment next line to enable logging
// mod log;
mod dependencies;
mod utils;
use crate::irust::IRust;
use crate::{args::ArgsResult, irust::options::Options};
use dependencies::{check_required_deps, warn_about_opt_deps};

use crate::args::handle_args;
use crossterm::style::Stylize;
use std::process::exit;

fn main() {
    let mut options = Options::new().unwrap_or_default();

    // Handle args
    let args_result = handle_args(&mut options);

    // Exit if there is nothing more todo
    if matches!(args_result, ArgsResult::Exit) {
        exit(0)
    }

    // Check required dependencies and exit if they're not present
    if !check_required_deps() {
        exit(1);
    }

    // Check optional dependencies and warn if they're not present
    warn_about_opt_deps(&mut options);

    // Create main IRust interface
    let mut irust = IRust::new(options);

    // If a script path was provided try to load it
    if let ArgsResult::ProceedWithScriptPath(script) = args_result {
        // Ignore if it fails
        let _ = irust.load_inner(script);
    }

    // Start IRust
    let err = if let Err(e) = irust.run() {
        Some(e)
    } else {
        None
    };

    // Now IRust has been dropped we can safely print to stderr
    if let Some(err) = err {
        eprintln!("{}", format!("\r\nIRust exited with error: {}", err).red());
    }
}
