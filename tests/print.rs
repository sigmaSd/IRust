use crossterm::style::Color;
use irust::irust::buffer::Buffer;
use irust::irust::printer::Printer;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
#[test]
fn write_from_terminal_start_cursor_pos_correct() -> Result<()> {
    let mut p = Printer::_new(std::io::sink());

    let origin_pos = p.cursor.pos;
    p.write_from_terminal_start("hello", Color::Red)?;
    assert_eq!(p.cursor.pos.current_pos.0, 5);
    assert_eq!(p.cursor.pos.current_pos.1, origin_pos.current_pos.1);

    Ok(())
}

#[test]
fn write_new_line_no_scroll() -> Result<()> {
    let mut p = Printer::_new(std::io::sink());

    let b = Buffer::from_str("Hello world");

    p.cursor.pos.starting_pos.0 = 0;
    p.cursor.pos.starting_pos.1 = 0;
    p.cursor.goto_start();
    assert_eq!(p.cursor.pos.current_pos, p.cursor.pos.starting_pos);

    let origin_pos = p.cursor.pos;
    p.write_newline(&b)?;

    assert_eq!(origin_pos.starting_pos.1 + 1, p.cursor.pos.starting_pos.1);
    assert_eq!(origin_pos.current_pos.1 + 1, p.cursor.pos.current_pos.1);

    Ok(())
}

#[test]
fn write_new_line_with_scroll() -> Result<()> {
    let mut p = Printer::_new(std::io::sink());
    let b = Buffer::from_str("Hello world");

    p.cursor.pos.starting_pos.0 = 0;
    p.cursor.pos.starting_pos.1 = p.cursor.bound.height - 1;
    p.cursor.goto_start();

    assert_eq!(p.cursor.pos.current_pos, p.cursor.pos.starting_pos);

    let origin_pos = p.cursor.pos;
    p.write_newline(&b)?;

    assert_eq!(origin_pos.starting_pos.1, p.cursor.pos.starting_pos.1);
    assert_eq!(origin_pos.current_pos.1, p.cursor.pos.current_pos.1);

    Ok(())
}

#[test]
fn scroll_up() -> Result<()> {
    let mut p = Printer::_new(std::io::sink());

    let origin_pos = p.cursor.pos;
    p.scroll_up(3);

    assert_eq!(
        origin_pos.starting_pos.1.saturating_sub(3),
        p.cursor.pos.starting_pos.1
    );
    assert_eq!(
        origin_pos.current_pos.1.saturating_sub(3),
        p.cursor.pos.current_pos.1
    );

    Ok(())
}

#[test]
fn scroll_because_input_needs_scroll() -> Result<()> {
    let mut p = Printer::_new(std::io::sink());
    let b = Buffer::from_str("\n\n\n");

    p.cursor.pos.starting_pos.0 = 0;
    p.cursor.pos.starting_pos.1 = p.cursor.bound.height - 1;
    p.cursor.goto_start();

    let original_pos = p.cursor.pos;
    p.scroll_if_needed_for_input(&b);

    assert_eq!(original_pos.starting_pos.1 - 3, p.cursor.pos.starting_pos.1);
    Ok(())
}

#[test]
fn dont_scroll_because_input_doesent_need_scroll() -> Result<()> {
    let mut p = Printer::_new(std::io::sink());
    let b = Buffer::from_str("\n\n\n");

    p.cursor.pos.starting_pos.0 = 0;
    p.cursor.pos.starting_pos.1 = 0;
    p.cursor.goto_start();

    let original_pos = p.cursor.pos;
    p.scroll_if_needed_for_input(&b);

    assert_eq!(original_pos.starting_pos.1, p.cursor.pos.starting_pos.1);
    Ok(())
}
