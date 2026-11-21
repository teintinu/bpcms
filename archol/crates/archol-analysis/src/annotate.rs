//! Semantic annotation: identifier paths during compilation, ids at the end.
//!
//! References are resolved to **paths** while walking the closure. Only after
//! the whole application is compiled do we assign ids (and only for used
//! paths); the AST is then rewritten with those ids. Unused declarations keep
//! no id.

use std::collections::{BTreeMap, BTreeSet};

use archol_core::{Artifact, ArtifactKind, Expr, HierarchyChain, Member, Span, Stmt};

use crate::application::Closure;
use crate::ids::{
    application_path, artifact_path, field_path, param_path, permission_path, service_path,
    value_path, Lock, LockEntry,
};
use crate::workspace::Workspace;

#[derive(Clone, Debug)]
pub struct DeclInfo {
    pub entry: LockEntry,
    pub kind: String,
    pub name: String,
    pub file: String,
    pub service: Option<String>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Usage {
    pub id: String,
    pub file: String,
    pub span: Span,
    pub kind: String,
}

#[derive(Clone, Debug, Default)]
pub struct SemanticIndex {
    pub lock: Lock,
    /// id -> declaration (only used identifiers).
    pub declarations: BTreeMap<String, DeclInfo>,
    /// References, in source order.
    pub usages: Vec<Usage>,
    /// Files outside the application closure (treated as commented out).
    pub outside: BTreeSet<String>,
}

#[derive(Default)]
struct Scope {
    decls: Vec<Decl>,
    by_name: BTreeMap<String, Vec<usize>>,
}

#[derive(Clone)]
struct FieldDecl {
    path: String,
    type_ref: Option<String>,
    span: Span,
}

#[derive(Clone)]
struct ValueDecl {
    path: String,
    span: Span,
}

#[derive(Clone)]
struct ParamDecl {
    path: String,
    span: Span,
}

#[derive(Clone)]
struct Decl {
    path: String,
    kind: String,
    name: String,
    service: Option<String>,
    file: String,
    span: Span,
    fields: BTreeMap<String, FieldDecl>,
    values: BTreeMap<String, ValueDecl>,
    params: BTreeMap<String, ParamDecl>,
}

impl Scope {
    fn artifact(&self, name: &str, service: Option<&str>) -> Option<&Decl> {
        let indices = self.by_name.get(name)?;
        indices
            .iter()
            .map(|index| &self.decls[*index])
            .find(|decl| decl.service.as_deref() == service)
            .or_else(|| indices.first().map(|index| &self.decls[*index]))
    }

    fn field_type(&self, type_name: &str, field: &str, service: Option<&str>) -> Option<String> {
        self.artifact(type_name, service)?
            .fields
            .get(field)
            .and_then(|field| field.type_ref.clone())
    }

    fn field_path(&self, type_name: &str, field: &str, service: Option<&str>) -> Option<String> {
        self.artifact(type_name, service)?
            .fields
            .get(field)
            .map(|field| field.path.clone())
    }

