//! Statements de processo: `if`, `try/catch`, `return`, `const`/`let`.

use crate::ast::{CatchClause, Member, Stmt};
use crate::span::Span;
use crate::token::TokenKind::{Assign, Colon, Eof, Ident, LParen, RBrace, RParen, Semicolon};

use super::Parser;

impl Parser<'_> {
    pub(crate) fn parse_if(&mut self) -> Member {
        let start = self.cur().span.start;
        self.advance(); // if
        let condition = if self.eat(LParen) {
            let cond = self.parse_expr(false);
            self.expect(RParen, "E_PARSE_IF");
            cond
        } else {
            self.parse_expr(false)
        };
        let then_branch = self.parse_body();
        let else_branch = if self.eat(crate::token::TokenKind::Keyword(
            crate::token::Keyword::Else,
        )) {
            if self.kind() == crate::token::TokenKind::Keyword(crate::token::Keyword::If) {
                let nested = self.parse_if();
                Some(vec![nested])
            } else {
                Some(self.parse_body())
            }
        } else {
            None
        };
        let span = Span::new(start, self.prev_end());
        Member::Statement {
            statement: Stmt::IfStatement {
                condition,
                then_branch,
                else_branch,
                span,
            },
            span,
        }
    }

    pub(crate) fn parse_try(&mut self) -> Member {
        let start = self.cur().span.start;
        self.advance(); // try
        let body = self.parse_body();
        let mut catches = Vec::new();
        while self.kind() == crate::token::TokenKind::Keyword(crate::token::Keyword::Catch) {
            let catch_start = self.cur().span.start;
            self.advance(); // catch
            let type_ref = if self.eat(LParen) {
                let mut ty = self.parse_type_ref();
                while self.eat(crate::token::TokenKind::Comma) {
                    if let Some(other) = self.parse_type_ref() {
                        let base = ty.clone().unwrap_or_default();
                        ty = Some(format!("{base}, {other}"));
                    }
                }
                self.expect(RParen, "E_PARSE_CATCH");
                ty
            } else {
                None
            };
            let catch_body = self.parse_body();
            catches.push(CatchClause {
                type_ref,
                body: catch_body,
                span: Span::new(catch_start, self.prev_end()),
            });
        }
        let span = Span::new(start, self.prev_end());
        Member::Statement {
            statement: Stmt::TryStatement {
                body,
                catches,
                span,
            },
            span,
        }
    }

    pub(crate) fn parse_return(&mut self) -> Member {
        let start = self.cur().span.start;
        self.advance(); // return
        let start_line = self.line_of(start);
        let has_value = !matches!(self.kind(), RBrace | Semicolon | Eof)
            && self.line_of(self.cur().span.start) == start_line;
        let value = if has_value {
            Some(self.parse_expr(true))
        } else {
            None
        };
        let span = Span::new(
            start,
            value.as_ref().map(|e| e.span().end).unwrap_or(start + 6),
        );
        Member::Statement {
            statement: Stmt::ReturnStatement { value, span },
            span,
        }
    }

    pub(crate) fn parse_var(&mut self, is_const: bool) -> Member {
        let start = self.cur().span.start;
        self.advance(); // const | let
        if self.kind() != Ident {
            return self.raw_statement(start);
        }
        let name_token = self.advance();
        let name = self.text(name_token.span).to_string();
        let type_ref = if self.eat(Colon) {
            self.parse_type_ref()
        } else {
            None
        };
        let init = if self.eat(Assign) {
            Some(self.parse_expr(true))
        } else {
            None
        };
        let span = Span::new(
            start,
            init.as_ref()
                .map(|e| e.span().end)
                .unwrap_or(self.prev_end()),
        );
        let statement = if is_const {
            Stmt::ConstDeclaration {
                name,
                type_ref,
                init,
                span,
            }
        } else {
            Stmt::LetDeclaration {
                name,
                type_ref,
                init,
                span,
            }
        };
        Member::Statement { statement, span }
    }
}
