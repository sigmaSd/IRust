use crate::cargo_cmds::CargoCmds;
use std::io;

#[derive(Clone)]
pub struct Repl {
    pub body: Vec<String>,
    cursor: usize,
    cargo_cmds: CargoCmds,
}

impl Repl {
    pub fn new() -> Self {
        Self {
            body: vec!["fn main() {\n".to_string(), "}".to_string()],
            cursor: 1,
            cargo_cmds: Default::default(),
        }
    }

    pub fn insert(&mut self, mut input: String) {
        input.insert(0, '\t');
        input.push('\n');
        self.body.insert(self.cursor, input);
        self.cursor += 1;
    }

    pub fn reset(&mut self) {
        self.prepare_ground().expect("Error while resetting Repl");
        *self = Self::new();
    }

    pub fn show(&self) -> String {
        format!("Current Repl Code:\n{}", self.body.join(""))
    }

    // prepare ground
    pub fn prepare_ground(&self) -> Result<(), io::Error> {
        self.cargo_cmds.cargo_new()?;
        Ok(())
    }

    pub fn eval(&self, input: String) -> io::Result<String> {
        let eval_statement = format!("println!(\"{{:?}}\", {{\n{}\n}});", input);
        let mut repl = self.clone();
        repl.insert(eval_statement);

        let code = repl.body.join("");

        Ok(self.cargo_cmds.cargo_run(code)?)
    }

    pub fn add_dep(&self, dep: &[String]) -> std::io::Result<std::process::Child> {
        Ok(self.cargo_cmds.cargo_add(dep)?)
    }

    pub fn build(&self) -> std::io::Result<std::process::Child> {
        self.cargo_cmds.cargo_build()
    }
}
