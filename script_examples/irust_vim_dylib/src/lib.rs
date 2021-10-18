use rscript::{FFiVec, Hook, ScriptInfo, ScriptType, VersionReq};
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
pub extern "C" fn script_info() -> FFiVec {
    let info = ScriptInfo::new(
        "VimDylib",
        ScriptType::DynamicLib,
        &[
            irust_api::InputEvent::NAME,
            irust_api::Shutdown::NAME,
            irust_api::Startup::NAME,
        ],
        VersionReq::parse(">=1.30.0").expect("correct version requirement"),
    );
    FFiVec::serialize_from(&info).unwrap()
}

#[no_mangle]
pub extern "C" fn script(hook: FFiVec, data: FFiVec) -> FFiVec {
    let hook: String = hook.deserialize().unwrap();
    match hook.as_str() {
        irust_api::InputEvent::NAME => {
            let hook: irust_api::InputEvent = data.deserialize().unwrap();
            let output: <irust_api::InputEvent as Hook>::Output =
                unsafe { VIM.handle_input_event(hook) };
            FFiVec::serialize_from(&output).unwrap()
        }
        irust_api::Shutdown::NAME => {
            let hook: irust_api::Shutdown = data.deserialize().unwrap();
            let output: <irust_api::Shutdown as Hook>::Output = unsafe { VIM.clean_up(hook) };
            FFiVec::serialize_from(&output).unwrap()
        }
        irust_api::Startup::NAME => {
            let hook: irust_api::Startup = data.deserialize().unwrap();
            let output: <irust_api::Startup as Hook>::Output = unsafe { VIM.start_up(hook) };
            FFiVec::serialize_from(&output).unwrap()
        }
        _ => unreachable!(),
    }
}
