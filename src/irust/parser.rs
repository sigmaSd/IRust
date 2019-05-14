use crate::irust::format::{format_eval_output, warn_about_common_mistakes};
use crate::irust::{output::Output, IRust};
use crate::utils::{remove_main, stdout_and_stderr};
use crossterm::Color;

const SUCESS: &str = "Ok!";

impl IRust {
    pub fn parse(&mut self) -> std::io::Result<Output> {
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

    fn reset(&mut self) -> std::io::Result<Output> {
        self.repl.reset();
        Ok(Output::new(SUCESS.to_string(), Color::Blue)
            .add_new_line()
            .finish()
            .add_new_line()
            .finish())
    }

    fn show(&mut self) -> std::io::Result<Output> {
        let output = Output::new(self.repl.show(), Color::Magenta);
        Ok(output)
    }

    fn add_dep(&mut self) -> std::io::Result<Output> {
        let dep: Vec<String> = self
            .buffer
            .split_whitespace()
            .skip(1)
            .map(ToOwned::to_owned)
            .collect();

        self.wait_add(self.repl.add_dep(&dep)?, "Add")?;
        self.wait_add(self.repl.build()?, "Build")?;

        Ok(Output::new(SUCESS.to_string(), Color::Blue)
            .add_new_line()
            .finish())
    }

    fn load_script(&mut self) -> std::io::Result<Output> {
        let script = self.buffer.split_whitespace().last().unwrap();

        let script_code = std::fs::read(script)?;
        if let Ok(mut s) = String::from_utf8(script_code) {
            remove_main(&mut s);
            self.repl.insert(s);
        }
        Ok(Output::new(SUCESS.to_string(), Color::Blue)
            .add_new_line()
            .finish())
    }

    fn run_cmd(&mut self) -> std::io::Result<Output> {
        // remove ::
        let buffer = &self.buffer[2..];

        let mut cmd = buffer.split_whitespace();

        let output = stdout_and_stderr(
            std::process::Command::new(cmd.next().unwrap_or_default())
                .args(&cmd.collect::<Vec<&str>>())
                .output()?,
        );

        Ok(Output::new(output, Color::Magenta))
    }

    fn parse_second_order(&mut self) -> std::io::Result<Output> {
        if self.buffer.ends_with(';') {
            self.repl.insert(self.buffer.clone());

            Ok(Output::default())
        } else {
            let mut output = Output::default();

            if let Some(warning) = warn_about_common_mistakes(&self.buffer) {
                output.append(warning);
                output.add_new_line();

                let eval_output = self.repl.eval(self.buffer.clone())?;
                if !eval_output.is_empty() {
                    output.append(format_eval_output(&eval_output));
                }
            } else {
                let eval_output = format_eval_output(&self.repl.eval(self.buffer.clone())?);
                output.append(eval_output);
                output.add_new_line();
            }

            Ok(output)
        }
    }
}
