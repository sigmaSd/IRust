#![feature(test)]
extern crate test;
use test::Bencher;

use printer::printer::{default_process_fn, Printer};

// last run 45ns/iter
#[bench]
fn bench_print_input(b: &mut Bencher) {
    let buffer = r#"\
        fn default() -> Self {
        crossterm::terminal::enable_raw_mode().expect("failed to enable raw_mode");
        let raw = Rc::new(RefCell::new(std::io::stdout()));
        Self {
            printer: Default::default(),
            writer: writer::Writer::new(raw.clone()),
            cursor: cursor::Cursor::new(raw),
        }
        "#
    .into();

    let mut printer = Printer::new(std::io::sink(), "".to_string());
    b.iter(|| printer.print_input(&default_process_fn, &buffer));
}
