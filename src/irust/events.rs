use crate::irust::printer::{Printer, PrinterItem, PrinterItemType};
use crate::irust::IRust;
use crate::utils::StringTools;
use crossterm::ClearType;
use std::error::Error;

impl IRust {
    pub fn handle_character(&mut self, c: char) -> std::io::Result<()> {
        self.racer_needs_update(true);
        StringTools::insert_at_char_idx(&mut self.buffer, self.internal_cursor.x, c);
        self.write_insert(c)?;
        Ok(())
    }

    pub fn handle_enter(&mut self) -> std::io::Result<()> {
        // unvalidate racer cache
        self.racer_needs_update(true);

        // clear suggestion
        self.clear_suggestion()?;

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
                    e.to_string(),
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

    pub fn handle_tab(&mut self) -> std::io::Result<()> {
        self.show_suggestions();
        Ok(())
    }

    pub fn handle_up(&mut self) -> std::io::Result<()> {
        self.racer_needs_update(true);
        self.internal_cursor.x = 0;
        self.move_cursor_to(4, None)?;
        self.terminal.clear(ClearType::UntilNewLine)?;
        let up = self.history.up();
        self.buffer = up.clone();
        self.write(&up)?;
        Ok(())
    }

    pub fn handle_down(&mut self) -> std::io::Result<()> {
        self.racer_needs_update(true);
        self.internal_cursor.x = 0;
        self.move_cursor_to(4, None)?;
        self.terminal.clear(ClearType::UntilNewLine)?;
        let down = self.history.down();
        self.buffer = down.clone();
        self.write(&down)?;
        Ok(())
    }

    pub fn handle_left(&mut self) -> std::io::Result<()> {
        // clear suggestion
        self.clear_suggestion()?;

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
        } else if let Some(racer) = self.racer.take() {
            if let Some(mut suggestion) = racer.current_suggestion() {
                StringTools::strings_unique(&self.buffer, &mut suggestion);
                self.write(&suggestion)?;
                self.buffer.push_str(&suggestion);
                self.racer = Some(racer);
            }
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
            // clear suggestion and unvalidate racer cahce
            self.clear_suggestion()?;
            self.racer_needs_update(true);

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
        #[cfg(unix)]
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
        self.clear_suggestion()?;
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
}
