//! Semantic annotation: stable ids for declarations and typed references.
//!
//! The AST keeps only lock ids on nodes (`id`, `ref`, `fieldRef`, `typeRefId`),
//! so JSON stays small and acyclic. Usages are collected into [`SemanticIndex`]
//! for the LSP and the builder.

use std::collections::{BTreeMap, BTreeSet};

use archol_core::{Artifact, Expr, Member, Span, Stmt};

use crate::application::Closure;
use crate::ids::{
    artifact_path, field_path, param_path, service_path, value_path, Lock, LockEntry,
};
use crate::workspace::Workspace;

#[derive(Clone, Debug)]
pub struct DeclInfo {
    pub entry: LockEntry,
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
    /// id -> declaration.
    pub declarations: BTreeMap<String, DeclInfo>,
    /// References collected while annotating, in source order.
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
    id: String,
    path: String,
    type_ref: Option<String>,
    span: Span,
}

#[derive(Clone)]
struct ValueDecl {
    id: String,
    path: String,
    span: Span,
}

#[derive(Clone)]
struct ParamDecl {
    id: String,
    path: String,
    span: Span,
}

#[derive(Clone)]
struct Decl {
    id: String,
    kind: String,
    name: String,
    service: Option<String>,
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

    fn field_id(&self, type_name: &str, field: &str, service: Option<&str>) -> Option<String> {
        self.artifact(type_name, service)?
            .fields
            .get(field)
            .map(|field| field.id.clone())
    }

    fn value_id(&self, type_name: &str, value: &str, service: Option<&str>) -> Option<String> {
        self.artifact(type_name, service)?
            .values
            .get(value)
            .map(|value| value.id.clone())
    }
}

/// Assigns ids to declarations in the closure and annotates references.
pub fn annotate(workspace: &mut Workspace, closure: &Closure, mut lock: Lock) -> SemanticIndex {
    let mut scope = Scope::default();
    let mut declarations = BTreeMap::new();
    let mut used_paths = BTreeSet::new();

    // Pass 1: declarations and their ids.
    for file_index in 0..workspace.files.len() {
        let path = workspace.files[file_index].path.clone();
        if !closure.contains_file(&path) {
            continue;
        }
        let service = workspace.files[file_index].service.clone();
        let artifacts_len = workspace.files[file_index].ast.items.len();
        for artifact_index in 0..artifacts_len {
            let (decl, decl_info) = {
                let mut artifact = workspace.files[file_index].ast.items[artifact_index].clone();
                let entry = assign_artifact(
                    &mut artifact,
                    service.as_deref(),
                    &path,
                    &mut lock,
                    &mut used_paths,
                );
                let decl = build_decl(&artifact, entry.clone(), service.clone());
                let decl_info = DeclInfo {
                    entry,
                    file: path.clone(),
                    service: service.clone(),
                    span: artifact.span,
                };
                (decl, decl_info)
            };
            // Re-apply only the mutated ids to the real artifact later.
            let decl_index = scope.decls.len();
            scope
                .by_name
                .entry(decl.name.clone())
                .or_default()
                .push(decl_index);
            declarations.insert(decl.id.clone(), decl_info);
            for field in decl.fields.values() {
                if let Some(entry) = lock.entry_for(&field.path) {
                    declarations.insert(
                        entry.id.clone(),
                        DeclInfo {
                            entry: entry.clone(),
                            file: path.clone(),
                            service: service.clone(),
                            span: field.span,
                        },
                    );
                }
            }
            for value in decl.values.values() {
                if let Some(entry) = lock.entry_for(&value.path) {
                    declarations.insert(
                        entry.id.clone(),
                        DeclInfo {
                            entry: entry.clone(),
                            file: path.clone(),
                            service: service.clone(),
                            span: value.span,
                        },
                    );
                }
            }
            for param in decl.params.values() {
                if let Some(entry) = lock.entry_for(&param.path) {
                    declarations.insert(
                        entry.id.clone(),
                        DeclInfo {
                            entry: entry.clone(),
                            file: path.clone(),
                            service: service.clone(),
                            span: param.span,
                        },
                    );
                }
            }
            scope.decls.push(decl);
            // Apply ids from scope to the actual AST node.
            apply_decl_ids(
                &mut workspace.files[file_index].ast.items[artifact_index],
                &scope.decls[decl_index],
            );
        }
    }

    // Pass 2: references.
    let mut usages = Vec::new();
    for file_index in 0..workspace.files.len() {
        let path = workspace.files[file_index].path.clone();
        if !closure.contains_file(&path) {
            continue;
        }
        let service = workspace.files[file_index].service.clone();
        let mut artifact_index = 0;
        while artifact_index < workspace.files[file_index].ast.items.len() {
            let artifact = &mut workspace.files[file_index].ast.items[artifact_index];
            let known = artifact
                .id
                .as_ref()
                .map(|id| scope.decls.iter().any(|decl| &decl.id == id))
                .unwrap_or(false);
            if known {
                let mut ctx = Ctx {
                    scope: &scope,
                    service: service.as_deref(),
                    locals: locals_of(artifact),
                    file: &path,
                    usages: Vec::new(),
                };
                annotate_param_types(artifact, &mut ctx);
                annotate_members(&mut artifact.body, &mut ctx);
                usages.extend(ctx.usages);
            }
            artifact_index += 1;
        }
    }

    lock.retain(&used_paths);
    SemanticIndex {
        lock,
        declarations,
        usages,
        outside: outside_files(workspace, closure),
    }
}

