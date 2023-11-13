mod rust_analyzer;
use self::rust_analyzer::RustAnalyzer;

use super::{
    highlight::{highlight, theme::Theme},
    Result,
};
use crate::utils::StringTools;
use crossterm::{style::Color, terminal::ClearType};
use irust_repl::Repl;
use printer::printer::{PrintQueue, Printer, PrinterItem};
use std::io::Write;
use std::path::Path;

pub enum Cycle {
    Up,
    Down,
}

pub struct Completer {
    pub rust_analyzer: RustAnalyzer,
    cursor: (usize, usize),
    // suggestions: (Name, definition)
    suggestions: Vec<(String, String)>,
    suggestion_idx: usize,
    cmds: [String; 30],
    update_lock: bool,
    pub active_suggestion: Option<String>,
}

impl Completer {
    pub fn start_ra(irust_dir: &Path, main_file: &Path, repl_body: String) -> Option<Completer> {
        let rust_analyzer = RustAnalyzer::start(irust_dir, main_file, repl_body).ok()?;

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

        Some(Completer {
            cursor,
            suggestions: vec![],
            suggestion_idx: 0,
            cmds,
            update_lock: false,
            active_suggestion: None,
            rust_analyzer,
        })
    }

    fn complete_code_ra(&mut self, main_file: &Path, text: String, buffer: &str) -> Result<()> {
        // check for lock
        if self.update_lock {
            return Ok(());
        }
        // reset suggestions
        self.suggestions.clear();
        self.goto_first_suggestion();

        self.rust_analyzer.document_did_change(main_file, text)?;

        let completions = self
            .rust_analyzer
            .document_completion(main_file, (self.cursor.0 - 1, self.cursor.1))?;

        // 1. walk buffer in reverse till first non alpha character
        let alpha_buffer = buffer
            .chars()
            .rev()
            .take_while(|c| c.is_alphabetic() || *c == '_')
            .collect::<String>()
            .chars()
            .rev()
            .collect::<String>();

        for completion in completions {
            if completion.starts_with(&alpha_buffer) {
                self.suggestions.push((completion, "".into()));
            }
        }

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
            self.suggestions.first().map(ToOwned::to_owned)
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

impl Completer {
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
            let ra = self;

            ra.cursor.0 = repl.lines_count() + StringTools::new_lines_count(&buffer);

            ra.cursor.1 = 0;
            for c in buffer.chars() {
                if c == '\n' {
                    ra.cursor.1 = 0;
                } else {
                    ra.cursor.1 += 1;
                }
            }

            let buf_ref = &buffer;
            repl.eval_in_tmp_repl(buffer.clone(), move |repl| -> Result<()> {
                ra.complete_code_ra(&repl.cargo.paths.main_file, repl.body(), buf_ref)
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
        let suggestions_num = std::cmp::min(self.suggestions.len(), options.ra_max_suggestions);

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
                    options.ra_inline_suggestion_color,
                )?;
            }
            Cycle::Up => self.write_previous_suggestion(
                printer,
                buffer,
                theme,
                options.ra_inline_suggestion_color,
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
            .set_fg(options.ra_suggestions_table_color)?;

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
            let mut suggestion = Completer::full_suggestion(suggestion);
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
                    .set_bg(options.ra_selected_suggestion_color)?;
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

    pub fn lock_ra_update(&mut self) -> Result<()> {
        self.update_lock = true;
        Ok(())
    }

    pub fn unlock_ra_update(&mut self) -> Result<()> {
        self.update_lock = false;
        Ok(())
    }
}
