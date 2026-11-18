//! Frames JSON-RPC 2.0 com `Content-Length` sobre stdio.

use std::io::{BufRead, Read, Write};

pub type Json = serde_json::Value;

pub struct Rpc {
    reader: std::io::StdinLock<'static>,
    writer: std::io::StdoutLock<'static>,
}

impl Rpc {
    pub fn stdio() -> Self {
        Self {
            reader: std::io::stdin().lock(),
            writer: std::io::stdout().lock(),
        }
    }

    pub fn read(&mut self) -> Option<Json> {
        let mut length: Option<usize> = None;
        loop {
            let mut line = String::new();
            if self.reader.read_line(&mut line).ok()? == 0 {
                return None;
            }
            let line = line.trim_end();
            if line.is_empty() {
                break;
            }
            if let Some(value) = line.strip_prefix("Content-Length:") {
                length = value.trim().parse().ok();
            }
        }
        let length = length?;
        let mut buffer = vec![0u8; length];
        self.reader.read_exact(&mut buffer).ok()?;
        serde_json::from_slice(&buffer).ok()
    }

    pub fn write(&mut self, message: Json) -> std::io::Result<()> {
        let body = serde_json::to_vec(&message).unwrap_or_default();
        write!(self.writer, "Content-Length: {}\r\n\r\n", body.len())?;
        self.writer.write_all(&body)?;
        self.writer.flush()
    }

    pub fn respond(&mut self, id: Json, result: Json) {
        let _ = self.write(serde_json::json!({ "jsonrpc": "2.0", "id": id, "result": result }));
    }

    pub fn respond_error(&mut self, id: Json, code: i64, message: &str) {
        let _ = self.write(serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": { "code": code, "message": message }
        }));
    }

    pub fn notify(&mut self, method: &str, params: Json) {
        let _ = self.write(serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params
        }));
    }
}
