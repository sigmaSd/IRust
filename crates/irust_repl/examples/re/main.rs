use irust_repl::{EvalConfig, Repl, ToolChain};
use once_cell::sync::Lazy;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    match args.len() {
        0 => panic!("No code provided"), // error
        1 => {
            // code
            eval(None, &args[0]);
        }
        2 => {
            // deps + code
            eval(Some(&args[0]), &args[1]);
        }
        _ => panic!("Extra arguments provided"), // extra arguments
    }
}

fn eval(deps: Option<&str>, code: &str) {
    let mut repl = Repl::new(ToolChain::Default).unwrap();
    if let Some(deps) = deps {
        let mut deps: Vec<String> = deps.split(',').map(ToOwned::to_owned).collect();
        deps.push("--offline".to_string());
        repl.add_dep(&deps).unwrap().wait().unwrap();
    }
    let result = repl
        .eval_with_configuration(EvalConfig {
            input: code,
            interactive_function: None,
            color: true,
            evaluator: &*DISPLAY_EVAL,
        })
        .unwrap();
    println!("{}", result.output);
}

static DISPLAY_EVAL: Lazy<[String; 2]> =
    Lazy::new(|| ["println!(\"{}\", {\n".into(), "\n});".into()]);

// Unreleated TODO
// eval should take &self
