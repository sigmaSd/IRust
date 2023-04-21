use irust_repl::Repl;
use serde::{Deserialize, Serialize};
use serde_json::Deserializer;
use std::io;

#[derive(Serialize, Deserialize)]
struct Message {
    code: String,
}
#[derive(Serialize, Deserialize)]
struct Response {
    result: String,
    inserted: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let stdin = io::stdin();
    let reader = stdin.lock();
    let mut deserializer = Deserializer::from_reader(reader).into_iter::<Message>();

    let mut repl = Repl::default();

    while let Some(json) = deserializer.next() {
        let message = json?;
        if message.code.ends_with(";") {
            repl.insert(&message.code);
            let output = serde_json::to_string(&Response {
                result: "".to_string(),
                inserted: true,
            })?;
            println!("{output}");
        } else {
            let result = repl.eval(message.code)?;
            let output = serde_json::to_string(&Response {
                result: result.output,
                inserted: false,
            })?;
            println!("{output}");
        }
    }

    Ok(())
}

