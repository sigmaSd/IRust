use super::{
    highlight::{highlight, theme::Theme},
    Result,
};
use crate::utils::{read_until_bytes, StringTools};
use crossterm::{style::Color, terminal::ClearType};
use irust_repl::Repl;
use printer::printer::{PrintQueue, Printer, PrinterItem};
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Read, Write};
use std::process::ChildStdout;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::{
    path::Path,
    process::{Child, Command, Stdio},
};

static ID: AtomicUsize = AtomicUsize::new(2);

pub enum Cycle {
    Up,
    Down,
}

pub struct Racer {
    process: Child,
    cursor: (usize, usize),
    // suggestions: (Name, definition)
    suggestions: Vec<(String, String)>,
    suggestion_idx: usize,
    cmds: [String; 30],
    update_lock: bool,
    stdout: Option<BufReader<ChildStdout>>,
    pub active_suggestion: Option<String>,
}

impl Racer {
    pub fn start_ra(irust_dir: &Path, main_file: &Path, text: String) -> Option<Racer> {
        let mut process = Command::new("rust-analyzer")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("Failed to start rust-analyzer");
        let mut stdin = process.stdin.as_mut().unwrap();
        let mut stdout = BufReader::new(process.stdout.take().unwrap());

        // Send a "initialize" request to the language server
        let initialize_request = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "processId": std::process::id(),
                "rootUri": format!("file://{}",irust_dir.display()),
                "capabilities": {
                    "textDocument": {
                        "completion": {
                            "completionItem": {
                                "documentationFormat": ["plaintext"]
                            },
                            "completionItemKind": {
                                "valueSet": [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35]
                            }
                        }
                    }
                }
            },
        });

        send_request(&mut stdin, &initialize_request);

        // Send an "initialized" notification to the language server
        let initialized_notification = json!({
            "jsonrpc": "2.0",
            "method": "initialized",
            "params": {},
        });
        send_request(&mut stdin, &initialized_notification);

        // Wait for "initialize" response
        let _initialize_response: Value = read_message(&mut stdout).unwrap();

        // Send a "textDocument/didOpen" notification to the language server
        let did_open_notification = json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": format!("file://{}",main_file.display()),
                    "languageId": "rust",
                    "version": 1,
                    "text": text,
                },
            },
        });
        send_request(&mut stdin, &did_open_notification);

        let cursor = (2, 0);
        let cmds = [
            "help".to_string(),
            "reset".to_string(),
            "show".to_string(),
            "pop".to_string(),
            "sync".to_string(),
            "exit".to_string(),
            "quit".to_string(),
            "edit".to_string(),
            "add".to_string(),
            "load".to_string(),
            "reload".to_string(),
            "type".to_string(),
            "del".to_string(),
            "dbg".to_string(),
            "cd".to_string(),
            "color".to_string(),
            "toolchain".to_string(),
            "theme".to_string(),
            "main_result".to_string(),
            "check_statements".to_string(),
            "time_release".to_string(),
            "time".to_string(),
            "bench".to_string(),
            "asm".to_string(),
            "expand".to_string(),
            "executor".to_string(),
            "evaluator".to_string(),
            "scripts".to_string(),
            "compile_time".to_string(),
            "compile_mode".to_string(),
        ];
        Some(Racer {
            process,
            cursor,
            suggestions: vec![],
            suggestion_idx: 0,
            cmds,
            update_lock: false,
            active_suggestion: None,
            stdout: Some(stdout),
        })
    }
    pub fn start() -> Option<Racer> {
        let process = Command::new("racer")
            .arg("daemon")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .ok()?;
        // Disable Racer if unable to start it
        //.map_err(|_| IRustError::RacerDisabled)?;
        let cursor = (2, 0);
        let cmds = [
            "help".to_string(),
            "reset".to_string(),
            "show".to_string(),
            "pop".to_string(),
            "sync".to_string(),
            "exit".to_string(),
            "quit".to_string(),
            "edit".to_string(),
            "add".to_string(),
            "load".to_string(),
            "reload".to_string(),
            "type".to_string(),
            "del".to_string(),
            "dbg".to_string(),
            "cd".to_string(),
            "color".to_string(),
            "toolchain".to_string(),
            "theme".to_string(),
            "main_result".to_string(),
            "check_statements".to_string(),
            "time_release".to_string(),
            "time".to_string(),
            "bench".to_string(),
            "asm".to_string(),
            "expand".to_string(),
            "executor".to_string(),
            "evaluator".to_string(),
            "scripts".to_string(),
            "compile_time".to_string(),
            "compile_mode".to_string(),
        ];

        Some(Racer {
            process,
            cursor,
            suggestions: vec![],
            suggestion_idx: 0,
            cmds,
            update_lock: false,
            active_suggestion: None,
            stdout: todo!(),
        })
    }

    pub fn did_change(&mut self, text: String, main_file: &Path) {
        let did_change_notification = json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didChange",
            "params": {
                "textDocument": {
                    "uri": format!("file://{}",main_file.display()),
                    "version": 2,
                },
                "contentChanges": [
                    {
                        "text":text,
                    }
                ]
            },
        });
        let mut stdin = self
            .process
            .stdin
            .as_mut()
            .ok_or("failed to acess ra stdin")
            .unwrap();
        send_request(&mut stdin, &did_change_notification);
    }
    pub fn config_update(&mut self) {
        let reload_msg = json!({
            "jsonrpc": "2.0",
            "id":  ID.fetch_add(1, Ordering::SeqCst),
            "method": "rust-analyzer/reloadWorkspace",
        });
        let mut stdin = self
            .process
            .stdin
            .as_mut()
            .ok_or("failed to acess ra stdin")
            .unwrap();
        send_request(&mut stdin, &reload_msg);
        read_message(self.stdout.as_mut().unwrap()).unwrap();
    }
    fn complete_code_ra(&mut self, main_file: &Path, text: String, buffer: &String) -> Result<()> {
        // check for lock
        if self.update_lock {
            return Ok(());
        }
        // reset suggestions
        self.suggestions.clear();
        self.goto_first_suggestion();

        let mut stdin = self
            .process
            .stdin
            .as_mut()
            .ok_or("failed to acess ra stdin")?;

        // Send a "textDocument/completion" request to the language server

        let completion_request = json!({
            "jsonrpc": "2.0",
            "id":  ID.fetch_add(1, Ordering::SeqCst),
            "method": "textDocument/completion",
            "params": {
                "textDocument": {
                    "uri": format!("file://{}",main_file.display()),
                },
                "position": {
                    "line": self.cursor.0 -1,
                    "character": self.cursor.1-1,
                },
            },
        });

        send_request(&mut stdin, &completion_request);
        let completion_response = loop {
        let completion_response = read_message(self.stdout.as_mut().unwrap()).unwrap();
            if completion_response.get("result").is_some() {
            break completion_response
            }
            std::thread::sleep_ms(100);
        };

        if let Some(result) = completion_response.get("result") {
            if let Some(items) = result.get("items") {
                for label in items
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|item| item.get("filterText").unwrap().to_string())
                    // remove quotes
                    .map(|item| item[1..item.len() - 1].to_owned())
                {
                    let Some(buffer) = buffer
                            .split(".")
                            .last()
                            .unwrap()
                            .split(":")
                            .last()
                            .unwrap()
                            .split_whitespace()
                            .last() else {
                            return Ok(())
                        };
                    // dbg!(&label, &buffer);
                    if label.starts_with(buffer) {
                        self.suggestions.push((label, "".into()));
                    }
                }
            }
            // sort suggestions by the ones starting by byffer
            // self.suggestions.sort_by(|a, b| {
            //     let a = &a.0;
            //     let b = &b.0;
            //     let buffer = buffer.to_string();
            //     if a.starts_with(&buffer) && !b.starts_with(&buffer) {
            //         std::cmp::Ordering::Less
            //     } else if !a.starts_with(&buffer) && b.starts_with(&buffer) {
            //         std::cmp::Ordering::Greater
            //     } else {
            //         a.cmp(&b)
            //     }
            // });
        }

        Ok(())
    }

    fn complete_code(&mut self, main_file: &Path) -> Result<()> {
        // check for lock
        if self.update_lock {
            return Ok(());
        }
        // reset suggestions
        self.suggestions.clear();
        self.goto_first_suggestion();

        let stdin = self
            .process
            .stdin
            .as_mut()
            .ok_or("failed to acess racer stdin")?;
        let stdout = self
            .process
            .stdout
            .as_mut()
            .ok_or("faied to acess racer stdout")?;

        match writeln!(
            stdin,
            "complete {} {} {}",
            self.cursor.0,
            self.cursor.1,
            main_file.display()
        ) {
            Ok(_) => (),
            Err(e) => {
                return Err(format!(
                    "\n\rError writing to racer, make sure it's properly configured\
                     \n\rCheckout https://github.com/racer-rust/racer/#configuration\
                     \n\rOr disable it in the configuration file.\
                     \n\rError: {e}"
                )
                .into());
            }
        };

        // read till END
        let mut raw_output = vec![];
        read_until_bytes(
            &mut std::io::BufReader::new(stdout),
            b"END\n",
            &mut raw_output,
        )?;
        let raw_output = String::from_utf8(raw_output.to_vec())
            .map_err(|_| "racer output did not contain valid UTF-8")?;

        for suggestion in raw_output.lines().skip(1) {
            if suggestion == "END" {
                break;
            }
            let mut try_parse = || -> Option<()> {
                let start_idx = suggestion.find("MATCH ")? + 6;
                let mut indices = suggestion.match_indices(',');
                let name = suggestion[start_idx..indices.next()?.0].to_owned();
                let definition = suggestion[indices.nth(3)?.0..].to_owned();
                self.suggestions.push((name, definition[1..].to_owned()));
                Some(())
            };

            try_parse();
        }

        // remove duplicates
        self.suggestions.sort();
        self.suggestions.dedup();

        Ok(())
    }

    fn goto_next_suggestion(&mut self) {
        if self.suggestion_idx >= self.suggestions.len() {
            self.suggestion_idx = 0
        }
        self.suggestion_idx += 1;
    }

    fn goto_previous_suggestion(&mut self) {
        self.suggestion_idx = self
            .suggestion_idx
            .checked_sub(1)
            .unwrap_or(self.suggestions.len());
        if self.suggestion_idx == 0 {
            self.suggestion_idx = self.suggestions.len();
        }
    }

    pub fn current_suggestion(&self) -> Option<(String, String)> {
        if self.suggestion_idx > 1 {
            self.suggestions
                .get(self.suggestion_idx - 1)
                .map(ToOwned::to_owned)
        } else {
            self.suggestions.get(0).map(ToOwned::to_owned)
        }
    }

    fn goto_first_suggestion(&mut self) {
        self.suggestion_idx = 0;
    }

    fn full_suggestion(s: &(String, String)) -> String {
        if !s.1.is_empty() {
            s.0.to_owned() + ": " + &s.1
        } else {
            s.0.to_owned()
        }
    }
}

