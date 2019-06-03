use crate::irust::printer::{Printer, PrinterItem, PrinterItemType};
use crate::irust::IRust;
use crossterm::{ClearType, Color};

impl IRust {
    pub fn wait_add(&mut self, add_cmd: std::process::Child, msg: &str) -> std::io::Result<()> {
        self.cursor.hide()?;
        self.color.set_fg(Color::Cyan)?;

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
        self.write_str_at(&format!(" {}ing dep [\\]", msg), 0, None)?;
        loop {
            match add_cmd.try_wait() {
                Ok(None) => {
                    self.write_str_at("\\", msg.len() + 10, None)?;
                    self.write_str_at("|", msg.len() + 10, None)?;
                    self.write_str_at("/", msg.len() + 10, None)?;
                    self.write_str_at("-", msg.len() + 10, None)?;
                    self.write_str_at("\\", msg.len() + 10, None)?;
                    self.write_str_at("|", msg.len() + 10, None)?;
                    self.write_str_at("/", msg.len() + 10, None)?;
                    self.write_str_at("-", msg.len() + 10, None)?;
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

        let default_msg = "Welcome to IRust".to_string();
        self.printer = Printer::new(PrinterItem::new(default_msg, PrinterItemType::Welcome));
        self.printer.add_new_line(2);

        self.write_out()?;

        Ok(())
    }

    pub fn fit_msg(&mut self, msg: &str) -> String {
        let slash_num = self.terminal.terminal_size().0 as usize - msg.len();
        let slash = std::iter::repeat('-')
            .take(slash_num / 2)
            .collect::<String>();

        format!("{0}{1}{0}", slash, msg)
    }
}
