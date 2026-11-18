//! Check semântico: valida valores de enum, tipos, papéis, referências e
//! duplicidade. Enum é resolvido no serviço do arquivo (incluindo `Alias.Tipo`).

use std::collections::{BTreeMap, HashMap};

use archol_core::{Artifact, Diagnostic, Expr, Member, Stmt};

use crate::index::{ident_list, Symbols, PRIMITIVES};
use crate::workspace::Workspace;

pub fn check(workspace: &Workspace, symbols: &Symbols) -> Vec<(String, Diagnostic)> {
    let mut diagnostics = Vec::new();
    let mut seen: BTreeMap<(Option<String>, String, String), usize> = BTreeMap::new();

    for (file, artifact) in workspace.artifacts() {
        let key = (
            file.service.clone(),
            artifact.artifact_kind.clone(),
            artifact.name.clone(),
        );
        if let Some(first) = seen.get(&key) {
            diagnostics.push((
                file.path.clone(),
                Diagnostic::error(
                    "E_DUP_ARTIFACT",
                    format!(
                        "{} {} duplicado no serviço (primeira declaração no offset {first})",
                        artifact.artifact_kind, artifact.name
                    ),
                    artifact.span,
                ),
            ));
        } else {
            seen.insert(key, artifact.span.start);
        }
        let mut local = Vec::new();
        check_artifact(symbols, file.service.as_deref(), artifact, &mut local);
        diagnostics.extend(local.into_iter().map(|d| (file.path.clone(), d)));
    }
    diagnostics
}

fn check_artifact(
    symbols: &Symbols,
    service: Option<&str>,
    artifact: &Artifact,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for param in &artifact.params {
        if let Some(type_ref) = &param.type_ref {
            check_type_name(symbols, service, type_ref, param.span, diagnostics);
        }
    }
    let mut locals: HashMap<String, Option<String>> = HashMap::new();
    for param in &artifact.params {
        locals.insert(param.name.clone(), param.type_ref.clone());
    }
    collect_locals(&artifact.body, &mut locals);
    check_members(symbols, service, &artifact.body, &locals, diagnostics);
}

fn collect_locals(members: &[Member], locals: &mut HashMap<String, Option<String>>) {
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
            Member::PropertyAssignment {
                value: Expr::Object { members, .. },
                ..
            } => collect_locals(members, locals),
            _ => {}
        }
    }
}

fn check_members(
    symbols: &Symbols,
    service: Option<&str>,
    members: &[Member],
    locals: &HashMap<String, Option<String>>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for member in members {
        match member {
            Member::PropertyAssignment { name, value, .. } => {
                check_value(symbols, service, name, value, locals, diagnostics);
            }
            Member::Block { body, .. } | Member::EnumValueDeclaration { body, .. } => {
                check_members(symbols, service, body, locals, diagnostics);
            }
            Member::Statement { statement, .. } => {
                check_statement(symbols, service, statement, locals, diagnostics)
            }
            _ => {}
        }
    }
}

fn check_statement(
    symbols: &Symbols,
    service: Option<&str>,
    statement: &Stmt,
    locals: &HashMap<String, Option<String>>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    match statement {
        Stmt::ConstDeclaration { type_ref, init, .. }
        | Stmt::LetDeclaration { type_ref, init, .. } => {
            if let (Some(type_ref), Some(init)) = (type_ref, init) {
                if let Some(enum_info) = symbols.resolve_enum(type_ref, service) {
                    if let Some(value_name) = init.as_ident() {
                        if !enum_info.values.iter().any(|v| v.name == value_name) {
                            diagnostics.push(Diagnostic::error(
                                "E_ENUM_VALUE",
                                format!(
                                    "'{value_name}' não é valor de {type_ref} (valores: {})",
                                    enum_values_text(enum_info)
                                ),
                                init.span(),
                            ));
                        }
                    }
                }
            }
            if let Some(init) = init {
                check_comparisons(symbols, service, init, locals, diagnostics);
            }
        }
        Stmt::IfStatement {
            condition,
            then_branch,
            else_branch,
            ..
        } => {
            check_comparisons(symbols, service, condition, locals, diagnostics);
            check_members(symbols, service, then_branch, locals, diagnostics);
            if let Some(else_branch) = else_branch {
                check_members(symbols, service, else_branch, locals, diagnostics);
            }
        }
        Stmt::TryStatement { body, catches, .. } => {
            check_members(symbols, service, body, locals, diagnostics);
            for catch in catches {
                check_members(symbols, service, &catch.body, locals, diagnostics);
            }
        }
        Stmt::ReturnStatement { value, .. } => {
            if let Some(value) = value {
                check_comparisons(symbols, service, value, locals, diagnostics);
            }
        }
        Stmt::ExpressionStatement { expression, .. } => {
            check_comparisons(symbols, service, expression, locals, diagnostics);
        }
        Stmt::RawStatement { .. } => {}
    }
}