impl Racer {
    pub fn update_suggestions(&mut self, buffer: &super::Buffer, repl: &mut Repl) -> Result<()> {
        // get the buffer as string
        let buffer: String = buffer.iter().take(buffer.buffer_pos).collect();

        // don't autocomplete shell commands
        if buffer.starts_with("::") {
            return Ok(());
        }

        self.show_suggestions_inner(buffer, repl)?;

        Ok(())
    }

    fn show_suggestions_inner(&mut self, buffer: String, repl: &mut Repl) -> Result<()> {
        if buffer.starts_with(':') {
            // Auto complete IRust commands
            self.suggestions = self
                .cmds
                .iter()
                .filter(|c| c.starts_with(&buffer[1..]))
                // place holder for IRust command definitions
                .map(|c| (c.to_owned(), String::new()))
                .collect();
        } else {
            // Auto complete rust code
            let mut racer = self;

            racer.cursor.0 = repl.lines_count() + StringTools::new_lines_count(&buffer);

            racer.cursor.1 = 0;
            for c in buffer.chars() {
                if c == '\n' {
                    racer.cursor.1 = 0;
                } else {
                    racer.cursor.1 += 1;
                }
            }

            let main_file = repl.cargo.paths.main_file.clone();
            let main_file = main_file.as_path();
            let buf2 = buffer.clone();
            let buf2 = &buf2;
            repl.eval_in_tmp_repl(buffer, move |repl| -> Result<()> {
                racer
                    .complete_code_ra(main_file, repl.body(), &buf2)
                    .map_err(From::from)
            })?;
        }

        Ok(())
    }

