//! Archol: léxico, AST e parser.
//!
//! Ponto de entrada: [`parse`], que devolve AST + diagnósticos. O check e o
//! link vivem em `archol-analysis`; o LSP e a CLI consomem esta crate.

pub mod ast;
pub mod diagnostic;
pub mod lexer;
pub mod parser;
pub mod span;
pub mod token;

pub use ast::{Artifact, Expr, Member, SourceFile, Stmt};
pub use diagnostic::{Diagnostic, Severity};
pub use lexer::{lex, Lexed};
pub use parser::{parse, ParseResult};
pub use span::{LineIndex, Span};
pub use token::{Keyword, Token, TokenKind};

/// Extensão canônica dos arquivos da linguagem.
pub const FILE_EXTENSION: &str = "archol";
