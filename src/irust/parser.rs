use crate::irust::format::{format_eval_output, warn_about_common_mistakes};
use crate::irust::printer::{Printer, PrinterItem, PrinterItemType};
use crate::irust::IRust;
use crate::utils::{remove_main, stdout_and_stderr};

const SUCCESS: &str = "Ok!";

impl IRust {
    pub fn parse(&mut self) -> std::io::Result<Printer> {
        match self.buffer.as_str() {
            ":help" => self.help(),
            ":reset" => self.reset(),
            ":show" => self.show(),
            cmd if cmd.starts_with("::") => self.run_cmd(),
            cmd if cmd.starts_with(":add") => self.add_dep(),
            cmd if cmd.starts_with(":load") => self.load_script(),
            _ => self.parse_second_order(),
        }
    }

    fn reset(&mut self) -> std::io::Result<Printer> {
        self.repl.reset();
        let mut outputs = Printer::new(PrinterItem::new(SUCCESS.to_string(), PrinterItemType::Ok));
        outputs.add_new_line(2);

        Ok(outputs)
    }

    fn show(&mut self) -> std::io::Result<Printer> {
        let outputs = Printer::new(PrinterItem::new(self.repl.show(), PrinterItemType::Show));

        Ok(outputs)
    }

    fn add_dep(&mut self) -> std::io::Result<Printer> {
        let dep: Vec<String> = self
            .buffer
            .split_whitespace()
            .skip(1)
            .map(ToOwned::to_owned)
            .collect();

        self.wait_add(self.repl.add_dep(&dep)?, "Add")?;
        self.wait_add(self.repl.build()?, "Build")?;

        let mut outputs = Printer::new(PrinterItem::new(SUCCESS.to_string(), PrinterItemType::Ok));
        outputs.add_new_line(1);

        Ok(outputs)
    }

    fn load_script(&mut self) -> std::io::Result<Printer> {
        let script = self.buffer.split_whitespace().last().unwrap();

        let script_code = std::fs::read(script)?;
        if let Ok(mut s) = String::from_utf8(script_code) {
            remove_main(&mut s);
            self.repl.insert(s);
        }

        let mut outputs = Printer::new(PrinterItem::new(SUCCESS.to_string(), PrinterItemType::Ok));
        outputs.add_new_line(1);

        Ok(outputs)
    }

    fn run_cmd(&mut self) -> std::io::Result<Printer> {
        // remove ::
        let buffer = &self.buffer[2..];

        let mut cmd = buffer.split_whitespace();

        let output = stdout_and_stderr(
            std::process::Command::new(cmd.next().unwrap_or_default())
                .args(&cmd.collect::<Vec<&str>>())
                .output()?,
        );

        Ok(Printer::new(PrinterItem::new(
            output,
            PrinterItemType::Shell,
        )))
    }

    fn parse_second_order(&mut self) -> std::io::Result<Printer> {
        if self.buffer.trim_end().ends_with(';') {
            self.repl.insert(self.buffer.clone());

            Ok(Printer::default())
        } else {
            let mut outputs = Printer::default();

            if let Some(mut warning) = warn_about_common_mistakes(&self.buffer) {
                outputs.append(&mut warning);
                outputs.add_new_line(1);

                let eval_output = self.repl.eval(self.buffer.clone())?;
                if !eval_output.is_empty() {
                    outputs.append(&mut format_eval_output(&eval_output));
                }
            } else {
                let mut eval_output = format_eval_output(&self.repl.eval(self.buffer.clone())?);
                outputs.append(&mut eval_output);
                outputs.add_new_line(1);
            }

            Ok(outputs)
        }
    }
}
