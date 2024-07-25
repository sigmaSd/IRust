use irust_repl::{EvalConfig, EvalResult, Repl, DEFAULT_EVALUATOR};
use serde::{Deserialize, Serialize};
use serde_json::Deserializer;
use std::{io, sync::OnceLock};

#[derive(Serialize, Deserialize)]
struct Message {
    code: String,
}

#[derive(Serialize, Deserialize)]
enum Action {
    Eval { value: String, mime_type: MimeType },
    Insert,
    AddDependency,
}
#[derive(Debug, Serialize, Deserialize)]
enum MimeType {
    #[serde(rename = "text/plain")]
    PlainText,
    #[serde(rename = "text/html")]
    Html,
    #[serde(rename = "image/png")]
    Png,
    #[serde(rename = "image/jpeg")]
    Jpeg,
}
impl MimeType {
    fn from_str(mime_type: &str) -> Self {
        match mime_type {
            "text/plain" => Self::PlainText,
            "text/html" => Self::Html,
            "image/png" => Self::Png,
            "image/jpeg" => Self::Jpeg,
            //NOTE: we should warn here
            _ => Self::PlainText,
        }
    }
}

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    let stdin = io::stdin();
    let reader = stdin.lock();
    let deserializer = Deserializer::from_reader(reader).into_iter::<Message>();

    let mut repl = Repl::default();

    // NOTE: errors should not exit this loop
    // In case of an error we log it and continue
    for json in deserializer {
        let result = (|| -> Result<()> {
            let message = json?;
            let mut code = message.code.trim();
            // detect `!irust` special comment
            if code.starts_with("//") && code.contains("!irust") {
                code = code.splitn(2, "!irust").nth(1).expect("checked").trim();
            }
            if code.ends_with(';') || is_a_statement(&code) {
                let EvalResult { output, status } = repl.eval_check(code.to_owned())?;
                if !status.success() {
                    let output = serde_json::to_string(&Action::Eval {
                        // NOTE: make show warnings configurable
                        value: format_err(&output, false, &repl.cargo.name),
                        mime_type: MimeType::PlainText,
                    })?;
                    println!("{output}");
                    return Ok(());
                }
                // No error, insert the code
                repl.insert(&code);
                let output = serde_json::to_string(&Action::Insert)?;
                println!("{output}");
            } else if code.starts_with(":add") {
                let cargo_add_arg = code
                    .strip_prefix(":add")
                    .expect("checked")
                    .split_whitespace()
                    .map(ToOwned::to_owned)
                    .collect::<Vec<_>>();
                repl.cargo.cargo_add_sync(&cargo_add_arg)?;
                let output = serde_json::to_string(&Action::AddDependency)?;
                println!("{output}");
            } else {
                // eval here
                let EvalResult {
                    output: value,
                    status,
                } = repl.eval_with_configuration(EvalConfig {
                    input: code,
                    interactive_function: None,
                    color: true,
                    evaluator: &*DEFAULT_EVALUATOR,
                    compile_mode: irust_repl::CompileMode::Debug,
                })?;

                // It errored, format the error and send it
                if !status.success() {
                    let output = serde_json::to_string(&Action::Eval {
                        // NOTE: make show warnings configurable
                        value: format_err(&value, false, &repl.cargo.name),
                        mime_type: MimeType::PlainText,
                    })?;
                    println!("{output}");
                    return Ok(());
                }

                // EVCXR
                if value.starts_with("EVCXR_BEGIN_CONTENT") {
                    let data = value.strip_prefix("EVCXR_BEGIN_CONTENT").expect("checked");
                    let data =
                        &data[..data.find("EVCXR_END_CONTENT").ok_or("malformed content")?];
                    let mut data = data.chars();
                    // mime_type = Regex::new("EVCXR_BEGIN_CONTENT ([^ ]+)")
                    let mime_type = data
                        .by_ref()
                        .skip_while(|c| c.is_whitespace())
                        .take_while(|c| !c.is_whitespace())
                        .collect::<String>();

                    let output = serde_json::to_string(&Action::Eval {
                        value: data.collect(),
                        mime_type: MimeType::from_str(&mime_type),
                    })?;
                    println!("{output}");
                    return Ok(());
                }

                let output = serde_json::to_string(&Action::Eval {
                    value,
                    mime_type: MimeType::PlainText,
                })?;
                println!("{output}");
            }
            Ok(())
        })();
        if result.is_err() {
            eprintln!("An error occurred: {result:?}");
            println!("{{}}"); // We still need to send a response so we send an empty object
        }
    }

    Ok(())
}

