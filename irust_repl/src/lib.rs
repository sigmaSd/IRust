pub mod cargo_cmds;
use cargo_cmds::*;
mod utils;
use std::process::ExitStatus;
use std::{
    io::{self, Write},
    process::Child,
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const FN_MAIN: &str = "fn main() {";

#[derive(Debug, Clone)]
pub struct Repl {
    body: Vec<String>,
    cursor: usize,
}

impl Repl {
    pub fn new() -> Self {
        Self {
            body: vec![
                FN_MAIN.to_string(),
                "} // Do not write past this line (it will corrupt the repl)".to_string(),
            ],
            cursor: 1,
        }
    }

    pub fn update_from_extern_main_file(&mut self) -> Result<()> {
        let main_file = std::fs::read_to_string(&*MAIN_FILE_EXTERN)?;
        let lines_num = main_file.lines().count();
        if lines_num < 2 {
            return Err("main.rs file corrupted, resetting irust..".into());
        }
        let cursor_pos = lines_num - 1;

        *self = Self {
            body: main_file.lines().map(ToOwned::to_owned).collect(),
            cursor: cursor_pos,
        };
        Ok(())
    }

    // Note: Insert must be followed by write_to_extern if persistance is needed
    // Or else it will be overwritten by the main_extern thread
    // Fix this
    pub fn insert(&mut self, input: impl ToString) {
        let input = input.to_string();
        // CRATE_ATTRIBUTE are special in the sense that they should be inserted outside of the main function
        // #![feature(unboxed_closures)]
        // fn main() {}
        const CRATE_ATTRIBUTE: &str = "#!";

        let outside_main = input.trim_start().starts_with(CRATE_ATTRIBUTE);
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

    pub fn reset(&mut self, toolchain: ToolChain) -> Result<()> {
        self.prepare_ground(toolchain)?;
        *self = Self::new();
        Ok(())
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
    pub fn prepare_ground(&self, toolchain: ToolChain) -> Result<()> {
        cargo_new(toolchain)?;
        Ok(())
    }

    pub fn eval(
        &mut self,
        input: impl ToString,
        toolchain: ToolChain,
    ) -> Result<(ExitStatus, String)> {
        self.eval_inner(input, toolchain, None, false)
    }
    //Note: These inputs should become a Config struct
    pub fn eval_with_configuration(
        &mut self,
        input: impl ToString,
        toolchain: ToolChain,
        interactive_function: fn(&mut Child) -> Result<()>,
        color: bool,
    ) -> Result<(ExitStatus, String)> {
        self.eval_inner(input, toolchain, Some(interactive_function), color)
    }

    fn eval_inner(
        &mut self,
        input: impl ToString,
        toolchain: ToolChain,
        interactive_function: Option<fn(&mut Child) -> Result<()>>,
        color: bool,
    ) -> Result<(ExitStatus, String)> {
        let input = input.to_string();
        // `\n{}\n` to avoid print appearing in error messages
        let eval_statement = format!("println!(\"{{:?}}\", {{\n{}\n}});", input);
        let mut eval_result = String::new();
        let mut status = None;

        self.eval_in_tmp_repl(eval_statement, || -> Result<()> {
            let (s, result) = cargo_run(color, false, toolchain, interactive_function)?;
            eval_result = result;
            status = Some(s);
            Ok(())
        })?;

        // remove trailing new line
        eval_result.pop();
        // status is guarenteed to be some
        Ok((status.unwrap(), eval_result))
    }

    pub fn eval_build(
        &mut self,
        input: String,
        toolchain: ToolChain,
    ) -> Result<(ExitStatus, String)> {
        let orig_body = self.body.clone();
        let orig_cursor = self.cursor;

        self.insert(input);
        self.write()?;
        let (status, output) = cargo_build_output(true, false, toolchain)?;

        self.body = orig_body;
        self.cursor = orig_cursor;
        Ok((status, output))
    }

    pub fn eval_in_tmp_repl(
        &mut self,
        input: String,
        mut f: impl FnMut() -> Result<()>,
    ) -> Result<()> {
        let orig_body = self.body.clone();
        let orig_cursor = self.cursor;

        self.insert(input);
        self.write()?;
        f()?;

        self.body = orig_body;
        self.cursor = orig_cursor;

        Ok(())
    }

    pub fn add_dep(&self, dep: &[String]) -> std::io::Result<std::process::Child> {
        cargo_add(dep)
    }

    pub fn build(&self, toolchain: ToolChain) -> std::io::Result<std::process::Child> {
        cargo_build(toolchain)
    }

    pub fn check(&mut self, buffer: String, toolchain: ToolChain) -> Result<String> {
        let mut result = String::new();
        self.eval_in_tmp_repl(buffer, || {
            result = cargo_check_output(toolchain)?;
            Ok(())
        })?;
        Ok(result)
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

    pub fn write_lib(&self) -> io::Result<()> {
        let mut lib_file = std::fs::File::create(&*LIB_FILE)?;
        let mut body = self.body.clone();

        // safe unwrap
        let main_idx = body.iter().position(|l| l == FN_MAIN).unwrap();
        body.remove(main_idx); // remove fn main
        body.pop(); // remove last }

        write!(lib_file, "{}", body.join("\n"))?;

        Ok(())
    }

    pub fn pop(&mut self) {
        if self.body.len() > 2 {
            self.body.remove(self.cursor - 1);
            self.cursor -= 1;
        }
    }

    pub fn del(&mut self, line_num: &str) -> Result<()> {
        if let Ok(line_num) = line_num.parse::<usize>() {
            if line_num != 0 && line_num + 1 < self.body.len() {
                self.body.remove(line_num);
                self.cursor -= 1;
                return Ok(());
            }
        }

        Err("Incorrect line number".into())
    }

    pub fn lines_count(&self) -> usize {
        self.body.len()
    }
}
