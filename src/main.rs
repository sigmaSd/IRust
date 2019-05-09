mod cargo_cmds;
mod history;
mod repl;
mod term;
mod utils;

use term::Term;

fn main() {
    let mut term = Term::new();
    term.run().expect("IRust Out");
}
