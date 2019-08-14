use super::cargo_cmds::*;
use super::IRustError;
use std::io::{self, Write};

#[derive(Clone)]
pub struct Repl {
    pub body: Vec<String>,
    cursor: usize,
}

impl Repl {
    pub fn new() -> Self {
        Self {
            body: vec!["fn main() {\n".to_string(), "}".to_string()],
            cursor: 1,
        }
    }

    pub fn insert(&mut self, input: String) {
        for line in input.lines() {
            let mut line = line.to_owned();
            //line.insert(0, '\t');
            line.push('\n');
            self.body.insert(self.cursor, line);
            self.cursor += 1;
        }
    }

    pub fn reset(&mut self) {
        self.prepare_ground().expect("Error while resetting Repl");
        *self = Self::new();
    }

    pub fn show(&self) -> String {
        let mut current_code = self.body.join("");
        // If cargo fmt is present foramt output else ignore
        if let Ok(fmt_code) = cargo_fmt(&current_code) {
            current_code = fmt_code;
        }
        format!("Current Repl Code:\n{}", current_code)
    }

    // prepare ground
    pub fn prepare_ground(&self) -> Result<(), io::Error> {
        cargo_new()?;
        Ok(())
    }

    pub fn eval(&mut self, input: String) -> Result<String, IRustError> {
        let eval_statement = format!("println!(\"{{:?}}\", {{\n{}\n}});", input);
        let mut eval_result = String::new();

        self.exec_in_tmp_repl(eval_statement, || -> Result<(), IRustError> {
            eval_result = cargo_run(true)?;
            Ok(())
        })?;

        Ok(eval_result)
    }

    pub fn add_dep(&self, dep: &[String]) -> std::io::Result<std::process::Child> {
        Ok(cargo_add(dep)?)
    }

    pub fn build(&self) -> std::io::Result<std::process::Child> {
        cargo_build()
    }

    pub fn write(&self) -> io::Result<()> {
        let mut main_file = std::fs::File::create(&*MAIN_FILE)?;
        write!(main_file, "{}", self.body.join(""))?;

        Ok(())
    }

    pub fn exec_in_tmp_repl(
        &mut self,
        input: String,
        mut f: impl FnMut() -> Result<(), IRustError>,
    ) -> Result<(), IRustError> {
        let orig_body = self.body.clone();
        let orig_cursor = self.cursor;

        self.insert(input);
        self.write()?;
        f()?;

        self.body = orig_body;
        self.cursor = orig_cursor;

        Ok(())
    }

    pub fn pop(&mut self) {
        if self.body.len() > 2 {
            self.body.remove(self.cursor - 1);
            self.cursor -= 1;
        }
    }

    pub fn del(&mut self, line_num: &str) -> Result<(), IRustError> {
        if let Ok(line_num) = line_num.parse::<usize>() {
            if line_num != 0 && line_num + 1 < self.body.len() {
                self.body.remove(line_num);
                self.cursor -= 1;
                return Ok(());
            }
        }

        Err(IRustError::Custom("Incorrect line number".into()))
    }
}
