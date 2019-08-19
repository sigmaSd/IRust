use crate::irust::options::Options;
use std::env;

const VERSION: &str = "0.7.3";

pub fn handle_args() -> std::io::Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();

    if !args.is_empty() {
        match args[0].as_str() {
            "--reset-config" => {
                if let Some(config_path) = Options::config_path() {
                    Options::reset_config(config_path);
                }
            }

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
            }

            "-v" | "--version" => println!("{}", VERSION),

            _ => (),
        }

        std::process::exit(0)
    }

    Ok(())
}
