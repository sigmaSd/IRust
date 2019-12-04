use super::cargo_cmds::{cargo_fmt, cargo_fmt_file, cargo_run, MAIN_FILE};
use super::highlight::highlight;
use crate::irust::format::format_eval_output;
use crate::irust::printer::{Printer, PrinterItem, PrinterItemType};
use crate::irust::{IRust, IRustError};
use crate::utils::{remove_main, stdout_and_stderr};

const SUCCESS: &str = "Ok!";

impl IRust {
    pub fn parse(&mut self) -> Result<Printer, IRustError> {
        match self.buffer.to_string().as_str() {
            ":help" => self.help(),
            ":reset" => self.reset(),
            ":show" => self.show(),
            ":pop" => self.pop(),
            ":irust" => self.irust(),
            cmd if cmd.starts_with("::") => self.run_cmd(),
            cmd if cmd.starts_with(":edit") => self.extern_edit(),
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
        outputs.add_new_line(1);

        Ok(outputs)
    }

    fn pop(&mut self) -> Result<Printer, IRustError> {
        self.repl.pop();
        let mut outputs = Printer::new(PrinterItem::new(SUCCESS.to_string(), PrinterItemType::Ok));
        outputs.add_new_line(1);

        Ok(outputs)
    }

    fn del(&mut self) -> Result<Printer, IRustError> {
        if let Some(line_num) = self.buffer.to_string().split_whitespace().last() {
            self.repl.del(line_num)?;
        }

        let mut outputs = Printer::new(PrinterItem::new(SUCCESS.to_string(), PrinterItemType::Ok));
        outputs.add_new_line(1);

        Ok(outputs)
    }

    fn show(&mut self) -> Result<Printer, IRustError> {
        let repl_code = highlight(&self.repl.show());

        Ok(repl_code)
    }

    fn add_dep(&mut self) -> Result<Printer, IRustError> {
        let dep: Vec<String> = self
            .buffer
            .to_string()
            .split_whitespace()
            .skip(1)
            .map(ToOwned::to_owned)
            .collect();

        self.cursor.save_position()?;
        self.wait_add(self.repl.add_dep(&dep)?, "Add")?;
        self.wait_add(self.repl.build()?, "Build")?;
        self.write_newline()?;

        let mut outputs = Printer::new(PrinterItem::new(SUCCESS.to_string(), PrinterItemType::Ok));
        outputs.add_new_line(1);

        Ok(outputs)
    }

    fn load_script(&mut self) -> Result<Printer, IRustError> {
        let buffer = self.buffer.to_string();
        let script = buffer.split_whitespace().last().unwrap();

        let script_code = std::fs::read(script)?;
        if let Ok(s) = String::from_utf8(script_code) {
            // Format script to make `remove_main` function work correctly
            let s = cargo_fmt(&s)?;
            let s = remove_main(&s);

            self.repl.insert(s);
        }

        let mut outputs = Printer::new(PrinterItem::new(SUCCESS.to_string(), PrinterItemType::Ok));
        outputs.add_new_line(1);

        Ok(outputs)
    }

    fn show_type(&mut self) -> Result<Printer, IRustError> {
        const TYPE_FOUND_MSG: &str = "found type `";
        const EMPTY_TYPE_MSG: &str = "dev [unoptimized + debuginfo]";

        let variable = self
            .buffer
            .to_string()
            .trim_start_matches(":type")
            .to_string();
        let mut raw_out = String::new();

        self.repl
            .eval_in_tmp_repl(variable, || -> Result<(), IRustError> {
                raw_out = cargo_run(false).unwrap();
                Ok(())
            })?;

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
        let buffer = &self.buffer.to_string()[2..];

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
        const FUNCTION_DEF: &str = "fn ";
        const ASYNC_FUNCTION_DEF: &str = "async fn ";
        const ENUM_DEF: &str = "enum ";
        const STRUCT_DEF: &str = "struct ";
        const TRAIT_DEF: &str = "trait ";
        const IMPL: &str = "impl ";

        // attribute exp:
        // #[derive(Debug)]
        // struct B{}
        const ATTRIBUTE: &str = "#";

        let buffer = self.buffer.to_string();
        let buffer = buffer.trim();

        if buffer.is_empty() {
            Ok(Printer::default())
        } else if buffer.ends_with(';')
            || buffer.starts_with(FUNCTION_DEF)
            || buffer.starts_with(ASYNC_FUNCTION_DEF)
            || buffer.starts_with(ENUM_DEF)
            || buffer.starts_with(STRUCT_DEF)
            || buffer.starts_with(TRAIT_DEF)
            || buffer.starts_with(IMPL)
            || buffer.starts_with(ATTRIBUTE)
        {
            self.repl.insert(self.buffer.to_string());

            let printer = Printer::default();

            Ok(printer)
        } else {
            let mut outputs = Printer::default();
            let mut eval_output = format_eval_output(&self.repl.eval(self.buffer.to_string())?);

            outputs.append(&mut eval_output);
            outputs.add_new_line(1);

            Ok(outputs)
        }
    }

    fn extern_edit(&mut self) -> Result<Printer, IRustError> {
        // exp: :edit vi
        let editor: String = match self.buffer.to_string().split_whitespace().nth(1) {
            Some(ed) => ed.to_string(),
            None => return Err(IRustError::Custom("No editor specified".to_string())),
        };

        self.raw_terminal.write_with_color(
            format!("waiting for {}...", editor),
            crossterm::style::Color::Magenta,
        )?;
        self.write_newline()?;

        // write current repl (to ensure eval leftover is cleaned)
        self.repl.write()?;
        // beautify code
        if self.repl.body.len() > 2 {
            let _ = cargo_fmt_file(&*MAIN_FILE);
        }

        std::process::Command::new(editor)
            .arg(&*MAIN_FILE)
            .spawn()?
            .wait()?;

        match self.repl.update_from_main_file() {
            Ok(_) => Ok(Printer::new(PrinterItem::new(
                SUCCESS.to_string(),
                PrinterItemType::Ok,
            ))),
            Err(e) => {
                self.repl.reset();
                Err(e)
            }
        }
    }

    fn irust(&mut self) -> Result<Printer, IRustError> {
        let irust = self.ferris();

        Ok(Printer::new(PrinterItem::new(
            irust,
            PrinterItemType::Custom(crossterm::style::Color::Red),
        )))
    }
}
