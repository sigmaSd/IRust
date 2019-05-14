use crate::irust::{
    output::{ColoredOutput, Output},
    OUT,
};
use crossterm::Color;

pub fn format_eval_output(output: &str) -> Output {
    let mut eval_output = Output::default();
    if output.contains("irust v0.1.0 (/tmp/irust)") {
        // Consider this an error
        let lines_count = output.lines().count();

        let actual_error = output
            .lines()
            .skip(1)
            .take(lines_count - 8)
            .collect::<Vec<&str>>()
            .join("\n");

        eval_output.append(actual_error.to_output(Color::White));
    } else {
        eval_output.append(OUT.to_output(Color::Red));
        eval_output.append(output.to_output(Color::White));
    }

    eval_output
}

pub fn warn_about_common_mistakes(input: &str) -> Option<Output> {
    let mut output = Output::new("IRust: ".to_string(), Color::DarkCyan);

    // if input = `x = something`
    if input.split('=').count() == 2 && input.split('=').map(str::trim).all(|s| !s.is_empty()) {
        output.append("Are you missing a `;` ?\n".to_output(Color::Cyan));
        return Some(output);
    }

    // if there were no mistakes return None
    None
}
