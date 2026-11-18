//! Parser recursivo de Archol.
//!
//! Estratégia: navegar pelo fonte sem exigir gramática fechada. Cada membro é
//! tentado como propriedade, bloco, valor de enum ou statement; o que não for
//! reconhecido vira `RawStatement` com span preservado. Assim o corpus real
//! sempre produz AST, e o check acusa apenas o que entende.
//!
//! Divisão: `parser/mod.rs` (arquivo/artefato/membros), `parser/expr.rs`
//! (expressões) e `parser/stmt.rs` (statements de processo).

mod expr;
mod member;
mod primary;
mod stmt;

use crate::ast::{Artifact, Member, Param, SourceFile};
use crate::diagnostic::Diagnostic;
use crate::lexer::lex;
use crate::span::{LineIndex, Span};
use crate::token::{Token, TokenKind};

pub struct ParseResult {
    pub ast: SourceFile,
    pub diagnostics: Vec<Diagnostic>,
    pub line_index: LineIndex,
}

pub fn parse(source: &str, path: impl Into<String>) -> ParseResult {
    let lexed = lex(source);
    let tokens = lexed
        .tokens
        .into_iter()
        .filter(|token| !token.kind.is_trivia())
        .collect();
    let mut parser = Parser {
        source,
        tokens,
        pos: 0,
        diagnostics: lexed.diagnostics,
        lines: LineIndex::new(source),
        depth: 0,
        depth_reported: false,
    };
    let ast = parser.parse_file(path.into());
    ParseResult {
        ast,
        diagnostics: parser.diagnostics,
        line_index: parser.lines,
    }
}

/// Profundidade máxima de aninhamento. Acima disso o parser recupera com
/// `E_PARSE_DEPTH` em vez de estourar a pilha.
const MAX_DEPTH: usize = 200;

pub(crate) struct Parser<'a> {
    source: &'a str,
    tokens: Vec<Token>,
    pos: usize,
    diagnostics: Vec<Diagnostic>,
    lines: LineIndex,
    depth: usize,
    depth_reported: bool,
}

impl<'a> Parser<'a> {
    // --- primitivas -----------------------------------------------------

    pub(crate) fn cur(&self) -> &Token {
        &self.tokens[self.pos.min(self.tokens.len().saturating_sub(1))]
    }

    pub(crate) fn kind(&self) -> TokenKind {
        self.cur().kind
    }

    pub(crate) fn peek(&self, offset: usize) -> TokenKind {
        self.tokens
            .get(self.pos + offset)
            .map(|t| t.kind)
            .unwrap_or(TokenKind::Eof)
    }

    pub(crate) fn at_eof(&self) -> bool {
        self.kind() == TokenKind::Eof
    }

    pub(crate) fn advance(&mut self) -> Token {
        let token = *self.cur();
        if !self.at_eof() {
            self.pos += 1;
        }
        token
    }

    pub(crate) fn eat(&mut self, kind: TokenKind) -> bool {
        if self.kind() == kind {
            self.advance();
            true
        } else {
            false
        }
    }

    pub(crate) fn expect(&mut self, kind: TokenKind, code: &str) -> Option<Token> {
        if self.kind() == kind {
            Some(self.advance())
        } else {
            self.error(
                code,
                format!("Esperado {kind:?}, encontrado {:?}", self.kind()),
            );
            None
        }
    }