fn assign_artifact(
    artifact: &mut Artifact,
    service: Option<&str>,
    file: &str,
    lock: &mut Lock,
    used: &mut BTreeSet<String>,
) -> LockEntry {
    let _ = file;
    let kind = artifact.artifact_kind.to_ascii_lowercase();
    let path = if artifact.artifact_kind == "Application" {
        format!("{}#application", artifact.name)
    } else if artifact.artifact_kind == "Service" {
        service_path(artifact.name.as_str())
    } else {
        artifact_path(
            service.unwrap_or(""),
            &artifact.artifact_kind,
            &artifact.name,
        )
    };
    let id = lock.ensure(&path, &kind, &artifact.name);
    artifact.id = Some(id.clone());
    used.insert(path.clone());

    let service_uri = service.unwrap_or("");
    if let Some(Member::Block { name, body, .. }) = artifact
        .body
        .iter_mut()
        .find(|member| matches!(member, Member::Block { name, .. } if name == "fields"))
    {
        let _ = name;
        for member in body {
            if let Member::PropertyAssignment { name, id, .. } = member {
                let field_path = field_path(service_uri, &artifact.name, name);
                let field_id = lock.ensure(&field_path, "field", name);
                *id = Some(field_id);
                used.insert(field_path);
            }
        }
    }
    for member in &mut artifact.body {
        if let Member::EnumValueDeclaration { name, id, .. } = member {
            let value_path = value_path(service_uri, &artifact.name, name);
            let value_id = lock.ensure(&value_path, "value", name);
            *id = Some(value_id);
            used.insert(value_path);
        }
    }
    for param in &mut artifact.params {
        let param_path = param_path(
            service_uri,
            &artifact.artifact_kind,
            &artifact.name,
            &param.name,
        );
        let param_id = lock.ensure(&param_path, "param", &param.name);
        param.id = Some(param_id);
        used.insert(param_path);
    }

    lock.entry_for(&path).cloned().unwrap_or(LockEntry {
        id,
        path,
        kind,
        name: artifact.name.clone(),
    })
}

fn annotate_param_types(artifact: &mut Artifact, ctx: &mut Ctx<'_>) {
    for param in &mut artifact.params {
        let Some(type_ref) = param.type_ref.clone() else {
            continue;
        };
        if let Some(decl) = ctx.scope.artifact(base_type(&type_ref), ctx.service) {
            param.type_ref_id = Some(decl.id.clone());
            ctx.usages.push(Usage {
                id: decl.id.clone(),
                file: ctx.file.to_string(),
                span: param.span,
                kind: "typeRef".to_string(),
            });
        }
    }
}

fn build_decl(artifact: &Artifact, entry: LockEntry, service: Option<String>) -> Decl {
    let mut fields = BTreeMap::new();
    let mut values = BTreeMap::new();
    let mut params = BTreeMap::new();

    if let Some(Member::Block { name, body, .. }) = artifact
        .body
        .iter()
        .find(|member| matches!(member, Member::Block { name, .. } if name == "fields"))
    {
        let _ = name;
        for member in body {
            if let Member::PropertyAssignment {
                name,
                value,
                span,
                id,
            } = member
            {
                fields.insert(
                    name.clone(),
                    FieldDecl {
                        id: id.clone().unwrap_or_default(),
                        path: field_path(service_ref(&service), &artifact.name, name),
                        type_ref: type_ref_of(value),
                        span: *span,
                    },
                );
            }
        }
    }
    for member in &artifact.body {
        if let Member::EnumValueDeclaration { name, span, id, .. } = member {
            values.insert(
                name.clone(),
                ValueDecl {
                    id: id.clone().unwrap_or_default(),
                    path: value_path(service_ref(&service), &artifact.name, name),
                    span: *span,
                },
            );
        }
    }
    for param in &artifact.params {
        params.insert(
            param.name.clone(),
            ParamDecl {
                id: param.id.clone().unwrap_or_default(),
                path: param_path(
                    service_ref(&service),
                    &artifact.artifact_kind,
                    &artifact.name,
                    &param.name,
                ),
                span: param.span,
            },
        );
    }

    Decl {
        id: entry.id.clone(),
        kind: artifact.artifact_kind.to_ascii_lowercase(),
        name: artifact.name.clone(),
        service,
        fields,
        values,
        params,
    }
}

