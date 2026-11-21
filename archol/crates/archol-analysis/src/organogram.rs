//! Derived organogram projection: the roles of a service with the hierarchy
//! turned into `manages` pointers and the role's own `inherits`/permissions.
//!
//! This is not a new source artifact: the `ArtifactRole` nodes stay in the AST
//! and in the lock. The organogram node only gains the derived `roles` array so
//! the builder can walk the graph without re-reading the hierarchy chains.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use archol_core::{
    Artifact, ArtifactKind, Expr, HierarchyChain, Member, OrganogramRole, PermissionPointer,
    RolePointer,
};

use crate::annotate::SemanticIndex;
use crate::application::Closure;
use crate::workspace::Workspace;

/// One role as read from its `role` artifact.
struct RoleData {
    id: String,
    path: String,
    line: usize,
    service: Option<String>,
    title: Option<Expr>,
    permissions: Vec<PermissionPointer>,
    inherits: Vec<String>,
}

/// Project the hierarchy and role data into every organogram of the workspace.
pub fn attach(workspace: &mut Workspace, semantic: &SemanticIndex, closure: &Closure) {
    let roles = role_data(workspace, semantic);
    if roles.is_empty() {
        return;
    }
    // Fill the declaration line on every hierarchy role reference.
    for file in &mut workspace.files {
        let service = file.service.clone();
        for artifact in &mut file.ast.items {
            if let Some(Member::Hierarchy { chains, .. }) = artifact
                .body
                .iter_mut()
                .find(|member| matches!(member, Member::Hierarchy { .. }))
            {
                for chain in chains {
                    for role in &mut chain.roles {
                        if let Some(data) = roles.get(&(service.clone(), role.name.clone())) {
                            role.line = Some(data.line);
                        }
                    }
                }
            }
        }
    }
    for file in &mut workspace.files {
        if !closure.contains_file(&file.path) {
            continue;
        }
        let service = file.service.clone();
        for artifact in &mut file.ast.items {
            if artifact.kind != ArtifactKind::Organogram {
                continue;
            }
            let chains = hierarchy_chains(artifact);
            let edges = hierarchy_edges(&chains);
            let mut projected = Vec::new();
            // Hierarchy order first (superior before subordinate), then the
            // remaining service roles alphabetically.
            let mut order: Vec<String> = Vec::new();
            for chain in &chains {
                for role in &chain.roles {
                    if !order.contains(&role.name) {
                        order.push(role.name.clone());
                    }
                }
            }
            for (role_service, name) in roles.keys() {
                if *role_service == service && !order.contains(name) {
                    order.push(name.clone());
                }
            }
            for name in order {
                let Some(data) = roles.get(&(service.clone(), name.clone())) else {
                    continue;
                };
                projected.push(OrganogramRole {
                    kind: "OrganogramRole",
                    name: name.clone(),
                    role_ref: data.id.clone(),
                    ref_kind: "role",
                    service: data.service.clone(),
                    path: data.path.clone(),
                    line: data.line,
                    title: data.title.clone(),
                    permissions: data.permissions.clone(),
                    inherits: pointers(&data.inherits, &roles, &service),
                    manages: pointers(&descendants(&name, &edges), &roles, &service),
                });
            }
            artifact.roles = projected;
        }
    }
}

/// Read every annotated role artifact into a by-service map.
fn role_data(
    workspace: &Workspace,
    semantic: &SemanticIndex,
) -> BTreeMap<(Option<String>, String), RoleData> {
    let mut roles = BTreeMap::new();
    for file in &workspace.files {
        for artifact in &file.ast.items {
            if artifact.kind != ArtifactKind::Role {
                continue;
            }
            let Some((id, _)) = semantic.declarations.iter().find(|(_, decl)| {
                decl.kind == "role" && decl.file == file.path && decl.name == artifact.name
            }) else {
                continue;
            };
            let line = file
                .lines
                .line_col(
                    semantic
                        .declarations
                        .get(id)
                        .map(|decl| decl.span.start)
                        .unwrap_or(0),
                )
                .0 as usize
                + 1;
            roles.insert(
                (file.service.clone(), artifact.name.clone()),
                RoleData {
                    id: id.clone(),
                    path: file.path.clone(),
                    line,
                    service: file.service.clone(),
                    title: property(&artifact.body, "title").cloned(),
                    permissions: property(&artifact.body, "permissions")
                        .map(|value| permission_pointers(value, file))
                        .unwrap_or_default(),
                    inherits: property(&artifact.body, "inherits")
                        .map(string_list)
                        .unwrap_or_default(),
                },
            );
        }
    }
    roles
}

