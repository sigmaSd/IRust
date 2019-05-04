use crate::term::{Term, OUT};
use crossterm::Color;

impl Term {
    pub fn parse(&mut self) -> std::io::Result<()> {
        match self.buffer.as_str() {
            "reset" => self.reset(),
            "show" => self.show()?,
            cmd if cmd.starts_with("add") => self.add_dep()?,
            cmd if cmd.starts_with("load") => self.load_script()?,
            _ => self.parse_second_order()?,
        }
        Ok(())
    }
    fn reset(&mut self) {
        self.repl.reset();
    }
    fn show(&self) -> std::io::Result<()> {
        let current_code = self.repl.show();
        self.terminal.write(&current_code)?;
        Ok(())
    }
    fn add_dep(&self) -> std::io::Result<()> {
        let dep: Vec<String> = self
            .buffer
            .split_whitespace()
            .skip(1)
            .map(ToOwned::to_owned)
            .collect();
        if dep.is_empty() {
            return Ok(());
        }
        self.repl.add_dep(&dep)?;

        Ok(())
    }
    fn load_script(&mut self) -> std::io::Result<()> {
        let script = match self.buffer.split_whitespace().last() {
            Some(s) => std::path::Path::new(s),
            None => return Ok(()),
        };

        let script_code = std::fs::read(script)?;
        if let Ok(s) = String::from_utf8(script_code) {
            self.repl.insert(s);
        }
        Ok(())
    }

    fn parse_second_order(&mut self) -> std::io::Result<()> {
        if self.buffer.ends_with(';') {
            self.repl.insert(self.buffer.drain(..).collect());
        } else {
            let output = self.repl.eval(self.buffer.drain(..).collect());
            self.color.set_fg(Color::Red)?;
            self.terminal.write(OUT)?;
            self.color.reset()?;
            self.terminal.write(output)?;
        }
        Ok(())
    }
}
