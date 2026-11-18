//! Consultas sobre o AST usadas por hover, completion, símbolos e definição.

use archol_core::{Artifact, Expr, Member, SourceFile, Span, Stmt};

pub fn word_at(text: &str, offset: usize) -> Option<(String, Span)> {
    let bytes = text.as_bytes();
    let mut start = offset.min(bytes.len());
    let mut end = start;
    while start > 0 && is_word(bytes[start - 1]) {
        start -= 1;
    }
    while end < bytes.len() && is_word(bytes[end]) {
        end += 1;
    }
    if start == end {
        return None;
    }
    Some((text[start..end].to_string(), Span::new(start, end)))
}

fn is_word(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

pub fn artifact_at(ast: &SourceFile, offset: usize) -> Option<&Artifact> {
    ast.items
        .iter()
        .filter(|item| contains(item.span, offset))
        .min_by_key(|item| item.span.end - item.span.start)
}

pub fn contains(span: Span, offset: usize) -> bool {
    span.start <= offset && offset <= span.end
}

/// Valor de enum (`value SIMPLES { ... }`) que contém o offset.
pub fn enum_value_at(ast: &SourceFile, offset: usize) -> Option<(&Artifact, &str)> {
    let artifact = artifact_at(ast, offset)?;
    let mut found = None;
    walk(&artifact.body, &mut |member| {
        if let Member::EnumValueDeclaration { name, span, .. } = member {
            if contains(*span, offset) {
                found = Some(name.as_str());
            }
        }
    });
    found.map(|name| (artifact, name))
}

pub fn const_at(ast: &SourceFile, offset: usize) -> Option<(&str, Option<&str>)> {
    let mut found = None;
    for artifact in &ast.items {
        walk(&artifact.body, &mut |member| {
            if let Member::Statement {
                statement:
                    Stmt::ConstDeclaration {
                        name,
                        type_ref,
                        span,
                        ..
                    }
                    | Stmt::LetDeclaration {
                        name,
                        type_ref,
                        span,
                        ..
                    },
                ..
            } = member
            {
                if contains(*span, offset) {
                    found = Some((name.as_str(), type_ref.as_deref()));
                }
            }
        });
    }
    found
}

pub fn param_type<'a>(artifact: &'a Artifact, name: &str) -> Option<&'a str> {
    artifact
        .params
        .iter()
        .find(|param| param.name == name)
        .and_then(|param| param.type_ref.as_deref())
}

/// Tipo declarado de uma variável local (parâmetro ou const/let).
pub fn local_type(artifact: &Artifact, name: &str) -> Option<String> {
    if let Some(ty) = param_type(artifact, name) {
        return Some(ty.to_string());
    }
    let mut found = None;
    walk(&artifact.body, &mut |member| {
        if let Member::Statement {
            statement:
                Stmt::ConstDeclaration {
                    name: decl_name,
                    type_ref: Some(ty),
                    ..
                }
                | Stmt::LetDeclaration {
                    name: decl_name,
                    type_ref: Some(ty),
                    ..
                },
            ..
        } = member
        {
            if decl_name == name {
                found = Some(ty.clone());
            }
        }
    });
    found
}

/// Nome da variável à esquerda de uma comparação que contém o offset.
pub fn comparison_variable_at(ast: &SourceFile, offset: usize) -> Option<&Artifact> {
    let artifact = artifact_at(ast, offset)?;
    let mut found = false;
    walk(&artifact.body, &mut |member| {
        if let Member::Statement {
            statement: Stmt::IfStatement { condition, .. },
            ..
        } = member
        {
            if contains(condition.span(), offset) {
                found = true;
            }
        }
    });
    if found {
        Some(artifact)
    } else {
        None
    }
}