fn check_value(
    symbols: &Symbols,
    service: Option<&str>,
    name: &str,
    value: &Expr,
    locals: &HashMap<String, Option<String>>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    match name {
        "roles" => {
            for role in ident_list(value) {
                if is_upper_name(role) && !symbols.roles.contains(role) {
                    diagnostics.push(Diagnostic::error(
                        "E_UNKNOWN_ROLE",
                        format!("Papel '{role}' não declarado no workspace"),
                        value.span(),
                    ));
                }
            }
        }
        "about" | "document" | "source" | "appliesTo" | "sources" | "fanOut" => {
            for target in ident_list(value) {
                check_artifact_ref(symbols, target, value, diagnostics);
            }
        }
        _ => {}
    }

    if let Expr::Object {
        type_ref: Some(type_ref),
        members,
        ..
    } = value
    {
        check_type_ref(symbols, service, type_ref, value, diagnostics);
        check_members(symbols, service, members, locals, diagnostics);
    }
    check_comparisons(symbols, service, value, locals, diagnostics);
}

fn check_type_ref(
    symbols: &Symbols,
    service: Option<&str>,
    type_ref: &str,
    value: &Expr,
    diagnostics: &mut Vec<Diagnostic>,
) {
    check_type_name(symbols, service, type_ref, value.span(), diagnostics);
}

fn check_type_name(
    symbols: &Symbols,
    _service: Option<&str>,
    type_ref: &str,
    span: archol_core::Span,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for part in type_ref.split('|') {
        let part = part.trim();
        if part.contains('.') {
            continue;
        }
        let base = part.trim_end_matches("[]").trim();
        if base.is_empty() || PRIMITIVES.contains(&base) {
            continue;
        }
        if is_upper_name(base) && !symbols.knows(base) {
            diagnostics.push(Diagnostic::error(
                "E_UNKNOWN_TYPE",
                format!("Tipo '{base}' não declarado no workspace"),
                span,
            ));
        }
    }
}

fn check_artifact_ref(
    symbols: &Symbols,
    target: &str,
    value: &Expr,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if !is_upper_name(target) {
        return;
    }
    if !symbols.knows(target) {
        diagnostics.push(Diagnostic::error(
            "E_UNKNOWN_REF",
            format!("Referência '{target}' não declarada no workspace"),
            value.span(),
        ));
    }
}

/// Valida `var == VALOR` quando `var` tem tipo de enum conhecido.
fn check_comparisons(
    symbols: &Symbols,
    service: Option<&str>,
    expr: &Expr,
    locals: &HashMap<String, Option<String>>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if let Expr::Binary {
        op, left, right, ..
    } = expr
    {
        if matches!(op.as_str(), "==" | "===" | "!=" | "!==") {
            if let Some((variable, literal)) = comparison_parts(left, right) {
                if let Some(Some(type_ref)) = locals.get(variable) {
                    if let Some(enum_info) = symbols.resolve_enum(type_ref, service) {
                        if let Some(value_name) = literal.as_ident() {
                            if !enum_info.values.iter().any(|v| v.name == value_name) {
                                diagnostics.push(Diagnostic::error(
                                    "E_ENUM_VALUE",
                                    format!(
                                        "'{value_name}' não é valor de {type_ref} (valores: {})",
                                        enum_values_text(enum_info)
                                    ),
                                    literal.span(),
                                ));
                            }
                        }
                    }
                }
            }
        }
        check_comparisons(symbols, service, left, locals, diagnostics);
        check_comparisons(symbols, service, right, locals, diagnostics);
    }
}

fn comparison_parts<'a>(left: &'a Expr, right: &'a Expr) -> Option<(&'a str, &'a Expr)> {
    match (left.as_ident(), right.as_ident()) {
        (Some(left_name), Some(right_name)) => {
            if is_upper_name(right_name) {
                Some((left_name, right))
            } else if is_upper_name(left_name) {
                Some((right_name, left))
            } else {
                None
            }
        }
        _ => None,
    }
}

fn enum_values_text(info: &crate::index::EnumInfo) -> String {
    info.values
        .iter()
        .map(|v| v.name.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}

pub fn is_upper_name(name: &str) -> bool {
    name.chars()
        .next()
        .map(|c| c.is_uppercase())
        .unwrap_or(false)
}
