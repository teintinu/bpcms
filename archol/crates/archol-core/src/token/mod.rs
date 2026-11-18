//! Léxico de Archol: `TokenKind`, `Token` e a tabela de palavras-chave.

mod keyword;

pub use keyword::Keyword;

use crate::span::Span;
use serde::Serialize;

/// Tipo de token produzido pelo lexer.
///
/// Os grupos seguem a organização de `docs/tokens.md`: pontuação, operadores,
/// literais e palavras-chave. Ao adicionar um token, adicione um teste em
/// `tests/lexer.rs` e a documentação correspondente.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TokenKind {
    /// `nome`, `Nome`, `pt-BR` (o hífen é separado, ver `token.lower_key`).
    Ident,
    /// `10`, `2.5`, `5d`, `15m`.
    Number,
    /// `"texto"`.
    String,
    /// `// comentário`.
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

/// Token com o intervalo que ocupa no fonte.
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
