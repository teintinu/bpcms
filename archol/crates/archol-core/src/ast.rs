//! Archol AST, serializable to JSON in the style of the TypeScript AST.
//!
//! Every node has `kind` (node name, e.g. `Artifact`, `PropertyAssignment`,
//! `Block`) and `span` (`start`/`end` in bytes). The AST explorer and the LSP
//! consume exactly this structure.

use crate::span::Span;
use serde::Serialize;

/// File root: list of top-level artifacts.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceFile {
    pub kind: &'static str,
    pub path: String,
    pub items: Vec<Artifact>,
}

impl SourceFile {
    pub fn new(path: impl Into<String>, items: Vec<Artifact>) -> Self {
        Self {
            kind: "SourceFile",
            path: path.into(),
            items,
        }
    }
}

/// Top-level declaration: `document Invoice { ... }`.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Artifact {
    pub kind: &'static str,
    pub artifact_kind: String,
    pub name: String,
    pub params: Vec<Param>,
    pub body: Vec<Member>,
    pub span: Span,
    /// Stable identifier from the application lock (assigned by the semantic pass).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

impl Artifact {
    pub fn new(
        artifact_kind: impl Into<String>,
        name: impl Into<String>,
        params: Vec<Param>,
        body: Vec<Member>,
        span: Span,
    ) -> Self {
        Self {
            kind: "Artifact",
            artifact_kind: artifact_kind.into(),
            name: name.into(),
            params,
            body,
            span,
            id: None,
        }
    }
}

/// Artifact parameter: `process MonthClose(fiscalPeriod: FiscalPeriod)`.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Param {
    pub name: String,
    #[serde(rename = "typeRef")]
    pub type_ref: Option<String>,
    #[serde(rename = "typeRefId", skip_serializing_if = "Option::is_none")]
    pub type_ref_id: Option<String>,
    pub span: Span,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

/// Item in the body of an artifact or block.
#[derive(Clone, Debug, Serialize)]
#[serde(tag = "kind")]
pub enum Member {
    /// `title: { ... }`, `version: 1`, `schedule: "0 3 1 * *"`.
    PropertyAssignment {
        name: String,
        value: Expr,
        span: Span,
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
    },
    /// `fields { ... }`, `on Order.create(...) { ... }`, `end Closed { ... }`.
    Block {
        keyword: Option<String>,
        name: String,
        body: Vec<Member>,
        span: Span,
    },
    /// `value SIMPLE { ... }` inside a classification/workflow shape.
    EnumValueDeclaration {
        name: String,
        body: Vec<Member>,
        span: Span,
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
    },
    /// `{ fiscalPeriod }` as an argument: key without an explicit value.
    ShorthandProperty { name: String, span: Span },
    /// `"example.com/services/orders" as Orders` inside an application's `services`.
    ServiceEntry {
        uri: String,
        alias: String,
        span: Span,
        /// Service declaration id from the application lock.
        #[serde(rename = "ref", skip_serializing_if = "Option::is_none")]
        target: Option<String>,
    },
    /// Process statement (`if`, `try`, `return`, call, ...).
    Statement { statement: Stmt, span: Span },
    /// `service "example.com/services/orders" as Orders` inside `imports`.
    ImportDeclaration {
        uri: String,
        alias: String,
        span: Span,
        /// Service declaration id from the application lock.
        #[serde(rename = "ref", skip_serializing_if = "Option::is_none")]
        target: Option<String>,
    },
    /// Trigger without a block: `on [Order.Created]`, `by Customer`.
    TriggerDeclaration {
        keyword: String,
        targets: Vec<Expr>,
        span: Span,
    },
}

impl Member {
    pub fn span(&self) -> Span {
        match self {
            Member::PropertyAssignment { span, .. }
            | Member::Block { span, .. }
            | Member::EnumValueDeclaration { span, .. }
            | Member::ShorthandProperty { span, .. }
            | Member::Statement { span, .. }
            | Member::ImportDeclaration { span, .. }
            | Member::ServiceEntry { span, .. }
            | Member::TriggerDeclaration { span, .. } => *span,
        }
    }

    pub fn name(&self) -> Option<&str> {
        match self {
            Member::PropertyAssignment { name, .. }
            | Member::Block { name, .. }
            | Member::EnumValueDeclaration { name, .. }
            | Member::ShorthandProperty { name, .. } => Some(name),
            Member::ImportDeclaration { alias, .. } | Member::ServiceEntry { alias, .. } => {
                Some(alias)
            }
            Member::TriggerDeclaration { .. } | Member::Statement { .. } => None,
        }
    }
}

