use crate::irust::IRust;
use crossterm::ClearType;
use std::error::Error;

impl IRust {
    pub fn handle_character(&mut self, c: char) -> std::io::Result<()> {
        if c == '\n' {
            self.handle_enter()?
        } else {
            self.buffer.insert(self.internal_cursor.x, c);
            self.write_insert(c)?;
        }
        Ok(())
    }

    fn handle_enter(&mut self) -> std::io::Result<()> {
        // create a new line
        self.write_newline()?;

        // parse and handle errors
        match self.parse() {
            Ok(Some(out)) => {
                if self.should_push_to_history(&self.buffer) {
                    self.history.push(self.buffer.drain(..).collect());
                }

                self.output = out;
            }
            Err(e) => {
                self.output = e.description().to_string();
            }
            Ok(None) => {
                self.output.clear();
            }
        }

        // ensure buffer is cleaned
        self.buffer.clear();

        // write out
        self.write_out()?;
        self.write_newline()?;
        self.write_in()?;

        Ok(())
    }

    pub fn handle_up(&mut self) -> std::io::Result<()> {
        self.internal_cursor.x = 0;

        let up = self.history.up();
        self.buffer = up.clone();
        self.write_str_at(&up, 4, None)?;
        Ok(())
    }

    pub fn handle_down(&mut self) -> std::io::Result<()> {
        self.internal_cursor.x = 0;

        let down = self.history.down();
        self.buffer = down.clone();
        self.write_str_at(&down, 4, None)?;
        Ok(())
    }

    pub fn handle_left(&mut self) -> std::io::Result<()> {
        if self.internal_cursor.x > 0 {
            self.cursor.move_left(1);
            self.internal_cursor.move_left();
        }
        Ok(())
    }

    pub fn handle_right(&mut self) -> std::io::Result<()> {
        if self.internal_cursor.x < self.buffer.len() {
            self.cursor.move_right(1);
            self.internal_cursor.move_right();
        }
        Ok(())
    }

    pub fn handle_backspace(&mut self) -> std::io::Result<()> {
        if self.internal_cursor.x > 0 {
            self.cursor.move_left(1);
            self.internal_cursor.move_left();
            if !self.buffer.is_empty() {
                self.buffer.remove(self.internal_cursor.x);
            }
            self.backspace()?;
        }
        Ok(())
    }

    pub fn exit(&mut self) -> std::io::Result<()> {
        if self.buffer.is_empty() {
            self.terminal.clear(ClearType::All)?;
            self.terminal.exit();
        }

        Ok(())
    }

    pub fn stop(&mut self) -> std::io::Result<()> {
        #[cfg(target_family = "unix")]
        {
            self.terminal.clear(ClearType::All)?;
            let _ = nix::sys::signal::kill(
                nix::unistd::Pid::this(),
                Some(nix::sys::signal::Signal::SIGTSTP),
            );
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
        self.internal_cursor.x = self.buffer.len();
        self.move_cursor_to(self.buffer.len() + 4, None)?;
        Ok(())
    }
}
