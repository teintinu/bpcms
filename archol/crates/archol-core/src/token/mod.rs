//! Archol lexer: `TokenKind`, `Token` and the keyword table.

mod keyword;

pub use keyword::Keyword;

use crate::span::Span;
use serde::Serialize;

/// Token type produced by the lexer.
///
/// The groups follow the organization in `docs/tokens.md`: punctuation,
/// operators, literals and keywords. When adding a token, add a test in
/// `tests/lexer.rs` and the corresponding documentation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TokenKind {
    /// `name`, `Name`, `pt-BR` (the hyphen is separate, see `token.lower_key`).
    Ident,
    /// `10`, `2.5`, `5d`, `15m`.
    Number,
    /// `"text"`.
    String,
    /// `// comment`.
    Comment,

    LBrace,
    RBrace,
    LParen,
    RParen,
    LBracket,
    RBracket,
    Comma,
    Colon,
    Semicolon,
    Dot,
    At,
    Question,
    QuestionQuestion,

    Assign,
    Eq,
    StrictEq,
    NotEq,
    StrictNotEq,
    Lt,
    Gt,
    LtEq,
    GtEq,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Bang,
    AndAnd,
    OrOr,
    Amp,
    Pipe,
    Arrow,
    FatArrow,

    Keyword(Keyword),

    Eof,
    Error,
}

impl TokenKind {
    pub fn is_trivia(self) -> bool {
        matches!(self, TokenKind::Comment)
    }

    pub fn is_keyword(self, keyword: Keyword) -> bool {
        matches!(self, TokenKind::Keyword(k) if k == keyword)
    }
}

/// Token with the range it occupies in the source.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }
}