// The next functions are extracted from irust
// They should be extracted to a separate common crate

pub fn is_a_statement(buffer_trimmed: &str) -> bool {
    match buffer_trimmed
        .split_whitespace()
        .collect::<Vec<_>>()
        .as_slice()
    {
        // async fn|const fn|unsafe fn
        [_, "fn", ..]
        | ["fn", ..]
        | ["enum", ..]
        | ["struct", ..]
        | ["trait", ..]
        | ["impl", ..]
        | ["pub", ..]
        | ["extern", ..]
        | ["macro", ..] => true,
        ["macro_rules!", ..] => true,
        // attribute exp:
        // #[derive(Debug)]
        // struct B{}
        [tag, ..] if tag.starts_with('#') => true,
        _ => false,
    }
}

static NO_COLOR: OnceLock<bool> = OnceLock::new();
/// Have the top precedence
fn no_color() -> bool {
    *NO_COLOR.get_or_init(|| std::env::var("NO_COLOR").is_ok())
}
pub fn format_err<'a>(original_output: &'a str, show_warnings: bool, repl_name: &str) -> String {
    const BEFORE_2021_END_TAG: &str = ": aborting due to ";
    // Relies on --color=always
    const ERROR_TAG: &str = "\u{1b}[0m\u{1b}[1m\u{1b}[38;5;9merror";
    const WARNING_TAG: &str = "\u{1b}[0m\u{1b}[1m\u{1b}[33mwarning";

    // These are more fragile, should be only used when NO_COLOR is on
    const ERROR_TAG_NO_COLOR: &str = "error[";
    const WARNING_TAG_NO_COLOR: &str = "warning: ";

    let go_to_start = |output: &'a str| -> Vec<&'a str> {
        if show_warnings {
            output
                .lines()
                .skip_while(|line| !line.contains(&format!("{repl_name} v0.1.0")))
                .skip(1)
                .collect()
        } else {
            output
                .lines()
                .skip_while(|line| {
                    if no_color() {
                        !line.starts_with(ERROR_TAG_NO_COLOR)
                    } else {
                        !line.starts_with(ERROR_TAG)
                    }
                })
                .collect()
        }
    };
    let go_to_end = |output: Box<dyn Iterator<Item = &str>>| -> String {
        if show_warnings {
            output
        } else {
            Box::new(output.take_while(|line| {
                if no_color() {
                    !line.starts_with(WARNING_TAG_NO_COLOR)
                } else {
                    !line.starts_with(WARNING_TAG)
                }
            }))
        }
        .collect::<Vec<_>>()
        .join("\n")
    };

    let handle_error = |output: &'a str| {
        go_to_start(output)
            .into_iter()
            .take_while(|line| !line.contains(BEFORE_2021_END_TAG))
    };
    let handle_error_2021 = |output: &'a str| {
        go_to_start(output)
            .into_iter()
            .rev()
            .skip_while(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
    };

    let output: Box<dyn Iterator<Item = &str>> = if original_output.contains(BEFORE_2021_END_TAG) {
        Box::new(handle_error(original_output))
    } else {
        Box::new(handle_error_2021(original_output))
    };

    let formatted_error = go_to_end(output);
    // The formatting logic is ad-hoc, there will always be a chance of failure with a rust update
    //
    // So we do a sanity check here, if the formatted_error is empty (which means we failed to
    // format the output), ask the user to open a bug report with the original_output
    if !formatted_error.is_empty() {
        formatted_error
    } else {
        format!("IRust: failed to format the error output.\nThis is a bug in IRust.\nFeel free to open a bug-report at https://github.com/sigmaSd/IRust/issues/new with the next text:\n\noriginal_output:\n{original_output}")
    }
}
