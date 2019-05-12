use crate::irust::IRust;
use crate::utils::{remove_main, stdout_and_stderr};

const SUCESS: &str = "Ok!";

impl IRust {
    pub fn parse(&mut self) -> std::io::Result<Option<String>> {
        match self.buffer.as_str() {
            ":reset" => self.reset(),
            ":show" => self.show(),
            cmd if cmd.starts_with("::") => self.run_cmd(),
            cmd if cmd.starts_with(":add") => self.add_dep(),
            cmd if cmd.starts_with(":load") => self.load_script(),
            _ => self.parse_second_order(),
        }
    }

    fn reset(&mut self) -> std::io::Result<Option<String>> {
        self.repl.reset();
        Ok(Some(SUCESS.to_string()))
    }

    fn show(&mut self) -> std::io::Result<Option<String>> {
        Ok(Some(self.repl.show()))
    }

    fn add_dep(&mut self) -> std::io::Result<Option<String>> {
        let dep: Vec<String> = self
            .buffer
            .split_whitespace()
            .skip(1)
            .map(ToOwned::to_owned)
            .collect();

        self.wait_add(self.repl.add_dep(&dep)?, "Add")?;
        self.wait_add(self.repl.build()?, "Build")?;

        Ok(Some(SUCESS.to_string()))
    }

    fn load_script(&mut self) -> std::io::Result<Option<String>> {
        let script = self.buffer.split_whitespace().last().unwrap();

        let script_code = std::fs::read(script)?;
        if let Ok(mut s) = String::from_utf8(script_code) {
            remove_main(&mut s);
            self.repl.insert(s);
        }
        Ok(Some(SUCESS.to_string()))
    }

    fn run_cmd(&mut self) -> std::io::Result<Option<String>> {
        // remove ::
        let buffer = &self.buffer[2..];

        let mut cmd = buffer.split_whitespace();

        Ok(Some(stdout_and_stderr(
            std::process::Command::new(cmd.next().unwrap_or_default())
                .args(&cmd.collect::<Vec<&str>>())
                .output()?,
        )))
    }

    fn parse_second_order(&mut self) -> std::io::Result<Option<String>> {
        let output = if self.buffer.ends_with(';') {
            self.repl.insert(self.buffer.clone());
            None
        } else {
            Some(self.repl.eval(self.buffer.clone())?)
        };

        Ok(output)
    }
}
