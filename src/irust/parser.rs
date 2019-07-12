#[cfg(feature = "highlight")]
use super::highlight::highlight;
use crate::irust::format::{format_eval_output, warn_about_common_mistakes};
use crate::irust::printer::{Printer, PrinterItem, PrinterItemType};
use crate::irust::{IRust, IRustError};
use crate::utils::{remove_main, stdout_and_stderr};

const SUCCESS: &str = "Ok!";

impl IRust {
    pub fn parse(&mut self) -> Result<Printer, IRustError> {
        match self.buffer.as_str() {
            ":help" => self.help(),
            ":reset" => self.reset(),
            ":show" => self.show(),
            ":pop" => self.pop(),
            cmd if cmd.starts_with("::") => self.run_cmd(),
            cmd if cmd.starts_with(":add") => self.add_dep(),
            cmd if cmd.starts_with(":load") => self.load_script(),
            cmd if cmd.starts_with(":type") => self.show_type(),
            cmd if cmd.starts_with(":del") => self.del(),
            _ => self.parse_second_order(),
        }
    }

    fn reset(&mut self) -> Result<Printer, IRustError> {
        self.repl.reset();
        let mut outputs = Printer::new(PrinterItem::new(SUCCESS.to_string(), PrinterItemType::Ok));
        outputs.add_new_line(2);

        Ok(outputs)
    }

    fn pop(&mut self) -> Result<Printer, IRustError> {
        self.repl.pop();
        let mut outputs = Printer::new(PrinterItem::new(SUCCESS.to_string(), PrinterItemType::Ok));
        outputs.add_new_line(2);

        Ok(outputs)
    }

    fn del(&mut self) -> Result<Printer, IRustError> {
        if let Some(line_num) = self.buffer.split_whitespace().last() {
            self.repl.del(line_num)?;
        }

        let mut outputs = Printer::new(PrinterItem::new(SUCCESS.to_string(), PrinterItemType::Ok));
        outputs.add_new_line(2);

        Ok(outputs)
    }

    fn show(&mut self) -> Result<Printer, IRustError> {
        #[cfg(feature = "highlight")]
        let code = highlight(&self.repl.show());

        // a default show method for dev builds (less compile time)
        // via cargo b --no-default-features
        #[cfg(not(feature = "highlight"))]
        let code = Printer::new(PrinterItem::new(
            self.repl.show(),
            PrinterItemType::Custom(crossterm::Color::DarkCyan),
        ));

        Ok(code)
    }

    fn add_dep(&mut self) -> Result<Printer, IRustError> {
        let dep: Vec<String> = self
            .buffer
            .split_whitespace()
            .skip(1)
            .map(ToOwned::to_owned)
            .collect();

        self.save_cursor_position()?;
        self.wait_add(self.repl.add_dep(&dep)?, "Add")?;
        self.wait_add(self.repl.build()?, "Build")?;
        self.write_newline()?;

        let mut outputs = Printer::new(PrinterItem::new(SUCCESS.to_string(), PrinterItemType::Ok));
        outputs.add_new_line(1);

        Ok(outputs)
    }

    fn load_script(&mut self) -> Result<Printer, IRustError> {
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

    fn show_type(&mut self) -> Result<Printer, IRustError> {
        const TYPE_FOUND_MSG: &str = "found type `";
        const EMPTY_TYPE_MSG: &str = "dev [unoptimized + debuginfo]";

        let mut tmp_repl = self.repl.clone();
        tmp_repl.insert(self.buffer.trim_start_matches(":type").to_string());
        tmp_repl.write()?;
        let raw_out = tmp_repl.cargo_cmds.cargo_run(false).unwrap();

        let var_type = if raw_out.find(TYPE_FOUND_MSG).is_some() {
            raw_out
                .lines()
                .find(|l| l.contains(TYPE_FOUND_MSG))
                .unwrap()
                .split('`')
                .nth(1)
                .unwrap()
                .to_string()
        } else if raw_out.find(EMPTY_TYPE_MSG).is_some() {
            "()".into()
        } else {
            "Uknown".into()
        };

        Ok(Printer::new(PrinterItem::new(
            var_type,
            PrinterItemType::Ok,
        )))
    }

    fn run_cmd(&mut self) -> Result<Printer, IRustError> {
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

    fn parse_second_order(&mut self) -> Result<Printer, IRustError> {
        if self.buffer.trim_end().ends_with(';') {
            self.repl.insert(self.buffer.clone());

            let printer = Printer::default();

            Ok(printer)
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
            }
            outputs.add_new_line(1);

            Ok(outputs)
        }
    }
}
