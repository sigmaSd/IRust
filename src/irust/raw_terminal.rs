use super::IRustError;
use crossterm::{cursor::*, execute, screen::RawScreen, style::*, terminal::*, Output};
use std::fmt::Display;
use std::io::{stdout, Write};

pub struct RawTerminal {}

impl RawTerminal {
    pub fn new() -> Self {
        Self {}
    }

    pub fn scroll_up(&self, n: u16) -> Result<(), IRustError> {
        execute!(stdout(), ScrollUp(n))?;
        Ok(())
    }

    pub fn clear(&self, clear_type: ClearType) -> Result<(), IRustError> {
        execute!(stdout(), Clear(clear_type))?;
        Ok(())
    }

    pub fn write<D: Display + Clone>(&self, value: D) -> Result<(), IRustError> {
        execute!(stdout(), Output(value))?;
        Ok(())
    }

    pub fn write_with_color<D: Display + Clone>(
        &self,
        value: D,
        color: Color,
    ) -> Result<(), IRustError> {
        self.set_fg(color)?;
        self.write(value)?;
        self.reset_color()?;
        Ok(())
    }

    pub fn size(&self) -> Result<(u16, u16), IRustError> {
        Ok(size()?)
    }

    pub fn reset_color(&self) -> Result<(), IRustError> {
        execute!(stdout(), ResetColor)?;
        Ok(())
    }

    pub fn set_fg(&self, color: Color) -> Result<(), IRustError> {
        execute!(stdout(), SetForegroundColor(color))?;
        Ok(())
    }

    pub fn set_bg(&self, color: Color) -> Result<(), IRustError> {
        execute!(stdout(), SetBackgroundColor(color))?;
        Ok(())
    }

    pub fn exit(status: i32) {
        let _ = RawScreen::disable_raw_mode();
        std::process::exit(status);
    }
}

pub struct RawCursor {}

impl RawCursor {
    pub fn new() -> RawCursor {
        Self {}
    }
    pub fn restore_position(&self) -> Result<(), IRustError> {
        execute!(stdout(), RestorePosition)?;
        Ok(())
    }

    pub fn save_position(&self) -> Result<(), IRustError> {
        execute!(stdout(), SavePosition)?;
        Ok(())
    }

    pub fn move_down(&self, n: u16) -> Result<(), IRustError> {
        execute!(stdout(), MoveDown(n))?;
        Ok(())
    }

    pub fn move_up(&self, n: u16) -> Result<(), IRustError> {
        execute!(stdout(), MoveUp(n))?;
        Ok(())
    }

    pub fn show(&self) -> Result<(), IRustError> {
        execute!(stdout(), Show)?;
        Ok(())
    }

    pub fn hide(&self) -> Result<(), IRustError> {
        execute!(stdout(), Hide)?;
        Ok(())
    }

    pub fn goto(&self, x: u16, y: u16) -> Result<(), IRustError> {
        execute!(stdout(), MoveTo(x, y))?;
        Ok(())
    }
}
