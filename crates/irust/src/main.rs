mod args;
mod dependencies;
mod irust;
mod utils;
use crate::irust::IRust;
use crate::{
    args::{handle_args, ArgsResult},
    irust::options::Options,
};
use crossterm::{style::Stylize, tty::IsTty};
use dependencies::{check_required_deps, warn_about_opt_deps};
use irust_repl::CompileMode;
use std::process::exit;

fn main() {
    let mut options = Options::new().unwrap_or_default();

    // Handle args
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args_result = if args.is_empty() {
        ArgsResult::Proceed
    } else {
        handle_args(&args, &mut options)
    };

    // Exit if there is nothing more todo
    if matches!(args_result, ArgsResult::Exit) {
        exit(0)
    }

    // If no argument are provided, check stdin for some oneshot usage
    if args.is_empty() {
        let mut stdin = std::io::stdin();
        if !stdin.is_tty() {
            // Something was piped to stdin
            // The users wants a oneshot evaluation
            use irust_repl::{EvalConfig, EvalResult, Repl, DEFAULT_EVALUATOR};
            use std::io::Read;

            let mut repl = Repl::default();
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
                            irust::format_err(&result.output, false, &repl.cargo.paths.name)
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
        eprintln!("{}", format!("\r\nIRust exited with error: {err}").red());
    }
}
