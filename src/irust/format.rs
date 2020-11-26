use crossterm::style::Color;

use crate::irust::{
    printer::{Printer, PrinterItem},
    OUT,
};

pub fn format_err(output: &str) -> Printer {
    let mut error = Printer::default();
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

pub fn format_eval_output(status: std::process::ExitStatus, output: String) -> Option<Printer> {
    if !status.success() {
        return Some(format_err(&output));
    }
    if output.trim() == "()" {
        return None;
    }

    let mut eval_output = Printer::default();
    eval_output.push(PrinterItem::String(OUT.into(), Color::Red));
    eval_output.push(PrinterItem::String(output, Color::White));
    Some(eval_output)
}

fn check_is_err(s: &str) -> bool {
    !s.contains("dev [unoptimized + debuginfo]")
}

pub fn format_check_output(output: String) -> Option<Printer> {
    if check_is_err(&output) {
        Some(format_err(&output))
    } else {
        None
    }
}
