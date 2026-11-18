//! Link: index ready for the builder.
//!
//! Beyond what the source declares, the link answers inverted questions, such
//! as "which artifacts require role X" and "which references each artifact
//! makes".

use std::collections::{BTreeMap, BTreeSet};

use archol_core::{Expr, Member};
use serde::Serialize;

use crate::index::{ident_list, Symbols};
use crate::workspace::Workspace;

#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct LinkIndex {
    pub services: Vec<ServiceLink>,
    pub roles: Vec<RoleLink>,
    pub artifacts: Vec<ArtifactLink>,
    pub enums: Vec<EnumLink>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceLink {
    pub uri: String,
    pub artifacts: Vec<ArtifactBrief>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactBrief {
    pub name: String,
    pub kind: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoleLink {
    pub name: String,
    pub used_by: Vec<Usage>,
}

#[derive(Serialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub struct Usage {
    pub artifact: String,
    pub artifact_kind: String,
    pub usage: String,
    pub path: String,
    pub line: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactLink {
    pub name: String,
    pub kind: String,
    pub service: Option<String>,
    pub path: String,
    pub span: (usize, usize),
    pub refs: Vec<Reference>,
}

#[derive(Serialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub struct Reference {
    pub kind: String,
    pub name: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnumLink {
    pub name: String,
    pub service: Option<String>,
    pub values: Vec<EnumValueLink>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnumValueLink {
    pub name: String,
    pub title: Option<String>,
}

pub fn retain_closure(index: &mut LinkIndex, closure: &crate::application::Closure) {
    index
        .services
        .retain(|service| closure.services.contains(&service.uri));
    index
        .artifacts
        .retain(|artifact| closure.files.contains(&artifact.path));
    for role in &mut index.roles {
        role.used_by
            .retain(|usage| closure.files.contains(&usage.path));
    }
    index.roles.retain(|role| !role.used_by.is_empty());
    index.enums.retain(|enum_info| {
        enum_info
            .service
            .as_ref()
            .map(|service| closure.services.contains(service))
            .unwrap_or(false)
    });
}

pub fn build(workspace: &Workspace, symbols: &Symbols) -> LinkIndex {
    let mut index = LinkIndex::default();
    let mut roles: BTreeMap<String, BTreeSet<Usage>> = BTreeMap::new();
    let mut artifacts = Vec::new();

    let mut services: BTreeMap<String, Vec<ArtifactBrief>> = BTreeMap::new();

    for (file, artifact) in workspace.artifacts() {
        let mut refs = BTreeSet::new();
        collect_refs(&artifact.body, &mut refs);
        collect_role_usages(
            &artifact.body,
            &artifact.name,
            &artifact.artifact_kind,
            &file.path,
            &file.lines,
            &mut roles,
        );

        if let Some(service) = &file.service {
            services
                .entry(service.clone())
                .or_default()
                .push(ArtifactBrief {
                    name: artifact.name.clone(),
                    kind: artifact.artifact_kind.clone(),
                });
        }

        artifacts.push(ArtifactLink {
            name: artifact.name.clone(),
            kind: artifact.artifact_kind.clone(),
            service: file.service.clone(),
            path: file.path.clone(),
            span: (artifact.span.start, artifact.span.end),
            refs: refs.into_iter().collect(),
        });
    }

    index.services = services
        .into_iter()
        .map(|(uri, artifacts)| ServiceLink { uri, artifacts })
        .collect();
    index.roles = roles
        .into_iter()
        .map(|(name, usages)| RoleLink {
            name,
            used_by: usages.into_iter().collect(),
        })
        .collect();
    index.artifacts = artifacts;
    index.enums = symbols
        .enums
        .iter()
        .map(|info| EnumLink {
            name: info.name.clone(),
            service: info.service.clone(),
            values: info
                .values
                .iter()
                .map(|value| EnumValueLink {
                    name: value.name.clone(),
                    title: value.title.clone(),
                })
                .collect(),
        })
        .collect();
    index
}

fn collect_role_usages(
    members: &[Member],
    artifact: &str,
    artifact_kind: &str,
    path: &str,
    lines: &archol_core::LineIndex,
    roles: &mut BTreeMap<String, BTreeSet<Usage>>,
) {
    for member in members {
        match member {
            Member::PropertyAssignment {
                name, value, span, ..
            } => {
                if name == "roles" {
                    let line = lines.line_col(span.start).0 as usize + 1;
                    for role in ident_list(value) {
                        roles.entry(role.to_string()).or_default().insert(Usage {
                            artifact: artifact.to_string(),
                            artifact_kind: artifact_kind.to_string(),
                            usage: "roles".to_string(),
                            path: path.to_string(),
                            line,
                        });
                    }
                }
                if let Expr::Object { members, .. } = value {
                    collect_role_usages(members, artifact, artifact_kind, path, lines, roles);
                }
            }
            Member::Block { body, .. } | Member::EnumValueDeclaration { body, .. } => {
                collect_role_usages(body, artifact, artifact_kind, path, lines, roles)
            }
            Member::Statement { statement, .. } => {
                collect_statement_roles(statement, artifact, artifact_kind, path, lines, roles)
            }
            _ => {}
        }
    }
}

fn collect_statement_roles(
    statement: &archol_core::Stmt,
    artifact: &str,
    artifact_kind: &str,
    path: &str,
    lines: &archol_core::LineIndex,
    roles: &mut BTreeMap<String, BTreeSet<Usage>>,
) {
    use archol_core::Stmt;
    match statement {
        Stmt::IfStatement {
            then_branch,
            else_branch,
            ..
        } => {
            collect_role_usages(then_branch, artifact, artifact_kind, path, lines, roles);
            if let Some(else_branch) = else_branch {
                collect_role_usages(else_branch, artifact, artifact_kind, path, lines, roles);
            }
        }
        Stmt::TryStatement { body, catches, .. } => {
            collect_role_usages(body, artifact, artifact_kind, path, lines, roles);
            for catch in catches {
                collect_role_usages(&catch.body, artifact, artifact_kind, path, lines, roles);
            }
        }
        _ => {}
    }
}

const REF_KEYS: &[&str] = &[
    "about",
    "document",
    "source",
    "sources",
    "appliesTo",
    "fanOut",
];

fn collect_refs(members: &[Member], refs: &mut BTreeSet<Reference>) {
    for member in members {
        match member {
            Member::PropertyAssignment { name, value, .. } => {
                if REF_KEYS.contains(&name.as_str()) {
                    for target in ident_list(value) {
                        if target
                            .chars()
                            .next()
                            .map(|c| c.is_ascii_uppercase())
                            .unwrap_or(false)
                        {
                            refs.insert(Reference {
                                kind: name.clone(),
                                name: target.to_string(),
                            });
                        }
                    }
                }
                if let Expr::Object { members, .. } = value {
                    collect_refs(members, refs);
                }
            }
            Member::Block { body, .. } | Member::EnumValueDeclaration { body, .. } => {
                collect_refs(body, refs)
            }
            Member::Statement { statement, .. } => collect_statement_refs(statement, refs),
            _ => {}
        }
    }
}

fn collect_statement_refs(statement: &archol_core::Stmt, refs: &mut BTreeSet<Reference>) {
    use archol_core::Stmt;
    match statement {
        Stmt::IfStatement {
            then_branch,
            else_branch,
            ..
        } => {
            collect_refs(then_branch, refs);
            if let Some(else_branch) = else_branch {
                collect_refs(else_branch, refs);
            }
        }
        Stmt::TryStatement { body, catches, .. } => {
            collect_refs(body, refs);
            for catch in catches {
                collect_refs(&catch.body, refs);
            }
        }
        _ => {}
    }
}
