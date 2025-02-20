#![allow(clippy::let_unit_value)]
use std::{sync::atomic::AtomicBool, thread};

use crossterm::{
    cursor::{Hide, MoveTo, MoveToColumn, RestorePosition, SavePosition, Show},
    style::Print,
};
use rscript::{
    Hook, ScriptInfo, ScriptType, VersionReq,
    scripting::{DynamicScript, FFiData, FFiStr},
};

#[unsafe(no_mangle)]
pub static SCRIPT: DynamicScript = DynamicScript {
    script,
    script_info,
};

static ANIMATION_FLAG: AtomicBool = AtomicBool::new(false);

extern "C" fn script_info() -> FFiData {
    let info = ScriptInfo::new(
        "Animation",
        ScriptType::DynamicLib,
        &[
            irust_api::BeforeCompiling::NAME,
            irust_api::AfterCompiling::NAME,
        ],
        VersionReq::parse(">=1.50.0").expect("correct version requirement"),
    );
    info.into_ffi_data()
}

extern "C" fn script(name: FFiStr, hook: FFiData) -> FFiData {
    match name.as_str() {
        irust_api::BeforeCompiling::NAME => {
            let hook: irust_api::BeforeCompiling = DynamicScript::read(hook);
            let output = start(hook);
            DynamicScript::write::<irust_api::BeforeCompiling>(&output)
        }
        irust_api::AfterCompiling::NAME => {
            let hook: irust_api::AfterCompiling = DynamicScript::read(hook);
            let output = end(hook);
            DynamicScript::write::<irust_api::AfterCompiling>(&output)
        }
        _ => unreachable!(),
    }
}

fn start(hook: irust_api::BeforeCompiling) {
    thread::spawn(move || {
        ANIMATION_FLAG.store(true, std::sync::atomic::Ordering::Relaxed);
        use crossterm::execute;
        use std::io::stdout;
        let globals = hook.0;
        let mut tick = 0;
        const STATUS: &[&str] = &["-", "/", "-", "\\"];

        while ANIMATION_FLAG.load(std::sync::atomic::Ordering::Relaxed) {
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
    });
}

fn end(_hook: irust_api::AfterCompiling) {
    ANIMATION_FLAG.store(false, std::sync::atomic::Ordering::Relaxed);
}