    fn value_path(&self, type_name: &str, value: &str, service: Option<&str>) -> Option<String> {
        self.artifact(type_name, service)?
            .values
            .get(value)
            .map(|value| value.path.clone())
    }
}

/// Compiles references to paths, then assigns ids only to used identifiers.
pub fn annotate(workspace: &mut Workspace, closure: &Closure, mut lock: Lock) -> SemanticIndex {
    let mut scope = Scope::default();
    let mut roots = BTreeSet::new();

    // Pass 1: declarations and their canonical paths (no ids yet).
    for file_index in 0..workspace.files.len() {
        let file_path = workspace.files[file_index].path.clone();
        if !closure.contains_file(&file_path) {
            continue;
        }
        let service = workspace.files[file_index].service.clone();
        let artifacts_len = workspace.files[file_index].ast.items.len();
        for artifact_index in 0..artifacts_len {
            let artifact = &mut workspace.files[file_index].ast.items[artifact_index];
            let decl = declare_artifact(artifact, service.clone(), file_path.clone());
            roots.insert(decl.path.clone());
            scope
                .by_name
                .entry(decl.name.clone())
                .or_default()
                .push(scope.decls.len());
            scope.decls.push(decl);
        }
    }

    // Pass 1b: auto-created permissions of each service, from role `permissions`.
    for permission in collect_permissions(workspace, closure) {
        let Some(service) = permission.service.clone() else {
            continue;
        };
        let path = permission_path(&service, &permission.name);
        let decl = Decl {
            path: path.clone(),
            kind: "permission".to_string(),
            name: permission.name.clone(),
            service: Some(service),
            file: permission.file,
            span: permission.span,
            fields: BTreeMap::new(),
            values: BTreeMap::new(),
            params: BTreeMap::new(),
        };
        roots.insert(path);
        scope
            .by_name
            .entry(decl.name.clone())
            .or_default()
            .push(scope.decls.len());
        scope.decls.push(decl);
    }

    // Pass 2: resolve references to paths.
    let mut usages = Vec::new();
    for file_index in 0..workspace.files.len() {
        let file_path = workspace.files[file_index].path.clone();
        if !closure.contains_file(&file_path) {
            continue;
        }
        let service = workspace.files[file_index].service.clone();
        let mut artifact_index = 0;
        while artifact_index < workspace.files[file_index].ast.items.len() {
            let artifact = &mut workspace.files[file_index].ast.items[artifact_index];
            let known = artifact
                .id
                .as_ref()
                .map(|path| scope.decls.iter().any(|decl| &decl.path == path))
                .unwrap_or(false);
            if known {
                let mut ctx = Ctx {
                    scope: &scope,
                    service: service.as_deref(),
                    locals: locals_of(artifact),
                    file: &file_path,
                    usages: Vec::new(),
                };
                annotate_param_types(artifact, &mut ctx);
                annotate_members(&mut artifact.body, &mut ctx);
                usages.extend(ctx.usages);
            }
            artifact_index += 1;
        }
    }

    // End of compilation: assign ids only to used paths, most referenced first.
    let mut weights: BTreeMap<String, usize> = roots.into_iter().map(|path| (path, 0)).collect();
    for usage in &usages {
        *weights.entry(usage.id.clone()).or_default() += 1;
    }
    let ids = lock.assign(&weights);

    // Rewrite the AST with final ids and drop temporary paths.
    for file_index in 0..workspace.files.len() {
        if !closure.contains_file(&workspace.files[file_index].path) {
            continue;
        }
        for artifact in &mut workspace.files[file_index].ast.items {
            finalize_artifact(artifact, &ids);
        }
    }
    let usages: Vec<Usage> = usages
        .into_iter()
        .map(|usage| Usage {
            id: ids.get(&usage.id).cloned().unwrap_or(usage.id),
            ..usage
        })
        .collect();
    let declarations = build_declarations(&scope, &ids);

    SemanticIndex {
        lock,
        declarations,
        usages,
        outside: outside_files(workspace, closure),
    }
}

fn declare_artifact(artifact: &mut Artifact, service: Option<String>, file: String) -> Decl {
    let artifact_kind = artifact.artifact_kind();
    let kind = artifact_kind.to_ascii_lowercase();
    let path = match kind.as_str() {
        "application" => application_path(&artifact.name),
        "service" => service_path(&artifact.name),
        _ => artifact_path(
            service.as_deref().unwrap_or(""),
            artifact_kind,
            &artifact.name,
        ),
    };
    artifact.id = Some(path.clone());

    let service_uri = service.as_deref().unwrap_or("");
    let artifact_name = artifact.name.clone();
    let mut fields = BTreeMap::new();
    if let Some(Member::Block { body, .. }) = artifact
        .body
        .iter_mut()
        .find(|member| matches!(member, Member::Block { name, .. } if name == "fields"))
    {
        for member in body {
            if let Member::PropertyAssignment {
                name,
                value,
                span,
                id,
                ..
            } = member
            {
                let field_path = field_path(service_uri, &artifact_name, name);
                *id = Some(field_path.clone());
                fields.insert(
                    name.clone(),
                    FieldDecl {
                        path: field_path,
                        type_ref: type_ref_of(value),
                        span: *span,
                    },
                );
            }
        }
    }

    let mut values = BTreeMap::new();
    for member in &mut artifact.body {
        if let Member::EnumValueDeclaration { name, span, id, .. } = member {
            let value_path = value_path(service_uri, &artifact_name, name);
            *id = Some(value_path.clone());
            values.insert(
                name.clone(),
                ValueDecl {
                    path: value_path,
                    span: *span,
                },
            );
        }
    }

    let mut params = BTreeMap::new();
    for param in &mut artifact.params {
        let param_path = param_path(service_uri, artifact_kind, &artifact_name, &param.name);
        param.id = Some(param_path.clone());
        params.insert(
            param.name.clone(),
            ParamDecl {
                path: param_path,
                span: param.span,
            },
        );
    }

    Decl {
        path,
        kind,
        name: artifact.name.clone(),
        service,
        file,
        span: artifact.span,
        fields,
        values,
        params,
    }
}

fn finalize_artifact(artifact: &mut Artifact, ids: &BTreeMap<String, String>) {
    artifact.id = finalize_opt(artifact.id.take(), ids);
    for param in &mut artifact.params {
        param.id = finalize_opt(param.id.take(), ids);
        param.type_ref_id = finalize_opt(param.type_ref_id.take(), ids);
    }
    finalize_members(&mut artifact.body, ids);
}

fn finalize_members(members: &mut [Member], ids: &BTreeMap<String, String>) {
    for member in members {
        match member {
            Member::PropertyAssignment { value, id, .. } => {
                *id = finalize_opt(id.take(), ids);
                finalize_expr(value, ids);
            }
            Member::EnumValueDeclaration { id, body, .. } => {
                *id = finalize_opt(id.take(), ids);
                finalize_members(body, ids);
            }
            Member::Block { body, .. } => finalize_members(body, ids),
            Member::Hierarchy { chains, .. } => {
                for chain in chains {
                    for role in &mut chain.roles {
                        role.role_ref = finalize_opt(role.role_ref.take(), ids);
                    }
                }
            }
            Member::ServiceEntry { target, .. } | Member::ImportDeclaration { target, .. } => {
                *target = finalize_opt(target.take(), ids);
            }
            Member::Statement { statement, .. } => finalize_statement(statement, ids),
            _ => {}
        }
    }
}

fn finalize_expr(expr: &mut Expr, ids: &BTreeMap<String, String>) {
    match expr {
        Expr::Reference { target, .. } => {
            if let Some(id) = ids.get(target) {
                *target = id.clone();
            }
        }
        Expr::MemberAccess {
            object, field_ref, ..
        } => {
            *field_ref = finalize_opt(field_ref.take(), ids);
            finalize_expr(object, ids);
        }
        Expr::Object {
            type_ref_id,
            members,
            ..
        } => {
            *type_ref_id = finalize_opt(type_ref_id.take(), ids);
            finalize_members(members, ids);
        }
        Expr::List { elements, .. } => {
            for element in elements {
                finalize_expr(element, ids);
            }
        }
        Expr::Call { args, .. } => {
            for arg in args {
                finalize_expr(arg, ids);
            }
        }
        Expr::Binary { left, right, .. } => {
            finalize_expr(left, ids);
            finalize_expr(right, ids);
        }
        Expr::Unary { operand, .. } => finalize_expr(operand, ids),
        Expr::NamedArgument { value, .. } => finalize_expr(value, ids),
        _ => {}
    }
}

fn finalize_statement(statement: &mut Stmt, ids: &BTreeMap<String, String>) {
    match statement {
        Stmt::ConstDeclaration { init, .. } | Stmt::LetDeclaration { init, .. } => {
            if let Some(init) = init {
                finalize_expr(init, ids);
            }
        }
        Stmt::IfStatement {
            condition,
            then_branch,
            else_branch,
            ..
        } => {
            finalize_expr(condition, ids);
            finalize_members(then_branch, ids);
            if let Some(else_branch) = else_branch {
                finalize_members(else_branch, ids);
            }
        }
        Stmt::TryStatement { body, catches, .. } => {
            finalize_members(body, ids);
            for catch in catches {
                finalize_members(&mut catch.body, ids);
            }
        }
        Stmt::ReturnStatement { value, .. } => {
            if let Some(value) = value {
                finalize_expr(value, ids);
            }
        }
        Stmt::ExpressionStatement { expression, .. } => finalize_expr(expression, ids),
        Stmt::RawStatement { .. } => {}
    }
}

fn finalize_opt(path: Option<String>, ids: &BTreeMap<String, String>) -> Option<String> {
    path.and_then(|path| ids.get(&path).cloned())
}

fn build_declarations(scope: &Scope, ids: &BTreeMap<String, String>) -> BTreeMap<String, DeclInfo> {
    let mut declarations = BTreeMap::new();
    for decl in &scope.decls {
        if let Some(id) = ids.get(&decl.path) {
            declarations.insert(
                id.clone(),
                DeclInfo {
                    entry: LockEntry {
                        id: id.clone(),
                        path: decl.path.clone(),
                    },
                    kind: decl.kind.clone(),
                    name: decl.name.clone(),
                    file: decl.file.clone(),
                    service: decl.service.clone(),
                    span: decl.span,
                },
            );
        }
        for (name, field) in &decl.fields {
            if let Some(id) = ids.get(&field.path) {
                declarations.insert(
                    id.clone(),
                    DeclInfo {
                        entry: LockEntry {
                            id: id.clone(),
                            path: field.path.clone(),
                        },
                        kind: "field".to_string(),
                        name: name.clone(),
                        file: decl.file.clone(),
                        service: decl.service.clone(),
                        span: field.span,
                    },
                );
            }
        }
        for (name, value) in &decl.values {
            if let Some(id) = ids.get(&value.path) {
                declarations.insert(
                    id.clone(),
                    DeclInfo {
                        entry: LockEntry {
                            id: id.clone(),
                            path: value.path.clone(),
                        },
                        kind: "value".to_string(),
                        name: name.clone(),
                        file: decl.file.clone(),
                        service: decl.service.clone(),
                        span: value.span,
                    },
                );
            }
        }
        for (name, param) in &decl.params {
            if let Some(id) = ids.get(&param.path) {
                declarations.insert(
                    id.clone(),
                    DeclInfo {
                        entry: LockEntry {
                            id: id.clone(),
                            path: param.path.clone(),
                        },
                        kind: "param".to_string(),
                        name: name.clone(),
                        file: decl.file.clone(),
                        service: decl.service.clone(),
                        span: param.span,
                    },
                );
            }
        }
    }
    declarations
}

fn type_ref_of(value: &Expr) -> Option<String> {
    match value {
        Expr::Object {
            type_ref: Some(ty), ..
        } => Some(ty.clone()),
        Expr::Identifier { name, .. } => Some(name.clone()),
        _ => None,
    }
}

struct Ctx<'a> {
    scope: &'a Scope,
    service: Option<&'a str>,
    locals: BTreeMap<String, Option<String>>,
    file: &'a str,
    usages: Vec<Usage>,
}

