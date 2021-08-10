pub mod cargo_cmds;
use cargo_cmds::*;
mod executor;
pub use executor::Executor;
mod toolchain;
pub use toolchain::ToolChain;
mod main_result;
pub use main_result::MainResult;

use once_cell::sync::Lazy;
mod utils;

use std::{
    io::{self, Write},
    process::{Child, ExitStatus},
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub static DEFAULT_EVALUATOR: Lazy<[String; 2]> =
    Lazy::new(|| ["println!(\"{:?}\", {\n".into(), "\n});".into()]);

pub struct EvalConfig<'a, S: ToString> {
    pub input: S,
    pub interactive_function: Option<fn(&mut Child) -> Result<()>>,
    pub color: bool,
    pub evaluator: &'a [String],
}

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
    main_body: Vec<String>,
    lib_body: Vec<String>,
    main_cursor: usize,
    lib_cursor: usize,
    toolchain: ToolChain,
    executor: Executor,
    main_result: MainResult,
    cargo_toml: CargoToml,
    target_file: TargetFile,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetFile {
    Main,
    Lib,
}
impl std::fmt::Display for TargetFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Main => {
                write!(f, "main")
            }
            Self::Lib => {
                write!(f, "lib")
            }
        }
    }
}

impl Default for Repl {
    fn default() -> Self {
        Repl::new(
            ToolChain::default(),
            Executor::default(),
            MainResult::default(),
        )
        .expect("Paniced while trying to create repl")
    }
}

impl Repl {
    pub fn new(toolchain: ToolChain, executor: Executor, main_result: MainResult) -> Result<Self> {
        cargo_new()?;
        // check for required dependencies (in case of async)
        if let Some(dependecy) = executor.dependecy() {
            // needs to be sync
            // repl::new(Tokio)
            // repl.eval(5); // cargo-edit may not have written to Cargo.toml yet
            cargo_add_sync(&dependecy)?;
        }
        cargo_build(toolchain)?;

        let (header, footer) = Self::generate_body_delimiters(executor, main_result);
        Ok(Self {
            main_body: vec![header, footer, "}".to_string()],
            lib_body: vec![],
            main_cursor: 1,
            lib_cursor: 0,
            toolchain,
            executor,
            main_result,
            cargo_toml: CargoToml { proc_macro: false },
            target_file: TargetFile::Main,
        })
    }
    pub fn proc_macro(&self) -> bool {
        self.cargo_toml.proc_macro
    }

    pub fn activate_proc_macro(&mut self) -> Result<()> {
        self.cargo_toml = CargoToml { proc_macro: true };
        write_cargo_toml(self.cargo_toml)?;
        // Reset lib for proc macro
        self.lib_body = vec![];
        self.lib_cursor = 0;
        self.touch_lib()?;
        Ok(())
    }
    pub fn deactivate_proc_macro(&mut self) -> Result<()> {
        self.cargo_toml = CargoToml { proc_macro: false };
        write_cargo_toml(self.cargo_toml).map_err(Into::into)
    }
    pub fn set_target_file(&mut self, target_file: TargetFile) {
        self.target_file = target_file;
    }
    pub fn target_file(&self) -> TargetFile {
        self.target_file
    }

    fn generate_body_delimiters(executor: Executor, main_result: MainResult) -> (String, String) {
        (
            executor.main() + " -> " + main_result.ttype() + "{",
            main_result.instance().to_string()
                + " // Do not write past this line (it will corrupt the repl)",
        )
    }

    pub fn set_executor(&mut self, executor: Executor) -> Result<()> {
        // remove old dependecy if it exists
        if let Some(dependecy) = self.executor.dependecy() {
            // cargo rm needs only the crate name
            cargo_rm_sync(&dependecy[0])?;
        }

        // use the new executor
        self.executor = executor;
        // check for required dependencies (in case of async)
        if let Some(dependecy) = executor.dependecy() {
            cargo_add_sync(&dependecy)?;
        }
        // finally set the correct main function
        let (header, footer) = Self::generate_body_delimiters(self.executor, self.main_result);
        let footer_pos = self.main_body.len() - 2;
        self.main_body[0] = header;
        self.main_body[footer_pos] = footer;
        Ok(())
    }