/// Copy the hierarchy chains out of an organogram body.
fn hierarchy_chains(artifact: &Artifact) -> Vec<HierarchyChain> {
    artifact
        .body
        .iter()
        .find_map(|member| match member {
            Member::Hierarchy { chains, .. } => Some(chains.clone()),
            _ => None,
        })
        .unwrap_or_default()
}

/// Direct reporting edges, in source order.
fn hierarchy_edges(chains: &[HierarchyChain]) -> Vec<(String, String)> {
    let mut edges = Vec::new();
    for chain in chains {
        for pair in chain.roles.windows(2) {
            edges.push((pair[0].name.clone(), pair[1].name.clone()));
        }
    }
    edges
}

/// Transitive subordinates of one role, breadth first, without the role itself.
fn descendants(role: &str, edges: &[(String, String)]) -> Vec<String> {
    let mut adjacency: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for (from, to) in edges {
        adjacency
            .entry(from.as_str())
            .or_default()
            .push(to.as_str());
    }
    let mut queue = VecDeque::from([role]);
    let mut seen = BTreeSet::from([role]);
    let mut result = Vec::new();
    while let Some(current) = queue.pop_front() {
        for next in adjacency.get(current).into_iter().flatten() {
            if !seen.insert(next) {
                continue;
            }
            result.push((*next).to_string());
            queue.push_back(next);
        }
    }
    result
}

/// Permission references of one role, with the permission artifact id and line.
fn permission_pointers(
    value: &Expr,
    file: &crate::workspace::ParsedFile,
) -> Vec<PermissionPointer> {
    let values: Vec<&Expr> = match value {
        Expr::List { elements, .. } => elements.iter().collect(),
        _ => vec![value],
    };
    let mut pointers = Vec::new();
    for value in values {
        let Expr::Reference {
            name, target, span, ..
        } = value
        else {
            continue;
        };
        pointers.push(PermissionPointer {
            kind: "PermissionPointer",
            name: name.clone(),
            permission_ref: target.clone(),
            ref_kind: "permission",
            line: file.lines.line_col(span.start).0 as usize + 1,
        });
    }
    pointers
}

/// Turn role names into builder pointers, preserving their order.
fn pointers(
    names: &[String],
    roles: &BTreeMap<(Option<String>, String), RoleData>,
    service: &Option<String>,
) -> Vec<RolePointer> {
    let mut pointers = Vec::new();
    for name in names {
        let Some(data) = roles.get(&(service.clone(), name.clone())) else {
            continue;
        };
        pointers.push(RolePointer {
            kind: "RolePointer",
            name: name.clone(),
            role_ref: data.id.clone(),
            ref_kind: "role",
            service: data.service.clone(),
            path: data.path.clone(),
            line: data.line,
        });
    }
    pointers
}

/// One named property of a role or organogram.
fn property<'a>(members: &'a [Member], name: &str) -> Option<&'a Expr> {
    members.iter().find_map(|member| match member {
        Member::PropertyAssignment {
            name: property,
            value,
            ..
        } if property == name => Some(value),
        _ => None,
    })
}

/// Read identifiers/member accesses from a list or single value.
fn string_list(value: &Expr) -> Vec<String> {
    match value {
        Expr::List { elements, .. } => elements.iter().filter_map(expr_text).collect(),
        _ => expr_text(value).into_iter().collect(),
    }
}

/// Source spelling of a permission or role reference.
fn expr_text(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Identifier { name, .. } => Some(name.clone()),
        Expr::MemberAccess { object, name, .. } => Some(format!("{}.{}", expr_text(object)?, name)),
        Expr::StringLiteral { value, .. } => Some(value.clone()),
        Expr::Reference { name, .. } => Some(name.clone()),
        _ => None,
    }
}
