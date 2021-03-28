#![cfg(feature = "ui-test")]
use crossterm::style::Color;
use irust::irust::highlight::highlight;
use irust::irust::printer::cursor::bound::BoundType;
use irust::irust::printer::Printer;
use irust::irust::{buffer::Buffer, highlight::theme::Theme};
use std::collections::HashMap;
use std::io::Write;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
#[test]
fn write_from_terminal_start_cursor_pos_correct() -> Result<()> {
    let mut p = Printer::new(std::io::sink());

    let origin_pos = p.cursor.pos;
    p.write_from_terminal_start("hello", Color::Red)?;
    assert_eq!(p.cursor.pos.current_pos.0, 5);
    assert_eq!(p.cursor.pos.current_pos.1, origin_pos.current_pos.1);

    Ok(())
}

#[test]
fn writenew_line_no_scroll() -> Result<()> {
    let mut p = Printer::new(std::io::sink());

    let b = Buffer::from_string("Hello world");

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
fn writenew_line_with_scroll() -> Result<()> {
    let mut p = Printer::new(std::io::sink());
    let b = Buffer::from_string("Hello world");

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
    let mut p = Printer::new(std::io::sink());

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
    let mut p = Printer::new(std::io::sink());
    let b = Buffer::from_string("\n\n\n");

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
    let mut p = Printer::new(std::io::sink());
    let b = Buffer::from_string("\n\n\n");

    p.cursor.pos.starting_pos.0 = 0;
    p.cursor.pos.starting_pos.1 = 0;
    p.cursor.goto_start();

    let original_pos = p.cursor.pos;
    p.scroll_if_needed_for_input(&b);

    assert_eq!(original_pos.starting_pos.1, p.cursor.pos.starting_pos.1);
    Ok(())
}

#[test]
fn calculate_bounds_correctly() -> Result<()> {
    const INPUT_START: usize = 4;
    let mut p = Printer::new(std::io::sink());
    let width = p.cursor.bound.width;
    let height = p.cursor.bound.height;
    let queue = highlight(
        &"alloc\nprint".chars().collect::<Vec<_>>(),
        &Theme::default(),
    );

    // 1
    move_to_and_modify_start(&mut p, 0, 0);
    p.recalculate_bounds(queue.clone())?;

    let expected_bound: HashMap<usize, BoundType> = vec![
        (0, BoundType::Bounded(INPUT_START + 5)),
        (1, BoundType::Bounded(INPUT_START + 5)),
    ]
    .into_iter()
    .collect();
    assert_eq!(expected_bound, p.cursor.bound.bound);
    Ok(())
}

#[test]
pub fn calculate_bounds_correctly2() -> Result<()> {
    const INPUT_START: usize = 4;
    let mut p = Printer::new(std::io::sink());
    let width = p.cursor.bound.width;
    let height = p.cursor.bound.height;
    let queue = highlight(&"A\t\nBC\n".chars().collect::<Vec<_>>(), &Theme::default());
    // 2
    move_to_and_modify_start(&mut p, 0, height - 5);
    p.recalculate_bounds(queue)?;

    let expected_bound: HashMap<usize, BoundType> = vec![
        (height - 5, BoundType::Bounded(INPUT_START + 2)),
        (height - 4, BoundType::Bounded(INPUT_START + 2)),
        (height - 3, BoundType::Bounded(INPUT_START)),
    ]
    .into_iter()
    .collect();
    assert_eq!(expected_bound, p.cursor.bound.bound);

    Ok(())
}

// helper
fn move_to_and_modify_start(printer: &mut Printer<impl Write>, x: usize, y: usize) {
    printer.cursor.pos.starting_pos.0 = x;
    printer.cursor.pos.starting_pos.1 = y;
    printer.cursor.goto_start();
}
