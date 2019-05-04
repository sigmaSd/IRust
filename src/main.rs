mod cargo_cmds;
mod repl;
mod term;

use term::Term;

fn main() {
    let mut term = Term::new();
    term.run().expect("Error while starting IRust");
}