/// Expression (right-hand side of a property or argument).
#[derive(Clone, Debug, Serialize)]
#[serde(tag = "kind")]
pub enum Expr {
    Identifier {
        name: String,
        span: Span,
    },
    StringLiteral {
        value: String,
        raw: String,
        span: Span,
    },
    NumberLiteral {
        value: String,
        span: Span,
    },
    BooleanLiteral {
        value: bool,
        span: Span,
    },
    NullLiteral {
        span: Span,
    },
    List {
        elements: Vec<Expr>,
        span: Span,
    },
    /// `{ pt-BR: "..." }` or `Store { code: ... }` (typed object).
    Object {
        #[serde(rename = "typeRef")]
        type_ref: Option<String>,
        #[serde(rename = "typeRefId", skip_serializing_if = "Option::is_none")]
        type_ref_id: Option<String>,
        members: Vec<Member>,
        span: Span,
    },
    /// Named argument in a call: `send(key: invoice, ...)`.
    NamedArgument {
        name: String,
        value: Box<Expr>,
        span: Span,
    },
    /// `FiscalPeriod.consolidate({ ... })`.
    Call {
        callee: String,
        args: Vec<Expr>,
        span: Span,
    },
    /// `a.b.c`.
    MemberAccess {
        object: Box<Expr>,
        name: String,
        span: Span,
        /// Field identifier (from the application lock) resolved by the semantic pass.
        #[serde(rename = "fieldRef", skip_serializing_if = "Option::is_none")]
        field_ref: Option<String>,
    },
    /// Resolved identifier reference; `ref` is the lock id of the declaration.
    Reference {
        #[serde(rename = "refKind")]
        ref_kind: String,
        name: String,
        #[serde(rename = "ref")]
        target: String,
        span: Span,
    },
    Binary {
        op: String,
        left: Box<Expr>,
        right: Box<Expr>,
        span: Span,
    },
    Unary {
        op: String,
        operand: Box<Expr>,
        span: Span,
    },
    /// Tolerant fallback for expressions the parser does not model.
    Raw {
        text: String,
        span: Span,
    },
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::Identifier { span, .. }
            | Expr::StringLiteral { span, .. }
            | Expr::NumberLiteral { span, .. }
            | Expr::BooleanLiteral { span, .. }
            | Expr::NullLiteral { span }
            | Expr::List { span, .. }
            | Expr::Object { span, .. }
            | Expr::NamedArgument { span, .. }
            | Expr::Call { span, .. }
            | Expr::MemberAccess { span, .. }
            | Expr::Reference { span, .. }
            | Expr::Binary { span, .. }
            | Expr::Unary { span, .. }
            | Expr::Raw { span, .. } => *span,
        }
    }

    pub fn as_ident(&self) -> Option<&str> {
        match self {
            Expr::Identifier { name, .. } => Some(name),
            _ => None,
        }
    }
}

/// Process statement (TypeScript-style code).
#[derive(Clone, Debug, Serialize)]
#[serde(tag = "kind")]
pub enum Stmt {
    ExpressionStatement {
        expression: Expr,
        span: Span,
    },
    ConstDeclaration {
        name: String,
        #[serde(rename = "typeRef")]
        type_ref: Option<String>,
        init: Option<Expr>,
        span: Span,
    },
    LetDeclaration {
        name: String,
        #[serde(rename = "typeRef")]
        type_ref: Option<String>,
        init: Option<Expr>,
        span: Span,
    },
    IfStatement {
        condition: Expr,
        #[serde(rename = "then")]
        then_branch: Vec<Member>,
        #[serde(rename = "else")]
        else_branch: Option<Vec<Member>>,
        span: Span,
    },
    TryStatement {
        body: Vec<Member>,
        catches: Vec<CatchClause>,
        span: Span,
    },
    ReturnStatement {
        value: Option<Expr>,
        span: Span,
    },
    RawStatement {
        tokens: String,
        span: Span,
    },
}

impl Stmt {
    pub fn span(&self) -> Span {
        match self {
            Stmt::ExpressionStatement { span, .. }
            | Stmt::ConstDeclaration { span, .. }
            | Stmt::LetDeclaration { span, .. }
            | Stmt::IfStatement { span, .. }
            | Stmt::TryStatement { span, .. }
            | Stmt::ReturnStatement { span, .. }
            | Stmt::RawStatement { span, .. } => *span,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CatchClause {
    #[serde(rename = "typeRef")]
    pub type_ref: Option<String>,
    pub body: Vec<Member>,
    pub span: Span,
}
