use rscript::{
    scripting::{DynamicScript, FFiData, FFiStr},
    Hook, ScriptInfo, ScriptType, VersionReq,
};
mod script;

#[no_mangle]
pub static SCRIPT: DynamicScript = DynamicScript {
    script,
    script_info,
};

static mut VIM: Vim = Vim::new();

extern "C" fn script_info() -> FFiData {
    let info = ScriptInfo::new(
        "VimDylib",
        ScriptType::DynamicLib,
        &[
            irust_api::InputEvent::NAME,
            irust_api::Shutdown::NAME,
            irust_api::Startup::NAME,
        ],
        VersionReq::parse(">=1.50.0").expect("correct version requirement"),
    );
    info.into_ffi_data()
}

extern "C" fn script(name: FFiStr, hook: FFiData) -> FFiData {
    match name.as_str() {
        irust_api::InputEvent::NAME => {
            let hook: irust_api::InputEvent = DynamicScript::read(hook);
            let output = unsafe { VIM.handle_input_event(hook) };
            DynamicScript::write::<irust_api::InputEvent>(&output)
        }
        irust_api::Shutdown::NAME => {
            let hook: irust_api::Shutdown = DynamicScript::read(hook);
            let output = unsafe { VIM.clean_up(hook) };
            DynamicScript::write::<irust_api::Shutdown>(&output)
        }
        irust_api::Startup::NAME => {
            let hook: irust_api::Startup = DynamicScript::read(hook);
            let output = unsafe { VIM.start_up(hook) };
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