/// Copies ids computed in `decl` onto the real AST node (and its children).
fn apply_decl_ids(artifact: &mut Artifact, decl: &Decl) {
    artifact.id = Some(decl.id.clone());
    if let Some(Member::Block { name, body, .. }) = artifact
        .body
        .iter_mut()
        .find(|member| matches!(member, Member::Block { name, .. } if name == "fields"))
    {
        let _ = name;
        for member in body {
            if let Member::PropertyAssignment { name, id, .. } = member {
                if let Some(field) = decl.fields.get(name) {
                    *id = Some(field.id.clone());
                }
            }
        }
    }
    for member in &mut artifact.body {
        if let Member::EnumValueDeclaration { name, id, .. } = member {
            if let Some(value) = decl.values.get(name) {
                *id = Some(value.id.clone());
            }
        }
    }
    for (index, param) in artifact.params.iter_mut().enumerate() {
        if let Some(decl_param) = decl.params.get(&param.name) {
            param.id = Some(decl_param.id.clone());
        }
        let _ = index;
    }
}

fn service_ref(service: &Option<String>) -> &str {
    service.as_deref().unwrap_or("")
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

fn annotate_members(members: &mut [Member], ctx: &mut Ctx<'_>) {
    for member in members {
        match member {
            Member::PropertyAssignment { name, value, .. } => {
                annotate_property(name, value, ctx);
            }
            Member::Block { body, .. } | Member::EnumValueDeclaration { body, .. } => {
                annotate_members(body, ctx);
            }
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

fn annotate_service_ref(uri: &str, span: Span, target: &mut Option<String>, ctx: &mut Ctx<'_>) {
    let Some(decl) = ctx.scope.artifact(uri, None) else {
        return;
    };
    if decl.kind != "service" {
        return;
    }
    *target = Some(decl.id.clone());
    ctx.usages.push(Usage {
        id: decl.id.clone(),
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
        "fanOut" => annotate_expr(value, ctx),
        _ => annotate_expr(value, ctx),
    }
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
                    let target = decl.id.clone();
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
            ..
        } => {
            if let Some(ty) = type_ref.as_deref() {
                if let Some(decl) = ctx.scope.artifact(base_type(ty), ctx.service) {
                    *type_ref_id = Some(decl.id.clone());
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
                if let Some(field_id) = ctx.scope.field_id(base_type(&ty), name, ctx.service) {
                    *field_ref = Some(field_id.clone());
                    let name_start = span.end.saturating_sub(name.len());
                    ctx.usages.push(Usage {
                        id: field_id,
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

/// `variable == LITERAL`: resolve the enum value and replace the literal node.
fn annotate_enum_literal(variable: &Expr, literal: &mut Expr, ctx: &mut Ctx<'_>) {
    let Some(ty) = expr_type(variable, ctx) else {
        return;
    };
    replace_enum_literal(literal, &ty, ctx);
}

/// Replaces `LITERAL` by a `Reference` when it names a value of enum `ty`.
fn replace_enum_literal(literal: &mut Expr, ty: &str, ctx: &mut Ctx<'_>) {
    let Some(name) = literal.as_ident().map(str::to_string) else {
        return;
    };
    let Some(value_id) = ctx.scope.value_id(base_type(ty), &name, ctx.service) else {
        return;
    };
    if let Expr::Identifier { span, .. } = literal {
        let span = *span;
        let name = name.clone();
        *literal = Expr::Reference {
            ref_kind: "enumValue".to_string(),
            name,
            target: value_id.clone(),
            span,
        };
        ctx.usages.push(Usage {
            id: value_id,
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
                        if let Some(value_id) =
                            ctx.scope.value_id(base_type(type_ref), &name, ctx.service)
                        {
                            if let Expr::Identifier { span, .. } = init {
                                let span = *span;
                                *init = Expr::Reference {
                                    ref_kind: "enumValue".to_string(),
                                    name,
                                    target: value_id.clone(),
                                    span,
                                };
                                ctx.usages.push(Usage {
                                    id: value_id,
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
