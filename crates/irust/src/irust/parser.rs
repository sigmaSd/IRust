use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Instant;
use std::{env, process};

use crossterm::style::Color;

use super::format::format_err_printqueue;
use super::highlight::highlight;
use crate::irust::{IRust, Result};
use crate::utils::{copy_dir, stdout_and_stderr};
use crate::utils::{find_workpace_root, patch_name_to};
use crate::{
    irust::format::{format_check_output, format_eval_output},
    utils::ctrlc_cancel,
};
use irust_repl::{CompileMode, EvalConfig, EvalResult, Executor, MainResult, ToolChain};
use printer::printer::{PrintQueue, PrinterItem};

const SUCCESS: &str = "Ok!";

macro_rules! success {
    () => {{
        let mut print_queue = PrintQueue::default();
        print_queue.push(PrinterItem::Str(SUCCESS, Color::Blue));
        print_queue.add_new_line(1);

        Ok(print_queue)
    }};
}
macro_rules! print_queue {
    ($item: expr, $color: expr) => {{
        let mut print_queue = PrintQueue::default();
        print_queue.push(PrinterItem::String($item, $color));
        print_queue.add_new_line(1);

        Ok(print_queue)
    }};
}

impl IRust {
    pub fn parse(&mut self, buffer: String) -> Result<PrintQueue> {
        // check if a script want to act upon the input
        // if so scripts have precedence over normal flow

        // Order matters in this match
        match buffer.as_str() {
            ":reset" => self.reset(),
            ":show" => Ok(self.show()),
            ":pop" => self.pop(),
            ":irust" => self.irust(),
            ":sync" => self.sync(),
            ":exit" | ":quit" => self.exit(),
            cmd if cmd.starts_with(":help") => self.help(buffer),
            cmd if cmd.starts_with("::") => self.run_cmd(buffer),
            cmd if cmd.starts_with(":edit") => self.extern_edit(buffer),
            cmd if cmd.starts_with(":add") => self.add_dep(buffer),
            cmd if cmd.starts_with(":hard_load_crate") => self.hard_load_crate(buffer),
            cmd if cmd.starts_with(":hard_load") => self.hard_load(buffer),
            cmd if cmd.starts_with(":load") => self.load(buffer),
            cmd if cmd.starts_with(":reload") => self.reload(),
            cmd if cmd.starts_with(":type") => self.show_type(),
            cmd if cmd.starts_with(":del") => self.del(buffer),
            cmd if cmd.starts_with(":dbg") => self.dbg(buffer),
            cmd if cmd.starts_with(":color") => self.color(buffer),
            cmd if cmd.starts_with(":cd") => self.cd(buffer),
            cmd if cmd.starts_with(":toolchain") => self.toolchain(buffer),
            cmd if cmd.starts_with(":theme") => self.theme(buffer),
            cmd if cmd.starts_with(":main_result") => self.main_result(buffer),
            cmd if cmd.starts_with(":check_statements") => self.check_statements(buffer),
            cmd if cmd.starts_with(":time_release") => self.time_release(buffer),
            cmd if cmd.starts_with(":time") => self.time(buffer),
            cmd if cmd.starts_with(":bench") => self.bench(),
            cmd if cmd.starts_with(":asm") => self.asm(buffer),
            cmd if cmd.starts_with(":executor") => self.executor(buffer),
            cmd if cmd.starts_with(":evaluator") => self.evaluator(buffer),
            cmd if cmd.starts_with(":scripts") => self.scripts(buffer),
            cmd if cmd.starts_with(":compile_time") => self.compile_time(buffer),
            cmd if cmd.starts_with(":compile_mode") => self.compile_mode(buffer),
            cmd if cmd.starts_with(":expand") => self.expand(buffer),
            cmd if self.options.shell_interpolate && cmd.contains("$$") => {
                let buffer = self.shell_interpolate(buffer)?;
                self.parse_second_order(buffer)
            }
            _ => self.parse_second_order(buffer),
        }
    }

    fn reset(&mut self) -> Result<PrintQueue> {
        self.repl.reset()?;
        success!()
    }

    fn pop(&mut self) -> Result<PrintQueue> {
        self.repl.pop();
        success!()
    }

