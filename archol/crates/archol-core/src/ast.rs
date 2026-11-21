//! Archol AST, serializable to JSON in the style of the TypeScript AST.
//!
//! Every node has `kind` (node name, e.g. `ArtifactDocument`, `PropertyAssignment`,
//! `Block`) and `span` (`start`/`end` in bytes). The AST explorer and the LSP
//! consume exactly this structure.

use std::collections::BTreeMap;

use crate::span::Span;
use serde::Serialize;

/// File root: list of top-level artifacts.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceFile {
    pub kind: &'static str,
    pub path: String,
    pub items: Vec<Artifact>,
    /// TODO/FIXME/REVIEW/OPTIMIZE/HACK comments found in the file.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub pending: Vec<Pending>,
}

impl SourceFile {
    pub fn new(path: impl Into<String>, items: Vec<Artifact>) -> Self {
        Self {
            kind: "SourceFile",
            path: path.into(),
            items,
            pending: Vec::new(),
        }
    }
}

/// Pending comment: `// TODO [2027-01-31] fix this`.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Pending {
    pub kind: &'static str,
    /// Canonical uppercase keyword (`TODO`, `FIXME`, `REVIEW`, `OPTIMIZE`, `HACK`).
    pub keyword: String,
    /// Valid due date (`YYYY-MM-DD`), when present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_date: Option<String>,
    /// Raw text inside `[...]`, including invalid dates.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_text: Option<String>,
    pub message: String,
    #[serde(rename = "loc")]
    pub span: Span,
    /// Span of the keyword itself, used by `archol format`.
    #[serde(rename = "keywordLoc")]
    pub keyword_span: Span,
}

/// Kind of a top-level artifact. Its `kind` in JSON is the typed node name
/// (`ArtifactDocument`, `ArtifactRole`, ...), one per artifact.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ArtifactKind {
    Application,
    Service,
    Document,
    Process,
    View,
    Form,
    Message,
    Shape,
    Role,
    Organogram,
    Policy,
    Decision,
    Integration,
    Parameter,
    Secret,
    Icon,
    Manual,
    Permission,
    Error,
}

impl ArtifactKind {
    /// Every artifact kind, in source-keyword order.
    pub const ALL: &'static [ArtifactKind] = &[
        ArtifactKind::Application,
        ArtifactKind::Service,
        ArtifactKind::Document,
        ArtifactKind::Process,
        ArtifactKind::View,
        ArtifactKind::Form,
        ArtifactKind::Message,
        ArtifactKind::Shape,
        ArtifactKind::Role,
        ArtifactKind::Organogram,
        ArtifactKind::Policy,
        ArtifactKind::Decision,
        ArtifactKind::Integration,
        ArtifactKind::Parameter,
        ArtifactKind::Secret,
        ArtifactKind::Icon,
        ArtifactKind::Manual,
        ArtifactKind::Permission,
        ArtifactKind::Error,
    ];

    /// Resolve the artifact name used by the parser (`Document`, `Role`, ...).
    pub fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "Application" => ArtifactKind::Application,
            "Service" => ArtifactKind::Service,
            "Document" => ArtifactKind::Document,
            "Process" => ArtifactKind::Process,
            "View" => ArtifactKind::View,
            "Form" => ArtifactKind::Form,
            "Message" => ArtifactKind::Message,
            "Shape" => ArtifactKind::Shape,
            "Role" => ArtifactKind::Role,
            "Organogram" => ArtifactKind::Organogram,
            "Policy" => ArtifactKind::Policy,
            "Decision" => ArtifactKind::Decision,
            "Integration" => ArtifactKind::Integration,
            "Parameter" => ArtifactKind::Parameter,
            "Secret" => ArtifactKind::Secret,
            "Icon" => ArtifactKind::Icon,
            "Manual" => ArtifactKind::Manual,
            "Permission" => ArtifactKind::Permission,
            "Error" => ArtifactKind::Error,
            _ => return None,
        })
    }

    /// Artifact name without the `Artifact` prefix (`Document`, `Role`, ...).
    pub fn name(self) -> &'static str {
        match self {
            ArtifactKind::Application => "Application",
            ArtifactKind::Service => "Service",
            ArtifactKind::Document => "Document",
            ArtifactKind::Process => "Process",
            ArtifactKind::View => "View",
            ArtifactKind::Form => "Form",
            ArtifactKind::Message => "Message",
            ArtifactKind::Shape => "Shape",
            ArtifactKind::Role => "Role",
            ArtifactKind::Organogram => "Organogram",
            ArtifactKind::Policy => "Policy",
            ArtifactKind::Decision => "Decision",
            ArtifactKind::Integration => "Integration",
            ArtifactKind::Parameter => "Parameter",
            ArtifactKind::Secret => "Secret",
            ArtifactKind::Icon => "Icon",
            ArtifactKind::Manual => "Manual",
            ArtifactKind::Permission => "Permission",
            ArtifactKind::Error => "Error",
        }
    }

    /// Typed node name carried by `kind` in JSON (`ArtifactDocument`, ...).
    pub fn node_kind(self) -> &'static str {
        match self {
            ArtifactKind::Application => "ArtifactApplication",
            ArtifactKind::Service => "ArtifactService",
            ArtifactKind::Document => "ArtifactDocument",
            ArtifactKind::Process => "ArtifactProcess",
            ArtifactKind::View => "ArtifactView",
            ArtifactKind::Form => "ArtifactForm",
            ArtifactKind::Message => "ArtifactMessage",
            ArtifactKind::Shape => "ArtifactShape",
            ArtifactKind::Role => "ArtifactRole",
            ArtifactKind::Organogram => "ArtifactOrganogram",
            ArtifactKind::Policy => "ArtifactPolicy",
            ArtifactKind::Decision => "ArtifactDecision",
            ArtifactKind::Integration => "ArtifactIntegration",
            ArtifactKind::Parameter => "ArtifactParameter",
            ArtifactKind::Secret => "ArtifactSecret",
            ArtifactKind::Icon => "ArtifactIcon",
            ArtifactKind::Manual => "ArtifactManual",
            ArtifactKind::Permission => "ArtifactPermission",
            ArtifactKind::Error => "ArtifactError",
        }
    }
}

