//! Service-side AST projection: every artifact owned by a service, plus the
//! permissions auto-created from its roles.
//!
//! Artifacts stay where they are declared; the `ArtifactService` node only gets
//! a typed index (`artifacts`) and the synthesized permission artifacts
//! (`permissions`) so the builder can start from the service.

use std::collections::BTreeMap;

use archol_core::{ArtifactKind, ArtifactPermission, ArtifactPointer};

use crate::annotate::SemanticIndex;
use crate::application::Closure;
use crate::workspace::Workspace;

/// Attach the service index and auto-created permissions to every
/// `ArtifactService` inside the application closure.
pub fn attach(workspace: &mut Workspace, semantic: &SemanticIndex, closure: &Closure) {
    let mut artifacts_by_service: BTreeMap<String, Vec<ArtifactPointer>> = BTreeMap::new();
    let mut permissions_by_service: BTreeMap<String, Vec<ArtifactPermission>> = BTreeMap::new();

    for file in &workspace.files {
        if !closure.contains_file(&file.path) {
            continue;
        }
        let Some(service) = file.service.clone() else {
            continue;
        };
        for artifact in &file.ast.items {
            if artifact.kind == ArtifactKind::Service {
                continue;
            }
            let kind = artifact.kind.name().to_ascii_lowercase();
            let Some(id) = declaration_id(semantic, &artifact.name, &kind, &service, &file.path)
            else {
                continue;
            };
            artifacts_by_service
                .entry(service.clone())
                .or_default()
                .push(ArtifactPointer {
                    kind: "ArtifactPointer",
                    artifact: artifact.kind,
                    name: artifact.name.clone(),
                    artifact_ref: id,
                    path: file.path.clone(),
                    line: file.lines.line_col(artifact.span.start).0 as usize + 1,
                });
        }
    }

    for (id, declaration) in &semantic.declarations {
        if declaration.kind != "permission" {
            continue;
        }
        let Some(service) = declaration.service.clone() else {
            continue;
        };
        let (namespace, action) = split_permission(&declaration.name);
        let line = workspace
            .files
            .iter()
            .find(|file| file.path == declaration.file)
            .map(|file| file.lines.line_col(declaration.span.start).0 as usize + 1)
            .unwrap_or(1);
        permissions_by_service
            .entry(service.clone())
            .or_default()
            .push(ArtifactPermission {
                kind: "ArtifactPermission",
                name: declaration.name.clone(),
                namespace: namespace.to_string(),
                action: action.to_string(),
                permission_ref: id.clone(),
                path: declaration.file.clone(),
                line,
                span: declaration.span,
            });
        artifacts_by_service
            .entry(service)
            .or_default()
            .push(ArtifactPointer {
                kind: "ArtifactPointer",
                artifact: ArtifactKind::Permission,
                name: declaration.name.clone(),
                artifact_ref: id.clone(),
                path: declaration.file.clone(),
                line,
            });
    }

    for file in &mut workspace.files {
        if !closure.contains_file(&file.path) {
            continue;
        }
        for artifact in &mut file.ast.items {
            if artifact.kind != ArtifactKind::Service {
                continue;
            }
            if let Some(mut pointers) = artifacts_by_service.remove(&artifact.name) {
                pointers.sort_by(|left, right| {
                    left.path
                        .cmp(&right.path)
                        .then(left.line.cmp(&right.line))
                        .then(left.name.cmp(&right.name))
                });
                artifact.artifacts = pointers;
            }
            if let Some(mut permissions) = permissions_by_service.remove(&artifact.name) {
                permissions.sort_by(|left, right| left.name.cmp(&right.name));
                artifact.permissions = permissions;
            }
        }
    }
}

/// Lock id of a declaration with the same kind/name/service/file.
fn declaration_id(
    semantic: &SemanticIndex,
    name: &str,
    kind: &str,
    service: &str,
    file: &str,
) -> Option<String> {
    semantic
        .declarations
        .iter()
        .find(|(_, declaration)| {
            declaration.name == name
                && declaration.kind == kind
                && declaration.file == file
                && declaration.service.as_deref() == Some(service)
        })
        .map(|(id, _)| id.clone())
}

/// Split `namespace.action` at the last dot; a bare action keeps an empty namespace.
fn split_permission(name: &str) -> (&str, &str) {
    match name.rsplit_once('.') {
        Some((namespace, action)) => (namespace, action),
        None => ("", name),
    }
}
