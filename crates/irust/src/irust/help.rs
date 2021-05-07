use super::highlight::{highlight, theme::Theme};
use crate::irust::{IRust, Result};
use crossterm::style::Color;
use printer::{
    buffer::Buffer,
    printer::{PrintQueue, PrinterItem},
};

impl IRust {
    pub fn help(&mut self) -> Result<PrintQueue> {
        #[cfg(unix)]
        let readme = include_str!("../../../../README.md");
        #[cfg(windows)]
        let readme = include_str!("..\\..\\..\\..\\README.md");

        Ok(parse_markdown(&readme.into(), &self.theme))
    }
}

fn parse_markdown(buffer: &Buffer, theme: &Theme) -> PrintQueue {
    let mut queue = PrintQueue::default();

    let buffer = buffer.to_string();
    let mut buffer = buffer.lines();

    (|| -> Option<()> {
        loop {
            let line = buffer.next()?;

            if line.trim_start().starts_with("##") {
                queue.push(PrinterItem::String(line.to_string(), Color::Yellow));
            } else if line.trim_start().starts_with('#') {
                queue.push(PrinterItem::String(line.to_string(), Color::Red));
            } else if line.trim_start().starts_with("```rust") {
                queue.push(PrinterItem::String(line.to_string(), Color::Cyan));
                // highlight rust code
                queue.add_new_line(1);

                // take_while takes ownership of the iterator
                let mut skipped_lines = 0;

                let code = buffer
                    .clone()
                    .take_while(|line| {
                        skipped_lines += 1;
                        !line.starts_with("```")
                    })
                    .collect::<Vec<&str>>()
                    .join("\n");

                for _ in 0..skipped_lines {
                    let _ = buffer.next();
                }

                queue.append(&mut highlight(&code.into(), theme));
            } else {
                let mut line = line.chars().peekable();

                (|| -> Option<()> {
                    loop {
                        let c = line.next()?;
                        match c {
                            '*' => {
                                let mut star = String::new();
                                star.push('*');

                                let mut pending = None;
                                let mut post_start_count = 0;

                                while line.peek().is_some() {
                                    let c = line.next().unwrap();
                                    if pending.is_none() && c != '*' {
                                        pending = Some(star.len());
                                    }
                                    star.push(c);

                                    if let Some(pending) = pending {
                                        if c == '*' {
                                            post_start_count += 1;
                                            if pending == post_start_count {
                                                break;
                                            }
                                        } else {
                                            post_start_count = post_start_count.saturating_sub(1);
                                        }
                                    }
                                }
                                queue.push(PrinterItem::String(star, Color::Magenta));
                            }
                            '`' => {
                                let mut quoted = String::new();
                                quoted.push('`');

                                while line.peek().is_some() && line.peek() != Some(&'`') {
                                    quoted.push(line.next().unwrap());
                                }
                                //push the closing quote
                                if line.peek().is_some() {
                                    quoted.push(line.next().unwrap());
                                }
                                queue.push(PrinterItem::String(quoted, Color::DarkGreen));
                            }
                            '=' | '>' | '(' | ')' | '-' | '|' => {
                                queue.push(PrinterItem::Char(c, Color::DarkRed))
                            }
                            c => queue.push(PrinterItem::Char(c, Color::White)),
                        }
                    }
                })();
            }
            queue.add_new_line(1);
        }
    })();
    queue
}
