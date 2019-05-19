use crate::irust::printer::{Printer, PrinterItem, PrinterItemType};
use crate::irust::IRust;
use crate::utils::StringTools;
use crossterm::{ClearType, Color};
use std::error::Error;

impl IRust {
    pub fn handle_character(&mut self, c: char) -> std::io::Result<()> {
        if c == '\n' {
            self.handle_enter()?
        } else {
            StringTools::insert_at_char_idx(&mut self.buffer, self.internal_cursor.x, c);

            //self.racer.complete();

            self.write_insert(c)?;
        }
        Ok(())
    }

    fn handle_enter(&mut self) -> std::io::Result<()> {
        // create a new line
        self.write_newline()?;

        // add commands to history
        if self.should_push_to_history(&self.buffer) {
            self.history.push(self.buffer.clone());
        }

        // parse and handle errors
        match self.parse() {
            Ok(out) => {
                self.printer = out;
            }
            Err(e) => {
                self.printer = Printer::new(PrinterItem::new(
                    e.description().to_string(),
                    PrinterItemType::Err,
                ));
                self.printer.add_new_line(1);
            }
        }

        // ensure buffer is cleaned
        self.buffer.clear();

        // write out
        if !self.printer.is_empty() {
            self.write_out()?;
            self.write_newline()?;
        }
        self.write_in()?;

        Ok(())
    }

    pub fn handle_up(&mut self) -> std::io::Result<()> {
        self.internal_cursor.x = 0;
        self.move_cursor_to(4, None)?;
        self.terminal.clear(ClearType::UntilNewLine)?;
        let up = self.history.up();
        self.buffer = up.clone();
        self.write(&up)?;
        Ok(())
    }

    pub fn handle_down(&mut self) -> std::io::Result<()> {
        self.internal_cursor.x = 0;
        self.move_cursor_to(4, None)?;
        self.terminal.clear(ClearType::UntilNewLine)?;
        let down = self.history.down();
        self.buffer = down.clone();
        self.write(&down)?;
        Ok(())
    }

    pub fn handle_left(&mut self) -> std::io::Result<()> {
        if self.internal_cursor.x > 0 {
            self.cursor.move_left(1);
            self.internal_cursor.move_left(1);
        }
        Ok(())
    }

    pub fn handle_right(&mut self) -> std::io::Result<()> {
        if self.internal_cursor.x < StringTools::chars_count(&self.buffer) {
            self.cursor.move_right(1);
            self.internal_cursor.move_right(1);
        }
        Ok(())
    }

    pub fn handle_backspace(&mut self) -> std::io::Result<()> {
        if self.internal_cursor.x > 0 {
            self.cursor.move_left(1);
            self.internal_cursor.move_left(1);
            if !self.buffer.is_empty() {
                StringTools::remove_at_char_idx(&mut self.buffer, self.internal_cursor.x);
            }
            self.backspace()?;
        }
        Ok(())
    }

    pub fn handle_ctrl_c(&mut self) -> std::io::Result<()> {
        if self.buffer.is_empty() {
            self.exit()?;
        } else {
            self.buffer.clear();
            self.write_newline()?;
            self.write_in()?;
        }

        Ok(())
    }

    pub fn handle_ctrl_d(&mut self) -> std::io::Result<()> {
        if self.buffer.is_empty() {
            self.exit()?;
        }

        Ok(())
    }

    fn exit(&mut self) -> std::io::Result<()> {
        self.terminal.clear(ClearType::All)?;
        self.terminal.exit();

        Ok(())
    }

    pub fn handle_ctrl_z(&mut self) -> std::io::Result<()> {
        #[cfg(target_family = "unix")]
        {
            use nix::{
                sys::signal::{kill, Signal},
                unistd::Pid,
            };
            self.terminal.clear(ClearType::All)?;
            let _ = kill(Pid::this(), Some(Signal::SIGTSTP));

            // display empty prompt after SIGCONT
            self.clear()?;
        }

        Ok(())
    }

    pub fn clear(&mut self) -> std::io::Result<()> {
        self.terminal.clear(ClearType::All)?;
        self.internal_cursor.reset();
        self.move_cursor_to(0, 1)?;
        self.buffer.clear();
        self.write_in()?;
        Ok(())
    }

    pub fn go_to_start(&mut self) -> std::io::Result<()> {
        self.internal_cursor.x = 0;
        self.move_cursor_to(4, None)?;
        Ok(())
    }

    pub fn go_to_end(&mut self) -> std::io::Result<()> {
        let end_idx = StringTools::chars_count(&self.buffer);
        self.internal_cursor.x = end_idx;
        self.move_cursor_to(end_idx + 4, None)?;
        Ok(())
    }

    pub fn show_suggestions(&mut self) -> std::io::Result<()> {
        let mut tmp_repl = self.repl.clone();
        let y_pos = tmp_repl.body.len();
        tmp_repl.insert(self.buffer.clone());
        tmp_repl.write()?;

        self.racer.cursor.0 = y_pos;
        self.racer.cursor.1 = self.buffer.len() + 1;
        //dbg!(&self.racer);

        self.racer.complete()?;

        if let Some(suggestion) = self.racer.next_suggestion() {
            self.color.set_fg(Color::Cyan)?;
            self.cursor.save_position()?;
            self.terminal.clear(ClearType::UntilNewLine)?;

            let mut idx = suggestion.len();
            let mut suggestion = suggestion.to_string();
            loop {
                if self.buffer.ends_with(&suggestion[..idx]) {
                    for _ in 0..idx {
                        suggestion.remove(0);
                    }

                    break;
                }
                if idx == 0 {
                    break;
                }

                idx -= 1;
            }

            self.terminal.write(&suggestion)?;
            self.cursor.reset_position()?;
            self.color.reset()?;
        }

        Ok(())
    }
}
