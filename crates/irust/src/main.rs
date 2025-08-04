mod args;
mod bare_repl;
mod dependencies;
mod irust;
mod utils;
use crate::irust::IRust;
use crate::{args::parse_args, irust::options::Options};
use crossterm::{style::Stylize, tty::IsTty};
use dependencies::{check_required_deps, warn_about_opt_deps};
use irust_repl::CompileMode;
use std::process::exit;

fn main() {
    let mut options = Options::new().unwrap_or_default();

    // Handle args
    let args: Vec<String> = std::env::args().skip(1).collect();
    let parsed = parse_args(&args);

    // Handle help/version and exit early
    if parsed.show_help {
        println!(
            "IRust: Cross Platform Rust REPL
version: {}
config file is in {}

USAGE:
    irust [FLAGS] [path_to_rust_file]

FLAGS:
    -h, --help            Show this help message and exit
    -v, --version         Show IRust version and exit
    --reset-config        Reset IRust configuration to default
    --default-config      Use the default configuration for this run (it will not be saved)
    --bare-repl           Start IRust in bare REPL mode

POSITIONAL ARGUMENTS:
    path_to_rust_file     Start IRust with the file loaded in the REPL
",
            crate::args::VERSION,
            Options::config_path()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| "??".into())
        );
        exit(0);
    }
    if parsed.show_version {
        println!("{}", crate::args::VERSION);
        exit(0);
    }

    // If no argument are provided, check stdin for some oneshot usage
    if args.is_empty() {
        let mut stdin = std::io::stdin();
        if !stdin.is_tty() {
            // Something was piped to stdin
            // The users wants a oneshot evaluation
            use irust_repl::{DEFAULT_EVALUATOR, EvalConfig, EvalResult, Repl};
            use std::io::Read;

            let mut repl = Repl::default();
            #[allow(clippy::blocks_in_conditions)]
            match (|| -> irust::Result<EvalResult> {
                let mut input = String::new();
                stdin.read_to_string(&mut input)?;
                let result = repl.eval_with_configuration(EvalConfig {
                    input,
                    interactive_function: None,
                    color: true,
                    evaluator: &*DEFAULT_EVALUATOR,
                    compile_mode: CompileMode::Debug,
                })?;
                Ok(result)
            })() {
                Ok(result) => {
                    if result.status.success() {
                        println!("{}", result.output);
                    } else {
                        println!(
                            "{}",
                            irust::format_err(&result.output, false, &repl.cargo.name)
                        );
                    }
                    exit(0)
                }
                Err(e) => {
                    eprintln!("failed to evaluate input, error: {e}");
                    exit(1)
                }
            }
        }
    }

    // Check required dependencies and exit if they're not present
    if !check_required_deps() {
        exit(1);
    }

    // Handle config flags
    if parsed.reset_config {
        options.reset();
    }

    // Create main IRust interface
    let mut irust = if parsed.default_config {
        let mut irust = IRust::new(Options::default());
        irust.dont_save_options();
        irust
    } else {
        // Check optional dependencies and warn if they're not present
        if !cfg!(feature = "no-welcome-screen") {
            warn_about_opt_deps(&mut options);
        }
        // main entry point
        IRust::new(options)
    };

    // If a script path was provided try to load it
    if let Some(script) = parsed.script_path.clone() {
        // Ignore if it fails
        let _ = irust.load_inner(script);
    }

    if parsed.bare_repl {
        // I think its better to not save, since this is expected to be used programmatically
        // Also remove the output prompt, its probably less surprising
        irust.dont_save_options();
        irust.options.output_prompt = String::new();

        if let Err(err) = bare_repl::run(irust) {
            eprintln!("{}", format!("\r\nIRust exited with error: {err}").red());
            exit(1);
        }
        exit(0);
    }

    // Warn about unknown arguments
    if !parsed.unknown_args.is_empty() {
        eprintln!("Unknown arguments: {:?}", parsed.unknown_args);
    }

    // Start IRust
    let err = irust.run().err();

    // Now IRust has been dropped we can safely print to stderr
    if let Some(err) = err {
        eprintln!("{}", format!("\r\nIRust exited with error: {err}").red());
        exit(1);
    }
}
