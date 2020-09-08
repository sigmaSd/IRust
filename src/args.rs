use crate::irust::IRustError;
use crate::irust::{options::Options, IRust};

use std::env;

const VERSION: &str = "0.8.15";

pub fn handle_args(irust: &mut IRust) -> Result<(), IRustError> {
    let args: Vec<String> = env::args().skip(1).collect();

    if !args.is_empty() {
        match args[0].as_str() {
            "-h" | "--help" => {
                println!(
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
                std::process::exit(0);
            }

            "-v" | "--version" => {
                println!("{}", VERSION);
                std::process::exit(0);
            }

            "--reset-config" => {
                irust.options.reset();
            }

            maybe_path => {
                let path = std::path::Path::new(maybe_path);
                if path.exists() {
                    irust.load_inner(path.to_path_buf())?;
                }
            }
        }
    }

    Ok(())
}
