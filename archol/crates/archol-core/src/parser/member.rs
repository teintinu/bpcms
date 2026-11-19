//! Members: properties, blocks, enum values, imports and triggers.

use crate::ast::Member;
use crate::span::Span;
use crate::token::{Keyword, TokenKind};

use super::{unquote, Parser};

impl Parser<'_> {
    // --- members ---------------------------------------------------------

    pub(crate) fn parse_member(&mut self) -> Option<Member> {
        if self.kind() == TokenKind::String {
            return Some(self.parse_service_entry());
        }
        match self.kind() {
            TokenKind::Keyword(Keyword::Service) if self.peek(1) == TokenKind::String => {
                return Some(self.parse_import())
            }
            TokenKind::Keyword(Keyword::On)
            | TokenKind::Keyword(Keyword::By)
            | TokenKind::Keyword(Keyword::Called)
                if self.peek(1) != TokenKind::Colon =>
            {
                return Some(self.parse_trigger_or_block())
            }
            TokenKind::Keyword(Keyword::If) => return Some(self.parse_if()),
            TokenKind::Keyword(Keyword::Try) => return Some(self.parse_try()),
            TokenKind::Keyword(Keyword::Return) => return Some(self.parse_return()),
            TokenKind::Keyword(Keyword::Const) => return Some(self.parse_var(true)),
            TokenKind::Keyword(Keyword::Let) => return Some(self.parse_var(false)),
            TokenKind::Keyword(Keyword::Value)
                if self.peek(1) == TokenKind::Ident && self.peek(2) == TokenKind::LBrace =>
            {
                return Some(self.parse_enum_value())
            }
            TokenKind::Keyword(Keyword::Transition) | TokenKind::Keyword(Keyword::End)
                if self.peek(1) == TokenKind::Ident && self.peek(2) == TokenKind::LBrace =>
            {
                return Some(self.parse_named_keyword_block());
            }
            _ => {}
        }

        if let Some(member) = self.try_parse_property() {
            return Some(member);
        }
        if self.is_block_start() {
            return Some(self.parse_block_member());
        }
        if let Some(member) = self.try_parse_header_block() {
            return Some(member);
        }
        Some(self.parse_expression_statement())
    }

    /// `name: value`, including hyphenated keys (`pt-BR: "..."`).
    /// `"example.com/services/orders" as Orders` (application `services` block).
    fn parse_service_entry(&mut self) -> Member {
        let start = self.cur().span.start;
        let uri_token = self.advance();
        let uri = unquote(self.text(uri_token.span)).to_string();
        let mut alias = String::new();
        if self.kind() == TokenKind::Keyword(Keyword::As) {
            self.advance();
            if self.kind() == TokenKind::Ident {
                let alias_token = self.advance();
                alias = self.text(alias_token.span).to_string();
            }
        }
        let span = Span::new(start, self.prev_end());
        Member::ServiceEntry {
            uri,
            alias,
            span,
            target: None,
        }
    }

    fn try_parse_property(&mut self) -> Option<Member> {
        if !matches!(self.kind(), TokenKind::Ident | TokenKind::Keyword(_)) {
            return None;
        }
        let start = self.cur().span.start;
        let mut name = self.text(self.cur().span).to_string();
        let mut lookahead = 1;
        while self.peek(lookahead) == TokenKind::Minus
            && self.peek(lookahead + 1) == TokenKind::Ident
        {
            name.push('-');
            name.push_str(&self.token_text_at(self.pos + lookahead + 1));
            lookahead += 2;
        }
        if self.peek(lookahead) != TokenKind::Colon {
            return None;
        }
        for _ in 0..lookahead {
            self.advance();
        }
        self.expect(TokenKind::Colon, "E_PARSE_COLON");
        let value = self.parse_expr(true);
        let end = value.span().end;
        let type_ref = super::property_type_ref(&name).map(str::to_string);
        Some(Member::PropertyAssignment {
            name,
            type_ref,
            value,
            span: Span::new(start, end),
            id: None,
        })
    }

    pub(crate) fn token_text_at(&self, index: usize) -> String {
        self.tokens
            .get(index)
            .map(|t| self.text(t.span).to_string())
            .unwrap_or_default()
    }

    fn is_block_start(&self) -> bool {
        matches!(self.kind(), TokenKind::Ident | TokenKind::Keyword(_))
            && self.peek(1) == TokenKind::LBrace
    }

    fn parse_block_member(&mut self) -> Member {
        let start = self.cur().span.start;
        let name_token = self.advance();
        let name = self.text(name_token.span).to_string();
        let body = self.parse_body();
        let end = self.prev_end();
        Member::Block {
            keyword: None,
            name,
            body,
            span: Span::new(start, end),
        }
    }

    fn parse_enum_value(&mut self) -> Member {
        let start = self.cur().span.start;
        self.advance();
        let name = if self.kind() == TokenKind::Ident {
            let token = self.advance();
            self.text(token.span).to_string()
        } else {
            self.error("E_PARSE_ENUM_VALUE", "Expected value name".to_string());
            String::new()
        };
        let body = self.parse_body();
        let end = self.prev_end();
        Member::EnumValueDeclaration {
            name,
            body,
            span: Span::new(start, end),
            id: None,
        }
    }

    fn parse_named_keyword_block(&mut self) -> Member {
        let start = self.cur().span.start;
        let keyword = self.advance();
        let keyword_text = self.text(keyword.span).to_string();
        let name = if self.kind() == TokenKind::Ident {
            let token = self.advance();
            self.text(token.span).to_string()
        } else {
            String::new()
        };
        let body = self.parse_body();
        let end = self.prev_end();
        Member::Block {
            keyword: Some(keyword_text),
            name,
            body,
            span: Span::new(start, end),
        }
    }

    fn parse_import(&mut self) -> Member {
        let start = self.cur().span.start;
        self.advance(); // service
        let uri_token = self.advance();
        let uri = unquote(self.text(uri_token.span)).to_string();
        let mut alias = String::new();
        if self.kind() == TokenKind::Keyword(Keyword::As) {
            self.advance();
            if self.kind() == TokenKind::Ident {
                let token = self.advance();
                alias = self.text(token.span).to_string();
            }
        }
        let span = Span::new(start, self.prev_end());
        Member::ImportDeclaration {
            uri,
            alias,
            span,
            target: None,
        }
    }

    fn parse_trigger_or_block(&mut self) -> Member {
        if let Some(block) = self.try_parse_header_block() {
            return block;
        }
        let start = self.cur().span.start;
        let keyword_token = self.advance();
        let keyword = self.text(keyword_token.span).to_string();
        let same_line = !self.at_eof()
            && self.line_of(self.cur().span.start) == self.line_of(start)
            && self.kind() != TokenKind::RBrace;
        let targets = if same_line {
            vec![self.parse_expr(true)]
        } else {
            Vec::new()
        };
        let end = targets
            .last()
            .map(|e| e.span().end)
            .unwrap_or(self.prev_end());
        Member::TriggerDeclaration {
            keyword,
            targets,
            span: Span::new(start, end),
        }
    }

    /// Generic header with a block: `open(a: string) {`, `loop X max 2 {`,
    /// `operator +(a: A, b: B): C {`, `send Message {`.
    pub(crate) fn try_parse_header_block(&mut self) -> Option<Member> {
        let mut depth = 0i32;
        let mut index = 0usize;
        let start_line = self.line_of(self.cur().span.start);
        loop {
            let kind = self.peek(index);
            match kind {
                TokenKind::Eof | TokenKind::RBrace | TokenKind::Semicolon => return None,
                TokenKind::LBrace if depth == 0 => break,
                TokenKind::LParen | TokenKind::LBracket | TokenKind::LBrace => depth += 1,
                TokenKind::RParen | TokenKind::RBracket => depth -= 1,
                _ => {}
            }
            if depth == 0 {
                let offset = self.tokens.get(self.pos + index)?.span.start;
                if self.line_of(offset) != start_line {
                    return None;
                }
            }
            index += 1;
            if index > 64 {
                return None;
            }
        }
        let start = self.cur().span.start;
        let brace_start = self.tokens[self.pos + index].span.start;
        let label = self.source[start..brace_start].trim().to_string();
        for _ in 0..index {
            self.advance();
        }
        let body = self.parse_body();
        let end = self.prev_end();
        Some(Member::Block {
            keyword: None,
            name: label,
            body,
            span: Span::new(start, end),
        })
    }

    /// Type of the token immediately after the `)` that closes the parenthesis
    /// opened at `self.pos + 1` (used to tell a call from a signature).
    pub(crate) fn token_after_matching_paren(&self) -> Option<TokenKind> {
        let mut depth = 0i32;
        let mut index = 0usize;
        loop {
            let kind = self.peek(index);
            match kind {
                TokenKind::Eof => return None,
                TokenKind::LParen => depth += 1,
                TokenKind::RParen => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(self.peek(index + 1));
                    }
                }
                _ => {}
            }
            index += 1;
            if index > 128 {
                return None;
            }
        }
    }

    /// `quote(a: A, b: B): Return` — signature without a body.
    fn signature_statement(&mut self, start: usize) -> Member {
        let line = self.line_of(start);
        while !self.at_eof()
            && self.kind() != TokenKind::RBrace
            && self.line_of(self.cur().span.start) == line
        {
            self.advance();
        }
        let span = Span::new(start, self.prev_end().max(start));
        Member::Statement {
            statement: crate::ast::Stmt::RawStatement {
                tokens: self.text(span).to_string(),
                span,
            },
            span,
        }
    }

    pub(crate) fn raw_statement(&mut self, start: usize) -> Member {
        let line = self.line_of(start);
        let mut depth = 0i32;
        while !self.at_eof() {
            let kind = self.kind();
            if depth == 0 && self.line_of(self.cur().span.start) != line {
                break;
            }
            match kind {
                TokenKind::RBrace if depth == 0 => break,
                TokenKind::LBrace | TokenKind::LParen | TokenKind::LBracket => depth += 1,
                TokenKind::RBrace | TokenKind::RParen | TokenKind::RBracket => depth -= 1,
                _ => {}
            }
            self.advance();
        }
        let span = Span::new(start, self.prev_end().max(start));
        Member::Statement {
            statement: crate::ast::Stmt::RawStatement {
                tokens: self.text(span).to_string(),
                span,
            },
            span,
        }
    }

    fn parse_expression_statement(&mut self) -> Member {
        let start = self.cur().span.start;
        if matches!(self.kind(), TokenKind::Ident)
            && self.peek(1) == TokenKind::LParen
            && self.token_after_matching_paren() == Some(TokenKind::Colon)
        {
            return self.signature_statement(start);
        }
        let expression = self.parse_expr(true);
        // Operation signature without a body: `quote(a: A): std.Money`.
        if self.kind() == TokenKind::Colon {
            self.advance();
            let _ = self.parse_type_ref();
            let line = self.line_of(start);
            while !self.at_eof()
                && self.kind() != TokenKind::RBrace
                && self.line_of(self.cur().span.start) == line
            {
                self.advance();
            }
            let span = Span::new(start, self.prev_end());
            return Member::Statement {
                statement: crate::ast::Stmt::RawStatement {
                    tokens: self.text(span).to_string(),
                    span,
                },
                span,
            };
        }
        let end = expression.span().end;
        Member::Statement {
            statement: crate::ast::Stmt::ExpressionStatement {
                expression,
                span: Span::new(start, end),
            },
            span: Span::new(start, end),
        }
    }
}
