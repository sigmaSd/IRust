use irust_api::script4;
use rscript::{Hook, ScriptInfo, ScriptType, VersionReq};
mod script;

struct Vim {
    state: State,
    mode: Mode,
}

#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq)]
enum State {
    Empty,
    c,
    ci,
    d,
    di,
    g,
    f,
    F,
    r,
}

#[derive(PartialEq)]
enum Mode {
    Normal,
    Insert,
}

static mut VIM: Vim = Vim::new();

#[no_mangle]
pub fn script_info() -> ScriptInfo {
    ScriptInfo::new(
        "VimDylib",
        ScriptType::DynamicLib,
        &[
            script4::InputEvent::NAME,
            script4::Shutdown::NAME,
            script4::Startup::NAME,
        ],
        VersionReq::parse(">=1.19.0").expect("correct version requirement"),
    )
}

/// # Safety
/// No stable ABI => Not safe
#[no_mangle]
pub unsafe fn script(hook: &str, data: Vec<u8>) -> Vec<u8> {
    match hook {
        script4::InputEvent::NAME => {
            let hook: script4::InputEvent = bincode::deserialize(&data).unwrap();
            let output = VIM.handle_input_event(hook);
            bincode::serialize(&output).unwrap()
        }
        script4::Shutdown::NAME => {
            let hook: script4::Shutdown = bincode::deserialize(&data).unwrap();
            let output = VIM.clean_up(hook);
            bincode::serialize(&output).unwrap()
        }
        script4::Startup::NAME => {
            let hook: script4::Startup = bincode::deserialize(&data).unwrap();
            let output = VIM.start_up(hook);
            bincode::serialize(&output).unwrap()
        }
        _ => unreachable!(),
    }
}