    fn check_statements(&mut self, buffer: String) -> Result<PrintQueue> {
        const ERROR: &str = "Invalid argument, accepted values are `false` `true`";
        let buffer = buffer.split_whitespace().nth(1).ok_or(ERROR)?;
        self.options.check_statements = buffer.parse().map_err(|_| ERROR)?;
        success!()
    }

    fn del(&mut self, buffer: String) -> Result<PrintQueue> {
        if let Some(line_num) = buffer.split_whitespace().last() {
            self.repl.del(line_num)?;
        }
        success!()
    }

    fn show(&mut self) -> PrintQueue {
        let code: Vec<char> = self.repl.show().chars().collect();
        highlight(&code.into(), &self.theme)
    }

    fn toolchain(&mut self, buffer: String) -> Result<PrintQueue> {
        let toolchain = buffer.split_whitespace().nth(1);

        if let Some(toolchain) = toolchain {
            let toolchain = ToolChain::from_str(toolchain)?;
            self.repl.set_toolchain(toolchain);
            self.options.toolchain = toolchain;
            success!()
        } else {
            print_queue!(self.options.toolchain.to_string(), Color::Blue)
        }
    }

    fn theme(&mut self, buffer: String) -> Result<PrintQueue> {
        let name = buffer.split_whitespace().nth(1);

        if let Some(name) = name {
            if let Ok(theme) = super::highlight::theme::theme(name.to_string()) {
                self.theme = theme;
                self.options.theme = name.to_string();
                success!()
            } else {
                Err("Failed to set theme".into())
            }
        } else {
            let installed_themes: Vec<_> = super::highlight::theme::installed_themes()
                .unwrap_or_default()
                .into_iter()
                .filter_map(|d| d.file_name().to_str().map(|s| s.to_string()))
                .collect();
            let mut msg = format!("Current theme is: {}\n", self.options.theme);
            if !installed_themes.is_empty() {
                msg.push_str(&format!(
                    "Installed themes:\n{}",
                    installed_themes
                        .into_iter()
                        .map(|t| format!("- {}", t.replace(".toml", "")))
                        .collect::<Vec<_>>()
                        .join("\n")
                ))
            }
            print_queue!(msg, Color::Blue)
        }
    }

    fn main_result(&mut self, buffer: String) -> Result<PrintQueue> {
        let main_result = buffer.split_whitespace().nth(1);

        if let Some(main_result) = main_result {
            let main_result = MainResult::from_str(main_result)?;
            self.repl.set_main_result(main_result);
            self.options.main_result = main_result;
            success!()
        } else {
            print_queue!(self.options.main_result.to_string(), Color::Blue)
        }
    }

    fn add_dep(&mut self, buffer: String) -> Result<PrintQueue> {
        let mut dep: Vec<String> = crate::utils::split_args(buffer);
        dep.remove(0); //drop :add

        // Try to canonicalize all arguments that corresponds to an existing path
        // This is necessary because `:add relative_path` doesn't work without it
        // Note this might be a bit too aggressive (an argument might be canonicalized, that the user didn't not intend for it to be considered as a path)
        // But the usefulness of this trick, outways this possible edge case
        // canonicalize is problamatic on windows -> need to handle extended path
        // #[cfg(unix)]
        // for p in dep.iter_mut() {
        //     let path = std::path::Path::new(p);
        //     if path.exists() {
        //         if let Ok(full_path) = path.canonicalize() {
        //             if let Some(full_path) = full_path.to_str() {
        //                 *p = full_path.to_string();
        //             }
        //         }
        //     }
        // }
        // But still the most common case is `:add .` so we can special case that
        // #[cfg(windows)]
        // for p in dep.iter_mut() {
        //     if p == "." {
        //         *p = self
        //             .global_variables
        //             .get_cwd()
        //             .to_str()
        //             .ok_or("Error parsing path to dependecy")?
        //             .to_string();
        //     }
        // }

        self.wait_add(self.repl.add_dep(&dep)?, "Add")?;
        self.wait_add(self.repl.build()?, "Build")?;

        if self.options.check_statements {
            self.wait_add(
                self.repl.cargo.cargo_check(self.options.toolchain)?,
                "Check",
            )?;
        }

        if let Some(ra) = self.completer.as_mut() {
            ra.rust_analyzer.reload_workspace()?;
        }

        success!()
    }

