use crate::term::Term;
use crossterm::Color;

impl Term {
    pub fn wait_add(&mut self, add_cmd: Vec<std::process::Child>) -> std::io::Result<()> {
        let _ = self.cursor.hide();
        let _ = self.color.set_fg(Color::DarkGreen);

        for mut child in add_cmd {
            loop {
                match child.try_wait() {
                    Ok(None) => {
                        let _ = self.write_str_at(" Adding dep [\\]", Some(0), None);
                        let _ = self.write_str_at(" Adding dep [|]", Some(0), None);
                        let _ = self.write_str_at(" Adding dep [/]", Some(0), None);
                        let _ = self.write_str_at(" Adding dep [-]", Some(0), None);
                        let _ = self.write_str_at(" Adding dep [\\]", Some(0), None);
                        let _ = self.write_str_at(" Adding dep [|]", Some(0), None);
                        let _ = self.write_str_at(" Adding dep [/]", Some(0), None);
                        let _ = self.write_str_at(" Adding dep [-]", Some(0), None);
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
        Ok(())
    }
}
