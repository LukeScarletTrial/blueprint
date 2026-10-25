use blueprint_compiler::compile;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{self, BufRead, Read, Write};

#[derive(Debug, Serialize, Deserialize)]
struct LspRequest {
    jsonrpc: String,
    id: Option<serde_json::Value>,
    method: String,
    params: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct LspResponse {
    jsonrpc: String,
    id: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<serde_json::Value>,
}

fn main() -> anyhow::Result<()> {
    let stdin = io::stdin();
    let mut stdin_lock = stdin.lock();
    let stdout = io::stdout();
    let mut stdout_lock = stdout.lock();

    let mut documents: HashMap<String, String> = HashMap::new();

    loop {
        let mut line = String::new();
        if stdin_lock.read_line(&mut line)? == 0 {
            break;
        }

        if line.starts_with("Content-Length:") {
            let len_str = line.trim_start_matches("Content-Length:").trim();
            let content_len: usize = len_str.parse()?;

            // Read trailing header lines until empty line
            loop {
                let mut header = String::new();
                stdin_lock.read_line(&mut header)?;
                if header.trim().is_empty() {
                    break;
                }
            }

            let mut body_bytes = vec![0u8; content_len];
            stdin_lock.read_exact(&mut body_bytes)?;
            let body_str = String::from_utf8(body_bytes)?;

            if let Ok(req) = serde_json::from_str::<LspRequest>(&body_str) {
                handle_request(&req, &mut documents, &mut stdout_lock)?;
            }
        }
    }

    Ok(())
}

fn send_response(
    id: Option<serde_json::Value>,
    result: serde_json::Value,
    writer: &mut impl Write,
) -> anyhow::Result<()> {
    let resp = LspResponse {
        jsonrpc: "2.0".to_string(),
        id,
        result: Some(result),
        error: None,
    };
    let json = serde_json::to_string(&resp)?;
    let msg = format!("Content-Length: {}\r\n\r\n{}", json.len(), json);
    writer.write_all(msg.as_bytes())?;
    writer.flush()?;
    Ok(())
}

fn send_notification(
    method: &str,
    params: serde_json::Value,
    writer: &mut impl Write,
) -> anyhow::Result<()> {
    let notif = serde_json::json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params
    });
    let json = serde_json::to_string(&notif)?;
    let msg = format!("Content-Length: {}\r\n\r\n{}", json.len(), json);
    writer.write_all(msg.as_bytes())?;
    writer.flush()?;
    Ok(())
}

fn handle_request(
    req: &LspRequest,
    documents: &mut HashMap<String, String>,
    writer: &mut impl Write,
) -> anyhow::Result<()> {
    match req.method.as_str() {
        "initialize" => {
            let result = serde_json::json!({
                "capabilities": {
                    "textDocumentSync": 1, // Full sync
                    "completionProvider": {
                        "triggerCharacters": [".", "@", ":"]
                    },
                    "hoverProvider": true,
                    "definitionProvider": true
                },
                "serverInfo": {
                    "name": "blueprint-lsp",
                    "version": "0.1.0"
                }
            });
            send_response(req.id.clone(), result, writer)?;
        }
        "textDocument/didOpen" => {
            if let Some(params) = &req.params {
                if let (Some(uri), Some(text)) = (
                    params.get("textDocument").and_then(|d| d.get("uri")).and_then(|u| u.as_str()),
                    params.get("textDocument").and_then(|d| d.get("text")).and_then(|t| t.as_str()),
                ) {
                    documents.insert(uri.to_string(), text.to_string());
                    publish_diagnostics(uri, text, writer)?;
                }
            }
        }
        "textDocument/didChange" => {
            if let Some(params) = &req.params {
                if let Some(uri) = params.get("textDocument").and_then(|d| d.get("uri")).and_then(|u| u.as_str()) {
                    if let Some(changes) = params.get("contentChanges").and_then(|c| c.as_array()) {
                        if let Some(last_change) = changes.last() {
                            if let Some(text) = last_change.get("text").and_then(|t| t.as_str()) {
                                documents.insert(uri.to_string(), text.to_string());
                                publish_diagnostics(uri, text, writer)?;
                            }
                        }
                    }
                }
            }
        }
        "textDocument/hover" => {
            let hover_result = serde_json::json!({
                "contents": {
                    "kind": "markdown",
                    "value": "**Blueprint Symbol**\n\nStatically checked entity."
                }
            });
            send_response(req.id.clone(), hover_result, writer)?;
        }
        "textDocument/completion" => {
            let keywords = vec![
                "func", "def", "defstruct", "return", "if", "else", "true", "false",
                "Blueprint.Setup", "output", "input", "str_len", "str_to_int", "str_to_float",
                "str_contains", "file_read", "file_write", "file_exists", "path_join",
                "env_get", "env_set", "time_now", "time_sleep", "math_sqrt", "math_pow",
                "int", "i8", "i16", "i32", "i64", "u8", "u16", "u32", "u64", "f32", "f64", "string", "bool"
            ];

            let items: Vec<serde_json::Value> = keywords
                .into_iter()
                .map(|kw| {
                    serde_json::json!({
                        "label": kw,
                        "kind": if kw.contains("()") || kw.starts_with("str_") || kw.starts_with("math_") || kw.starts_with("file_") { 3 } else { 14 },
                        "detail": "Blueprint Builtin"
                    })
                })
                .collect();

            send_response(req.id.clone(), serde_json::json!({ "isIncomplete": false, "items": items }), writer)?;
        }
        _ => {
            if req.id.is_some() {
                send_response(req.id.clone(), serde_json::Value::Null, writer)?;
            }
        }
    }

    Ok(())
}

fn publish_diagnostics(uri: &str, text: &str, writer: &mut impl Write) -> anyhow::Result<()> {
    let mut diagnostics = Vec::new();

    if let Err(compile_err) = compile(text) {
        let (line, col, msg) = match &compile_err {
            blueprint_compiler::CompileError::Lex(e) => (e.location.line, e.location.column, e.message.clone()),
            blueprint_compiler::CompileError::Parse(e) => (e.location.line, e.location.column, e.message.clone()),
            blueprint_compiler::CompileError::Semantic(e) => (e.location.line, e.location.column, e.message.clone()),
            blueprint_compiler::CompileError::Runtime(e) => (e.location.line, e.location.column, e.message.clone()),
            blueprint_compiler::CompileError::General { message } => (1, 1, message.clone()),
        };

        let l = line.saturating_sub(1);
        let c = col.saturating_sub(1);

        diagnostics.push(serde_json::json!({
            "range": {
                "start": { "line": l, "character": c },
                "end": { "line": l, "character": c + 1 }
            },
            "severity": 1, // Error
            "source": "blueprint",
            "message": msg
        }));
    }

    send_notification(
        "textDocument/publishDiagnostics",
        serde_json::json!({
            "uri": uri,
            "diagnostics": diagnostics
        }),
        writer,
    )?;

    Ok(())
}