    fn color(&mut self, buffer: String) -> Result<PrintQueue> {
        let mut buffer = buffer.split_whitespace().skip(1).peekable();

        // reset theme
        if buffer.peek() == Some(&"reset") {
            self.theme.reset();
            return success!();
        }

        let mut parse = || -> Result<()> {
            let key = buffer.next().ok_or("Key not specified")?;
            let value = buffer.next().ok_or("Value not specified")?;

            let mut theme = toml::Value::try_from(&self.theme)?;
            // test key
            *theme.get_mut(key).ok_or("key doesn't exist")? = value.into();

            // test Value
            if super::highlight::theme::theme_color_to_term_color(value).is_none() {
                return Err("Value is incorrect".into());
            }

            self.theme = theme.try_into()?;
            Ok(())
        };

        parse()?;

        success!()
    }

    fn load(&mut self, buffer: String) -> Result<PrintQueue> {
        let path = if let Some(path) = buffer.split_whitespace().nth(1) {
            std::path::Path::new(&path).to_path_buf()
        } else {
            return Err("No path specified").map_err(|e| e.into());
        };
        self.load_inner(path)
    }

    fn reload(&mut self) -> Result<PrintQueue> {
        let path = if let Some(path) = self.global_variables.get_last_loaded_coded_path() {
            path
        } else {
            return Err("No saved path").map_err(|e| e.into());
        };
        self.load_inner(path)
    }

    pub fn hard_load_crate(&mut self, buffer: String) -> Result<PrintQueue> {
        // 0- Load to repl
        let _ = self.hard_load(buffer.clone())?;
        // Copy crate workspace
        // 1- find file path
        let file_path = buffer.split_whitespace().nth(1).expect("already checked");
        // 2- find crate root
        let crate_root = {
            let metadata = String::from_utf8(
                std::process::Command::new("cargo")
                    .arg("metadata")
                    .current_dir(
                        std::path::Path::new(file_path)
                            .parent()
                            .ok_or("file must have a parent directory")?,
                    )
                    .output()?
                    .stdout,
            )?;
            std::path::PathBuf::from(
                find_workpace_root(metadata).ok_or("Could not find workspace root")?,
            )
        };
        // 3- Copy Cargo.toml
        std::fs::copy(
            crate_root.join("Cargo.toml"),
            &self.repl.cargo.paths.cargo_toml_file,
        )?;
        // 4- Patch crate to name to `irust_host_repl`
        patch_name_to(
            &self.repl.cargo.paths.cargo_toml_file,
            &self.repl.cargo.name,
        )?;
        // 5- Copy src directory
        // 5-1 Remove original src
        std::fs::remove_dir_all(&self.repl.cargo.paths.irust_src_dir)?;
        // 5-2 The actual copy
        copy_dir(
            &crate_root.join("src"),
            &self.repl.cargo.paths.irust_src_dir,
        )?;

        success!()
    }

    pub fn hard_load(&mut self, buffer: String) -> Result<PrintQueue> {
        let buffer = buffer.split_whitespace().collect::<Vec<_>>();
        if buffer.len() != 3 {
            return Err("Incorrect unsage".into());
        }
        let code = std::fs::read_to_string(buffer[1])?;
        let cursor: usize = buffer[2].parse()?;
        self.repl.hard_load(code, cursor);
        success!()
    }

    pub fn load_inner(&mut self, path: PathBuf) -> Result<PrintQueue> {
        // save path
        self.global_variables
            .set_last_loaded_coded_path(path.clone());

        // reset repl
        self.repl.reset()?;

        // read code
        let code = std::fs::read_to_string(path)?;

        // build the code
        let EvalResult { output, status } = self.repl.eval_build(code.clone())?;

        if !status.success() {
            Ok(format_err_printqueue(
                &output,
                self.options.show_warnings,
                &self.repl.cargo.name,
            ))
        } else {
            self.repl.insert(code);
            success!()
        }
    }

