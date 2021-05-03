use std::io::stdout;

use crossterm::{
    cursor::{Hide, MoveTo, RestorePosition, SavePosition, Show},
    execute,
    style::Print,
};

fn main() {
    let stdin = std::io::stdin();
    let handle = stdin.lock();

    let globals: irust_api::GlobalVariables = bincode::deserialize_from(handle).unwrap();

    let mut tick = 0;
    const STATUS: &[&str] = &["-", "/", "-", "\\"];
    loop {
        let msg = format!("In [{}]: ", STATUS[tick % STATUS.len()]);
        let _ = execute!(
            stdout(),
            SavePosition,
            MoveTo(
                globals.prompt_position.0 as u16,
                globals.prompt_position.1 as u16,
            ),
            Hide,
            Print(msg),
            Show,
            RestorePosition
        );
        tick += 1;
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}
