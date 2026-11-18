//! Lexer manual, sem dependências externas.
//!
//! Regra de ouro: todo token tem `span` exato no fonte. Comentários viram
//! tokens (`Comment`) para que a ferramenta de tokens e o explorador de AST
//! possam mostrá-los; o parser os ignora.

use crate::diagnostic::Diagnostic;
use crate::span::Span;
use crate::token::{Keyword, Token, TokenKind};

/// Resultado da tokenização.
#[derive(Debug, Default)]
pub struct Lexed {
    pub tokens: Vec<Token>,
    pub diagnostics: Vec<Diagnostic>,
}

pub fn lex(source: &str) -> Lexed {
    let mut lexer = Lexer {
        source,
        pos: 0,
        tokens: Vec::new(),
        diagnostics: Vec::new(),
    };
    lexer.run();
    Lexed {
        tokens: lexer.tokens,
        diagnostics: lexer.diagnostics,
    }
}

struct Lexer<'a> {
    source: &'a str,
    pos: usize,
    tokens: Vec<Token>,
    diagnostics: Vec<Diagnostic>,
}

impl<'a> Lexer<'a> {
    fn run(&mut self) {
        while self.pos < self.source.len() {
            let start = self.pos;
            let byte = match self.byte_at(self.pos) {
                Some(byte) => byte,
                None => break,
            };
            match byte {
                b' ' | b'\t' | b'\r' | b'\n' => self.pos += 1,
                b'/' if self.byte_at(self.pos + 1) == Some(b'/') => self.comment(),
                b'"' => self.string(),
                b'0'..=b'9' => self.number(),
                b'a'..=b'z' | b'A'..=b'Z' | b'_' => self.ident(),
                _ => {
                    if self.operator() {
                        continue;
                    }
                    let ch = self.char_at(start).unwrap_or('?');
                    if ch.is_alphabetic() {
                        self.ident();
                        continue;
                    }
                    self.pos += ch.len_utf8();
                    self.error(
                        "E_LEX_CHAR",
                        format!("Caractere inesperado: {ch}"),
                        start,
                        self.pos,
                    );
                }
            }
        }
        self.push(TokenKind::Eof, self.pos, self.pos);
    }

    fn push(&mut self, kind: TokenKind, start: usize, end: usize) {
        self.tokens.push(Token::new(kind, Span::new(start, end)));
    }

    fn error(&mut self, code: &str, message: String, start: usize, end: usize) {
        self.diagnostics
            .push(Diagnostic::error(code, message, Span::new(start, end)));
        self.push(TokenKind::Error, start, end);
    }

    fn byte_at(&self, pos: usize) -> Option<u8> {
        self.source.as_bytes().get(pos).copied()
    }

    fn char_at(&self, pos: usize) -> Option<char> {
        self.source.get(pos..).and_then(|s| s.chars().next())
    }

    fn starts_with(&self, text: &str) -> bool {
        self.source[self.pos..].starts_with(text)
    }

    fn comment(&mut self) {
        let start = self.pos;
        while let Some(byte) = self.byte_at(self.pos) {
            if byte == b'\n' {
                break;
            }
            self.pos += 1;
        }
        self.push(TokenKind::Comment, start, self.pos);
    }

    fn string(&mut self) {
        let start = self.pos;
        self.pos += 1;
        while let Some(byte) = self.byte_at(self.pos) {
            match byte {
                b'\\' => self.pos = (self.pos + 2).min(self.source.len()),
                b'"' => {
                    self.pos += 1;
                    self.push(TokenKind::String, start, self.pos);
                    return;
                }
                _ => {
                    let ch = self.char_at(self.pos).unwrap_or('?');
                    self.pos += ch.len_utf8();
                }
            }
        }
        self.error(
            "E_LEX_STRING",
            "String não terminada".to_string(),
            start,
            self.source.len(),
        );
    }

    fn number(&mut self) {
        let start = self.pos;
        while matches!(self.byte_at(self.pos), Some(b'0'..=b'9')) {
            self.pos += 1;
        }
        if self.byte_at(self.pos) == Some(b'.')
            && matches!(self.byte_at(self.pos + 1), Some(b'0'..=b'9'))
        {
            self.pos += 1;
            while matches!(self.byte_at(self.pos), Some(b'0'..=b'9')) {
                self.pos += 1;
            }
        }
        // Sufixos de duração e unidade: 5d, 15m, 1h, 30s, 2026y.
        while matches!(self.byte_at(self.pos), Some(b'a'..=b'z' | b'A'..=b'Z')) {
            self.pos += 1;
        }
        self.push(TokenKind::Number, start, self.pos);
    }