impl Serialize for ArtifactKind {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.node_kind())
    }
}

/// Top-level declaration: `document Invoice { ... }`.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Artifact {
    /// Typed node name (`ArtifactDocument`, `ArtifactRole`, ...).
    pub kind: ArtifactKind,
    pub name: String,
    pub params: Vec<Param>,
    pub body: Vec<Member>,
    #[serde(rename = "loc")]
    pub span: Span,
    /// Stable identifier from the application lock (assigned by the semantic pass).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Derived roles of an organogram: manages/inherits/permissions resolved for
    /// the builder. Empty for every other artifact and in the syntax-only AST.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub roles: Vec<OrganogramRole>,
    /// Every artifact owned by a service, for a builder-side service index.
    /// Present on `ArtifactService` after the semantic pass.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub artifacts: Vec<ArtifactPointer>,
    /// Permissions auto-created for a service from its roles' `permissions`.
    /// Present on `ArtifactService` after the semantic pass.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub permissions: Vec<ArtifactPermission>,
}

impl Artifact {
    pub fn new(
        kind: ArtifactKind,
        name: impl Into<String>,
        params: Vec<Param>,
        body: Vec<Member>,
        span: Span,
    ) -> Self {
        Self {
            kind,
            name: name.into(),
            params,
            body,
            span,
            id: None,
            roles: Vec::new(),
            artifacts: Vec::new(),
            permissions: Vec::new(),
        }
    }