    fn show_type(&mut self) -> Result<PrintQueue> {
        // TODO
        // We should probably use the `Any` trait instead of the current method
        // Current method might break with compiler updates
        // On the other hand `Any` is more limited

        const TYPE_FOUND_MSG: &str = "expected `()`, found ";
        const EMPTY_TYPE_MSG: &str = "dev [unoptimized + debuginfo]";

        let variable = self
            .buffer
            .to_string()
            .trim_start_matches(":type")
            .to_string();
        let mut raw_out = String::new();

        let toolchain = self.options.toolchain;
        let get_type = format!("let _:() = {variable};");

        let cargo = self.repl.cargo.clone();
        self.repl.eval_in_tmp_repl(get_type, |_| -> Result<()> {
            let (_status, out) = cargo.cargo_build_output(false, false, toolchain)?;
            raw_out = out;
            Ok(())
        })?;

        let var_type = if raw_out.contains(TYPE_FOUND_MSG) {
            raw_out
                .lines()
                // there is a case where there could be 2 found msg
                // the second one is more detailed
                .rev()
                .find(|l| l.contains("found"))
                // safe
                .unwrap()
                .rsplit("found ")
                .next()
                // safe
                .unwrap()
                .to_string()
        } else if raw_out.contains(EMPTY_TYPE_MSG) {
            "()".into()
        } else {
            "Unknown".into()
        };

        print_queue!(var_type, self.options.ok_color)
    }

    fn run_cmd(&mut self, buffer: String) -> Result<PrintQueue> {
        // remove ::
        let buffer = &buffer[2..];

        let mut cmd = buffer.split_whitespace();
        let output = stdout_and_stderr(if cfg!(windows) {
            std::process::Command::new("cmd")
                .arg("/C")
                .arg(cmd.next().unwrap_or_default())
                .args(cmd.collect::<Vec<&str>>())
                .output()?
        } else {
            std::process::Command::new(cmd.next().unwrap_or_default())
                .args(cmd.collect::<Vec<&str>>())
                .output()?
        })
        .trim()
        .to_owned();

        print_queue!(output, self.options.shell_color)
    }

    fn parse_second_order(&mut self, buffer: String) -> Result<PrintQueue> {
        // Time irust compiling (includes rustc compiling + irust code)
        let timer = if self.options.compile_time {
            Some(Instant::now())
        } else {
            None
        };

        let buffer = {
            let mut buffer = buffer;
            // check for replace marker option
            if self.options.replace_output_with_marker {
                if let Some(output) = self.global_variables.get_last_output() {
                    buffer = buffer.replace(&self.options.replace_marker, output);
                }
            }
            buffer
        };

        // This trimmed buffer should not be inserted nor evaluated
        let buffer_trimmed = buffer.trim();

        let mut print_queue = if buffer_trimmed.is_empty() {
            PrintQueue::default()
        } else if buffer_trimmed.ends_with(';')
            || (self.options.auto_insert_semicolon && is_a_statement(buffer_trimmed))
        {
            let mut print_queue = PrintQueue::default();

            let mut insert_flag = true;

            if self.options.check_statements {
                self.before_compiling_hook();
                let check_result = self.repl.eval_check(buffer.clone());
                self.after_compiling_hook();
                if let Some(mut e) = format_check_output(
                    check_result?.output,
                    self.options.show_warnings,
                    &self.repl.cargo.name,
                ) {
                    print_queue.append(&mut e);
                    insert_flag = false;
                }
            }

            // if cargo_check is disabled or if cargo_check is enabled but returned no error
            if insert_flag {
                self.repl.insert(buffer);
                self.repl.write_to_extern()?;
            }

            print_queue
        } else {
            let mut outputs = PrintQueue::default();

            self.before_compiling_hook();
            let result = self.repl.eval_with_configuration(EvalConfig {
                input: buffer,
                interactive_function: Some(ctrlc_cancel),
                color: true,
                evaluator: &self.options.evaluator,
                compile_mode: self.options.compile_mode,
            });
            self.after_compiling_hook();
            let EvalResult { output, status } = result?;

            // Save output if it was a success
            if status.success() {
                self.global_variables.set_last_output(output.clone());
            }

            let output_prompt = self.get_output_prompt();
            if let Some(mut eval_output) = format_eval_output(
                status,
                output,
                output_prompt,
                self.options.show_warnings,
                &self.repl.cargo.name,
                self.options.new_lines_after_output,
            ) {
                outputs.append(&mut eval_output);
            }

            outputs
        };

        // Print compile time
        if let Some(timer) = timer {
            let time = PrinterItem::String(
                format!(
                    "[-] compiling took: {} millisseconds",
                    timer.elapsed().as_millis()
                ),
                Color::Magenta,
            );
            print_queue.add_new_line(1);
            print_queue.push(time);
        }

        Ok(print_queue)
    }

