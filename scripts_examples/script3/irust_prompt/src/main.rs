use irust_api::{GlobalVariables, Hook, Message, ScriptInfo};
use std::io::Write;

fn main() {
    let stdin = std::io::stdin();
    let mut handle = stdin.lock();

    let message: Message = bincode::deserialize_from(&mut handle).unwrap();

    if message == Message::Greeting {
        let script_info = ScriptInfo {
            hooks: vec![Hook::SetInputPrompt, Hook::SetOutputPrompt],
            path: std::env::current_exe().unwrap(),
            is_daemon: false,
        };
        bincode::serialize_into(std::io::stdout(), &script_info).unwrap();
        std::io::stdout().flush().unwrap();
        return;
    }

    let hook: Hook = bincode::deserialize_from(&mut handle).unwrap();
    let globals: GlobalVariables = bincode::deserialize_from(&mut handle).unwrap();

    match hook {
        Hook::SetInputPrompt => {
            let prompt = format!("In [{}]: ", globals.operation_number);
            bincode::serialize_into(std::io::stdout(), &prompt).unwrap();
            std::io::stdout().flush().unwrap();
        }
        Hook::SetOutputPrompt => {
            let prompt = format!("Out [{}]: ", globals.operation_number);
            bincode::serialize_into(std::io::stdout(), &prompt).unwrap();
            std::io::stdout().flush().unwrap();
        }
        _ => unreachable!(),
    }
}
