use irust_repl::{EvalConfig, Repl, ToolChain};

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
        let deps: Vec<String> = deps.split(',').map(ToOwned::to_owned).collect();
        repl.add_dep(&deps).unwrap();
    }
    let result = repl
        .eval_with_configuration(EvalConfig {
            input: code,
            interactive_function: None,
            color: true,
            evaluator: display_eval,
        })
        .unwrap();
    println!("{}", result.output);
}

pub fn display_eval(code: String) -> String {
    format!("println!(\"{{}}\", {{\n{}\n}});", code)
}

// Unreleated TODO
// eval should take &self
