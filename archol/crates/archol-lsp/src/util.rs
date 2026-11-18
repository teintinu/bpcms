//! Conversões entre URIs, caminhos e posições LSP (UTF-16).

use std::path::{Path, PathBuf};

pub fn uri_to_path(uri: &str) -> Option<PathBuf> {
    let rest = uri.strip_prefix("file://")?;
    let decoded = percent_decode(rest);
    // Windows: file:///C:/... -> /C:/... (mantém como está; Path lida)
    Some(PathBuf::from(decoded))
}

pub fn path_to_uri(path: &Path) -> String {
    let text = path.to_string_lossy().replace('\\', "/");
    let encoded = text
        .chars()
        .map(|ch| match ch {
            ' ' => "%20".to_string(),
            '#' => "%23".to_string(),
            '?' => "%3F".to_string(),
            _ => ch.to_string(),
        })
        .collect::<String>();
    if encoded.starts_with('/') {
        format!("file://{encoded}")
    } else {
        format!("file:///{encoded}")
    }
}

fn percent_decode(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' && index + 2 < bytes.len() {
            if let Ok(byte) = u8::from_str_radix(&text[index + 1..index + 3], 16) {
                out.push(byte);
                index += 3;
                continue;
            }
        }
        out.push(bytes[index]);
        index += 1;
    }
    String::from_utf8_lossy(&out).to_string()
}

pub fn offset_of(text: &str, line: u32, character: u32) -> usize {
    let mut offset = 0;
    for (current, segment) in text.split_inclusive('\n').enumerate() {
        if current as u32 == line {
            let mut units = 0u32;
            for (byte, ch) in segment.char_indices() {
                if units >= character {
                    return offset + byte;
                }
                units += ch.len_utf16() as u32;
            }
            return offset + segment.len();
        }
        offset += segment.len();
    }
    text.len()
}

pub fn position_of(text: &str, offset: usize) -> (u32, u32) {
    let mut line = 0u32;
    let mut units = 0u32;
    for (byte, ch) in text.char_indices() {
        if byte >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            units = 0;
        } else {
            units += ch.len_utf16() as u32;
        }
    }
    (line, units)
}