fn locals_of(artifact: &Artifact) -> BTreeMap<String, Option<String>> {
    let mut locals = BTreeMap::new();
    for param in &artifact.params {
        locals.insert(param.name.clone(), param.type_ref.clone());
    }
    collect_locals(&artifact.body, &mut locals);
    locals
}

fn collect_locals(members: &[Member], locals: &mut BTreeMap<String, Option<String>>) {
    for member in members {
        match member {
            Member::Block { body, .. } | Member::EnumValueDeclaration { body, .. } => {
                collect_locals(body, locals)
            }
            Member::Statement { statement, .. } => match statement {
                Stmt::ConstDeclaration { name, type_ref, .. }
                | Stmt::LetDeclaration { name, type_ref, .. } => {
                    locals.insert(name.clone(), type_ref.clone());
                }
                Stmt::IfStatement {
                    then_branch,
                    else_branch,
                    ..
                } => {
                    collect_locals(then_branch, locals);
                    if let Some(else_branch) = else_branch {
                        collect_locals(else_branch, locals);
                    }
                }
                Stmt::TryStatement { body, catches, .. } => {
                    collect_locals(body, locals);
                    for catch in catches {
                        collect_locals(&catch.body, locals);
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }
}

fn annotate_param_types(artifact: &mut Artifact, ctx: &mut Ctx<'_>) {
    for param in &mut artifact.params {
        let Some(type_ref) = param.type_ref.clone() else {
            continue;
        };
        if let Some(decl) = ctx.scope.artifact(base_type(&type_ref), ctx.service) {
            param.type_ref_id = Some(decl.path.clone());
            ctx.usages.push(Usage {
                id: decl.path.clone(),
                file: ctx.file.to_string(),
                span: param.span,
                kind: "typeRef".to_string(),
            });
        }
    }
}

fn annotate_members(members: &mut [Member], ctx: &mut Ctx<'_>) {
    for member in members {
        match member {
            Member::PropertyAssignment { name, value, .. } => annotate_property(name, value, ctx),
            Member::Block { body, .. } | Member::EnumValueDeclaration { body, .. } => {
                annotate_members(body, ctx)
            }
            Member::Hierarchy { chains, .. } => annotate_hierarchy(chains, ctx),
            Member::Statement { statement, .. } => annotate_statement(statement, ctx),
            Member::ServiceEntry {
                uri, span, target, ..
            }
            | Member::ImportDeclaration {
                uri, span, target, ..
            } => annotate_service_ref(uri, *span, target, ctx),
            _ => {}
        }
    }
}

fn annotate_hierarchy(chains: &mut [HierarchyChain], ctx: &mut Ctx<'_>) {
    for chain in chains {
        for role in &mut chain.roles {
            let name = role.name.clone();
            let span = role.span;
            let Some(decl) = ctx.scope.artifact(&name, ctx.service) else {
                continue;
            };
            if decl.kind != "role" {
                continue;
            }
            role.role_ref = Some(decl.path.clone());
            role.service = decl.service.clone();
            role.path = Some(decl.file.clone());
            ctx.usages.push(Usage {
                id: decl.path.clone(),
                file: ctx.file.to_string(),
                span,
                kind: "roleRef".to_string(),
            });
        }
    }
}

fn annotate_service_ref(uri: &str, span: Span, target: &mut Option<String>, ctx: &mut Ctx<'_>) {
    let Some(decl) = ctx.scope.artifact(uri, None) else {
        return;
    };
    if decl.kind != "service" {
        return;
    }
    *target = Some(decl.path.clone());
    ctx.usages.push(Usage {
        id: decl.path.clone(),
        file: ctx.file.to_string(),
        span,
        kind: "serviceRef".to_string(),
    });
}

fn annotate_property(name: &str, value: &mut Expr, ctx: &mut Ctx<'_>) {
    match name {
        "about" | "document" | "source" | "appliesTo" | "sources" => {
            annotate_artifact_refs(value, None, "artifactRef", ctx);
        }
        "roles" => annotate_artifact_refs(value, Some("role"), "roleRef", ctx),
        "permissions" => annotate_permission_refs(value, ctx),
        "fanOut" => annotate_expr(value, ctx),
        _ => annotate_expr(value, ctx),
    }
}

/// Permissions used by a role become references to the service's auto-created
/// permission artifacts, so the builder follows `ref` instead of re-parsing text.
fn annotate_permission_refs(value: &mut Expr, ctx: &mut Ctx<'_>) {
    match value {
        Expr::List { elements, .. } => {
            for element in elements {
                annotate_permission_refs(element, ctx);
            }
        }
        Expr::Identifier { .. } | Expr::MemberAccess { .. } => {
            let Some(name) = expr_text(value) else {
                return;
            };
            let span = value.span();
            let Some(decl) = ctx.scope.artifact(&name, ctx.service) else {
                return;
            };
            if decl.kind != "permission" {
                return;
            }
            *value = Expr::Reference {
                ref_kind: "permission".to_string(),
                name,
                target: decl.path.clone(),
                span,
            };
            ctx.usages.push(Usage {
                id: decl.path.clone(),
                file: ctx.file.to_string(),
                span,
                kind: "permissionRef".to_string(),
            });
        }
        _ => {}
    }
}

/// Source spelling of a permission (`invoice.view`, `exam.perform.hemogram`).
fn expr_text(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Identifier { name, .. } => Some(name.clone()),
        Expr::MemberAccess { object, name, .. } => Some(format!("{}.{}", expr_text(object)?, name)),
        Expr::Reference { name, .. } => Some(name.clone()),
        _ => None,
    }
}

/// One permission expression found in a role.
struct FoundPermission {
    service: Option<String>,
    name: String,
    file: String,
    span: Span,
}

fn collect_permissions(workspace: &Workspace, closure: &Closure) -> Vec<FoundPermission> {
    let mut found: Vec<FoundPermission> = Vec::new();
    for file in &workspace.files {
        if !closure.contains_file(&file.path) {
            continue;
        }
        for artifact in &file.ast.items {
            if artifact.kind != ArtifactKind::Role {
                continue;
            }
            let Some(value) = property_value(&artifact.body, "permissions") else {
                continue;
            };
            for expr in permission_exprs(value) {
                let Some(name) = expr_text(expr) else {
                    continue;
                };
                let duplicate = found
                    .iter()
                    .any(|entry| entry.service == file.service && entry.name == name);
                if duplicate {
                    continue;
                }
                found.push(FoundPermission {
                    service: file.service.clone(),
                    name,
                    file: file.path.clone(),
                    span: expr.span(),
                });
            }
        }
    }
    found
}

fn permission_exprs(value: &Expr) -> Vec<&Expr> {
    match value {
        Expr::List { elements, .. } => elements.iter().collect(),
        _ => vec![value],
    }
}

fn property_value<'a>(members: &'a [Member], wanted: &str) -> Option<&'a Expr> {
    members.iter().find_map(|member| match member {
        Member::PropertyAssignment { name, value, .. } if name == wanted => Some(value),
        _ => None,
    })
}

fn annotate_artifact_refs(
    value: &mut Expr,
    kind_filter: Option<&str>,
    usage_kind: &str,
    ctx: &mut Ctx<'_>,
) {
    match value {
        Expr::Identifier { name, span } => {
            let name = name.clone();
            let span = *span;
            if let Some(decl) = ctx.scope.artifact(&name, ctx.service) {
                if kind_filter.map(|kind| decl.kind == kind).unwrap_or(true) {
                    let target = decl.path.clone();
                    let ref_kind = decl.kind.clone();
                    *value = Expr::Reference {
                        ref_kind,
                        name,
                        target: target.clone(),
                        span,
                    };
                    ctx.usages.push(Usage {
                        id: target,
                        file: ctx.file.to_string(),
                        span,
                        kind: usage_kind.to_string(),
                    });
                }
            }
        }
        Expr::List { elements, .. } => {
            for element in elements {
                annotate_artifact_refs(element, kind_filter, usage_kind, ctx);
            }
        }
        Expr::Binary { left, right, .. } => {
            annotate_artifact_refs(left, kind_filter, usage_kind, ctx);
            annotate_expr(right, ctx);
        }
        _ => {}
    }
}

fn annotate_expr(expr: &mut Expr, ctx: &mut Ctx<'_>) {
    match expr {
        Expr::List { elements, .. } => {
            for element in elements {
                annotate_expr(element, ctx);
            }
        }
        Expr::Object {
            type_ref,
            type_ref_id,
            members,
            span,
        } => {
            if let Some(ty) = type_ref.as_deref() {
                if let Some(decl) = ctx.scope.artifact(base_type(ty), ctx.service) {
                    *type_ref_id = Some(decl.path.clone());
                    ctx.usages.push(Usage {
                        id: decl.path.clone(),
                        file: ctx.file.to_string(),
                        span: *span,
                        kind: "typeRef".to_string(),
                    });
                }
            }
            annotate_members(members, ctx);
        }
        Expr::Call { args, .. } => {
            for arg in args {
                annotate_expr(arg, ctx);
            }
        }
        Expr::MemberAccess {
            object,
            name,
            span,
            field_ref,
        } => {
            annotate_expr(object, ctx);
            if let Some(ty) = expr_type(object, ctx) {
                if let Some(field_path) = ctx.scope.field_path(base_type(&ty), name, ctx.service) {
                    *field_ref = Some(field_path.clone());
                    let name_start = span.end.saturating_sub(name.len());
                    ctx.usages.push(Usage {
                        id: field_path,
                        file: ctx.file.to_string(),
                        span: Span::new(name_start, span.end),
                        kind: "fieldRef".to_string(),
                    });
                }
            }
        }
        Expr::Binary {
            op, left, right, ..
        } => {
            annotate_expr(left, ctx);
            annotate_expr(right, ctx);
            if matches!(op.as_str(), "==" | "===" | "!=" | "!==") {
                annotate_enum_literal(left, right, ctx);
                annotate_enum_literal(right, left, ctx);
            }
            if op == "where" {
                // `fanOut: Invoice where status == Overdue`: the subject type is
                // the left side; the compared identifier is a field of it.
                let subject = left.as_ident().map(str::to_string);
                annotate_artifact_refs(left, None, "artifactRef", ctx);
                if let (
                    Some(subject),
                    Expr::Binary {
                        op,
                        left: var,
                        right: lit,
                        ..
                    },
                ) = (subject, right.as_mut())
                {
                    if matches!(op.as_str(), "==" | "===" | "!=" | "!==") {
                        if let Some(field) = var.as_ident() {
                            if let Some(ty) = ctx.scope.field_type(&subject, field, ctx.service) {
                                replace_enum_literal(lit.as_mut(), &ty, ctx);
                            }
                        }
                    }
                }
            }
        }
        Expr::Unary { operand, .. } => annotate_expr(operand, ctx),
        Expr::NamedArgument { value, .. } => annotate_expr(value, ctx),
        _ => {}
    }
}

fn annotate_enum_literal(variable: &Expr, literal: &mut Expr, ctx: &mut Ctx<'_>) {
    let Some(ty) = expr_type(variable, ctx) else {
        return;
    };
    replace_enum_literal(literal, &ty, ctx);
}

fn replace_enum_literal(literal: &mut Expr, ty: &str, ctx: &mut Ctx<'_>) {
    let Some(name) = literal.as_ident().map(str::to_string) else {
        return;
    };
    let Some(target) = ctx.scope.value_path(base_type(ty), &name, ctx.service) else {
        return;
    };
    if let Expr::Identifier { span, .. } = literal {
        let span = *span;
        *literal = Expr::Reference {
            ref_kind: "enumValue".to_string(),
            name,
            target: target.clone(),
            span,
        };
        ctx.usages.push(Usage {
            id: target,
            file: ctx.file.to_string(),
            span,
            kind: "enumValue".to_string(),
        });
    }
}

fn expr_type(expr: &Expr, ctx: &mut Ctx<'_>) -> Option<String> {
    match expr {
        Expr::Identifier { name, .. } => {
            if let Some(ty) = ctx.locals.get(name).and_then(|ty| ty.clone()) {
                return Some(ty);
            }
            ctx.scope
                .artifact(name, ctx.service)
                .map(|decl| decl.name.clone())
        }
        Expr::MemberAccess { object, name, .. } => {
            let base = expr_type(object, ctx)?;
            ctx.scope.field_type(base_type(&base), name, ctx.service)
        }
        _ => None,
    }
}

fn base_type(ty: &str) -> &str {
    let ty = ty.split('|').next().unwrap_or(ty).trim();
    ty.trim_end_matches("[]").trim()
}

fn annotate_statement(statement: &mut Stmt, ctx: &mut Ctx<'_>) {
    match statement {
        Stmt::ConstDeclaration { type_ref, init, .. }
        | Stmt::LetDeclaration { type_ref, init, .. } => {
            if let Some(init) = init {
                if let Some(type_ref) = type_ref {
                    if let Some(name) = init.as_ident().map(str::to_string) {
                        if let Some(target) =
                            ctx.scope
                                .value_path(base_type(type_ref), &name, ctx.service)
                        {
                            if let Expr::Identifier { span, .. } = init {
                                let span = *span;
                                *init = Expr::Reference {
                                    ref_kind: "enumValue".to_string(),
                                    name,
                                    target: target.clone(),
                                    span,
                                };
                                ctx.usages.push(Usage {
                                    id: target,
                                    file: ctx.file.to_string(),
                                    span,
                                    kind: "enumValue".to_string(),
                                });
                            }
                        }
                    }
                }
                annotate_expr(init, ctx);
            }
        }
        Stmt::IfStatement {
            condition,
            then_branch,
            else_branch,
            ..
        } => {
            annotate_expr(condition, ctx);
            annotate_members(then_branch, ctx);
            if let Some(else_branch) = else_branch {
                annotate_members(else_branch, ctx);
            }
        }
        Stmt::TryStatement { body, catches, .. } => {
            annotate_members(body, ctx);
            for catch in catches {
                annotate_members(&mut catch.body, ctx);
            }
        }
        Stmt::ReturnStatement { value, .. } => {
            if let Some(value) = value {
                annotate_expr(value, ctx);
            }
        }
        Stmt::ExpressionStatement { expression, .. } => annotate_expr(expression, ctx),
        Stmt::RawStatement { .. } => {}
    }
}

fn outside_files(workspace: &Workspace, closure: &Closure) -> BTreeSet<String> {
    workspace
        .files
        .iter()
        .map(|file| file.path.clone())
        .filter(|path| !closure.contains_file(path))
        .collect()
}

#[allow(dead_code)]
fn _unused(_: &Lock) {}
