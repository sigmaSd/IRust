use super::{
    cargo_cmds::MAIN_FILE,
    highlight::{highlight, theme::Theme},
    Result,
};
use crate::irust::printer::PrintQueue;
use crate::irust::printer::PrinterItem;
use crate::utils::{read_until_bytes, StringTools};
use crossterm::{style::Color, terminal::ClearType};
use std::io::Write;
use std::process::{Child, Command, Stdio};

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
    cmds: [String; 16],
    update_lock: bool,
    pub active_suggestion: Option<String>,
}

impl Racer {
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
            "show".to_string(),
            "help".to_string(),
            "pop".to_string(),
            "del".to_string(),
            "add".to_string(),
            "reset".to_string(),
            "load".to_string(),
            "reload".to_string(),
            "type".to_string(),
            "cd".to_string(),
            "color".to_string(),
            "toolchain".to_string(),
            "check_statements".to_string(),
            "time".to_string(),
            "time_release".to_string(),
            "bench".to_string(),
        ];

        Some(Racer {
            process,
            cursor,
            suggestions: vec![],
            suggestion_idx: 0,
            cmds,
            update_lock: false,
            active_suggestion: None,
        })
    }

    fn complete_code(&mut self) -> Result<()> {
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
            MAIN_FILE.display()
        ) {
            Ok(_) => (),
            Err(e) => {
                return Err(format!(
                    "\n\rError writing to racer, make sure it's properly configured\
                     \n\rCheckout https://github.com/racer-rust/racer/#configuration\
                     \n\rOr disable it in the configuration file.\
                     \n\rError: {}",
                    e
                )
                .into());
            }
        };

        // read till END
        let mut raw_output = vec![];
        read_until_bytes(
            &mut std::io::BufReader::new(stdout),
            b"END",
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
            .unwrap_or_else(|| self.suggestions.len());
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
    pub fn update_suggestions(
        &mut self,
        buffer: &super::Buffer,
        repl: &mut crate::irust::repl::Repl,
    ) -> Result<()> {
        // get the buffer as string
        let buffer: String = buffer.iter().take(buffer.buffer_pos).collect();

        // don't autocomplete shell commands
        if buffer.starts_with("::") {
            return Ok(());
        }

        self.show_suggestions_inner(buffer, repl)?;

        Ok(())
    }

    fn show_suggestions_inner(
        &mut self,
        buffer: String,
        repl: &mut crate::irust::repl::Repl,
    ) -> Result<()> {
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

            racer.cursor.0 = repl.body.len() + StringTools::new_lines_count(&buffer);

            racer.cursor.1 = 0;
            for c in buffer.chars() {
                if c == '\n' {
                    racer.cursor.1 = 0;
                } else {
                    racer.cursor.1 += 1;
                }
            }

            repl.eval_in_tmp_repl(buffer, move || -> Result<()> {
                racer.complete_code().map_err(From::from)
            })?;
        }

        Ok(())
    }

    fn write_next_suggestion(
        &mut self,
        printer: &mut super::printer::Printer<impl std::io::Write>,
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
        printer: &mut super::printer::Printer<impl std::io::Write>,
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
        printer: &mut crate::irust::printer::Printer<impl std::io::Write>,
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
                    .collect::<Vec<char>>(),
                theme,
            );

            let mut sug = PrintQueue::default();
            sug.push(PrinterItem::String(suggestion.clone(), color));

            let mut post = highlight(
                &buffer
                    .iter()
                    .skip(buffer.buffer_pos)
                    .copied()
                    .collect::<Vec<char>>(),
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
        printer: &mut super::printer::Printer<impl Write>,
        buffer: &super::Buffer,
        theme: &super::Theme,
        cycle: Cycle,
        options: &super::options::Options,
    ) -> Result<()> {
        // Max suggestions number to show
        let suggestions_num = std::cmp::min(self.suggestions.len(), options.racer_max_suggestions);

        // if The total input + suggestion >  screen height don't draw the suggestions
        if printer.cursor.buffer_pos_to_cursor_pos(&buffer).1 + suggestions_num
            >= printer.cursor.bound.height - 1
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
        printer.cursor.move_to_input_last_row(&buffer);

        let max_width = printer.cursor.bound.width - 1;
        printer.cursor.pos.current_pos.0 = 0;
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
        printer.recalculate_bounds(highlight(&buffer.buffer, &theme))?;

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