pub fn walk<'a>(members: &'a [Member], visit: &mut impl FnMut(&'a Member)) {
    for member in members {
        visit(member);
        match member {
            Member::Block { body, .. } | Member::EnumValueDeclaration { body, .. } => {
                walk(body, visit)
            }
            Member::Statement { statement, .. } => walk_stmt(statement, visit),
            Member::PropertyAssignment { value, .. } => walk_expr(value, visit),
            _ => {}
        }
    }
}

fn walk_stmt<'a>(statement: &'a Stmt, visit: &mut impl FnMut(&'a Member)) {
    match statement {
        Stmt::IfStatement {
            then_branch,
            else_branch,
            ..
        } => {
            walk(then_branch, visit);
            if let Some(else_branch) = else_branch {
                walk(else_branch, visit);
            }
        }
        Stmt::TryStatement { body, catches, .. } => {
            walk(body, visit);
            for catch in catches {
                walk(&catch.body, visit);
            }
        }
        _ => {}
    }
}

fn walk_expr<'a>(expr: &'a Expr, visit: &mut impl FnMut(&'a Member)) {
    if let Expr::Object { members, .. } = expr {
        walk(members, visit);
    }
}

/// Expressão mais profunda que contém o offset.
pub fn expr_at(ast: &SourceFile, offset: usize) -> Option<&Expr> {
    let mut best: Option<&Expr> = None;
    for artifact in &ast.items {
        walk_exprs(&artifact.body, &mut |expr| {
            if contains(expr.span(), offset) {
                let size = expr.span().end - expr.span().start;
                let better = best
                    .map(|current| size < current.span().end - current.span().start)
                    .unwrap_or(true);
                if better {
                    best = Some(expr);
                }
            }
        });
    }
    best
}

/// Propriedade mais interna (`nome: valor`) que contém o offset.
pub fn property_at(ast: &SourceFile, offset: usize) -> Option<(&str, &Expr)> {
    let mut best: Option<(&str, &Expr)> = None;
    for artifact in &ast.items {
        walk_properties(&artifact.body, &mut |name, value| {
            if contains(value.span(), offset) {
                let size = value.span().end - value.span().start;
                let better = best
                    .map(|(_, current)| size < current.span().end - current.span().start)
                    .unwrap_or(true);
                if better {
                    best = Some((name, value));
                }
            }
        });
    }
    best
}

/// Comparação (`==`, `===`, `!=`, `!==`) mais interna que contém o offset.
pub fn comparison_at(ast: &SourceFile, offset: usize) -> Option<&Expr> {
    let mut best: Option<&Expr> = None;
    for artifact in &ast.items {
        walk_exprs(&artifact.body, &mut |expr| {
            if let Expr::Binary { op, .. } = expr {
                if matches!(op.as_str(), "==" | "===" | "!=" | "!==")
                    && contains(expr.span(), offset)
                {
                    let size = expr.span().end - expr.span().start;
                    let better = best
                        .map(|current| size < current.span().end - current.span().start)
                        .unwrap_or(true);
                    if better {
                        best = Some(expr);
                    }
                }
            }
        });
    }
    best
}

pub fn walk_exprs<'a>(members: &'a [Member], visit: &mut impl FnMut(&'a Expr)) {
    for member in members {
        match member {
            Member::PropertyAssignment { value, .. } => visit_expr(value, visit),
            Member::Block { body, .. } | Member::EnumValueDeclaration { body, .. } => {
                walk_exprs(body, visit)
            }
            Member::Statement { statement, .. } => walk_stmt_exprs(statement, visit),
            _ => {}
        }
    }
}

fn visit_expr<'a>(expr: &'a Expr, visit: &mut impl FnMut(&'a Expr)) {
    visit(expr);
    match expr {
        Expr::List { elements, .. } => {
            for element in elements {
                visit_expr(element, visit);
            }
        }
        Expr::Object { members, .. } => walk_exprs(members, visit),
        Expr::Call { args, .. } => {
            for arg in args {
                visit_expr(arg, visit);
            }
        }
        Expr::MemberAccess { object, .. } => visit_expr(object, visit),
        Expr::Binary { left, right, .. } => {
            visit_expr(left, visit);
            visit_expr(right, visit);
        }
        Expr::Unary { operand, .. } => visit_expr(operand, visit),
        Expr::NamedArgument { value, .. } => visit_expr(value, visit),
        _ => {}
    }
}