    fn write_next_suggestion(
        &mut self,
        printer: &mut Printer<impl std::io::Write>,
        buffer: &super::Buffer,
        theme: &Theme,
        color: Color,
    ) -> Result<()> {
        self.goto_next_suggestion();
        self.write_current_suggestion(printer, buffer, theme, color)?;

        Ok(())
    }

    fn write_previous_suggestion(
        &mut self,
        printer: &mut Printer<impl std::io::Write>,
        buffer: &super::Buffer,
        theme: &super::Theme,
        color: Color,
    ) -> Result<()> {
        self.goto_previous_suggestion();
        self.write_current_suggestion(printer, buffer, theme, color)?;

        Ok(())
    }

    fn write_current_suggestion(
        &mut self,
        printer: &mut Printer<impl std::io::Write>,
        buffer: &super::Buffer,
        theme: &super::Theme,
        color: Color,
    ) -> Result<()> {
        if let Some(suggestion) = self.current_suggestion() {
            let mut suggestion = suggestion.0;
            let mut buffer = buffer.clone();
            StringTools::strings_unique(
                &buffer.iter().take(buffer.buffer_pos).collect::<String>(),
                &mut suggestion,
            );
            buffer.insert_str(&suggestion);

            let mut pre = highlight(
                &buffer
                    .iter()
                    .take(buffer.buffer_pos - StringTools::chars_count(&suggestion))
                    .copied()
                    .collect(),
                theme,
            );

            let mut sug = PrintQueue::default();
            sug.push(PrinterItem::String(suggestion.clone(), color));

            let mut post = highlight(
                &buffer.iter().skip(buffer.buffer_pos).copied().collect(),
                theme,
            );

            pre.append(&mut sug);
            pre.append(&mut post);
            printer.print_input_from_queue(pre, &buffer)?;

            self.active_suggestion = Some(suggestion);
        }

        Ok(())
    }

