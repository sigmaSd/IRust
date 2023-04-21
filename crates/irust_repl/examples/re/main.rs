use irust_repl::Repl;
use serde::{Deserialize, Serialize};
use serde_json::Deserializer;
use std::io;

#[derive(Serialize, Deserialize)]
struct Message {
    code: String,
}

#[derive(Serialize, Deserialize)]
enum Action {
    Eval { value: String, mime_type: MimeType },
    Insert,
    AddDependency,
}
#[derive(Debug, Serialize, Deserialize)]
enum MimeType {
    #[serde(rename = "text/plain")]
    PlainText,
    #[serde(rename = "text/html")]
    Html,
    #[serde(rename = "image/png")]
    Png,
    #[serde(rename = "image/jpeg")]
    Jpeg,
}
impl MimeType {
    fn from_str(mime_type: &str) -> Self {
        match mime_type {
            "text/plain" => Self::PlainText,
            "text/html" => Self::Html,
            "image/png" => Self::Png,
            "image/jpeg" => Self::Jpeg,
            //NOTE: we should warn here
            _ => Self::PlainText,
        }
    }
}

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    let stdin = io::stdin();
    let reader = stdin.lock();
    let mut deserializer = Deserializer::from_reader(reader).into_iter::<Message>();

    let mut repl = Repl::default();

    // NOTE: errors should not exit this loop
    // In case of an error we log it and continue
    while let Some(json) = deserializer.next() {
        let result = (|| -> Result<()> {
            let message = json?;
            if message.code.ends_with(";") {
                repl.insert(&message.code);
                let output = serde_json::to_string(&Action::Insert)?;
                println!("{output}");
            } else if message.code.starts_with(":add") {
                let cargo_add_arg = message
                    .code
                    .strip_prefix(":add")
                    .expect("checked")
                    .split_whitespace()
                    .map(ToOwned::to_owned)
                    .collect::<Vec<_>>();
                repl.cargo.cargo_add_sync(&cargo_add_arg)?;
                let output = serde_json::to_string(&Action::AddDependency)?;
                println!("{output}");
            } else {
                // eval here
                let value = repl.eval(message.code)?.output;

                // EVCXR
                if value.starts_with("EVCXR_BEGIN_CONTENT") {
                    let data = value.strip_prefix("EVCXR_BEGIN_CONTENT").expect("checked");
                    let data =
                        &data[..data.find("EVCXR_END_CONTENT").ok_or("malformed content")?];
                    let mut data = data.chars();
                    // mime_type = Regex::new("EVCXR_BEGIN_CONTENT ([^ ]+)")
                    let mime_type = data
                        .by_ref()
                        .skip_while(|c| c.is_whitespace())
                        .take_while(|c| !c.is_whitespace())
                        .collect::<String>();

                    let output = serde_json::to_string(&Action::Eval {
                        value: data.collect(),
                        mime_type: MimeType::from_str(&mime_type),
                    })?;
                    println!("{output}");
                    return Ok(());
                }

                let output = serde_json::to_string(&Action::Eval {
                    value,
                    mime_type: MimeType::PlainText,
                })?;
                println!("{output}");
            }
            Ok(())
        })();
        if result.is_err() {
            eprintln!("An error occurred: {result:?}");
            println!("{{}}"); // We still need to send a response so we send an empty object
        }
    }

    Ok(())
}
