use std::{
    io::Read,
    process::{Child, Output},
    sync::mpsc,
};

use crate::Result;

pub fn stdout_and_stderr(out: Output) -> String {
    let out = if !out.stdout.is_empty() {
        out.stdout
    } else {
        out.stderr
    };

    String::from_utf8(out).unwrap_or_default()
}

pub trait ProcessUtils {
    fn interactive_output(self, function: Option<fn(&mut Child) -> Result<()>>) -> Result<Output>;
}

impl ProcessUtils for Child {
    fn interactive_output(
        mut self,
        function: Option<fn(&mut Child) -> Result<()>>,
    ) -> Result<Output> {
        let mut stdout = self.stdout.take().expect("stdout is piped");
        let mut stderr = self.stderr.take().expect("stderr is piped");

        let (tx_out, rx) = mpsc::channel();
        let tx_err = tx_out.clone();
        enum OutType {
            Stdout(Vec<u8>),
            Stderr(Vec<u8>),
        }

        std::thread::spawn(move || {
            let mut out = Vec::new();
            let _ = stdout.read_to_end(&mut out);
            let _ = tx_out.send(OutType::Stdout(out));
        });

        std::thread::spawn(move || {
            let mut err = Vec::new();
            let _ = stderr.read_to_end(&mut err);
            let _ = tx_err.send(OutType::Stderr(err));
        });

        while self.try_wait()?.is_none() {
            if let Some(ref function) = function {
                function(&mut self)?;
            }
        }
        let mut stdout = None;
        let mut stderr = None;
        for _ in 0..2 {
            match rx.recv()? {
                OutType::Stdout(out) => stdout = Some(out),
                OutType::Stderr(err) => stderr = Some(err),
            }
        }

        Ok(Output {
            status: self.wait()?,
            stdout: stdout.unwrap(),
            stderr: stderr.unwrap(),
        })
    }
}

pub fn is_allowed_in_lib(s: &str) -> bool {
    match s.split_whitespace().collect::<Vec<_>>().as_slice() {
        // async fn|const fn|unsafe fn
        [_, "fn", ..]
        | ["fn", ..]
        | [_, "use", ..]
        | ["use", ..]
        | ["enum", ..]
        | ["struct", ..]
        | ["trait", ..]
        | ["impl", ..]
        | ["pub", ..]
        | ["extern", ..]
        | ["macro", ..] => true,
        ["macro_rules!", ..] => true,
        // attribute exp:
        // #[derive(Debug)]
        // struct B{}
        [tag, ..] if tag.starts_with('#') => true,
        _ => false,
    }
}

pub fn remove_semi_col_if_exists(mut s: String) -> String {
    if !s.ends_with(';') {
        return s;
    }
    s.pop();
    s
}

pub fn is_use_stmt(l: &str) -> bool {
    let l = l.trim_start();
    l.starts_with("use") || l.starts_with("#[allow(unused_imports)]use")
}
