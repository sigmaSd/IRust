pub mod cargo_cmds;
use cargo_cmds::*;
mod executor;
pub use executor::Executor;
mod toolchain;
pub use toolchain::ToolChain;
mod main_result;
pub use main_result::MainResult;
mod edition;
pub use edition::Edition;
mod compile_mode;
pub use compile_mode::CompileMode;

use once_cell::sync::Lazy;
mod utils;

use std::{
    io::{self, Write},
    path::PathBuf,
    process::{Child, ExitStatus},
};

use anyhow::{Result, bail, anyhow};
// type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub static DEFAULT_EVALUATOR: Lazy<[String; 2]> =
    Lazy::new(|| ["println!(\"{:?}\", {\n".into(), "\n});".into()]);

pub struct EvalConfig<'a, S: ToString> {
    pub input: S,
    pub interactive_function: Option<fn(&mut Child) -> Result<()>>,
    pub color: bool,
    pub evaluator: &'a [String],
    pub compile_mode: CompileMode,
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
    body: Vec<String>,
    cursor: usize,
    toolchain: ToolChain,
    executor: Executor,
    main_result: MainResult,
    edition: Edition,
    prelude: Option<PathBuf>,
    pub cargo: Cargo,
}
impl Default for Repl {
    fn default() -> Self {
        Repl::new(
            ToolChain::default(),
            Executor::default(),
            MainResult::default(),
            Edition::default(),
            None,
        )
        .expect("Paniced while trying to create repl")
    }
}
impl Drop for Repl {
    fn drop(&mut self) {
        let _ = self.cargo.delete_project();
    }
}

const PRELUDE_NAME: &str = "irust_prelude";

impl Repl {
    pub fn new(
        toolchain: ToolChain,
        executor: Executor,
        main_result: MainResult,
        edition: Edition,
        prelude_parent_path: Option<PathBuf>,
    ) -> Result<Self> {
        let cargo = Cargo::default();
        // NOTE: All the code in new should always not block
        cargo.cargo_new(edition)?;
        if let Some(ref path) = prelude_parent_path {
            cargo.cargo_new_lib_simple(path, PRELUDE_NAME)?;
            cargo.cargo_add_prelude(path.join(PRELUDE_NAME), PRELUDE_NAME)?;
        }
        // check for required dependencies (in case of async)
        if let Some(dependecy) = executor.dependecy() {
            // needs to be sync
            // repl::new(Tokio)
            // repl.eval(5); // cargo-edit may not have written to Cargo.toml yet
            //
            // NOTE: This code blocks
            cargo.cargo_add_sync(&dependecy)?;
        }
        cargo.cargo_build(toolchain)?;

        let (header, footer) = Self::generate_body_delimiters(executor, main_result);
        let (body, cursor) = if prelude_parent_path.is_some() {
            (
                vec![
                    header,
                    format!("#[allow(unused_imports)]use {PRELUDE_NAME}::*;"),
                    footer,
                    "}".to_string(),
                ],
                2,
            )
        } else {
            (vec![header, footer, "}".to_string()], 1)
        };
        Ok(Self {
            body,
            cursor,
            toolchain,
            executor,
            main_result,
            edition,
            prelude: prelude_parent_path,
            cargo,
        })
    }

    fn generate_body_delimiters(executor: Executor, main_result: MainResult) -> (String, String) {
        (
            executor.main() + " -> " + main_result.ttype() + "{",
            "#[allow(unreachable_code)]".to_string()
                + main_result.instance()
                + " // Do not write past this line (it will corrupt the repl)",
        )
    }

    pub fn set_executor(&mut self, executor: Executor) -> Result<()> {
        // remove old dependecy if it exists
        if let Some(dependecy) = self.executor.dependecy() {
            // cargo rm needs only the crate name
            self.cargo.cargo_rm_sync(&dependecy[0])?;
        }

        // use the new executor
        self.executor = executor;
        // check for required dependencies (in case of async)
        if let Some(dependecy) = executor.dependecy() {
            self.cargo.cargo_add_sync(&dependecy)?;
        }
        // finally set the correct main function
        let (header, footer) = Self::generate_body_delimiters(self.executor, self.main_result);
        let footer_pos = self.body.len() - 2;
        self.body[0] = header;
        self.body[footer_pos] = footer;
        Ok(())
    }

    pub fn update_from_extern_main_file(&mut self) -> Result<()> {
        let main_file = std::fs::read_to_string(&self.cargo.paths.main_file_extern)?;
        let lines_num = main_file.lines().count();
        if lines_num < 2 {
            bail!("main.rs file corrupted, resetting irust..");
        }
        let cursor_pos = lines_num - 2;

        *self = Self {
            body: main_file.lines().map(ToOwned::to_owned).collect(),
            cursor: cursor_pos,
            toolchain: self.toolchain,
            executor: self.executor,
            main_result: self.main_result,
            edition: self.edition,
            prelude: self.prelude.clone(),
            cargo: self.cargo.clone(),
        };
        Ok(())
    }

