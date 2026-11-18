//! Primary expressions: literals, lists, typed objects, calls and types.

use crate::ast::{Expr, Member};
use crate::parser::unquote;
use crate::span::Span;
use crate::token::TokenKind::{
    self, Colon, Comma, Dot, Ident, LBrace, LBracket, LParen, Minus, Number, Percent, Pipe, RBrace,
    RBracket, RParen, Semicolon, Slash, Star,
};

use super::Parser;

impl Parser<'_> {
    pub(crate) fn parse_postfix(&mut self, newline_sensitive: bool) -> Expr {
        let mut expr = self.parse_primary();
        loop {
            if newline_sensitive
                && self.line_of(self.cur().span.start) > self.line_of(expr.span().end)
            {
                break;
            }
            match self.kind() {
                Dot => {
                    self.advance();
                    let name = if matches!(self.kind(), Ident | TokenKind::Keyword(_)) {
                        let token = self.advance();
                        self.text(token.span).to_string()
                    } else {
                        break;
                    };
                    if self.kind() == LParen {
                        let args = self.parse_call_args();
                        let callee = format!("{}.{name}", render(&expr));
                        let span = Span::new(expr.span().start, self.prev_end());
                        expr = Expr::Call { callee, args, span };
                    } else {
                        let span = Span::new(expr.span().start, self.prev_end());
                        expr = Expr::MemberAccess {
                            object: Box::new(expr),
                            name,
                            span,
                        };
                    }
                }
                LParen => {
                    let args = self.parse_call_args();
                    let callee = render(&expr);
                    let span = Span::new(expr.span().start, self.prev_end());
                    expr = Expr::Call { callee, args, span };
                }
                LBrace if matches!(expr, Expr::Identifier { .. } | Expr::MemberAccess { .. }) => {
                    let name = render(&expr);
                    let members = self.parse_object_members();
                    let span = Span::new(expr.span().start, self.prev_end());
                    expr = Expr::Object {
                        type_ref: Some(name),
                        members,
                        span,
                    };
                }
                _ => break,
            }
        }
        expr
    }

    fn parse_primary(&mut self) -> Expr {
        let token = *self.cur();
        match token.kind {
            Ident => {
                self.advance();
                let mut name = self.text(token.span).to_string();
                // `InvoiceItem[] { ... }`: list suffix in a type.
                if self.kind() == Slash
                    && self.peek(1) == Star
                    && matches!(self.peek(2), RBracket | Comma | RParen)
                {
                    self.advance();
                    self.advance();
                    name.push_str("/*");
                }
                if self.kind() == LBracket && self.peek(1) == RBracket {
                    self.advance();
                    self.advance();
                    name.push_str("[]");
                }
                Expr::Identifier {
                    name,
                    span: token.span,
                }
            }
            TokenKind::String => {
                self.advance();
                Expr::StringLiteral {
                    value: unquote(self.text(token.span)).to_string(),
                    raw: self.text(token.span).to_string(),
                    span: token.span,
                }
            }
            Number => {
                self.advance();
                let mut value = self.text(token.span).to_string();
                let mut end = token.span.end;
                if self.kind() == Ident
                    && self.line_of(self.cur().span.start) == self.line_of(token.span.start)
                {
                    let unit = self.advance();
                    value.push(' ');
                    value.push_str(self.text(unit.span));
                    end = unit.span.end;
                }
                Expr::NumberLiteral {
                    value,
                    span: Span::new(token.span.start, end),
                }
            }
            TokenKind::Keyword(crate::token::Keyword::True)
            | TokenKind::Keyword(crate::token::Keyword::False) => {
                self.advance();
                Expr::BooleanLiteral {
                    value: token.kind.is_keyword(crate::token::Keyword::True),
                    span: token.span,
                }
            }
            TokenKind::Keyword(crate::token::Keyword::Null) => {
                self.advance();
                Expr::NullLiteral { span: token.span }
            }
            Percent => {
                self.advance();
                Expr::Identifier {
                    name: "%".to_string(),
                    span: token.span,
                }
            }
            TokenKind::Keyword(_) => {
                self.advance();
                Expr::Identifier {
                    name: self.text(token.span).to_string(),
                    span: token.span,
                }
            }
            LBracket => {
                let start = token.span.start;
                self.advance();
                let mut elements = Vec::new();
                while self.kind() != RBracket && !self.at_eof() {
                    elements.push(self.parse_expr(false));
                    if !self.eat(Comma) {
                        break;
                    }
                }
                self.expect(RBracket, "E_PARSE_LIST");
                Expr::List {
                    elements,
                    span: Span::new(start, self.prev_end()),
                }
            }
            LBrace => {
                let start = token.span.start;
                let members = self.parse_object_members();
                Expr::Object {
                    type_ref: None,
                    members,
                    span: Span::new(start, self.prev_end()),
                }
            }
            LParen => {
                let start = token.span.start;
                self.advance();
                let _inner = self.parse_expr(false);
                self.expect(RParen, "E_PARSE_PAREN");
                let span = Span::new(start, self.prev_end());
                Expr::Raw {
                    text: self.text(span).to_string(),
                    span,
                }
            }
            _ => {
                let span = token.span;
                self.error(
                    "E_PARSE_EXPR",
                    format!("Unexpected expression: {:?}", token.kind),
                );
                if !self.at_eof() && self.kind() != RBrace {
                    self.advance();
                }
                Expr::Raw {
                    text: self.text(span).to_string(),
                    span,
                }
            }
        }
    }

    /// Members of `{ ... }` (object literal or type body).
    pub(crate) fn parse_object_members(&mut self) -> Vec<Member> {
        let mut members = Vec::new();
        if !self.enter_depth() {
            return members;
        }
        if !self.eat(LBrace) {
            self.leave_depth();
            return members;
        }
        loop {
            while self.eat(Comma) || self.eat(Semicolon) {}
            if self.kind() == RBrace {
                self.advance();
                break;
            }
            if self.at_eof() {
                self.error("E_PARSE_UNCLOSED", "Unclosed object".to_string());
                break;
            }
            if matches!(self.kind(), Ident)
                && self.peek(1) == LParen
                && self.token_after_matching_paren() == Some(LBrace)
            {
                if let Some(block) = self.try_parse_header_block() {
                    members.push(block);
                    continue;
                }
            }
            if let Some(property) = self.try_parse_property_in_object() {
                members.push(property);
                continue;
            }
            if matches!(self.kind(), Ident | TokenKind::Keyword(_)) {
                let token = self.advance();
                members.push(Member::ShorthandProperty {
                    name: self.text(token.span).to_string(),
                    span: token.span,
                });
                continue;
            }
            self.advance();
        }
        self.leave_depth();
        members
    }

    fn try_parse_property_in_object(&mut self) -> Option<Member> {
        if !matches!(self.kind(), Ident | TokenKind::Keyword(_)) {
            return None;
        }
        let start = self.cur().span.start;
        let mut name = self.text(self.cur().span).to_string();
        let mut lookahead = 1;
        while self.peek(lookahead) == Minus && self.peek(lookahead + 1) == Ident {
            name.push('-');
            name.push_str(&self.token_text_at(self.pos + lookahead + 1));
            lookahead += 2;
        }
        if self.peek(lookahead) != Colon {
            return None;
        }
        for _ in 0..lookahead {
            self.advance();
        }
        self.expect(Colon, "E_PARSE_COLON");
        let value = self.parse_expr(false);
        let end = value.span().end;
        Some(Member::PropertyAssignment {
            name,
            value,
            span: Span::new(start, end),
        })
    }

    fn parse_call_args(&mut self) -> Vec<Expr> {
        let mut args = Vec::new();
        self.expect(LParen, "E_PARSE_CALL");
        while self.kind() != RParen && !self.at_eof() {
            if matches!(self.kind(), Ident) && self.peek(1) == Colon {
                let start = self.cur().span.start;
                let name_token = self.advance();
                let name = self.text(name_token.span).to_string();
                self.advance(); // :
                let value = self.parse_expr(false);
                let span = Span::new(start, value.span().end);
                args.push(Expr::NamedArgument {
                    name,
                    value: Box::new(value),
                    span,
                });
                self.eat(Comma);
                continue;
            }
            args.push(self.parse_expr(false));
            self.eat(Comma);
        }
        self.expect(RParen, "E_PARSE_CALL");
        args
    }

    /// Type name: `Invoice`, `std.Money`, `InvoiceItem[]`.
    pub(crate) fn parse_type_ref(&mut self) -> Option<String> {
        let mut parts = vec![self.parse_type_segment()?];
        while self.kind() == Pipe {
            self.advance();
            match self.parse_type_segment() {
                Some(segment) => parts.push(segment),
                None => break,
            }
        }
        Some(parts.join(" | "))
    }

    fn parse_type_segment(&mut self) -> Option<String> {
        if !matches!(self.kind(), Ident) {
            return None;
        }
        let first = self.advance();
        let mut name = self.text(first.span).to_string();
        while self.kind() == Dot && self.peek(1) == Ident {
            self.advance();
            name.push('.');
            let part = self.advance();
            name.push_str(self.text(part.span));
        }
        if self.kind() == LBracket && self.peek(1) == RBracket {
            self.advance();
            self.advance();
            name.push_str("[]");
        }
        Some(name)
    }
}

fn render(expr: &Expr) -> String {
    match expr {
        Expr::Identifier { name, .. } => name.clone(),
        Expr::MemberAccess { object, name, .. } => format!("{}.{}", render(object), name),
        other => format!("{:?}", other.span()),
    }
}
