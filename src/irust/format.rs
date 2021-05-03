use crossterm::style::Color;

use printer::printer::{PrintQueue, PrinterItem};

pub fn format_err(output: &str) -> PrintQueue {
    let mut error = PrintQueue::default();
    let lines_count = output.lines().count();
    let actual_error = if lines_count > 8 {
        output
            .lines()
            .skip(1)
            .take(lines_count - 8)
            .collect::<Vec<&str>>()
            .join("\n")
    } else {
        output.to_string()
    };
    error.push(PrinterItem::String(actual_error, Color::Red));
    error.add_new_line(1);
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
