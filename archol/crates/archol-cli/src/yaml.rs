//! Minimal block-style YAML for the AST snapshot.
//!
//! Strings (keys and values) always use JSON double-quoted syntax, which is
//! valid YAML. The output is stable and easy to diff.

use serde_json::Value;

/// Serializes a JSON value to block-style YAML with two-space indentation.
pub fn to_yaml(value: &Value) -> String {
    let mut out = String::new();
    emit(value, 0, &mut out);
    out
}

fn emit(value: &Value, indent: usize, out: &mut String) {
    match value {
        Value::Object(map) if !map.is_empty() => {
            for (key, item) in map {
                out.push_str(&" ".repeat(indent));
                out.push_str(&quote(key));
                out.push(':');
                if flow(item) {
                    out.push(' ');
                    out.push_str(&flow_text(item));
                    out.push('\n');
                } else {
                    out.push('\n');
                    emit(item, indent + 2, out);
                }
            }
        }
        Value::Array(items) if !items.is_empty() => {
            for item in items {
                out.push_str(&" ".repeat(indent));
                out.push('-');
                if flow(item) {
                    out.push(' ');
                    out.push_str(&flow_text(item));
                    out.push('\n');
                } else {
                    out.push('\n');
                    emit(item, indent + 2, out);
                }
            }
        }
        other => {
            out.push_str(&" ".repeat(indent));
            out.push_str(&flow_text(other));
            out.push('\n');
        }
    }
}

fn flow(value: &Value) -> bool {
    match value {
        Value::Object(map) => map.is_empty(),
        Value::Array(items) => items.is_empty(),
        _ => true,
    }
}

fn flow_text(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(value) => value.to_string(),
        Value::Number(value) => value.to_string(),
        Value::String(value) => quote(value),
        Value::Array(_) => "[]".to_string(),
        Value::Object(_) => "{}".to_string(),
    }
}

fn quote(value: &str) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "\"\"".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn emits_block_style_yaml_in_canonical_order() {
        let value = json!({
            "kind": "SourceFile",
            "path": "examples/a.archol",
            "items": [
                { "kind": "Artifact", "name": "A", "empty": [], "flags": [true, null] }
            ]
        });
        let value = archol_core::order::ordered(value);
        assert_eq!(
            to_yaml(&value),
            "\"kind\": \"SourceFile\"\n\"items\":\n  -\n    \"kind\": \"Artifact\"\n    \"name\": \"A\"\n    \"empty\": []\n    \"flags\":\n      - true\n      - null\n\"path\": \"examples/a.archol\"\n"
        );
    }

    #[test]
    fn quotes_strings_and_escapes() {
        let value = json!({ "text": "line\nquote \"x\"" });
        assert_eq!(to_yaml(&value), "\"text\": \"line\\nquote \\\"x\\\"\"\n");
    }
}
