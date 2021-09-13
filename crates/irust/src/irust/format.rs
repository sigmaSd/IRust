use crossterm::style::Color;

use printer::printer::{PrintQueue, PrinterItem};

pub fn format_err<'a>(output: &'a str, show_warnings: bool) -> PrintQueue {
    const BEFORE_2021_END_TAG: &str = ": aborting due to ";
    // Relies on --color=always
    const ERROR_TAG: &str = "\u{1b}[0m\u{1b}[1m\u{1b}[38;5;9merror";
    const WARNING_TAG: &str = "\u{1b}[0m\u{1b}[1m\u{1b}[33mwarning";

    if output.lines().count() <= 8 {
        return PrinterItem::String(output.into(), Color::Red).into();
    }

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

    let output: Box<dyn Iterator<Item = &str>> = if output.contains(BEFORE_2021_END_TAG) {
        Box::new(handle_error(output))
    } else {
        Box::new(handle_error_2021(output))
    };
    PrinterItem::String(go_to_end(output), Color::Red).into()
}

pub fn format_eval_output(
    status: std::process::ExitStatus,
    output: String,
    prompt: String,
    show_warnings: bool,
) -> Option<PrintQueue> {
    if !status.success() {
        return Some(format_err(&output, show_warnings));
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
        Some(format_err(&output, show_warnings))
    } else {
        None
    }
}