    pub fn update_from_extern_main_file(&mut self) -> Result<()> {
        let main_file = std::fs::read_to_string(&*MAIN_FILE_EXTERN)?;
        let lines_num = main_file.lines().count();
        if lines_num < 2 {
            return Err("main.rs file corrupted, resetting irust..".into());
        }
        let cursor_pos = lines_num - 2;

        *self = Self {
            main_body: main_file.lines().map(ToOwned::to_owned).collect(),
            main_cursor: cursor_pos,
            lib_cursor: self.lib_cursor,
            toolchain: self.toolchain,
            executor: self.executor,
            main_result: self.main_result,
            cargo_toml: self.cargo_toml,
            target_file: self.target_file,
            lib_body: self.lib_body.clone(),
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

        let (body, cursor) = match self.target_file {
            TargetFile::Main => (&mut self.main_body, &mut self.main_cursor),
            TargetFile::Lib => (&mut self.lib_body, &mut self.lib_cursor),
        };

        if outside_main {
            for line in input.lines() {
                body.insert(0, line.to_owned());
                *cursor += 1;
            }
        } else {
            for line in input.lines() {
                body.insert(*cursor, line.to_owned());
                *cursor += 1;
            }
        }
    }

    pub fn reset(&mut self) -> Result<()> {
        *self = Self::new(self.toolchain, self.executor, self.main_result)?;
        Ok(())
    }

    pub fn show(&self) -> String {
        let mut current_code = self.main_body.join("\n");
        // If cargo fmt is present format output else ignore
        if let Ok(fmt_code) = cargo_fmt(&current_code) {
            current_code = fmt_code;
        }
        format!("Current Repl Code:\n{}", current_code)
    }

    pub fn eval(&mut self, input: impl ToString) -> Result<EvalResult> {
        self.eval_inner(input, None, false, &*DEFAULT_EVALUATOR)
    }
    //Note: These inputs should become a Config struct
    pub fn eval_with_configuration(
        &mut self,
        eval_config: EvalConfig<impl ToString>,
    ) -> Result<EvalResult> {
        let EvalConfig {
            input,
            interactive_function,
            color,
            evaluator,
        } = eval_config;
        self.eval_inner(input, interactive_function, color, evaluator)
    }

    fn eval_inner(
        &mut self,
        input: impl ToString,
        interactive_function: Option<fn(&mut Child) -> Result<()>>,
        color: bool,
        evaluator: &[String],
    ) -> Result<EvalResult> {
        // eval will always write to main
        let original_target = self.target_file;
        self.target_file = TargetFile::Main;

        let input = input.to_string();
        // `\n{}\n` to avoid print appearing in error messages
        let eval_statement = format!("{}{}{}", evaluator[0], input, evaluator[1]);
        let toolchain = self.toolchain;

        let (status, mut eval_result) = self.eval_in_tmp_repl(eval_statement, || {
            cargo_run(color, false, toolchain, interactive_function)
        })?;

        // remove trailing new line
        eval_result.pop();
        // restore target
        self.target_file = original_target;
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
        let (orig_body, orig_cursor) = match self.target_file {
            TargetFile::Main => (self.main_body.clone(), self.main_cursor),
            TargetFile::Lib => (self.lib_body.clone(), self.lib_cursor),
        };

        self.insert(input);
        self.write()?;
        let result = f();

        match self.target_file {
            TargetFile::Main => {
                self.main_body = orig_body;
                self.main_cursor = orig_cursor;
            }
            TargetFile::Lib => {
                self.lib_body = orig_body;
                self.lib_cursor = orig_cursor;
            }
        };

        result
    }

    pub fn toolchain(&self) -> ToolChain {
        self.toolchain
    }

    pub fn set_toolchain(&mut self, toolchain: ToolChain) {
        self.toolchain = toolchain;
    }

    pub fn set_main_result(&mut self, main_result: MainResult) {
        self.main_result = main_result;
        // rebuild main fn
        let (header, footer) = Self::generate_body_delimiters(self.executor, self.main_result);
        let footer_pos = self.main_body.len() - 2;
        self.main_body[0] = header;
        self.main_body[footer_pos] = footer;
    }

    pub fn add_dep(&self, dep: &[String]) -> std::io::Result<std::process::Child> {
        cargo_add(dep)
    }

    pub fn build(&self) -> std::io::Result<std::process::Child> {
        cargo_build(self.toolchain)
    }

    pub fn write(&self) -> io::Result<()> {
        let mut main_file = std::fs::File::create(&*MAIN_FILE)?;
        write!(main_file, "{}", self.main_body.join("\n"))?;
        let mut lib_file = std::fs::File::create(&*LIB_FILE)?;
        write!(lib_file, "{}", self.lib_body.join("\n"))?;

        Ok(())
    }

    // Used for external editors
    pub fn write_to_extern(&self) -> io::Result<()> {
        let mut main_file = std::fs::File::create(&*MAIN_FILE_EXTERN)?;
        write!(main_file, "{}", self.main_body.join("\n"))?;

        Ok(())
    }

    pub fn touch_lib(&self) -> io::Result<()> {
        std::fs::File::create(&*LIB_FILE).map(|_| ())
    }
    pub fn write_lib(&self) -> io::Result<()> {
        let mut lib_file = std::fs::File::create(&*LIB_FILE)?;
        let mut body = self.main_body.clone();

        // safe unwrap
        let main_idx = body
            .iter()
            .position(|line| {
                line == &Self::generate_body_delimiters(self.executor, self.main_result).0
            })
            .unwrap();
        body.remove(main_idx); // remove fn main
        body.pop(); // remove result type [() | Ok(())]
        body.pop(); // remove last }

        write!(lib_file, "{}", body.join("\n"))?;

        Ok(())
    }

    pub fn pop(&mut self) {
        if self.main_body.len() > 2 {
            self.main_body.remove(self.main_cursor - 1);
            self.main_cursor -= 1;
        }
    }

    pub fn del(&mut self, line_num: &str) -> Result<()> {
        if let Ok(line_num) = line_num.parse::<usize>() {
            if line_num != 0 && line_num + 1 < self.main_body.len() {
                self.main_body.remove(line_num);
                self.main_cursor -= 1;
                return Ok(());
            }
        }

        Err("Incorrect line number".into())
    }

    pub fn lines<'a>(&'a self) -> impl Iterator<Item = &String> + 'a {
        self.main_body.iter()
    }
    pub fn lines_count(&self) -> usize {
        self.main_body.len() - 1
    }
}
