use crate::term::Term;

impl Term {
    pub fn parse(&mut self) -> std::io::Result<()> {
        match self.buffer.as_str() {
            "reset" => self.reset(),
            "show" => self.show()?,
            cmd if cmd.starts_with("add") => self.add_dep()?,
            cmd if cmd.starts_with("load") => self.load_script()?,
            _ => self.parse_second_order(),
        }
        Ok(())
    }
    fn reset(&mut self) {
        self.repl.reset();
    }
    fn show(&mut self) -> std::io::Result<()> {
        self.output = self.repl.show();
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

    fn parse_second_order(&mut self) {
        if self.buffer.ends_with(';') {
            self.repl.insert(self.buffer.drain(..).collect());
        } else {
            self.output = self.repl.eval(self.buffer.drain(..).collect())
        }
    }
}
