use crate::irust::{
    printer::{Printer, PrinterItem, PrinterItemType},
    OUT,
};

pub fn output_is_err(output: &str) -> bool {
    output.contains("could not compile `irust`.")
}

pub fn format_err(output: &str) -> Printer {
    let mut eval_output = Printer::default();
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
    eval_output.push(PrinterItem::new(actual_error, PrinterItemType::Err));
    eval_output
}

pub fn format_eval_output(output: &str) -> Option<Printer> {
    if output.trim() == "()" {
        return None;
    }

    if output_is_err(&output) {
        Some(format_err(&output))
    } else {
        let mut eval_output = Printer::default();
        eval_output.push(PrinterItem::new(OUT.into(), PrinterItemType::Out));
        eval_output.push(PrinterItem::new(output.into(), PrinterItemType::Eval));
        Some(eval_output)
    }
}
