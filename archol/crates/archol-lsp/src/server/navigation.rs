//! LSP navigation: definition, references, highlight and folding.

use std::path::Path;

use archol_core::Span;
use serde_json::{json, Value};

use crate::app;
use crate::features::{self, range};
use crate::util::{offset_of, path_to_uri};

use super::Server;

impl Server {
    pub(super) fn definition(&mut self, params: &Value) -> Value {
        let Some((path, text)) = self.document_text(params) else {
            return Value::Null;
        };
        if !self.in_current_application(&path) {
            return Value::Null;
        }
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
        let target = crate::resolve::semantic_definition(analysis, &path, offset)
            .or_else(|| crate::resolve::definition(analysis, &path, offset));
        let Some(target) = target else {
            return Value::Null;
        };
        json!({
            "uri": path_to_uri(Path::new(&target.path)),
            "range": range(
                analysis
                    .workspace
                    .files
                    .iter()
                    .find(|file| file.path == target.path)
                    .map(|file| file.source.as_str())
                    .unwrap_or(""),
                target.span
            )
        })
    }

    pub(super) fn references(&mut self, params: &Value, highlight: bool) -> Value {
        let Some((path, text)) = self.document_text(params) else {
            return json!([]);
        };
        if !self.in_current_application(&path) {
            return json!([]);
        }
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
        let Some((word, _)) = features::word_at(&text, offset) else {
            return json!([]);
        };
        let analysis = self.analysis();
        let mut found = Vec::new();
        for file in &analysis.workspace.files {
            if !app::in_application(analysis, &file.path) {
                continue;
            }
            if highlight && file.path != path {
                continue;
            }
            let lexed = archol_core::lex(&file.source);
            for token in &lexed.tokens {
                if matches!(token.kind, archol_core::TokenKind::Ident)
                    && file.source[token.span.start..token.span.end] == word
                {
                    found.push(json!({
                        "uri": path_to_uri(Path::new(&file.path)),
                        "range": range(&file.source, token.span)
                    }));
                }
            }
        }
        if highlight {
            return Value::Array(found);
        }
        Value::Array(found)
    }

    pub(super) fn folding(&mut self, params: &Value) -> Value {
        let Some((path, text)) = self.document_text(params) else {
            return json!([]);
        };
        if !self.in_current_application(&path) {
            return json!([]);
        }
        let analysis = self.analysis();
        let Some(file) = analysis
            .workspace
            .files
            .iter()
            .find(|file| file.path == path)
        else {
            return json!([]);
        };
        let mut ranges = Vec::new();
        for item in &file.ast.items {
            add_folding(&text, item.span, &mut ranges);
            features::walk(&item.body, &mut |member| {
                add_folding(&text, member.span(), &mut ranges)
            });
        }
        Value::Array(ranges)
    }
}

fn add_folding(text: &str, span: Span, ranges: &mut Vec<Value>) {
    let (start_line, _) = crate::util::position_of(text, span.start);
    let (end_line, _) = crate::util::position_of(text, span.end);
    if end_line > start_line {
        ranges.push(json!({
            "startLine": start_line,
            "endLine": end_line.saturating_sub(1),
            "kind": "region"
        }));
    }
}
