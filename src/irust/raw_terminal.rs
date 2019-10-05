use super::IRustError;
use crossterm::{ClearType, Color, Terminal, TerminalColor};
use std::fmt::Display;

pub struct RawTerminal {
    terminal: Terminal,
    color: TerminalColor,
}

impl RawTerminal {
    pub fn new() -> Self {
        Self {
            terminal: Terminal::new(),
            color: TerminalColor::new(),
        }
    }

    pub fn scroll_up(&self, n: u16) -> Result<(), IRustError> {
        self.terminal.scroll_up(n)?;
        Ok(())
    }

    pub fn clear(&self, clear_type: ClearType) -> Result<(), IRustError> {
        self.terminal.clear(clear_type)?;
        Ok(())
    }

    pub fn write<D: Display>(&self, value: D) -> Result<(), IRustError> {
        self.terminal.write(value)?;
        Ok(())
    }

    pub fn write_with_color<D: Display>(&self, value: D, color: Color) -> Result<(), IRustError> {
        self.set_fg(color)?;
        self.terminal.write(value)?;
        self.reset_color()?;
        Ok(())
    }

    pub fn size(&self) -> Result<(u16, u16), IRustError> {
        Ok(self.terminal.size()?)
    }

    pub fn reset_color(&self) -> Result<(), IRustError> {
        self.color.reset()?;
        Ok(())
    }

    pub fn set_fg(&self, color: Color) -> Result<(), IRustError> {
        self.color.set_fg(color)?;
        Ok(())
    }

    pub fn set_bg(&self, color: Color) -> Result<(), IRustError> {
        self.color.set_bg(color)?;
        Ok(())
    }

    pub fn exit(status: i32) {
        let _ = crossterm::RawScreen::disable_raw_mode();
        std::process::exit(status);
    }
}
