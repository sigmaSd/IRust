use crossterm::style::Color;

use printer::printer::{PrintQueue, PrinterItem};

pub fn format_err(output: &str) -> PrintQueue {
    const BEFORE_2021_END_TAG: &str = ": aborting due to ";
    let mut error = PrintQueue::default();
    let lines_count = output.lines().count();
    let actual_error = if lines_count > 8 {
        let handle_error = |output: &str| {
            output
                .lines()
                // skips some messages (maybe in case of panic)
                .skip_while(|line| !line.contains("irust_host_repl v0.1.0"))
                .skip(1)
                .take_while(|line| !line.contains(BEFORE_2021_END_TAG))
                .collect::<Vec<&str>>()
                .join("\n")
        };
        let handle_error_2021 = |output: &str| {
            output
                .lines()
                // skips some messages (maybe in case of panic)
                .skip_while(|line| !line.contains("irust_host_repl v0.1.0"))
                .skip(1)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .skip_while(|line| !line.is_empty())
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect::<Vec<&str>>()
                .join("\n")
        };
        if output.contains(BEFORE_2021_END_TAG) {
            handle_error(output)
        } else {
            handle_error_2021(output)
        }
    } else {
        output.to_string()
    };
    error.push(PrinterItem::String(actual_error, Color::Red));
    error
}

pub fn format_eval_output(
    status: std::process::ExitStatus,
    output: String,
    prompt: String,
) -> Option<PrintQueue> {
    if !status.success() {
        return Some(format_err(&output));
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

pub fn format_check_output(output: String) -> Option<PrintQueue> {
    if check_is_err(&output) {
        Some(format_err(&output))
    } else {
        None
    }
}
