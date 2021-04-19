use crossterm::style::Color;

use super::cargo_cmds::{cargo_asm, cargo_bench, ToolChain};
use super::cargo_cmds::{cargo_fmt, cargo_fmt_file, cargo_run, MAIN_FILE_EXTERN};
use super::highlight::highlight;
use crate::irust::format::{format_check_output, format_err, format_eval_output};
use crate::irust::{IRust, Result};
use crate::utils::{remove_main, stdout_and_stderr};
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
    pub fn parse(&mut self) -> Result<PrintQueue> {
        // Order matters in this match
        match self.buffer.to_string().as_str() {
            ":help" => self.help(),
            ":reset" => self.reset(),
            ":show" => Ok(self.show()),
            ":pop" => self.pop(),
            ":irust" => self.irust(),
            ":sync" => self.sync(),
            cmd if cmd.starts_with("::") => self.run_cmd(),
            cmd if cmd.starts_with(":edit") => self.extern_edit(),
            cmd if cmd.starts_with(":add") => self.add_dep(),
            cmd if cmd.starts_with(":load") => self.load(),
            cmd if cmd.starts_with(":reload") => self.reload(),
            cmd if cmd.starts_with(":type") => self.show_type(),
            cmd if cmd.starts_with(":del") => self.del(),
            cmd if cmd.starts_with(":cd") => self.cd(),
            cmd if cmd.starts_with(":color") => self.color(),
            cmd if cmd.starts_with(":toolchain") => self.toolchain(),
            cmd if cmd.starts_with(":check_statements") => self.check_statements(),
            cmd if cmd.starts_with(":time_release") => self.time_release(),
            cmd if cmd.starts_with(":time") => self.time(),
            cmd if cmd.starts_with(":bench") => self.bench(),
            cmd if cmd.starts_with(":asm") => self.asm(),
            _ => self.parse_second_order(),
        }
    }

    fn reset(&mut self) -> Result<PrintQueue> {
        self.repl.reset(self.options.toolchain)?;
        success!()
    }

    fn pop(&mut self) -> Result<PrintQueue> {
        self.repl.pop();
        success!()
    }

    fn check_statements(&mut self) -> Result<PrintQueue> {
        const ERROR: &str = "Invalid argument, accepted values are `false` `true`";
        let buffer = self.buffer.to_string();
        let buffer = buffer.split_whitespace().nth(1).ok_or(ERROR)?;
        self.options.check_statements = buffer.parse().map_err(|_| ERROR)?;
        success!()
    }

    fn del(&mut self) -> Result<PrintQueue> {
        if let Some(line_num) = self.buffer.to_string().split_whitespace().last() {
            self.repl.del(line_num)?;
        }
        success!()
    }

    fn show(&mut self) -> PrintQueue {
        let code: Vec<char> = self.repl.show().chars().collect();
        highlight(&code.into(), &self.theme)
    }

    fn toolchain(&mut self) -> Result<PrintQueue> {
        self.options.toolchain = ToolChain::from_str(
            self.buffer
                .to_string()
                .split_whitespace()
                .nth(1)
                .unwrap_or("?"),
        )?;
        success!()
    }

    fn add_dep(&mut self) -> Result<PrintQueue> {
        let mut dep: Vec<String> = crate::utils::split_args(self.buffer.to_string());
        dep.remove(0); //drop :add

        // Try to canonicalize all arguments that corresponds to an existing path
        // This is necessary because `:add relative_path` doesn't work without it
        // Note this might be a bit too aggressive (an argument might be canonicalized, that the user didn't not intend for it to be considered as a path)
        // But the usefulness of this trick, outways this possible edge case
        // canonicalize is problamatic on windows -> need to handle extended path
        #[cfg(unix)]
        for p in dep.iter_mut() {
            let path = std::path::Path::new(p);
            if path.exists() {
                if let Ok(full_path) = path.canonicalize() {
                    if let Some(full_path) = full_path.to_str() {
                        *p = full_path.to_string();
                    }
                }
            }
        }
        // But still the most common case is `:add .` so we can special case that
        #[cfg(windows)]
        for p in dep.iter_mut() {
            if p == "." {
                *p = self
                    .global_variables
                    .get_cwd()
                    .to_str()
                    .ok_or("Error parsing path to dependecy")?
                    .to_string();
            }
        }

        self.wait_add(self.repl.add_dep(&dep)?, "Add")?;
        self.wait_add(self.repl.build(self.options.toolchain)?, "Build")?;

        if self.options.check_statements {
            self.wait_add(
                super::cargo_cmds::cargo_check(self.options.toolchain)?,
                "Check",
            )?;
        }

        success!()
    }

    fn color(&mut self) -> Result<PrintQueue> {
        let buffer = self.buffer.to_string();
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

        if let Err(e) = parse() {
            return Err(e);
        }

        success!()
    }

    fn load(&mut self) -> Result<PrintQueue> {
        let buffer = self.buffer.to_string();
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

    pub fn load_inner(&mut self, path: std::path::PathBuf) -> Result<PrintQueue> {
        // save path
        self.global_variables
            .set_last_loaded_coded_path(path.clone());

        // reset repl
        self.repl.reset(self.options.toolchain)?;

        // read code
        let path_code = std::fs::read(path)?;
        let code = if let Ok(code) = String::from_utf8(path_code) {
            code
        } else {
            return Err("The specified file is not utf8 encoded").map_err(Into::into);
        };

        // Format code to make `remove_main` function work correctly
        let code = cargo_fmt(&code)?;
        let code = remove_main(&code);

        // build the code
        let (status, output) = self.repl.eval_build(code.clone(), self.options.toolchain)?;

        if !status.success() {
            Ok(format_err(&output))
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
        self.repl.eval_in_tmp_repl(variable, || -> Result<()> {
            let (_status, out) = cargo_run(false, false, toolchain)?;
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
            "Uknown".into()
        };

        print_queue!(var_type, self.options.ok_color)
    }

    fn run_cmd(&mut self) -> Result<PrintQueue> {
        // remove ::
        let buffer = &self.buffer.to_string()[2..];

        let mut cmd = buffer.split_whitespace();
        let output = stdout_and_stderr(
            std::process::Command::new(cmd.next().unwrap_or_default())
                .args(&cmd.collect::<Vec<&str>>())
                .output()?,
        )
        .trim()
        .to_owned();

        print_queue!(output, self.options.shell_color)
    }

    fn parse_second_order(&mut self) -> Result<PrintQueue> {
        // these consts are used to detect statements that don't require to be terminated with ';'
        // `loop` can return a value so we don't add it here, exp: `loop {break 4}`
        const FUNCTION_DEF: &str = "fn ";
        const ASYNC_FUNCTION_DEF: &str = "async fn ";
        const ENUM_DEF: &str = "enum ";
        const STRUCT_DEF: &str = "struct ";
        const TRAIT_DEF: &str = "trait ";
        const IMPL: &str = "impl ";
        const PUB: &str = "pub ";
        const WHILE: &str = "while ";
        const EXTERN: &str = "extern ";

        // attribute exp:
        // #[derive(Debug)]
        // struct B{}
        const ATTRIBUTE: &str = "#";

        let buffer = {
            let mut buffer = self.buffer.to_string();
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

        if buffer_trimmed.is_empty() {
            Ok(PrintQueue::default())
        } else if buffer_trimmed.ends_with(';')
            || self.options.auto_insert_semicolon
                && (buffer_trimmed.starts_with(FUNCTION_DEF)
                    || buffer_trimmed.starts_with(ASYNC_FUNCTION_DEF)
                    || buffer_trimmed.starts_with(ENUM_DEF)
                    || buffer_trimmed.starts_with(STRUCT_DEF)
                    || buffer_trimmed.starts_with(TRAIT_DEF)
                    || buffer_trimmed.starts_with(IMPL)
                    || buffer_trimmed.starts_with(ATTRIBUTE)
                    || buffer_trimmed.starts_with(PUB)
                    || buffer_trimmed.starts_with(WHILE)
                    || buffer_trimmed.starts_with(EXTERN))
        {
            let mut print_queue = PrintQueue::default();

            let mut insert_flag = true;

            if self.options.check_statements {
                if let Some(mut e) =
                    format_check_output(self.repl.check(buffer.clone(), self.options.toolchain)?)
                {
                    print_queue.append(&mut e);
                    insert_flag = false;
                }
            }

            // if cargo_check is disabled or if cargo_check is enabled but returned no error
            if insert_flag {
                self.repl.insert(buffer);
            }

            Ok(print_queue)
        } else {
            let mut outputs = PrintQueue::default();
            let (status, out) = self.repl.eval(buffer, self.options.toolchain)?;
            // Save output if it was a success
            if status.success() {
                self.global_variables.set_last_output(out.clone());
            }

            let output_prompt = self.get_output_prompt();
            if let Some(mut eval_output) = format_eval_output(status, out, output_prompt) {
                outputs.append(&mut eval_output);
            }

            Ok(outputs)
        }
    }

    pub fn sync(&mut self) -> Result<PrintQueue> {
        match self.repl.update_from_extern_main_file() {
            Ok(_) => success!(),
            Err(e) => {
                self.repl.reset(self.options.toolchain)?;
                Err(e)
            }
        }
    }

    fn extern_edit(&mut self) -> Result<PrintQueue> {
        // exp: :edit vi
        let editor: String = match self.buffer.to_string().split_whitespace().nth(1) {
            Some(ed) => ed.to_string(),
            None => return Err("No editor specified".into()),
        };

        self.printer.writer.raw.write_with_color(
            format!("waiting for {}...", editor),
            crossterm::style::Color::Magenta,
        )?;

        // Write repl to disk
        self.repl.write_to_extern()?;

        // beautify code
        cargo_fmt_file(&*MAIN_FILE_EXTERN);

        // some commands are not detected from path but still works  with cmd /C
        #[cfg(windows)]
        std::process::Command::new("cmd")
            .arg("/C")
            .arg(editor)
            .arg(&*MAIN_FILE_EXTERN)
            .spawn()?
            .wait()?;

        #[cfg(not(windows))]
        std::process::Command::new(editor)
            .arg(&*MAIN_FILE_EXTERN)
            .spawn()?
            .wait()?;

        self.sync()
    }

    fn irust(&mut self) -> Result<PrintQueue> {
        print_queue!(self.ferris(), Color::Red)
    }

    fn cd(&mut self) -> Result<PrintQueue> {
        use std::env::*;
        let buffer = self.buffer.to_string();
        let buffer = buffer
            .split(":cd")
            .skip(1)
            .collect::<String>()
            .trim()
            .to_string();
        match buffer.as_str() {
            "" => {
                if let Some(dir) = dirs_next::home_dir() {
                    set_current_dir(dir)?;
                }
            }
            "-" => {
                set_current_dir(self.global_variables.get_pwd())?;
            }
            path => {
                let mut dir = current_dir()?;
                dir.push(&path);
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

    fn time(&mut self) -> Result<PrintQueue> {
        self.inner_time(":time", false)
    }
    fn time_release(&mut self) -> Result<PrintQueue> {
        self.inner_time(":time_release", true)
    }

    fn inner_time(&mut self, pattern: &str, release: bool) -> Result<PrintQueue> {
        let buffer = self.buffer.to_string();
        let fnn = buffer
            .splitn(2, pattern)
            .nth(1)
            .ok_or("No function specified")?;

        if fnn.is_empty() {
            return Err("No function specified".into());
        }

        let time = format!(
            "\
        use std::time::Instant;
        let now = Instant::now();
        {};
        println!(\"{{:?}}\", now.elapsed());
        ",
            fnn,
        );

        let toolchain = self.options.toolchain;
        let mut raw_out = String::new();
        let mut status = None;
        self.repl.eval_in_tmp_repl(time, || -> Result<()> {
            let (s, out) = cargo_run(true, release, toolchain)?;
            raw_out = out;
            status = Some(s);
            Ok(())
        })?;

        let output_prompt = self.get_output_prompt();
        // safe unwrap
        Ok(format_eval_output(status.unwrap(), raw_out, output_prompt)
            .ok_or("failed to bench function")?)
    }

    fn bench(&mut self) -> Result<PrintQueue> {
        //make sure we have the latest changes in main.rs
        self.repl.write()?;
        let out = cargo_bench(self.options.toolchain)?.trim().to_owned();

        print_queue!(out, self.options.eval_color)
    }

    fn asm(&mut self) -> Result<PrintQueue> {
        let buffer = self.buffer.to_string();
        let fnn = buffer.strip_prefix(":asm").expect("already checked").trim();
        if fnn.is_empty() {
            return Err("No function specified".into());
        }

        self.repl.write_lib()?;
        let asm = cargo_asm(fnn, self.options.toolchain)?;

        print_queue!(asm, self.options.eval_color)
    }
}
