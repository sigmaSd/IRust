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
            body: vec!["fn main() {".to_string(), "}".to_string()],
            cursor: 1,
        }
    }

    pub fn update_from_main_file(&mut self) -> Result<(), IRustError> {
        let main_file = std::fs::read_to_string(&*MAIN_FILE_EXTERN)?;
        let lines_num = main_file.lines().count();
        if lines_num < 2 {
            return Err(IRustError::Custom(
                "main.rs file corrupted, resetting irust..".to_string(),
            ));
        }
        let cursor_pos = lines_num - 1;

        *self = Self {
            body: main_file.lines().map(ToOwned::to_owned).collect(),
            cursor: cursor_pos,
        };
        Ok(())
    }

    pub fn insert(&mut self, input: String, outside_main: bool) {
        if outside_main {
            for line in input.lines() {
                self.body.insert(0, line.to_owned());
                self.cursor += 1;
            }
        } else {
            for line in input.lines() {
                self.body.insert(self.cursor, line.to_owned());
                self.cursor += 1;
            }
        }
    }

    pub fn reset(&mut self, toolchain: ToolChain) {
        self.prepare_ground(toolchain)
            .expect("Error while resetting Repl");
        *self = Self::new();
    }

    pub fn show(&self) -> String {
        let mut current_code = self.body.join("\n");
        // If cargo fmt is present foramt output else ignore
        if let Ok(fmt_code) = cargo_fmt(&current_code) {
            current_code = fmt_code;
        }
        format!("Current Repl Code:\n{}", current_code)
    }

    // prepare ground
    pub fn prepare_ground(&self, toolchain: ToolChain) -> Result<(), IRustError> {
        cargo_new(toolchain)?;
        Ok(())
    }

    pub fn eval(&mut self, input: String, toolchain: ToolChain) -> Result<String, IRustError> {
        // `\n{}\n` to avoid print appearing in error messages
        let eval_statement = format!("println!(\"{{:?}}\", {{\n{}\n}});", input);
        let mut eval_result = String::new();

        self.eval_in_tmp_repl(eval_statement, || -> Result<(), IRustError> {
            eval_result = cargo_run(true, toolchain)?;
            Ok(())
        })?;

        Ok(eval_result)
    }

    pub fn eval_build(
        &mut self,
        input: String,
        toolchain: ToolChain,
    ) -> Result<String, IRustError> {
        let orig_body = self.body.clone();
        let orig_cursor = self.cursor;

        self.insert(input, false);
        self.write()?;
        let output = cargo_build_output(true, toolchain)?;

        self.body = orig_body;
        self.cursor = orig_cursor;
        Ok(output)
    }

    pub fn eval_in_tmp_repl(
        &mut self,
        input: String,
        mut f: impl FnMut() -> Result<(), IRustError>,
    ) -> Result<(), IRustError> {
        let orig_body = self.body.clone();
        let orig_cursor = self.cursor;

        self.insert(input, false);
        self.write()?;
        f()?;

        self.body = orig_body;
        self.cursor = orig_cursor;

        Ok(())
    }

    pub fn add_dep(&self, dep: &[String]) -> std::io::Result<std::process::Child> {
        Ok(cargo_add(dep)?)
    }

    pub fn build(&self, toolchain: ToolChain) -> std::io::Result<std::process::Child> {
        cargo_build(toolchain)
    }

    pub fn write(&self) -> io::Result<()> {
        let mut main_file = std::fs::File::create(&*MAIN_FILE)?;
        write!(main_file, "{}", self.body.join("\n"))?;

        Ok(())
    }

    // Used for external editors
    pub fn write_to_extern(&self) -> io::Result<()> {
        let mut main_file = std::fs::File::create(&*MAIN_FILE_EXTERN)?;
        write!(main_file, "{}", self.body.join("\n"))?;

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
