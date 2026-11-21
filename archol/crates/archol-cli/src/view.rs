//! Compact preview payload (`archol view --json`) consumed by editor integrations.
//!
//! The payload is deliberately smaller than the AST: editors need token spans
//! for syntax highlighting, a flat artifact outline, pending-comment markers and
//! position diagnostics, not the whole syntax tree. Span offsets are byte
//! offsets, exactly like the AST, so both outputs agree.

use archol_core::{lex, Diagnostic, LineIndex, Severity, SourceFile, TokenKind};
use serde_json::{json, Value};

/// Everything one preview payload can carry. Fields stay optional so the same
/// builder serves a syntax-only read and an application-annotated read.
pub struct ViewInput<'a> {
    /// Path reported back to the caller (as given, not canonicalized).
    pub path: &'a str,
    pub source: &'a str,
    pub ast: &'a SourceFile,
    /// Diagnostics that belong to this file only.
    pub diagnostics: &'a [Diagnostic],
    /// Application URI and file path when the file was compiled inside one.
    pub application: Option<(&'a str, &'a str)>,
}

/// Build the JSON payload for one Archol source file.
pub fn payload(input: ViewInput<'_>) -> Value {
    let lines = LineIndex::new(input.source);
    let tokens = lex(input.source);
    let token_list: Vec<Value> = tokens
        .tokens
        .iter()
        .filter(|token| token.kind != TokenKind::Eof)
        .map(|token| {
            let text = input
                .source
                .get(token.span.start..token.span.end)
                .unwrap_or("");
            json!({
                "class": class_of(token.kind, text),
                "start": token.span.start,
                "end": token.span.end,
            })
        })
        .collect();
    let outline: Vec<Value> = input
        .ast
        .items
        .iter()
        .map(|artifact| {
            let (line, column) = lines.line_col(artifact.span.start);
            json!({
                "kind": artifact.artifact_kind(),
                "name": artifact.name,
                "line": line + 1,
                "column": column + 1,
                "start": artifact.span.start,
                "end": artifact.span.end,
            })
        })
        .collect();
    let pending = input.ast.pending.len();
    let diagnostics: Vec<Value> = input
        .diagnostics
        .iter()
        .map(|diagnostic| {
            let (line, column) = lines.line_col(diagnostic.span.start);
            json!({
                "severity": severity_name(diagnostic.severity),
                "code": diagnostic.code,
                "message": diagnostic.message,
                "line": line + 1,
                "column": column + 1,
            })
        })
        .collect();
    let mut payload = json!({
        "kind": "ArcholView",
        "path": input.path,
        "language": archol_core::FILE_EXTENSION,
        "bytes": input.source.len(),
        "version": env!("CARGO_PKG_VERSION"),
        "tokens": token_list,
        "outline": outline,
        "pending": pending,
        "diagnostics": diagnostics,    });
    if let Some((uri, path)) = input.application {
        payload["application"] = json!({ "uri": uri, "path": path });
    }
    payload
}

/// Highlighting class of one token, derived from the lexer kind and the token text.
fn class_of(kind: TokenKind, text: &str) -> &'static str {
    match kind {
        TokenKind::Ident => {
            if text.chars().next().is_some_and(char::is_uppercase) {
                "type"
            } else {
                "ident"
            }
        }
        TokenKind::Number => "number",
        TokenKind::String => "string",
        TokenKind::Comment => "comment",
        TokenKind::Keyword(_) => "keyword",
        TokenKind::Error => "error",
        TokenKind::LBrace
        | TokenKind::RBrace
        | TokenKind::LParen
        | TokenKind::RParen
        | TokenKind::LBracket
        | TokenKind::RBracket
        | TokenKind::Comma
        | TokenKind::Colon
        | TokenKind::Semicolon
        | TokenKind::Dot => "punctuation",
        TokenKind::Eof => "eof",
        _ => "operator",
    }
}

fn severity_name(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Info => "info",
        Severity::Hint => "hint",
    }
}
