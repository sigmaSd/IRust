use crossterm::{ClearType, Color, Terminal, TerminalColor};
use std::fmt::Display;
use std::io;

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

    pub fn scroll_up(&self, n: i16) -> io::Result<()> {
        self.terminal.scroll_up(n)?;
        Ok(())
    }

    pub fn clear(&self, clear_type: ClearType) -> io::Result<()> {
        self.terminal.clear(clear_type)?;
        Ok(())
    }

    pub fn write<D: Display>(&self, value: D) -> io::Result<()> {
        self.terminal.write(value)?;
        Ok(())
    }

    pub fn terminal_size(&self) -> (u16, u16) {
        self.terminal.terminal_size()
    }

    pub fn reset_color(&self) -> io::Result<()> {
        self.color.reset()?;
        Ok(())
    }

    pub fn set_fg(&self, color: Color) -> io::Result<()> {
        self.color.set_fg(color)?;
        Ok(())
    }

    pub fn set_bg(&self, color: Color) -> io::Result<()> {
        self.color.set_bg(color)?;
        Ok(())
    }

    pub fn exit(&self) {
        let _ = crossterm::RawScreen::disable_raw_mode();
        std::process::exit(0);
    }
}
