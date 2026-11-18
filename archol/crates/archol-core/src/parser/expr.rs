//! Expressions and types.

use crate::ast::Expr;
use crate::span::Span;
use crate::token::TokenKind::{
    self, AndAnd, Arrow, Assign, Bang, Eq, Gt, GtEq, Ident, Lt, LtEq, Minus, NotEq, OrOr, Percent,
    Plus, Slash, Star, StrictEq, StrictNotEq,
};

use super::Parser;

impl Parser<'_> {
    pub(crate) fn parse_expr(&mut self, newline_sensitive: bool) -> Expr {
        if !self.enter_depth() {
            let span = self.cur().span;
            return Expr::Raw {
                text: self.text(span).to_string(),
                span,
            };
        }
        let left = self.parse_unary(newline_sensitive);
        let result = self.parse_binary_rest(left, 0, newline_sensitive);
        self.leave_depth();
        result
    }

    fn parse_binary_rest(&mut self, left: Expr, min_prec: u8, newline_sensitive: bool) -> Expr {
        let mut left = left;
        loop {
            if newline_sensitive
                && self.line_of(self.cur().span.start) > self.line_of(left.span().end)
            {
                break;
            }
            let (op, prec) = match self.binary_op() {
                Some(pair) => pair,
                None => break,
            };
            if prec < min_prec {
                break;
            }
            self.advance();
            let right = self.parse_unary(false);
            let right = self.parse_binary_rest(right, prec + 1, false);
            let span = left.span().merge(right.span());
            left = Expr::Binary {
                op: op.to_string(),
                left: Box::new(left),
                right: Box::new(right),
                span,
            };
        }
        left
    }

    fn binary_op(&self) -> Option<(&'static str, u8)> {
        if self.kind() == Ident {
            let text = self.text(self.cur().span);
            let op = match text {
                "and" => Some(("and", 2)),
                "or" => Some(("or", 1)),
                _ => None,
            };
            if op.is_some() {
                return op;
            }
        }
        let op = match self.kind() {
            Assign => ("=", 1),
            OrOr => ("||", 2),
            AndAnd => ("&&", 3),
            TokenKind::Keyword(crate::token::Keyword::In) => ("in", 4),
            Arrow => ("->", 1),
            Eq => ("==", 5),
            StrictEq => ("===", 5),
            NotEq => ("!=", 5),
            StrictNotEq => ("!==", 5),
            Lt => ("<", 6),
            Gt => (">", 6),
            LtEq => ("<=", 6),
            GtEq => (">=", 6),
            Plus => ("+", 7),
            Minus => ("-", 7),
            Star => ("*", 8),
            Slash => ("/", 8),
            Percent => ("%", 8),
            TokenKind::Keyword(crate::token::Keyword::Where) => ("where", 1),
            TokenKind::QuestionQuestion => ("??", 1),
            _ => return None,
        };
        Some(op)
    }

    fn parse_unary(&mut self, newline_sensitive: bool) -> Expr {
        if !self.enter_depth() {
            let span = self.cur().span;
            return Expr::Raw {
                text: self.text(span).to_string(),
                span,
            };
        }
        let result = self.parse_unary_inner(newline_sensitive);
        self.leave_depth();
        result
    }

    fn parse_unary_inner(&mut self, newline_sensitive: bool) -> Expr {
        let (op, is_unary) = match self.kind() {
            Bang => ("!", true),
            Minus => ("-", true),
            _ => ("", false),
        };
        if is_unary {
            let start = self.cur().span.start;
            self.advance();
            let operand = self.parse_unary(newline_sensitive);
            let span = Span::new(start, operand.span().end);
            return Expr::Unary {
                op: op.to_string(),
                operand: Box::new(operand),
                span,
            };
        }
        self.parse_postfix(newline_sensitive)
    }

    pub(crate) fn prev_end(&self) -> usize {
        self.tokens
            .get(self.pos.saturating_sub(1))
            .map(|t| t.span.end)
            .unwrap_or(0)
    }
}