    pub fn cycle_suggestions(
        &mut self,
        printer: &mut Printer<impl Write>,
        buffer: &super::Buffer,
        theme: &super::Theme,
        cycle: Cycle,
        options: &super::options::Options,
    ) -> Result<()> {
        // Max suggestions number to show
        let suggestions_num = std::cmp::min(self.suggestions.len(), options.racer_max_suggestions);

        // if The total input + suggestion >  screen height don't draw the suggestions
        if printer.cursor.buffer_pos_to_cursor_pos(buffer).1 + suggestions_num
            >= printer.cursor.height() - 1
        {
            return Ok(());
        }

        // Write inline suggestion
        match cycle {
            Cycle::Down => {
                self.write_next_suggestion(
                    printer,
                    buffer,
                    theme,
                    options.racer_inline_suggestion_color,
                )?;
            }
            Cycle::Up => self.write_previous_suggestion(
                printer,
                buffer,
                theme,
                options.racer_inline_suggestion_color,
            )?,
        }

        // No suggestions to show
        if self.suggestions.is_empty() {
            return Ok(());
        }

        // Handle screen height overflow
        let height_overflow = printer
            .cursor
            .screen_height_overflow_by_new_lines(buffer, suggestions_num + 1);

        if height_overflow != 0 {
            printer.scroll_up(height_overflow);
        }

        printer.cursor.save_position();
        printer.cursor.move_to_input_last_row(buffer);

        let max_width = printer.cursor.width() - 1;
        printer.cursor.current_pos().0 = 0;
        printer.cursor.goto_internal_pos();
        printer.cursor.raw.move_down(1)?;
        printer.writer.raw.clear(ClearType::FromCursorDown)?;
        printer.cursor.raw.move_up(1)?;

        printer
            .writer
            .raw
            .set_fg(options.racer_suggestions_table_color)?;

        let current_suggestion = self.current_suggestion();

        for (idx, suggestion) in self
            .suggestions
            .iter()
            .skip(((self.suggestion_idx - 1) / suggestions_num) * suggestions_num)
            .take(suggestions_num)
            .enumerate()
        {
            let suggestion_c = suggestion.clone();
            // trancuate long suggestions
            let mut suggestion = Racer::full_suggestion(suggestion);
            if suggestion.len() > max_width {
                suggestion.truncate(max_width - 3);
                suggestion.push_str("...");
            }
            // move one + idx row down
            printer.cursor.raw.move_down(idx as u16 + 1)?;

            // write suggestion
            printer.cursor.raw.save_position()?;

            if Some(&suggestion_c) == current_suggestion.as_ref() {
                printer
                    .writer
                    .raw
                    .set_bg(options.racer_selected_suggestion_color)?;
            }

            printer.writer.raw.write(&suggestion)?;
            printer.writer.raw.set_bg(crossterm::style::Color::Reset)?;
            printer.cursor.raw.restore_position()?;
            printer.cursor.move_up(idx as u16 + 1);
        }

        // reset to input position and color
        printer.writer.raw.reset_color()?;
        printer.cursor.restore_position();
        printer.cursor.goto_internal_pos();
        printer.recalculate_bounds(highlight(buffer, theme))?;

        Ok(())
    }

