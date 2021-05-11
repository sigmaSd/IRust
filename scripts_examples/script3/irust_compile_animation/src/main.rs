use std::io::{stdout, Write};

use irust_api::{GlobalVariables, Hook, Message, ScriptInfo};

use crossterm::{
    cursor::{Hide, MoveTo, MoveToColumn, RestorePosition, SavePosition, Show},
    execute,
    style::Print,
};

fn main() {
    let stdin = std::io::stdin();
    let mut handle = stdin.lock();

    let message: Message = bincode::deserialize_from(&mut handle).unwrap();
    if message == Message::Greeting {
        let script_info = ScriptInfo {
            name: "Compile Animation".to_string(),
            hooks: vec![Hook::WhileCompiling],
            path: std::env::current_exe().unwrap(),
            is_daemon: false,
        };
        bincode::serialize_into(std::io::stdout(), &script_info).unwrap();
        std::io::stdout().flush().unwrap();
        return;
    } /*else {
          // message is Hook
          continue
      }*/

    let hook: Hook = bincode::deserialize_from(&mut handle).unwrap();

    match hook {
        Hook::WhileCompiling => {
            let globals: GlobalVariables = bincode::deserialize_from(&mut handle).unwrap();
            let mut tick = 0;
            const STATUS: &[&str] = &["-", "/", "-", "\\"];
            loop {
                let msg = format!("In [{}]: ", STATUS[tick % STATUS.len()]);
                execute!(
                    stdout(),
                    SavePosition,
                    Hide,
                    MoveTo(
                        globals.prompt_position.0 as u16,
                        globals.prompt_position.1 as u16
                    ),
                    Print(" ".repeat(globals.prompt_len)),
                    MoveToColumn(0),
                    Print(msg),
                    Show,
                    RestorePosition
                )
                .unwrap();

                tick += 1;
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        }
        _ => unreachable!(),
    }
}
