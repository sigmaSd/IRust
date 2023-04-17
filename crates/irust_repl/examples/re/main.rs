use irust_repl::{EvalConfig, Repl, DEFAULT_EVALUATOR};

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
    let mut repl = Repl::default();
    if let Some(deps) = deps {
        let deps: Vec<String> = split_args(deps.to_string());
        repl.add_dep(&deps).unwrap().wait().unwrap();
    }
    let result = repl
        .eval_with_configuration(EvalConfig {
            input: code,
            interactive_function: None,
            color: true,
            evaluator: &*DEFAULT_EVALUATOR,
            compile_mode: irust_repl::CompileMode::Debug,
        })
        .unwrap();
    println!("{}", result.output);
}

fn split_args(s: String) -> Vec<String> {
    let mut args = vec![];
    let mut tmp = String::new();
    let mut quote = false;

    for c in s.chars() {
        match c {
            ' ' => {
                if !quote && !tmp.is_empty() {
                    args.push(tmp.drain(..).collect());
                } else {
                    tmp.push(' ');
                }
            }
            '"' => {
                quote = !quote;
            }
            _ => tmp.push(c),
        }
    }
    if !tmp.is_empty() {
        args.push(tmp);
    }
    args
}