    pub fn lock_racer_update(&mut self) -> Result<()> {
        self.update_lock = true;
        Ok(())
    }

    pub fn unlock_racer_update(&mut self) -> Result<()> {
        self.update_lock = false;
        Ok(())
    }
}

fn send_request(stdin: &mut std::process::ChildStdin, request: &Value) {
    let request_str = serde_json::to_string(request).unwrap();
    let content_length = request_str.len();
    writeln!(stdin, "Content-Length: {}\r", content_length).unwrap();
    writeln!(stdin, "\r").unwrap();
    write!(stdin, "{}", request_str).unwrap();
    stdin.flush().unwrap();
}

fn read_response(stdout: &mut BufReader<&mut std::process::ChildStdout>) -> Value {
    let mut content_length = None;
    let mut buf = vec![];
    loop {
        buf.clear();
        stdout.read_until(b'\n', &mut buf).unwrap();
        let line = String::from_utf8_lossy(&buf);
        if line == "\r\n" {
            break;
        }
        if line.starts_with("Content-Length: ") {
            let content_length_str = line
                .trim_start_matches("Content-Length: ")
                .trim()
                .to_string();
            content_length = Some(content_length_str.parse::<usize>().unwrap());
        }
    }

    let content_length = content_length.unwrap();
    let mut response_buf = vec![0; content_length];
    stdout.read_exact(&mut response_buf).unwrap();

    let response_str = String::from_utf8_lossy(&response_buf).to_string();
    serde_json::from_str(&response_str).unwrap()
}

// fn read_response(stdout: &mut BufReader<&mut std::process::ChildStdout>) -> Value {
fn read_message(reader: &mut BufReader<std::process::ChildStdout>) -> Result<Value> {
    let content_length = get_content_length(reader)?;
    let mut content = vec![0; content_length];

    reader.read_exact(&mut content)?;
    let json_string = String::from_utf8(content)?;
    let message = serde_json::from_str(&json_string)?;
    Ok(message)
}

fn get_content_length(reader: &mut BufReader<std::process::ChildStdout>) -> Result<usize> {
    let mut line = String::new();
    let mut blank_line = String::new();

    let mut _bytes_read = reader.read_line(&mut line)?;
    let idx = line.find("Content-Length").unwrap();
    let mut split = line[idx..].trim().split(": ");

    if split.next() == Some("Content-Length") {
        _bytes_read = reader.read_line(&mut blank_line)?;
        Ok(split
            .next()
            .and_then(|value_string| value_string.parse().ok())
            .ok_or("TODO")?)
    } else {
        return Err("malformed rpc message".into());
    }
}