    pub fn sync(&mut self) -> Result<PrintQueue> {
        match self.repl.update_from_extern_main_file() {
            Ok(_) => success!(),
            Err(e) => {
                self.repl.reset()?;
                Err(e)
            }
        }
    }

    fn extern_edit(&mut self, buffer: String) -> Result<PrintQueue> {
        // exp: :edit vi
        let editor: String = match buffer.split_whitespace().nth(1) {
            Some(ed) => ed.to_string(),
            None => {
                if let Ok(ed) = env::var("EDITOR") {
                    ed
                } else {
                    return Err("No editor specified".into());
                }
            }
        };

        self.printer.writer.raw.write_with_color(
            format!("waiting for {editor}..."),
            crossterm::style::Color::Magenta,
        )?;

        // Write repl to disk
        self.repl.write_to_extern()?;

        // beautify code
        self.repl
            .cargo
            .cargo_fmt_file(&self.repl.cargo.paths.main_file_extern);

        // some commands are not detected from path but still works  with cmd /C
        #[cfg(windows)]
        std::process::Command::new("cmd")
            .arg("/C")
            .arg(editor)
            .arg(&self.repl.cargo.paths.main_file_extern)
            .spawn()?
            .wait()?;

        #[cfg(not(windows))]
        std::process::Command::new(editor)
            .arg(&self.repl.cargo.paths.main_file_extern)
            .spawn()?
            .wait()?;

        self.sync()
    }

    fn irust(&mut self) -> Result<PrintQueue> {
        print_queue!(self.ferris(), Color::Red)
    }

    fn cd(&mut self, buffer: String) -> Result<PrintQueue> {
        use std::env::*;
        let buffer = buffer
            .split(":cd")
            .skip(1)
            .collect::<String>()
            .trim()
            .to_string();
        match buffer.as_str() {
            "" => {
                if let Some(dir) = crate::utils::irust_dirs::home_dir() {
                    set_current_dir(dir)?;
                }
            }
            "-" => {
                set_current_dir(self.global_variables.get_pwd())?;
            }
            path => {
                let mut dir = current_dir()?;
                dir.push(path);
                set_current_dir(dir)?;
            }
        }
        // Update cwd and the terminal title accordingly
        let cwd = current_dir()?;
        self.global_variables.update_cwd(cwd.clone());
        self.printer
            .writer
            .raw
            .set_title(&format!("IRust: {}", cwd.display()))?;

        print_queue!(cwd.display().to_string(), self.options.ok_color)
    }

    fn time(&mut self, buffer: String) -> Result<PrintQueue> {
        self.inner_time(buffer, ":time", false)
    }
    fn time_release(&mut self, buffer: String) -> Result<PrintQueue> {
        self.inner_time(buffer, ":time_release", true)
    }

