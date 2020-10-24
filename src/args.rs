use crate::irust::{options::Options, IRust};

use std::env;

const VERSION: &str = "0.8.60";

pub fn handle_args(irust: &mut IRust) -> bool {
    let args: Vec<String> = env::args().skip(1).collect();

    if !args.is_empty() {
        match args[0].as_str() {
            "-h" | "--help" => {
                print!(
                    "IRust: Cross Platform Rust REPL
        version: {}\n
        config file is in {}\n
        --help => shows this message
        --reset-config => reset IRust configuration to default",
                    VERSION,
                    Options::config_path()
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_else(|| "??".into())
                );
                return true;
            }

            "-v" | "--version" => {
                print!("{}", VERSION);
                return true;
            }

            "--reset-config" => {
                irust.options.reset();
            }

            maybe_path => {
                let path = std::path::Path::new(maybe_path);
                if path.exists() {
                    if let Err(e) = irust.load_inner(path.to_path_buf()) {
                        eprintln!("Could not read path {}\n\rError: {}", path.display(), e);
                    }
                } else {
                    eprintln!("Uknown argument: {}", maybe_path)
                }
            }
        }
    }

    false
}
