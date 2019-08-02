use crate::irust::{
    printer::{Printer, PrinterItem, PrinterItemType},
    OUT,
};

pub fn format_eval_output(output: &str) -> Printer {
    let mut eval_output = Printer::default();
    if output.contains("irust v0.1.0 (/tmp/irust)") {
        // Consider this an error
        let lines_count = output.lines().count();

        let actual_error: String = if main_panic(&output) {
            // example:
            // thread 'main' panicked at 'attempt to multiply with overflow',
            let mut output: Vec<&str> = output.lines().nth(3).unwrap().split(',').collect();
            output.pop();

            output.join(",")
        } else {
            output
                .lines()
                .skip(1)
                .take(lines_count - 8)
                .collect::<Vec<&str>>()
                .join("\n")
        };
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

fn main_panic(s: &str) -> bool {
    s.contains("thread 'main' panicked")
}
