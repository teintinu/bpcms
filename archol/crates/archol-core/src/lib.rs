//! Archol: lexer, AST and parser.
//!
//! Entry point: [`parse`], which returns AST + diagnostics. The check and
//! link live in `archol-analysis`; the LSP and the CLI consume this crate.

pub mod ast;
pub mod diagnostic;
pub mod lexer;
pub mod parser;
pub mod pending;
pub mod span;
pub mod token;

pub use ast::{Artifact, Expr, Member, Pending, ShapeValue, SourceFile, Stmt};
pub use diagnostic::{Diagnostic, Severity};
pub use lexer::{lex, Lexed};
pub use parser::{parse, ParseResult};
pub use pending::Date;
pub use span::{LineIndex, Span};
pub use token::{Keyword, Token, TokenKind};

/// Canonical file extension of the language.
pub const FILE_EXTENSION: &str = "archol";
