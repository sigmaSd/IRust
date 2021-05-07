pub mod cargo_cmds;
use cargo_cmds::*;
mod executor;
pub use executor::Executor;
mod toolchain;
pub use toolchain::ToolChain;
mod utils;

use std::{
    io::{self, Write},
    process::{Child, ExitStatus},
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug)]
pub struct EvalResult {
    pub output: String,
    pub status: ExitStatus,
}

impl From<(ExitStatus, String)> for EvalResult {
    fn from(result: (ExitStatus, String)) -> Self {
        Self {
            output: result.1,
            status: result.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Repl {
    body: Vec<String>,
    cursor: usize,
    toolchain: ToolChain,
    executor: Executor,
}

impl Repl {
    pub fn new(toolchain: ToolChain) -> Result<Self> {
        Self::new_with_executor(toolchain, Executor::Sync)
    }

    pub fn new_with_executor(toolchain: ToolChain, executor: Executor) -> Result<Self> {
        cargo_new()?;
        // check for required dependencies (in case of async)
        if let Some(dependecy) = executor.dependecy() {
            // needs to be sync
            // repl::new(Tokio)
            // repl.eval(5); // cargo-edit may not have written to Cargo.toml yet
            cargo_add_sync(&dependecy)?;
        }
        cargo_build(toolchain)?;

        Ok(Self {
            body: vec![
                executor.main(),
                "} // Do not write past this line (it will corrupt the repl)".to_string(),
            ],
            cursor: 1,
            toolchain,
            executor,
        })
    }

    pub fn set_executor(&mut self, executor: Executor) -> Result<()> {
        // remove old dependecy if it exists
        if let Some(dependecy) = self.executor.dependecy() {
            cargo_rm(&dependecy)?;
        }

        // use the new executor
        self.executor = executor;
        // check for required dependencies (in case of async)
        if let Some(dependecy) = executor.dependecy() {
            cargo_add_sync(&dependecy)?;
        }
        // finally set the correct main function
        self.body[0] = executor.main();
        Ok(())
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
            toolchain: self.toolchain,
            executor: self.executor,
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

    pub fn reset(&mut self) -> Result<()> {
        *self = Self::new_with_executor(self.toolchain, self.executor)?;
        Ok(())
    }

    pub fn show(&self) -> String {
        let mut current_code = self.body.join("\n");
        // If cargo fmt is present format output else ignore
        if let Ok(fmt_code) = cargo_fmt(&current_code) {
            current_code = fmt_code;
        }
        format!("Current Repl Code:\n{}", current_code)
    }

    pub fn eval(&mut self, input: impl ToString) -> Result<EvalResult> {
        self.eval_inner(input, None, false)
    }
    //Note: These inputs should become a Config struct
    pub fn eval_with_configuration(
        &mut self,
        input: impl ToString,
        interactive_function: fn(&mut Child) -> Result<()>,
        color: bool,
    ) -> Result<EvalResult> {
        self.eval_inner(input, Some(interactive_function), color)
    }

    fn eval_inner(
        &mut self,
        input: impl ToString,
        interactive_function: Option<fn(&mut Child) -> Result<()>>,
        color: bool,
    ) -> Result<EvalResult> {
        let input = input.to_string();
        // `\n{}\n` to avoid print appearing in error messages
        let eval_statement = format!("println!(\"{{:?}}\", {{\n{}\n}});", input);
        let toolchain = self.toolchain;

        let (status, mut eval_result) = self.eval_in_tmp_repl(eval_statement, || {
            cargo_run(color, false, toolchain, interactive_function)
        })?;

        // remove trailing new line
        eval_result.pop();
        Ok((status, eval_result).into())
    }

    pub fn eval_build(&mut self, input: impl ToString) -> Result<EvalResult> {
        let input = input.to_string();
        let toolchain = self.toolchain;
        Ok(self
            .eval_in_tmp_repl(input, || -> Result<(ExitStatus, String)> {
                Ok(cargo_build_output(true, false, toolchain)?)
            })?
            .into())
    }

    pub fn eval_check(&mut self, buffer: String) -> Result<EvalResult> {
        let toolchain = self.toolchain;
        Ok(self
            .eval_in_tmp_repl(buffer, || Ok(cargo_check_output(toolchain)?))?
            .into())
    }

    pub fn eval_in_tmp_repl<T>(
        &mut self,
        input: String,
        mut f: impl FnMut() -> Result<T>,
    ) -> Result<T> {
        let orig_body = self.body.clone();
        let orig_cursor = self.cursor;

        self.insert(input);
        self.write()?;
        let result = f();

        self.body = orig_body;
        self.cursor = orig_cursor;

        result
    }

    pub fn toolchain(&self) -> ToolChain {
        self.toolchain
    }

    pub fn set_toolchain(&mut self, toolchain: ToolChain) {
        self.toolchain = toolchain;
    }

    pub fn add_dep(&self, dep: &[String]) -> std::io::Result<std::process::Child> {
        cargo_add(dep)
    }

    pub fn build(&self) -> std::io::Result<std::process::Child> {
        cargo_build(self.toolchain)
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
        let main_idx = body
            .iter()
            .position(|line| line == &self.executor.main())
            .unwrap();
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

    pub fn lines<'a>(&'a self) -> impl Iterator<Item = &String> + 'a {
        self.body.iter()
    }
    pub fn lines_count(&self) -> usize {
        self.body.len()
    }
}
