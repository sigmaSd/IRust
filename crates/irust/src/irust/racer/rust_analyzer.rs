use crate::irust::Result;
use serde_json::{json, Value};
use std::io::Write;
use std::io::{BufRead, Read};
use std::process::{Child, ChildStdin, ChildStdout};
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::time::Duration;
use std::{
    io::BufReader,
    path::Path,
    process::{Command, Stdio},
};

static ID: AtomicUsize = AtomicUsize::new(1);

pub struct RustAnalyzer {
    _process: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl RustAnalyzer {
    pub fn start(root_uri: &Path, uri: &Path, text: String) -> Result<RustAnalyzer> {
        let mut process = Command::new("rust-analyzer")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null()) // comment out to debug lsp
            .spawn()?;
        let mut stdin = process.stdin.take().expect("piped");
        let mut stdout = BufReader::new(process.stdout.take().expect("piped"));

        // Send a "initialize" request to the language server
        let initialize_request = json!({
            "jsonrpc": "2.0",
            "id":  ID.fetch_add(1, Ordering::SeqCst),
            "method": "initialize",
            "params": {
                "processId": std::process::id(),
                "rootUri": format!("file://{}",root_uri.display()),
                "capabilities": {
                    "textDocument": {
                        "completion": {
                            "completionItem": {
                                "documentationFormat": ["plaintext"]
                            },
                            "completionItemKind": {
                                "valueSet": [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35]
                            }
                        }
                    }
                }
            },
        });
        send_request(&mut stdin, &initialize_request)?;

        // Send an "initialized" notification to the language server
        let initialized_notification = json!({
            "jsonrpc": "2.0",
            "method": "initialized",
            "params": {},
        });
        send_request(&mut stdin, &initialized_notification)?;

        // Wait for "initialize" response
        let _initialize_response = read_response(&mut stdout)?;

        // Send a "textDocument/didOpen" notification to the language server
        let did_open_notification = json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": format!("file://{}",uri.display()),
                    "languageId": "rust",
                    "version": 1,
                    "text": text,
                },
            },
        });
        send_request(&mut stdin, &did_open_notification)?;

        Ok(RustAnalyzer {
            _process: process,
            stdin,
            stdout,
        })
    }

    pub fn document_did_change(&mut self, uri: &Path, text: String) -> Result<()> {
        let did_change_notification = json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didChange",
            "params": {
                "textDocument": {
                    "uri": format!("file://{}",uri.display()),
                    "version": 2,
                },
                "contentChanges": [
                    {
                        "text":text,
                    }
                ]
            },
        });
        send_request(&mut self.stdin, &did_change_notification)?;
        Ok(())
    }

    pub fn reload_workspace(&mut self) -> Result<()> {
        let reload_msg = json!({
            "jsonrpc": "2.0",
            "id":  ID.fetch_add(1, Ordering::SeqCst),
            "method": "rust-analyzer/reloadWorkspace",
        });
        send_request(&mut self.stdin, &reload_msg)?;
        read_response(&mut self.stdout)?;
        Ok(())
    }

    pub fn document_completion(
        &mut self,
        uri: &Path,
        (line, character): (usize, usize),
    ) -> Result<Vec<String>> {
        // Send a "textDocument/completion" request to the language server
        let completion_request = json!({
            "jsonrpc": "2.0",
            "id":  ID.fetch_add(1, Ordering::SeqCst),
            "method": "textDocument/completion",
            "params": {
                "textDocument": {
                    "uri": format!("file://{}",uri.display()),
                },
                "position": {
                    "line": line,
                    "character": character
                },
            },
        });
        send_request(&mut self.stdin, &completion_request)?;

        let completion_response = loop {
            let completion_response = read_response(&mut self.stdout)?;
            if completion_response.get("result").is_some() {
                break completion_response;
            }
            // NOTE: we block until we get a completion
            std::thread::sleep(Duration::from_millis(100));
        };
        if let Some(result) = completion_response.get("result") {
            if let Some(items) = result.get("items") {
                return Ok(items
                    .as_array()
                    .ok_or("ra items is not an array")?
                    .iter()
                    .filter_map(|item| item.get("filterText"))
                    .map(|item| item.to_string())
                    // remove quotes
                    .map(|item| item[1..item.len() - 1].to_owned())
                    .collect());
            }
        }

        Ok(vec![])
    }
}

fn send_request(stdin: &mut std::process::ChildStdin, request: &Value) -> Result<()> {
    let request_str = serde_json::to_string(request)?;
    let content_length = request_str.len();
    writeln!(stdin, "Content-Length: {}\r", content_length)?;
    writeln!(stdin, "\r")?;
    write!(stdin, "{}", request_str)?;
    stdin.flush()?;
    Ok(())
}

fn read_response(reader: &mut BufReader<std::process::ChildStdout>) -> Result<Value> {
    let content_length = get_content_length(reader)?;
    let mut content = vec![0; content_length];

    reader.read_exact(&mut content)?;
    let json_string = String::from_utf8(content)?;
    let message = serde_json::from_str(&json_string)?;
    Ok(message)
}

fn get_content_length(reader: &mut BufReader<std::process::ChildStdout>) -> Result<usize> {
    let mut line = String::new();
    let mut blank_line = String::new();

    let mut _bytes_read = reader.read_line(&mut line)?;
    let mut split = line.trim().split(": ");

    if split.next() == Some("Content-Length") {
        _bytes_read = reader.read_line(&mut blank_line)?;
        Ok(split
            .next()
            .and_then(|value_string| value_string.parse().ok())
            .ok_or("malformed rpc message")?)
    } else {
        Err("malformed rpc message".into())
    }
}
