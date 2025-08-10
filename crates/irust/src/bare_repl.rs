use crate::irust::IRust;
use std::io::{Read, Write};

pub fn run(mut irust: IRust) -> crate::irust::Result<()> {
    irust.bare_repl = true;
    let mut stdin = std::io::stdin();

    let mut input = String::new();
    let mut input_buf = [0; 1024];

    loop {
        let n = stdin.read(&mut input_buf)?;
        input.push_str(&String::from_utf8_lossy(&input_buf[..n]));

        let input_c = input.clone();
        let Some((before, after)) = input_c.split_once("IRUST_INPUT_END") else {
            continue;
        };
        input = after.to_string();

        let Some((_, to_eval)) = before.rsplit_once("IRUST_INPUT_START") else {
            return Err("Invalid input format: missing start marker".into());
        };
        let output = irust.parse(to_eval.to_string())?;
        print!("IRUST_OUTPUT_START");
        for part in output {
            match part {
                printer::printer::PrinterItem::RcString(s, range, _color) => {
                    print!("{}", &s[range.start..range.end]);
                }
                printer::printer::PrinterItem::Char(c, _color) => {
                    print!("{c}");
                }
                printer::printer::PrinterItem::String(s, _color) => {
                    print!("{s}");
                }
                printer::printer::PrinterItem::Str(s, _color) => {
                    print!("{s}");
                }
                printer::printer::PrinterItem::NewLine => {
                    println!();
                }
            }
        }
        print!("IRUST_OUTPUT_END");
        std::io::stdout().flush()?;
    }
}