fn walk_stmt_exprs<'a>(statement: &'a Stmt, visit: &mut impl FnMut(&'a Expr)) {
    match statement {
        Stmt::ExpressionStatement { expression, .. } => visit_expr(expression, visit),
        Stmt::ConstDeclaration { init, .. } | Stmt::LetDeclaration { init, .. } => {
            if let Some(init) = init {
                visit_expr(init, visit);
            }
        }
        Stmt::IfStatement {
            condition,
            then_branch,
            else_branch,
            ..
        } => {
            visit_expr(condition, visit);
            walk_exprs(then_branch, visit);
            if let Some(else_branch) = else_branch {
                walk_exprs(else_branch, visit);
            }
        }
        Stmt::TryStatement { body, catches, .. } => {
            walk_exprs(body, visit);
            for catch in catches {
                walk_exprs(&catch.body, visit);
            }
        }
        Stmt::ReturnStatement { value, .. } => {
            if let Some(value) = value {
                visit_expr(value, visit);
            }
        }
        Stmt::RawStatement { .. } => {}
    }
}

fn walk_properties<'a>(members: &'a [Member], visit: &mut impl FnMut(&'a str, &'a Expr)) {
    for member in members {
        if let Member::PropertyAssignment { name, value, .. } = member {
            visit(name, value);
            if let Expr::Object { members, .. } = value {
                walk_properties(members, visit);
            }
        }
        match member {
            Member::Block { body, .. } | Member::EnumValueDeclaration { body, .. } => {
                walk_properties(body, visit)
            }
            Member::Statement { statement, .. } => walk_stmt_properties(statement, visit),
            _ => {}
        }
    }
}

fn walk_stmt_properties<'a>(statement: &'a Stmt, visit: &mut impl FnMut(&'a str, &'a Expr)) {
    match statement {
        Stmt::IfStatement {
            then_branch,
            else_branch,
            ..
        } => {
            walk_properties(then_branch, visit);
            if let Some(else_branch) = else_branch {
                walk_properties(else_branch, visit);
            }
        }
        Stmt::TryStatement { body, catches, .. } => {
            walk_properties(body, visit);
            for catch in catches {
                walk_properties(&catch.body, visit);
            }
        }
        _ => {}
    }
}

/// Símbolos do documento: artefatos e seus membros de primeiro nível.
pub fn document_symbols(ast: &SourceFile, text: &str) -> Vec<serde_json::Value> {
    ast.items
        .iter()
        .map(|artifact| {
            let children: Vec<_> = artifact
                .body
                .iter()
                .filter_map(|member| {
                    let name = member.name()?;
                    if name.is_empty() {
                        return None;
                    }
                    Some(serde_json::json!({
                        "name": name,
                        "kind": member_kind(member),
                        "range": range(text, member.span()),
                        "selectionRange": range(text, member.span()),
                    }))
                })
                .collect();
            serde_json::json!({
                "name": artifact.name,
                "detail": artifact.artifact_kind,
                "kind": 5,
                "range": range(text, artifact.span),
                "selectionRange": range(text, artifact.span),
                "children": children,
            })
        })
        .collect()
}

fn member_kind(member: &Member) -> i64 {
    match member {
        Member::EnumValueDeclaration { .. } => 22,
        Member::Statement { .. } => 12,
        Member::Block { .. } => 2,
        _ => 7,
    }
}

pub fn range(text: &str, span: Span) -> serde_json::Value {
    let (sl, sc) = crate::util::position_of(text, span.start);
    let (el, ec) = crate::util::position_of(text, span.end);
    serde_json::json!({
        "start": { "line": sl, "character": sc },
        "end": { "line": el, "character": ec }
    })
}
