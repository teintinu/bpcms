//! LSP handlers for hover, completion, definition, references and folding.

use std::path::Path;

use archol_analysis::Analysis;
use archol_core::Span;
use serde_json::{json, Value};

use crate::features::{self, range};
use crate::util::{offset_of, path_to_uri};

use super::Server;

impl Server {
    pub(super) fn hover(&mut self, params: &Value) -> Value {
        let Some((path, text)) = self.document_text(params) else {
            return Value::Null;
        };
        let offset = params
            .pointer("/position/line")
            .and_then(Value::as_u64)
            .zip(
                params
                    .pointer("/position/character")
                    .and_then(Value::as_u64),
            )
            .map(|(line, character)| offset_of(&text, line as u32, character as u32))
            .unwrap_or(0);
        let analysis = self.analysis();
        let Some(file) = analysis
            .workspace
            .files
            .iter()
            .find(|file| file.path == path)
        else {
            return Value::Null;
        };
        let Some((word, word_span)) = features::word_at(&text, offset) else {
            return Value::Null;
        };
        let service = file.service.clone();
        let service_ref = service.as_deref();
        let artifact = features::artifact_at(&file.ast, offset);
        let mut parts: Vec<String> = Vec::new();

        if let Some((artifact, value)) = features::enum_value_at(&file.ast, offset) {
            let title = analysis
                .symbols
                .resolve_enum(&artifact.name, service_ref)
                .and_then(|info| info.values.iter().find(|v| v.name == value))
                .and_then(|v| v.title.clone());
            parts.push(format!("**{value}** — value of `{}`", artifact.name));
            if let Some(title) = title {
                parts.push(title);
            }
        } else if let Some(param) = artifact.and_then(|artifact| {
            artifact
                .params
                .iter()
                .find(|param| features::contains(param.span, offset))
        }) {
            let ty = param
                .type_ref
                .clone()
                .unwrap_or_else(|| "inferred".to_string());
            parts.push(format!("**{}** · parameter `{ty}`", param.name));
        } else if let Some((info, member)) = analysis.symbols.find_value(&word) {
            if !analysis.symbols.enums.iter().any(|e| e.name == word) {
                parts.push(format!("**{}** — value of `{}`", member.name, info.name));
                if let Some(title) = &member.title {
                    parts.push(title.clone());
                }
            }
        }
        if parts.is_empty() {
            if let Some(info) = analysis.symbols.resolve_enum(&word, service_ref) {
                let values = info
                    .values
                    .iter()
                    .map(|v| {
                        v.title
                            .as_ref()
                            .map(|title| format!("`{}` — {title}", v.name))
                            .unwrap_or_else(|| format!("`{}`", v.name))
                    })
                    .collect::<Vec<_>>()
                    .join("\n\n");
                parts.push(format!("**shape `{word}`** · enum\n\n{values}"));
            } else if let Some(info) = analysis.symbols.artifacts.get(&word) {
                if let Some(first) = info.first() {
                    let service = first
                        .service
                        .as_ref()
                        .map(|s| format!("\n\nservice: `{s}`"))
                        .unwrap_or_default();
                    parts.push(format!("**{} `{}`**{service}", first.kind, first.name));
                }
            } else if analysis.symbols.roles.contains(&word) {
                parts.push(format!("**role `{word}`**"));
            } else if let Some((name, ty)) = features::const_at(&file.ast, offset) {
                let ty = ty.unwrap_or("inferred");
                parts.push(format!("**{name}** · variable `{ty}`"));
            }
        }

        if parts.is_empty() {
            return Value::Null;
        }
        parts.push(ast_link(&path, &text, word_span));
        json!({
            "contents": { "kind": "markdown", "value": parts.join("\n\n") },
            "range": range(&text, word_span)
        })
    }

    pub(super) fn completion(&mut self, params: &Value) -> Value {
        let Some((path, text)) = self.document_text(params) else {
            return json!({ "isIncomplete": false, "items": [] });
        };
        let offset = params
            .pointer("/position/line")
            .and_then(Value::as_u64)
            .zip(
                params
                    .pointer("/position/character")
                    .and_then(Value::as_u64),
            )
            .map(|(line, character)| offset_of(&text, line as u32, character as u32))
            .unwrap_or(0);
        let line_start = text[..offset].rfind('\n').map(|i| i + 1).unwrap_or(0);
        let prefix = &text[line_start..offset];

        let analysis = self.analysis();
        let enum_type = expected_enum(analysis, &path, offset, prefix);
        if let Some(enum_type) = enum_type {
            let service = analysis
                .workspace
                .files
                .iter()
                .find(|file| file.path == path)
                .and_then(|file| file.service.clone());
            if let Some(info) = analysis
                .symbols
                .resolve_enum(&enum_type, service.as_deref())
            {
                let items: Vec<Value> = info
                    .values
                    .iter()
                    .map(|value| {
                        json!({
                            "label": value.name,
                            "kind": 20,
                            "detail": enum_type,
                            "documentation": value.title.clone().unwrap_or_default(),
                            "insertText": value.name,
                        })
                    })
                    .collect();
                return json!({ "isIncomplete": false, "items": items });
            }
        }

        if prefix.contains("roles") && prefix.contains('[') {
            let items: Vec<Value> = analysis
                .symbols
                .roles
                .iter()
                .map(|role| json!({ "label": role, "kind": 7, "detail": "role" }))
                .collect();
            return json!({ "isIncomplete": false, "items": items });
        }

        let items: Vec<Value> = analysis
            .workspace
            .files
            .iter()
            .flat_map(|file| file.ast.items.iter())
            .take(400)
            .map(|item| {
                json!({
                    "label": item.name,
                    "kind": 5,
                    "detail": item.artifact_kind,
                })
            })
            .collect();
        json!({ "isIncomplete": false, "items": items })
    }
}

fn expected_enum(analysis: &Analysis, path: &str, offset: usize, prefix: &str) -> Option<String> {
    let file = analysis
        .workspace
        .files
        .iter()
        .find(|file| file.path == path)?;
    let trimmed = prefix.trim_end();
    if trimmed.ends_with('=') && !trimmed.ends_with("==") && !trimmed.ends_with("!=") {
        if let Some((_, ty)) = features::const_at(&file.ast, offset) {
            return ty.map(|ty| ty.to_string());
        }
        return None;
    }
    if trimmed.ends_with("==") || trimmed.ends_with("!=") {
        let artifact = features::comparison_variable_at(&file.ast, offset)?;
        let var = trimmed
            .trim_end_matches('=')
            .trim_end()
            .rsplit(|c: char| !(c.is_alphanumeric() || c == '_'))
            .next()?;
        return features::local_type(artifact, var);
    }
    None
}

fn ast_link(path: &str, text: &str, span: Span) -> String {
    let (line, character) = crate::util::position_of(text, span.start);
    let args = serde_json::to_string(&json!([path_to_uri(Path::new(path)), line, character]))
        .unwrap_or_default();
    format!(
        "[Explore AST](command:archol.showAst?{})",
        encode_component(&args)
    )
}

fn encode_component(text: &str) -> String {
    let mut out = String::new();
    for byte in text.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char)
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}