    fn ident(&mut self) {
        let start = self.pos;
        while let Some(ch) = self.char_at(self.pos) {
            if !(ch.is_alphanumeric() || ch == '_') {
                break;
            }
            self.pos += ch.len_utf8();
        }
        let text = &self.source[start..self.pos];
        let kind = match Keyword::from_word(text) {
            Some(keyword) => TokenKind::Keyword(keyword),
            None => TokenKind::Ident,
        };
        self.push(kind, start, self.pos);
    }

    /// Operadores e pontuação, do mais longo para o mais curto.
    fn operator(&mut self) -> bool {
        const TABLE: &[(&str, TokenKind)] = &[
            ("!==", TokenKind::StrictNotEq),
            ("===", TokenKind::StrictEq),
            ("->", TokenKind::Arrow),
            ("=>", TokenKind::FatArrow),
            ("&&", TokenKind::AndAnd),
            ("||", TokenKind::OrOr),
            ("==", TokenKind::Eq),
            ("!=", TokenKind::NotEq),
            ("!", TokenKind::Bang),
            ("<=", TokenKind::LtEq),
            (">=", TokenKind::GtEq),
            ("{", TokenKind::LBrace),
            ("}", TokenKind::RBrace),
            ("(", TokenKind::LParen),
            (")", TokenKind::RParen),
            ("[", TokenKind::LBracket),
            ("]", TokenKind::RBracket),
            (",", TokenKind::Comma),
            (":", TokenKind::Colon),
            (";", TokenKind::Semicolon),
            (".", TokenKind::Dot),
            ("@", TokenKind::At),
            ("??", TokenKind::QuestionQuestion),
            ("?", TokenKind::Question),
            ("=", TokenKind::Assign),
            ("<", TokenKind::Lt),
            (">", TokenKind::Gt),
            ("+", TokenKind::Plus),
            ("-", TokenKind::Minus),
            ("*", TokenKind::Star),
            ("/", TokenKind::Slash),
            ("%", TokenKind::Percent),
            ("&", TokenKind::Amp),
            ("|", TokenKind::Pipe),
        ];
        for (text, kind) in TABLE {
            if self.starts_with(text) {
                let start = self.pos;
                self.pos += text.len();
                self.push(*kind, start, self.pos);
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(source: &str) -> Vec<TokenKind> {
        lex(source)
            .tokens
            .into_iter()
            .map(|t| t.kind)
            .filter(|k| !k.is_trivia())
            .collect()
    }

    #[test]
    fn lexa_artefato_minimo() {
        assert_eq!(
            kinds("document Invoice { }"),
            vec![
                TokenKind::Keyword(Keyword::Document),
                TokenKind::Ident,
                TokenKind::LBrace,
                TokenKind::RBrace,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn lexa_comentario_e_string_sem_quebrar_chaves() {
        let lexed = lex("// a { b\nkey: \"{x}\"");
        assert_eq!(
            lexed.tokens.iter().map(|t| t.kind).collect::<Vec<_>>(),
            vec![
                TokenKind::Comment,
                TokenKind::Ident,
                TokenKind::Colon,
                TokenKind::String,
                TokenKind::Eof,
            ]
        );
        assert!(lexed.diagnostics.is_empty());
    }

    #[test]
    fn lexa_numeros_com_sufixo() {
        assert_eq!(
            kinds("5d 15m 1h 2.5 10%"),
            vec![
                TokenKind::Number,
                TokenKind::Number,
                TokenKind::Number,
                TokenKind::Number,
                TokenKind::Number,
                TokenKind::Percent,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn lexa_operadores_de_tres_caracteres() {
        assert_eq!(
            kinds("a !== b === c && d"),
            vec![
                TokenKind::Ident,
                TokenKind::StrictNotEq,
                TokenKind::Ident,
                TokenKind::StrictEq,
                TokenKind::Ident,
                TokenKind::AndAnd,
                TokenKind::Ident,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn reporta_string_nao_terminada() {
        let lexed = lex("x: \"abc");
        assert_eq!(lexed.diagnostics.len(), 1);
        assert_eq!(lexed.diagnostics[0].code, "E_LEX_STRING");
    }
}
