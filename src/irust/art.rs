use crate::irust::IRust;
use crossterm::{ClearType, Color};

impl IRust {
    pub fn wait_add(&mut self, add_cmd: std::process::Child, msg: &str) -> std::io::Result<()> {
        self.cursor.hide()?;
        self.color.set_fg(Color::DarkGreen)?;

        match self.wait_add_inner(add_cmd, msg) {
            Ok(status) => {
                self.write_newline()?;
                self.cursor.show()?;
                self.color.reset()?;

                if status.success() {
                    Ok(())
                } else {
                    Err(std::io::Error::last_os_error())
                }
            }
            Err(e) => {
                self.write_newline()?;
                self.cursor.show()?;
                self.color.reset()?;
                Err(e)
            }
        }
    }

    fn wait_add_inner(
        &mut self,
        mut add_cmd: std::process::Child,
        msg: &str,
    ) -> std::io::Result<std::process::ExitStatus> {
        loop {
            match add_cmd.try_wait() {
                Ok(None) => {
                    self.write_str_at(&format!(" {}ing dep [\\]", msg), 0, None)?;
                    self.write_str_at(&format!(" {}ing dep [|]", msg), 0, None)?;
                    self.write_str_at(&format!(" {}ing dep [/]", msg), 0, None)?;
                    self.write_str_at(&format!(" {}ing dep [-]", msg), 0, None)?;
                    self.write_str_at(&format!(" {}ing dep [\\]", msg), 0, None)?;
                    self.write_str_at(&format!(" {}ing dep [|]", msg), 0, None)?;
                    self.write_str_at(&format!(" {}ing dep [/]", msg), 0, None)?;
                    self.write_str_at(&format!(" {}ing dep [-]", msg), 0, None)?;
                    continue;
                }
                Err(e) => {
                    return Err(e);
                }
                Ok(Some(status)) => return Ok(status),
            }
        }
    }

    pub fn welcome(&mut self) -> std::io::Result<()> {
        self.terminal.clear(ClearType::All)?;

        self.color.set_fg(Color::Blue)?;
        let slash = std::iter::repeat('-')
            .take(self.terminal.terminal_size().0 as usize / 3)
            .collect::<String>();

        self.terminal
            .write(format!("       {0}Welcome to IRust{0}\n", slash))?;

        self.color.reset()?;

        Ok(())
    }
}
