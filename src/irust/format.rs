use crate::irust::{
    output::{Output, OutputPrinter, OutputType},
    OUT,
};

pub fn format_eval_output(output: &str) -> OutputPrinter {
    let mut eval_output = OutputPrinter::default();
    if output.contains("irust v0.1.0 (/tmp/irust)") {
        // Consider this an error
        let lines_count = output.lines().count();

        let actual_error = output
            .lines()
            .skip(1)
            .take(lines_count - 8)
            .collect::<Vec<&str>>()
            .join("\n");

        eval_output.push(Output::new(actual_error, OutputType::Err));
    } else {
        eval_output.push(Output::new(OUT.into(), OutputType::Out));
        eval_output.push(Output::new(output.into(), OutputType::Eval));
    }

    eval_output
}

pub fn warn_about_common_mistakes(input: &str) -> Option<OutputPrinter> {
    let mut outputs = OutputPrinter::new(Output::new("IRust: ".into(), OutputType::IRust));

    // if input = `x = something`
    if input.split('=').count() == 2 && input.split('=').map(str::trim).all(|s| !s.is_empty()) {
        outputs.push(Output::new(
            "Are you missing a `;` ?".into(),
            OutputType::Warn,
        ));
        return Some(outputs);
    }

    // if there were no mistakes return None
    None
}