    fn inner_time(
        &mut self,
        buffer: String,
        pattern: &'static str,
        release: bool,
    ) -> Result<PrintQueue> {
        let fnn = buffer
            .split_once(pattern)
            .map(|buf| buf.1)
            .ok_or("No function specified")?;

        if fnn.is_empty() {
            return Err("No function specified".into());
        }

        let time = format!(
            "\
        use std::time::Instant;
        let now = Instant::now();
        {fnn};
        println!(\"{{:?}}\", now.elapsed());
        ",
        );

        let toolchain = self.options.toolchain;
        let mut raw_out = String::new();
        let mut status = None;

        let cargo = self.repl.cargo.clone();
        self.repl.eval_in_tmp_repl(time, |_| -> Result<()> {
            let (s, out) = cargo.cargo_run(true, release, toolchain, Some(ctrlc_cancel))?;
            raw_out = out;
            status = Some(s);
            Ok(())
        })?;

        let output_prompt = self.get_output_prompt();
        // safe unwrap
        Ok(format_eval_output(
            status.unwrap(),
            raw_out,
            output_prompt,
            self.options.show_warnings,
            &self.repl.cargo.name,
            self.options.new_lines_after_output,
        )
        .ok_or("failed to bench function")?)
    }

    fn bench(&mut self) -> Result<PrintQueue> {
        //make sure we have the latest changes in main.rs
        self.repl.write()?;
        let out = self
            .repl
            .cargo
            .cargo_bench(self.options.toolchain)?
            .trim()
            .to_owned();

        print_queue!(out, self.options.eval_color)
    }

    fn asm(&mut self, buffer: String) -> Result<PrintQueue> {
        let fnn = buffer.strip_prefix(":asm").expect("already checked").trim();
        if fnn.is_empty() {
            return Err("No function specified".into());
        }

        let asm = self
            .repl
            .with_lib(|| self.repl.cargo.cargo_asm(fnn, self.options.toolchain))??;

        print_queue!(asm, self.options.eval_color)
    }

    fn executor(&mut self, buffer: String) -> Result<PrintQueue> {
        let executor = buffer.split_whitespace().nth(1);
        if let Some(executor) = executor {
            let executor = Executor::from_str(executor.trim())?;
            self.repl.set_executor(executor)?;
            // save executor
            self.options.executor = executor;
            success!()
        } else {
            print_queue!(self.options.executor.to_string(), Color::Blue)
        }
    }

    fn evaluator(&mut self, buffer: String) -> Result<PrintQueue> {
        let evaluator = buffer
            .strip_prefix(":evaluator")
            .expect("already checked")
            .trim();
        // get
        if evaluator.is_empty() {
            return print_queue!(self.options.evaluator.join("$$"), Color::Blue);
        }

        // reset
        if evaluator == "reset" {
            self.options.reset_evaluator();
            return success!();
        }

        // Sanity checks
        // set
        if !evaluator.contains("$$") {
            return Err(
                "evaluator must contain `$$`, `$$` will be replaced by IRust by the code input"
                    .into(),
            );
        }
        if !evaluator.ends_with(';') {
            return Err("evaluator must end with ;".into());
        }

        let evaluator: Vec<String> = evaluator.split("$$").map(ToOwned::to_owned).collect();
        if evaluator.len() != 2 {
            return Err("evaluator requires two parts".into());
        }

        self.options.evaluator = evaluator;
        success!()
    }

    fn scripts(&mut self, buffer: String) -> Result<PrintQueue> {
        let scripts_list = if let Some(scripts_list) = self.scripts_list() {
            scripts_list
        } else {
            return Err("No scripts found".into());
        };

        let buffer: Vec<&str> = buffer
            .strip_prefix(":scripts")
            .expect("already checked")
            .split_whitespace()
            .collect();

        //TODO: This code can be improved *a lot* by doing formatting here, instead of letting each script manager do its own thing

        match buffer.len() {
            0 => {
                //NOTE: Hack formatting because its less code
                //Assumes that script state is the last column

                if scripts_list.is_empty() {
                    return print_queue!("Script list is empty".to_string(), Color::Blue);
                }

                let mut queue = PrintQueue::default();
                let mut scripts_list = scripts_list.lines();

                let header = scripts_list.next().expect("exists");
                queue.push(PrinterItem::String(header.to_string(), Color::Blue));
                queue.add_new_line(2);

                let mut list: Vec<_> = scripts_list.collect();
                list.sort_by_key(|line| line.ends_with("false"));
                list.iter().for_each(|line| {
                    if line.ends_with("true") {
                        queue.push(PrinterItem::String(line.to_string(), Color::Green))
                    } else {
                        queue.push(PrinterItem::String(line.to_string(), Color::Red))
                    }
                    queue.add_new_line(1);
                });

                Ok(queue)
            }
            1 => {
                // Print script
                if let Some(script) = scripts_list
                    .lines()
                    .skip(1)
                    .find(|line| line.contains(buffer[0]))
                {
                    let header = scripts_list
                        .lines()
                        .next()
                        .expect("header should be always present")
                        .to_string();
                    print_queue!(header + "\n" + script, Color::Blue)
                } else {
                    Err(format!("script: {} not found", &buffer[0]).into())
                }
            }
            2 => {
                // Set script state {0:script_name} {1:[activate|deactivate]}
                if let Some(script) = scripts_list.lines().skip(1).find_map(|line| {
                    if line.contains(buffer[0]) {
                        Some(line.split_whitespace().next()?)
                    } else {
                        None
                    }
                }) {
                    match buffer[1] {
                        "activate" => {
                            if let Some(command) = self.activate_script(script)? {
                                // script start up command
                                self.execute(command)?;
                            }
                            success!()
                        }
                        "deactivate" => {
                            if let Some(command) = self.deactivate_script(script)? {
                                // script clean up command
                                self.execute(command)?;
                            }
                            success!()
                        }
                        _ => Err(format!("Unknown argument: {}", &buffer[1]).into()),
                    }
                } else {
                    Err(format!("script: {} not found", &buffer[0]).into())
                }
            }
            _ => Err("Incorrect number of arguments for `:scripts` command".into()),
        }
    }
    fn compile_time(&mut self, buffer: String) -> Result<PrintQueue> {
        let buffer: Vec<&str> = buffer
            .strip_prefix(":compile_time")
            .expect("already checked")
            .split_whitespace()
            .collect();
        match buffer.len() {
            0 => {
                print_queue!(self.options.compile_time.to_string(), Color::Blue)
            }
            1 => match buffer[0].to_lowercase().as_str() {
                "on" => {
                    self.options.compile_time = true;
                    success!()
                }
                "off" => {
                    self.options.compile_time = false;
                    success!()
                }
                _ => Err("Invalid argument (only accepts on/off)".into()),
            },
            _ => Err("Invalid number of arguments".into()),
        }
    }

    fn compile_mode(&mut self, buffer: String) -> Result<PrintQueue> {
        let mode = buffer
            .strip_prefix(":compile_mode")
            .expect("already checked")
            .trim();

        if mode.is_empty() {
            return print_queue!(self.options.compile_mode.to_string(), Color::Blue);
        }

        self.options.compile_mode = CompileMode::from_str(mode)?;
        success!()
    }

    fn dbg(&mut self, buffer: String) -> Result<PrintQueue> {
        let expression = buffer
            .strip_prefix(":dbg")
            .expect("already checked")
            .trim()
            .to_owned();
        let expression = if expression.is_empty() {
            "println!(); // Compiler black box".into()
        } else {
            format!(
                "print!(\"{{:?}}\", {expression}); // Print to make sure that the compiler doesn't remove the expression (blackbox requires nightly)"
            )
        };

        let (debugger, debugger_arg) = match self.options.debugger {
            crate::irust::options::Debugger::LLDB => ("rust-lldb", "-s"),
            crate::irust::options::Debugger::GDB => ("rust-gdb", "-x"),
        };

        self.printer
            .writer
            .raw
            .write_with_color("Waiting for debugger...", crossterm::style::Color::Magenta)?;

        let expr_line_num = self.repl.lines_count();

        let cargo = self.repl.cargo.clone();
        self.repl.eval_in_tmp_repl(expression, |_| -> Result<()> {
            let (status, _out) = cargo.cargo_build_output(true, false, self.options.toolchain)?;
            if !status.success() {
                return Err("Failed to execute expression".into());
            }

            let dbg_cmds_path = crate::utils::irust_dirs::temp_dir().join("irust_dbg_cmds");
            {
                let mut dbg_cmds = std::fs::File::create(&dbg_cmds_path)?;
                writeln!(&mut dbg_cmds, "b {expr_line_num}")?;
                writeln!(&mut dbg_cmds, "r")?;
            }

            crossterm::terminal::disable_raw_mode()?;
            println!();
            crossterm::execute!(std::io::stdout(), crossterm::cursor::Show)?;

            #[cfg(windows)]
            std::process::Command::new("cmd")
                .arg("/C")
                .arg(debugger)
                .args([debugger_arg, &dbg_cmds_path.display().to_string()])
                .arg(&cargo.paths.exe_path)
                .spawn()?
                .wait()?;

            #[cfg(not(windows))]
            {
                std::process::Command::new(debugger)
                    .args([debugger_arg, &dbg_cmds_path.display().to_string()])
                    .arg(&cargo.paths.exe_path)
                    .spawn()?
                    .wait()?;
            }

            crossterm::terminal::enable_raw_mode()?;
            Ok(())
        })?;

        success!()
    }
    fn shell_interpolate(&mut self, buffer: String) -> Result<String> {
        // Replace shell expression with rust expression
        // The shell expression is delimited by `$$` and `$$`
        // There could be multiple shell expressions
        let mut buffer = buffer.chars().peekable();
        let mut res = String::new();

        let mut maybe_error = None;
        (|| -> Option<()> {
            loop {
                let c = buffer.next()?;

                if c == '$' && buffer.peek() == Some(&'$') {
                    buffer.next();
                    let mut shell_expr = String::new();
                    while let Some(s) = buffer.next() {
                        if s == '$' && buffer.peek() == Some(&'$') {
                            buffer.next();
                            break;
                        }
                        shell_expr.push(s);
                    }
                    let mut shell_expr = shell_expr.split_whitespace();
                    let shell_res = if cfg!(windows) {
                        process::Command::new("cmd")
                            .arg("/C")
                            .arg(shell_expr.next()?)
                            .args(shell_expr.collect::<Vec<_>>())
                            .output()
                    } else {
                        process::Command::new(shell_expr.next()?)
                            .args(shell_expr.collect::<Vec<_>>())
                            .output()
                    };
                    let shell_res = match shell_res {
                        Ok(res) => stdout_and_stderr(res),
                        Err(e) => {
                            maybe_error = Some(e);
                            break None;
                        }
                    };

                    struct F<'a>(&'a [u8]);
                    impl std::fmt::Display for F<'_> {
                        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                            write!(f, "String::from_utf8_lossy(&[")?;
                            for b in self.0.iter() {
                                write!(f, "{b},")?;
                            }
                            write!(f, "]).to_string()")
                        }
                    }
                    let shell_res = format!("{}", F(shell_res.as_bytes()));
                    res.push_str(&shell_res);
                } else {
                    res.push(c);
                }
            }
        })();
        if let Some(e) = maybe_error {
            return Err(e.into());
        }

        Ok(res)
    }

    fn expand(&mut self, buffer: String) -> Result<PrintQueue> {
        let fnn = buffer
            .strip_prefix(":expand")
            .expect("already checked")
            .trim();
        if !fnn.is_empty() {
            let r = self.repl.with_lib(|| {
                self.repl
                    .cargo
                    .cargo_expand(Some(fnn), self.options.toolchain)
            })??;
            print_queue!(r, Color::White)
        } else {
            print_queue!(
                self.repl.cargo.cargo_expand(None, self.options.toolchain)?,
                Color::White
            )
        }
    }

    fn exit(&mut self) -> Result<PrintQueue> {
        self.exit_flag = true;
        Ok(PrintQueue::default())
    }
}

// These patterns are used to detect statements that don't require to be terminated with ';'
// Note: `loop` can return a value so we don't add it here, exp: `loop {break 4}`
pub fn is_a_statement(buffer_trimmed: &str) -> bool {
    match buffer_trimmed
        .split_whitespace()
        .collect::<Vec<_>>()
        .as_slice()
    {
        // async fn|const fn|unsafe fn
        [_, "fn", ..]
        | ["fn", ..]
        | ["enum", ..]
        | ["struct", ..]
        | ["trait", ..]
        | ["impl", ..]
        | ["pub", ..]
        | ["extern", ..]
        | ["macro", ..] => true,
        ["macro_rules!", ..] => true,
        // attribute exp:
        // #[derive(Debug)]
        // struct B{}
        [tag, ..] if tag.starts_with('#') => true,
        _ => false,
    }
}