    pub fn hard_load(&mut self, code: impl ToString, cursor: usize) {
        self.body = code.to_string().lines().map(ToOwned::to_owned).collect();
        self.cursor = cursor;
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
        *self = Self::new(
            self.toolchain,
            self.executor,
            self.main_result,
            self.edition,
            self.prelude.clone(),
        )?;
        Ok(())
    }

    pub fn show(&self) -> String {
        let mut current_code = self.body.join("\n");
        // If cargo fmt is present format output else ignore
        if let Ok(fmt_code) = self.cargo.cargo_fmt(&current_code) {
            current_code = fmt_code;
        }
        format!("Current Repl Code:\n{current_code}")
    }

    pub fn eval(&mut self, input: impl ToString) -> Result<EvalResult> {
        self.eval_inner(input, None, false, &*DEFAULT_EVALUATOR, CompileMode::Debug)
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
            compile_mode,
        } = eval_config;
        self.eval_inner(input, interactive_function, color, evaluator, compile_mode)
    }

    fn eval_inner(
        &mut self,
        input: impl ToString,
        interactive_function: Option<fn(&mut Child) -> Result<()>>,
        color: bool,
        evaluator: &[String],
        compile_mode: CompileMode,
    ) -> Result<EvalResult> {
        let input = input.to_string();
        // `\n{}\n` to avoid print appearing in error messages
        let eval_statement = format!(
            "{}{}{}std::process::exit(0);", // exit(0) allows :hard_load functions to inspect variables that are used after this line
            evaluator[0], input, evaluator[1]
        );
        let toolchain = self.toolchain;

        let cargo = self.cargo.clone();
        let (status, mut eval_result) = self.eval_in_tmp_repl(eval_statement, |_| {
            cargo.cargo_run(
                color,
                compile_mode.is_release(),
                toolchain,
                interactive_function,
            )
        })?;

        // remove trailing new line
        eval_result.pop();
        Ok((status, eval_result).into())
    }

    pub fn eval_build(&mut self, input: impl ToString) -> Result<EvalResult> {
        let input = input.to_string();
        let toolchain = self.toolchain;
        let cargo = self.cargo.clone();
        Ok(self
            .eval_in_tmp_repl(input, |_| -> Result<(ExitStatus, String)> {
                Ok(cargo.cargo_build_output(true, false, toolchain)?)
            })?
            .into())
    }

    pub fn eval_check(&mut self, buffer: String) -> Result<EvalResult> {
        let toolchain = self.toolchain;
        let cargo = self.cargo.clone();
        Ok(self
            .eval_in_tmp_repl(buffer, |_| Ok(cargo.cargo_check_output(toolchain)?))?
            .into())
    }

    pub fn eval_in_tmp_repl_without_io<T>(
        &mut self,
        input: String,
        mut f: impl FnMut(&Self) -> Result<T>,
    ) -> Result<T> {
        let orig_body = self.body.clone();
        let orig_cursor = self.cursor;

        self.insert(input);
        // self.write()?;
        let result = f(self);

        self.body = orig_body;
        self.cursor = orig_cursor;

        result
    }
    pub fn eval_in_tmp_repl<T>(
        &mut self,
        input: String,
        mut f: impl FnMut(&Self) -> Result<T>,
    ) -> Result<T> {
        let orig_body = self.body.clone();
        let orig_cursor = self.cursor;

        self.insert(input);
        self.write()?;
        let result = f(self);

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

    pub fn set_main_result(&mut self, main_result: MainResult) {
        self.main_result = main_result;
        // rebuild main fn
        let (header, footer) = Self::generate_body_delimiters(self.executor, self.main_result);
        let footer_pos = self.body.len() - 2;
        self.body[0] = header;
        self.body[footer_pos] = footer;
    }

    pub fn add_dep(&self, dep: &[String]) -> std::io::Result<std::process::Child> {
        self.cargo.cargo_add(dep)
    }

    pub fn build(&self) -> std::io::Result<std::process::Child> {
        self.cargo.cargo_build(self.toolchain)
    }

    pub fn write(&self) -> io::Result<()> {
        let mut main_file = std::fs::File::create(&self.cargo.paths.main_file)?;
        write!(main_file, "{}", self.body.join("\n"))?;

        Ok(())
    }

    pub fn body(&self) -> String {
        self.body.join("\n")
    }

    // Used for external editors
    pub fn write_to_extern(&self) -> io::Result<()> {
        let mut main_file = std::fs::File::create(&self.cargo.paths.main_file_extern)?;
        write!(main_file, "{}", self.body.join("\n"))?;

        Ok(())
    }

    fn write_lib(&self) -> io::Result<()> {
        let mut lib_file = std::fs::File::create(&self.cargo.paths.lib_file)?;
        let mut body = self.body.clone();

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

    fn remove_lib(&self) -> io::Result<()> {
        std::fs::remove_file(&self.cargo.paths.lib_file)
    }

    pub fn with_lib<T>(&self, f: impl Fn() -> T) -> io::Result<T> {
        self.write_lib()?;
        let r = f();
        self.remove_lib()?;
        Ok(r)
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

        Err(anyhow!("Incorrect line number"))
    }

    pub fn lines(&self) -> impl Iterator<Item = &String> {
        self.body.iter()
    }
    pub fn lines_count(&self) -> usize {
        self.body.len() - 1
    }
}
