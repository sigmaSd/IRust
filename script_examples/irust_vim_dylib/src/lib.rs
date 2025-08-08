use rscript::{
    Hook, ScriptInfo, ScriptType, VersionReq,
    scripting::{DynamicScript, FFiData, FFiStr},
};
use std::sync::{LazyLock, Mutex};
mod script;

#[unsafe(no_mangle)]
pub static SCRIPT: DynamicScript = DynamicScript {
    script,
    script_info,
};

static VIM: LazyLock<Mutex<Vim>> = LazyLock::new(|| Mutex::new(Vim::new()));
fn vim() -> std::sync::MutexGuard<'static, Vim> {
    VIM.lock().unwrap()
}

extern "C" fn script_info() -> FFiData {
    let info = ScriptInfo::new(
        "VimDylib",
        ScriptType::DynamicLib,
        &[
            irust_api::InputEvent::NAME,
            irust_api::Shutdown::NAME,
            irust_api::Startup::NAME,
        ],
        VersionReq::parse(">=1.67.3").expect("correct version requirement"),
    );
    info.into_ffi_data()
}

extern "C" fn script(name: FFiStr, hook: FFiData) -> FFiData {
    match name.as_str() {
        irust_api::InputEvent::NAME => {
            let hook: irust_api::InputEvent = DynamicScript::read(hook);
            let output = vim().handle_input_event(hook);
            DynamicScript::write::<irust_api::InputEvent>(&output)
        }
        irust_api::Shutdown::NAME => {
            let hook: irust_api::Shutdown = DynamicScript::read(hook);
            let output = vim().clean_up(hook);
            DynamicScript::write::<irust_api::Shutdown>(&output)
        }
        irust_api::Startup::NAME => {
            let hook: irust_api::Startup = DynamicScript::read(hook);
            let output = vim().start_up(hook);
            DynamicScript::write::<irust_api::Startup>(&output)
        }
        _ => unreachable!(),
    }
}

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
