use crossterm::{
    event::{KeyCode, KeyEvent, KeyModifiers},
    style::Color,
};
use printer::{
    buffer::Buffer,
    printer::{default_process_fn, PrintQueue, Printer, PrinterItem},
    Result,
};

fn main() -> Result<()> {
    let mut printer = Printer::new(std::io::stdout(), "In: ".into());
    printer.print_prompt_if_set()?;
    std::io::Write::flush(&mut printer.writer.raw)?;

    let mut buffer = Buffer::new();

    loop {
        let inp = crossterm::event::read()?;
        match inp {
            crossterm::event::Event::Key(key) => match key {
                KeyEvent {
                    code: KeyCode::Char(c),
                    modifiers: KeyModifiers::NONE,
                    ..
                } => {
                    buffer.insert(c);
                    printer.print_input(&default_process_fn, &buffer)?;
                    printer.cursor.move_right_unbounded();
                }
                KeyEvent {
                    code: KeyCode::Backspace,
                    ..
                } => {
                    if !buffer.is_at_start() {
                        buffer.move_backward();
                        printer.cursor.move_left();
                        buffer.remove_current_char();
                        printer.print_input(&default_process_fn, &buffer)?;
                    }
                }
                KeyEvent {
                    code: KeyCode::Enter,
                    ..
                } => {
                    if let Some(mut output) = eval(buffer.to_string()) {
                        output.push_front(PrinterItem::NewLine);

                        printer.print_output(output)?;
                    }
                    buffer.clear();
                    printer.print_prompt_if_set()?;
                }
                KeyEvent {
                    code: KeyCode::Char('c'),
                    modifiers: KeyModifiers::CONTROL,
                    ..
                } => break,
                _ => (),
            },
            _ => (),
        }
        std::io::Write::flush(&mut printer.writer.raw)?;
    }
    Ok(())
}

fn eval(buffer: String) -> Option<PrintQueue> {
    let mut buffer = buffer.split_whitespace();
    let cmd = buffer.next()?;
    let args: Vec<&str> = buffer.collect();

    match (|| -> Result<PrinterItem> {
        let output = std::process::Command::new(cmd).args(args).output()?;
        if output.status.success() {
            Ok(PrinterItem::String(
                String::from_utf8(output.stdout)?,
                Color::Blue,
            ))
        } else {
            Ok(PrinterItem::String(
                String::from_utf8(output.stderr)?,
                Color::Red,
            ))
        }
    })() {
        Ok(result) => Some(result.into()),
        Err(e) => Some(PrinterItem::String(e.to_string(), Color::Red).into()),
    }
}
