use crate::irust::options::Options;

use std::{
    env,
    path::{Path, PathBuf},
};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub enum ArgsResult {
    Exit,
    Proceed,
    ProceedWithScriptPath(PathBuf),
}

pub fn handle_args(options: &mut Options) -> ArgsResult {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        return ArgsResult::Proceed;
    }

    match args[0].as_str() {
        "-h" | "--help" => {
            println!(
                "IRust: Cross Platform Rust REPL
        version: {}\n
        config file is in {}\n
        irust {{path_to_rust_file}} will start IRust with the file loaded in the repl
        --help => shows this message
        --reset-config => reset IRust configuration to default",
                VERSION,
                Options::config_path()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| "??".into())
            );
            ArgsResult::Exit
        }

        "-v" | "--version" => {
            println!("{}", VERSION);
            ArgsResult::Exit
        }

        "--reset-config" => {
            options.reset();
            ArgsResult::Proceed
        }
        maybe_path => {
            let path = Path::new(&maybe_path);
            if path.exists() {
                ArgsResult::ProceedWithScriptPath(path.to_path_buf())
            } else {
                eprintln!("Unknown argument: {}", maybe_path);
                ArgsResult::Proceed
            }
        }
    }
}