    /// Artifact name without the `Artifact` prefix (`Document`, `Role`, ...).
    pub fn artifact_kind(&self) -> &'static str {
        self.kind.name()
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
    #[serde(rename = "loc")]
    pub span: Span,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

/// Item in the body of an artifact or block.
#[derive(Clone, Debug, Serialize)]
#[serde(tag = "kind")]
pub enum Member {
    /// `title: { ... }`, `schedule: "0 3 1 * *"`.
    PropertyAssignment {
        name: String,
        /// Semantic type when the compiler recognizes one (`I18N`).
        #[serde(rename = "typeRef", skip_serializing_if = "Option::is_none")]
        type_ref: Option<String>,
        value: Expr,
        #[serde(rename = "loc")]
        span: Span,
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
    },
    /// `fields { ... }`, `on Order.create(...) { ... }`, `end Closed { ... }`.
    Block {
        keyword: Option<String>,
        name: String,
        body: Vec<Member>,
        #[serde(rename = "loc")]
        span: Span,
    },
    /// `hierarchy { Manager > Clerk > Customer }`: ordered reporting chains.
    Hierarchy {
        chains: Vec<HierarchyChain>,
        #[serde(rename = "loc")]
        span: Span,
    },
    /// `value SIMPLE { ... }` inside a classification/workflow shape.
    EnumValueDeclaration {
        name: String,
        body: Vec<Member>,
        #[serde(rename = "loc")]
        span: Span,
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
    },
    /// `{ fiscalPeriod }` as an argument: key without an explicit value.
    ShorthandProperty {
        name: String,
        #[serde(rename = "loc")]
        span: Span,
    },
    /// `"example.com/services/orders" as Orders` inside an application's `services`.
    ServiceEntry {
        uri: String,
        alias: String,
        #[serde(rename = "loc")]
        span: Span,
        /// Service declaration id from the application lock.
        #[serde(rename = "ref", skip_serializing_if = "Option::is_none")]
        target: Option<String>,
    },
    /// Process statement (`if`, `try`, `return`, call, ...).
    Statement {
        statement: Stmt,
        #[serde(rename = "loc")]
        span: Span,
    },
    /// `service "example.com/services/orders" as Orders` inside `imports`.
    ImportDeclaration {
        uri: String,
        alias: String,
        #[serde(rename = "loc")]
        span: Span,
        /// Service declaration id from the application lock.
        #[serde(rename = "ref", skip_serializing_if = "Option::is_none")]
        target: Option<String>,
    },
    /// Trigger without a block: `on [Order.Created]`, `by Customer`.
    TriggerDeclaration {
        keyword: String,
        targets: Vec<Expr>,
        #[serde(rename = "loc")]
        span: Span,
    },
}

impl Member {
    pub fn span(&self) -> Span {
        match self {
            Member::PropertyAssignment { span, .. }
            | Member::Block { span, .. }
            | Member::Hierarchy { span, .. }
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
            Member::TriggerDeclaration { .. }
            | Member::Statement { .. }
            | Member::Hierarchy { .. } => None,
        }
    }
}

/// One reporting chain: `Manager > Clerk > Customer`.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HierarchyChain {
    pub kind: &'static str,
    pub roles: Vec<RoleRef>,
    #[serde(rename = "loc")]
    pub span: Span,
}

/// One role occurrence inside a hierarchy chain. After the semantic pass,
/// `ref` is the role's lock id and `service`/`path` locate its declaration.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoleRef {
    pub kind: &'static str,
    pub name: String,
    #[serde(rename = "ref", skip_serializing_if = "Option::is_none")]
    pub role_ref: Option<String>,
    #[serde(rename = "refKind")]
    pub ref_kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// 1-based line of the role declaration, filled by the semantic pass.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
    #[serde(rename = "loc")]
    pub span: Span,
}

/// A role projected into its organogram: own data plus the hierarchy-derived
/// `manages` and the `inherits` declared by the role artifact.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrganogramRole {
    pub kind: &'static str,
    pub name: String,
    #[serde(rename = "ref")]
    pub role_ref: String,
    #[serde(rename = "refKind")]
    pub ref_kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<String>,
    pub path: String,
    /// 1-based line of the role declaration.
    pub line: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<Expr>,
    pub permissions: Vec<PermissionPointer>,
    pub inherits: Vec<RolePointer>,
    pub manages: Vec<RolePointer>,
}

/// Fast builder pointer to another role.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RolePointer {
    pub kind: &'static str,
    pub name: String,
    #[serde(rename = "ref")]
    pub role_ref: String,
    #[serde(rename = "refKind")]
    pub ref_kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<String>,
    pub path: String,
    /// 1-based line of the role declaration.
    pub line: usize,
}

/// Fast builder pointer to a permission artifact.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionPointer {
    pub kind: &'static str,
    pub name: String,
    #[serde(rename = "ref")]
    pub permission_ref: String,
    #[serde(rename = "refKind")]
    pub ref_kind: &'static str,
    /// 1-based line of the permission usage that created the artifact.
    pub line: usize,
}

/// One artifact of a service, with its typed kind and lock id.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactPointer {
    pub kind: &'static str,
    /// Typed artifact kind (`ArtifactDocument`, `ArtifactPermission`, ...).
    pub artifact: ArtifactKind,
    pub name: String,
    #[serde(rename = "ref")]
    pub artifact_ref: String,
    pub path: String,
    /// 1-based line of the declaration (or first usage, for permissions).
    pub line: usize,
}

/// Permission auto-created for a service from its roles' `permissions`.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactPermission {
    pub kind: &'static str,
    pub name: String,
    pub namespace: String,
    pub action: String,
    #[serde(rename = "ref")]
    pub permission_ref: String,
    pub path: String,
    /// 1-based line of the first usage that created the permission.
    pub line: usize,
    #[serde(rename = "loc")]
    pub span: Span,
}

