use crate::irust::{
    printer::{Printer, PrinterItem, PrinterItemType},
    OUT,
};

pub fn format_eval_output(output: &str) -> Printer {
    let mut eval_output = Printer::default();
    if output.contains("irust v0.1.0 (/tmp/irust)") {
        // Consider this an error
        let lines_count = output.lines().count();

        let actual_error = output
            .lines()
            .skip(1)
            .take(lines_count - 8)
            .collect::<Vec<&str>>()
            .join("\n");

        eval_output.push(PrinterItem::new(actual_error, PrinterItemType::Err));
    } else {
        eval_output.push(PrinterItem::new(OUT.into(), PrinterItemType::Out));

        if output.trim() == "()" {
            eval_output.push(PrinterItem::new(
                "IRust: Are you missing a `;` ?".into(),
                PrinterItemType::Warn,
            ));
            eval_output.add_new_line(1);
        }

        eval_output.push(PrinterItem::new(output.into(), PrinterItemType::Eval));
    }

    eval_output
}

pub fn _warn_about_common_mistakes(input: &str) -> Option<Printer> {
    let mut outputs = Printer::new(PrinterItem::new("IRust: ".into(), PrinterItemType::_IRust));

    // if input = `x = something`
    if input.split('=').count() == 2 && input.split('=').map(str::trim).all(|s| !s.is_empty()) {
        outputs.push(PrinterItem::new(
            "Are you missing a `;` ?".into(),
            PrinterItemType::Warn,
        ));
        return Some(outputs);
    }

    // if there were no mistakes return None
    None
}
