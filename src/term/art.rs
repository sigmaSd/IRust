use crate::term::Term;
use crossterm::Color;

impl Term {
    pub fn wait_add(&mut self, mut add_cmd: std::process::Child, msg: &str) -> std::io::Result<()> {
        let _ = self.cursor.hide();
        let _ = self.color.set_fg(Color::DarkGreen);

        loop {
            match add_cmd.try_wait() {
                Ok(None) => {
                    let _ = self.write_str_at(&format!(" {}ing dep [\\]", msg), 0, None);
                    let _ = self.write_str_at(&format!(" {}ing dep [|]", msg), 0, None);
                    let _ = self.write_str_at(&format!(" {}ing dep [/]", msg), 0, None);
                    let _ = self.write_str_at(&format!(" {}ing dep [-]", msg), 0, None);
                    let _ = self.write_str_at(&format!(" {}ing dep [\\]", msg), 0, None);
                    let _ = self.write_str_at(&format!(" {}ing dep [|]", msg), 0, None);
                    let _ = self.write_str_at(&format!(" {}ing dep [/]", msg), 0, None);
                    let _ = self.write_str_at(&format!(" {}ing dep [-]", msg), 0, None);
                    continue;
                }
                Err(e) => {
                    let _ = self.write_newline();
                    let _ = self.cursor.show();
                    let _ = self.color.reset();
                    return Err(e);
                }
                Ok(Some(status)) => {
                    let _ = self.write_newline();
                    let _ = self.cursor.show();
                    let _ = self.color.reset();
                    if status.success() {
                        return Ok(());
                    } else {
                        return Err(std::io::Error::last_os_error());
                    }
                }
            }
        }
    }
}
