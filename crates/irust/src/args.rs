use std::path::PathBuf;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone, Default)]
pub struct ParsedArgs {
    pub show_help: bool,
    pub show_version: bool,
    pub reset_config: bool,
    pub default_config: bool,
    pub bare_repl: bool,
    pub script_path: Option<PathBuf>,
    pub unknown_args: Vec<String>,
}

pub fn parse_args(args: &[String]) -> ParsedArgs {
    let mut parsed = ParsedArgs::default();

    for arg in args {
        match arg.as_str() {
            "-h" | "--help" => parsed.show_help = true,
            "-v" | "--version" => parsed.show_version = true,
            "--reset-config" => parsed.reset_config = true,
            "--default-config" => parsed.default_config = true,
            "--bare-repl" => parsed.bare_repl = true,
            _ => {
                // If it's a file path, set script_path
                if arg.ends_with(".rs") && parsed.script_path.is_none() {
                    parsed.script_path = Some(PathBuf::from(arg));
                } else {
                    parsed.unknown_args.push(arg.clone());
                }
            }
        }
    }

    parsed
}
