use crate::Result;
use std::process::{Child, Output};

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
        while self.try_wait()?.is_none() {
            if let Some(ref function) = function {
                function(&mut self)?;
            }
        }
        self.wait_with_output().map_err(Into::into)
    }
}