/// Expression (right-hand side of a property or argument).
#[derive(Clone, Debug, Serialize)]
#[serde(tag = "kind")]
pub enum Expr {
    Identifier {
        name: String,
        #[serde(rename = "loc")]
        span: Span,
    },
    StringLiteral {
        value: String,
        raw: String,
        #[serde(rename = "loc")]
        span: Span,
    },
    NumberLiteral {
        value: String,
        #[serde(rename = "loc")]
        span: Span,
    },
    BooleanLiteral {
        value: bool,
        #[serde(rename = "loc")]
        span: Span,
    },
    NullLiteral {
        #[serde(rename = "loc")]
        span: Span,
    },
    List {
        elements: Vec<Expr>,
        #[serde(rename = "loc")]
        span: Span,
    },
    /// `{ pt-BR: "..." }` or `Store { code: ... }` (typed object).
    Object {
        #[serde(rename = "typeRef")]
        type_ref: Option<String>,
        #[serde(rename = "typeRefId", skip_serializing_if = "Option::is_none")]
        type_ref_id: Option<String>,
        members: Vec<Member>,
        #[serde(rename = "loc")]
        span: Span,
    },
    /// Named argument in a call: `send(key: invoice, ...)`.
    NamedArgument {
        name: String,
        value: Box<Expr>,
        #[serde(rename = "loc")]
        span: Span,
    },
    /// `FiscalPeriod.consolidate({ ... })`.
    Call {
        callee: String,
        args: Vec<Expr>,
        #[serde(rename = "loc")]
        span: Span,
    },
    /// Localized value of a Shape: `title: { pt-BR: "..." }`.
    ShapeValue {
        #[serde(rename = "typeRef")]
        type_ref: String,
        value: BTreeMap<String, ShapeData>,
        #[serde(rename = "loc")]
        loc: Span,
    },
    /// Parsed literal of a Shape: `"30d"` assigned to `std.Duration`.
    ShapeLiteral {
        /// Shape reference as written (`std.Duration`).
        shape: String,
        /// Literal text without quotes.
        raw: String,
        /// Parsed value; shape-dependent (number, text, or object).
        value: ShapeData,
        /// Base unit after normalization, for quantity shapes.
        #[serde(skip_serializing_if = "Option::is_none")]
        unit: Option<String>,
        #[serde(rename = "loc")]
        span: Span,
    },
    /// `a.b.c`.
    MemberAccess {
        object: Box<Expr>,
        name: String,
        #[serde(rename = "loc")]
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
        #[serde(rename = "loc")]
        span: Span,
    },
    Binary {
        op: String,
        left: Box<Expr>,
        right: Box<Expr>,
        #[serde(rename = "loc")]
        span: Span,
    },
    Unary {
        op: String,
        operand: Box<Expr>,
        #[serde(rename = "loc")]
        span: Span,
    },
    /// Tolerant fallback for expressions the parser does not model.
    Raw {
        text: String,
        #[serde(rename = "loc")]
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
            | Expr::ShapeValue { loc: span, .. }
            | Expr::ShapeLiteral { span, .. }
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

/// Parsed value of a `ShapeLiteral`, serialized untagged.
#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
pub enum ShapeData {
    Integer(i64),
    Decimal(f64),
    Text(String),
    Object(BTreeMap<String, ShapeData>),
    List(Vec<ShapeData>),
}

/// Process statement (TypeScript-style code).
#[derive(Clone, Debug, Serialize)]
#[serde(tag = "kind")]
pub enum Stmt {
    ExpressionStatement {
        expression: Expr,
        #[serde(rename = "loc")]
        span: Span,
    },
    ConstDeclaration {
        name: String,
        #[serde(rename = "typeRef")]
        type_ref: Option<String>,
        init: Option<Expr>,
        #[serde(rename = "loc")]
        span: Span,
    },
    LetDeclaration {
        name: String,
        #[serde(rename = "typeRef")]
        type_ref: Option<String>,
        init: Option<Expr>,
        #[serde(rename = "loc")]
        span: Span,
    },
    IfStatement {
        condition: Expr,
        #[serde(rename = "then")]
        then_branch: Vec<Member>,
        #[serde(rename = "else")]
        else_branch: Option<Vec<Member>>,
        #[serde(rename = "loc")]
        span: Span,
    },
    TryStatement {
        body: Vec<Member>,
        catches: Vec<CatchClause>,
        #[serde(rename = "loc")]
        span: Span,
    },
    ReturnStatement {
        value: Option<Expr>,
        #[serde(rename = "loc")]
        span: Span,
    },
    RawStatement {
        tokens: String,
        #[serde(rename = "loc")]
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
    #[serde(rename = "loc")]
    pub span: Span,
}