    pub(crate) fn text(&self, span: Span) -> &'a str {
        let start = span.start.min(self.source.len());
        let end = span.end.min(self.source.len()).max(start);
        &self.source[start..end]
    }

    pub(crate) fn line_of(&self, offset: usize) -> u32 {
        self.lines.line_col(offset).0
    }

    pub(crate) fn error(&mut self, code: &str, message: String) {
        let span = Span::new(self.cur().span.start, self.cur().span.end);
        self.diagnostics
            .push(Diagnostic::error(code, message, span));
    }

    // --- arquivo ---------------------------------------------------------

    fn parse_file(&mut self, path: String) -> SourceFile {
        let mut items = Vec::new();
        while !self.at_eof() {
            if self.eat(TokenKind::Semicolon) {
                continue;
            }
            let is_modifier = matches!(self.kind(), TokenKind::Ident)
                && matches!(self.peek(1), TokenKind::Keyword(k) if k.is_artifact());
            if is_modifier {
                self.advance();
                continue;
            }
            let is_artifact = matches!(self.kind(), TokenKind::Keyword(k) if k.is_artifact());
            if !is_artifact {
                self.error(
                    "E_PARSE_TOP",
                    format!("Esperado artefato de topo, encontrado {:?}", self.kind()),
                );
                self.skip_top_level();
                continue;
            }
            if let Some(artifact) = self.parse_artifact() {
                items.push(artifact);
            }
        }
        SourceFile::new(path, items)
    }

    fn skip_top_level(&mut self) {
        let start_line = self.line_of(self.cur().span.start);
        while !self.at_eof() {
            if matches!(self.kind(), TokenKind::Keyword(k) if k.is_artifact())
                && self.line_of(self.cur().span.start) != start_line
            {
                return;
            }
            self.advance();
        }
    }

    fn parse_artifact(&mut self) -> Option<Artifact> {
        let keyword = match self.kind() {
            TokenKind::Keyword(keyword) if keyword.is_artifact() => keyword,
            _ => return None,
        };
        let start = self.cur().span.start;
        self.advance();

        let name = match self.kind() {
            TokenKind::String => {
                let token = self.advance();
                unquote(self.text(token.span)).to_string()
            }
            TokenKind::Ident => {
                let token = self.advance();
                self.text(token.span).to_string()
            }
            _ => {
                self.error("E_PARSE_NAME", "Esperado nome do artefato".to_string());
                return None;
            }
        };

        let mut params = Vec::new();
        if self.kind() == TokenKind::LParen {
            self.advance();
            params = self.parse_params();
        }
        if matches!(self.kind(), TokenKind::Ident) && self.text(self.cur().span) == "extends" {
            self.advance();
            let _ = self.parse_type_ref();
        }

        if self.kind() != TokenKind::LBrace {
            self.error("E_PARSE_BLOCK", format!("Esperado '{{' após {name}"));
            return None;
        }
        let body = self.parse_body();
        let end = self.prev_end();
        Some(Artifact::new(
            keyword.artifact_name(),
            name,
            params,
            body,
            Span::new(start, end),
        ))
    }

    fn parse_params(&mut self) -> Vec<Param> {
        let mut params = Vec::new();
        while self.kind() != TokenKind::RParen && !self.at_eof() {
            if self.kind() != TokenKind::Ident {
                self.error("E_PARSE_PARAM", "Esperado nome de parâmetro".to_string());
                self.skip_until(&[TokenKind::Comma, TokenKind::RParen]);
                self.eat(TokenKind::Comma);
                continue;
            }
            let start = self.cur().span.start;
            let name_token = self.advance();
            let name = self.text(name_token.span).to_string();
            let type_ref = if self.eat(TokenKind::Colon) {
                self.parse_type_ref()
            } else {
                None
            };
            let end = self.tokens[self.pos.saturating_sub(1)].span.end;
            params.push(Param {
                name,
                type_ref,
                span: Span::new(start, end),
            });
            if !self.eat(TokenKind::Comma) {
                break;
            }
        }
        self.expect(TokenKind::RParen, "E_PARSE_PARAM_CLOSE");
        params
    }

    /// Consome `{ ... }` e devolve os membros do corpo.
    pub(crate) fn parse_body(&mut self) -> Vec<Member> {
        let mut members = Vec::new();
        if !self.enter_depth() {
            return members;
        }
        if !self.eat(TokenKind::LBrace) {
            self.leave_depth();
            self.error("E_PARSE_BLOCK", "Esperado '{'".to_string());
            return members;
        }
        loop {
            while self.eat(TokenKind::Semicolon) || self.eat(TokenKind::Comma) {}
            if self.kind() == TokenKind::RBrace {
                self.advance();
                break;
            }
            if self.at_eof() {
                self.error("E_PARSE_UNCLOSED", "Bloco não fechado".to_string());
                break;
            }
            let before = self.pos;
            match self.parse_member() {
                Some(member) => members.push(member),
                None => self.recover_member(before),
            }
        }
        self.leave_depth();
        members
    }

    fn recover_member(&mut self, before: usize) {
        if self.pos == before {
            self.advance();
        }
        let level = self.line_of(self.cur().span.start);
        while !self.at_eof() && self.kind() != TokenKind::RBrace {
            if self.line_of(self.cur().span.start) != level {
                break;
            }
            self.advance();
        }
    }

    pub(crate) fn enter_depth(&mut self) -> bool {
        if self.depth >= MAX_DEPTH {
            if !self.depth_reported {
                self.depth_reported = true;
                self.error(
                    "E_PARSE_DEPTH",
                    "Aninhamento profundo demais; trecho tratado como não estruturado".to_string(),
                );
            }
            self.skip_balanced();
            return false;
        }
        self.depth += 1;
        true
    }

    pub(crate) fn leave_depth(&mut self) {
        self.depth = self.depth.saturating_sub(1);
    }

    /// Consome tokens até o `}` correspondente (ou EOF), para recuperação.
    pub(crate) fn skip_balanced(&mut self) {
        let mut depth = 0i32;
        while !self.at_eof() {
            match self.kind() {
                TokenKind::LBrace => {
                    depth += 1;
                    self.advance();
                }
                TokenKind::RBrace => {
                    self.advance();
                    if depth == 0 {
                        return;
                    }
                    depth -= 1;
                }
                _ => {
                    self.advance();
                }
            }
        }
    }

    pub(crate) fn skip_until(&mut self, stop: &[TokenKind]) {
        while !self.at_eof() && !stop.contains(&self.kind()) {
            self.advance();
        }
    }
}

pub(crate) fn unquote(raw: &str) -> &str {
    raw.strip_prefix('"')
        .and_then(|s| s.strip_suffix('"'))
        .unwrap_or(raw)
}
