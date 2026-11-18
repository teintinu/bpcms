//! Symbol index: artifacts, enums, roles, imports and references.

use std::collections::{BTreeMap, BTreeSet};

use archol_core::{Artifact, Expr, Member};

use crate::workspace::Workspace;

pub const PRIMITIVES: &[&str] = &[
    "string", "decimal", "boolean", "int", "number", "DateTime", "Date", "Time", "Map", "void",
    "any", "null", "richtext", "JSON", "Duration", "Count",
];

#[derive(Clone, Debug)]
pub struct ArtifactInfo {
    pub name: String,
    pub kind: String,
    pub service: Option<String>,
    pub path: String,
    pub span_start: usize,
    pub span_end: usize,
}

#[derive(Clone, Debug)]
pub struct EnumMember {
    pub name: String,
    pub title: Option<String>,
    pub span: (usize, usize),
}

#[derive(Clone, Debug)]
pub struct EnumInfo {
    pub name: String,
    pub service: Option<String>,
    pub path: String,
    pub span_start: usize,
    pub values: Vec<EnumMember>,
}

#[derive(Default)]
pub struct Symbols {
    /// Name -> declarations (the same name may exist in different services).
    pub artifacts: BTreeMap<String, Vec<ArtifactInfo>>,
    /// All enums; names may repeat across services.
    pub enums: Vec<EnumInfo>,
    pub roles: BTreeSet<String>,
    /// Service URI -> import alias -> imported service URI.
    pub imports: BTreeMap<String, BTreeMap<String, String>>,
}

impl Symbols {
    pub fn build(workspace: &Workspace) -> Self {
        let mut symbols = Symbols::default();
        for (file, artifact) in workspace.artifacts() {
            let info = ArtifactInfo {
                name: artifact.name.clone(),
                kind: artifact.artifact_kind.clone(),
                service: file.service.clone(),
                path: file.path.clone(),
                span_start: artifact.span.start,
                span_end: artifact.span.end,
            };
            symbols
                .artifacts
                .entry(artifact.name.clone())
                .or_default()
                .push(info);
            if artifact.artifact_kind == "Role" {
                symbols.roles.insert(artifact.name.clone());
            }
            if artifact.artifact_kind == "Service" {
                let mut map = BTreeMap::new();
                walk_members(&artifact.body, &mut |member| {
                    if let Member::ImportDeclaration { uri, alias, .. } = member {
                        if !alias.is_empty() {
                            map.insert(alias.clone(), uri.clone());
                        }
                    }
                });
                symbols.imports.insert(artifact.name.clone(), map);
            }
            if artifact.artifact_kind == "Shape" {
                let values = enum_values(artifact);
                if !values.is_empty() {
                    symbols.enums.push(EnumInfo {
                        name: artifact.name.clone(),
                        service: file.service.clone(),
                        path: file.path.clone(),
                        span_start: artifact.span.start,
                        values,
                    });
                }
            }
        }
        symbols
    }

    pub fn knows(&self, name: &str) -> bool {
        self.artifacts.contains_key(name) || PRIMITIVES.contains(&name)
    }

    /// Resolves an enum type, honoring the service and the `imports` aliases.
    pub fn resolve_enum(&self, type_ref: &str, service: Option<&str>) -> Option<&EnumInfo> {
        let base = type_ref.trim_end_matches("[]").trim();
        if let Some((alias, local)) = base.split_once('.') {
            let target = service
                .and_then(|service| self.imports.get(service))
                .and_then(|imports| imports.get(alias))?;
            return self.enums.iter().find(|info| {
                info.name == local && info.service.as_deref() == Some(target.as_str())
            });
        }
        if let Some(service) = service {
            if let Some(info) = self
                .enums
                .iter()
                .find(|info| info.name == base && info.service.as_deref() == Some(service))
            {
                return Some(info);
            }
        }
        self.enums.iter().find(|info| info.name == base)
    }

    pub fn find_value(&self, value: &str) -> Option<(&EnumInfo, &EnumMember)> {
        for info in &self.enums {
            if let Some(member) = info.values.iter().find(|member| member.name == value) {
                return Some((info, member));
            }
        }
        None
    }

    pub fn enum_of_artifact(&self, name: &str) -> Option<&EnumInfo> {
        self.enums.iter().find(|info| info.name == name)
    }
}

/// Enum values declared with `value Name { ... }`.
pub fn enum_values(artifact: &Artifact) -> Vec<EnumMember> {
    artifact
        .body
        .iter()
        .filter_map(|member| match member {
            Member::EnumValueDeclaration {
                name, body, span, ..
            } => Some(EnumMember {
                name: name.clone(),
                title: title_of(body),
                span: (span.start, span.end),
            }),
            _ => None,
        })
        .collect()
}

fn title_of(members: &[Member]) -> Option<String> {
    let expr = property(members, "title")?;
    let Expr::Object { members, .. } = expr else {
        return None;
    };
    members.iter().find_map(|member| match member {
        Member::PropertyAssignment {
            value: Expr::StringLiteral { value, .. },
            ..
        } => Some(value.clone()),
        _ => None,
    })
}

pub fn property<'a>(members: &'a [Member], name: &str) -> Option<&'a Expr> {
    members.iter().find_map(|member| match member {
        Member::PropertyAssignment {
            name: key, value, ..
        } if key == name => Some(value),
        _ => None,
    })
}

/// List of identifiers: `[A, B]`, `A` or empty.
pub fn ident_list(expr: &Expr) -> Vec<&str> {
    match expr {
        Expr::Identifier { name, .. } => vec![name.as_str()],
        Expr::MemberAccess { name, .. } => vec![name.as_str()],
        Expr::List { elements, .. } => elements.iter().filter_map(Expr::as_ident).collect(),
        _ => Vec::new(),
    }
}

/// Walks all members recursively (blocks, statements, objects).
pub fn walk_members<'a>(members: &'a [Member], visit: &mut impl FnMut(&'a Member)) {
    for member in members {
        visit(member);
        match member {
            Member::Block { body, .. } | Member::EnumValueDeclaration { body, .. } => {
                walk_members(body, visit)
            }
            Member::Statement { statement, .. } => walk_statement(statement, visit),
            Member::PropertyAssignment { value, .. } => walk_expr(value, visit),
            _ => {}
        }
    }
}

fn walk_statement<'a>(statement: &'a archol_core::Stmt, visit: &mut impl FnMut(&'a Member)) {
    use archol_core::Stmt;
    match statement {
        Stmt::IfStatement {
            then_branch,
            else_branch,
            ..
        } => {
            walk_members(then_branch, visit);
            if let Some(else_branch) = else_branch {
                walk_members(else_branch, visit);
            }
        }
        Stmt::TryStatement { body, catches, .. } => {
            walk_members(body, visit);
            for catch in catches {
                walk_members(&catch.body, visit);
            }
        }
        _ => {}
    }
}

fn walk_expr<'a>(expr: &'a Expr, visit: &mut impl FnMut(&'a Member)) {
    if let Expr::Object { members, .. } = expr {
        walk_members(members, visit);
    }
}
