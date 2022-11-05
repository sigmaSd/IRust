use crossterm::style::Color;
use printer::printer::{PrintQueue, PrinterItem};

pub fn format_err<'a>(original_output: &'a str, show_warnings: bool) -> String {
    const BEFORE_2021_END_TAG: &str = ": aborting due to ";
    // Relies on --color=always
    const ERROR_TAG: &str = "\u{1b}[0m\u{1b}[1m\u{1b}[38;5;9merror";
    const WARNING_TAG: &str = "\u{1b}[0m\u{1b}[1m\u{1b}[33mwarning";

    let go_to_start = |output: &'a str| -> Vec<&'a str> {
        if show_warnings {
            output
                .lines()
                .skip_while(|line| !line.contains("irust_host_repl v0.1.0"))
                .skip(1)
                .collect()
        } else {
            output
                .lines()
                .skip_while(|line| !line.starts_with(ERROR_TAG))
                .collect()
        }
    };
    let go_to_end = |output: Box<dyn Iterator<Item = &str>>| -> String {
        if show_warnings {
            output
        } else {
            Box::new(output.take_while(|line| !line.starts_with(WARNING_TAG)))
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
        format!(
            "IRust: failed to format the error output.\nThis is a bug in IRust.\nFeel free to open a bug-report at https://github.com/sigmaSd/IRust/issues/new with the next text:\n\noriginal_output:\n{original_output}"
        )
    }
}

pub fn format_err_printqueue(output: &str, show_warnings: bool) -> PrintQueue {
    PrinterItem::String(format_err(output, show_warnings), Color::Red).into()
}

pub fn format_eval_output(
    status: std::process::ExitStatus,
    output: String,
    prompt: String,
    show_warnings: bool,
) -> Option<PrintQueue> {
    if !status.success() {
        return Some(format_err_printqueue(&output, show_warnings));
    }
    if output.trim() == "()" {
        return None;
    }

    let mut eval_output = PrintQueue::default();
    eval_output.push(PrinterItem::String(prompt, Color::Red));
    eval_output.push(PrinterItem::String(output, Color::White));
    eval_output.add_new_line(1);
    Some(eval_output)
}

fn check_is_err(s: &str) -> bool {
    !s.contains("dev [unoptimized + debuginfo]")
}

pub fn format_check_output(output: String, show_warnings: bool) -> Option<PrintQueue> {
    if check_is_err(&output) {
        Some(format_err_printqueue(&output, show_warnings))
    } else {
        None
    }
}
