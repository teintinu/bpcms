//! Compile-time shape literals.
//!
//! A string assigned to a value whose declared type is a Shape is parsed with
//! the shape's declarative `parse` metadata. The AST node carries the shape
//! reference, the raw text, the normalized value and, for quantities, the base
//! unit. Invalid literals are compile errors.

use std::collections::BTreeMap;

use archol_core::{Artifact, Diagnostic, Expr, Member, ShapeData, Stmt};

use crate::application::Closure;
use crate::index::Symbols;
use crate::workspace::Workspace;

#[derive(Clone, Debug, Default)]
struct ShapeInfo {
    name: String,
    base: Option<String>,
    parse: Option<String>,
    validate: Option<String>,
    normalize: Option<String>,
    units: BTreeMap<String, f64>,
    unit: Option<String>,
    fields: BTreeMap<String, String>,
}

impl ShapeInfo {
    fn base_unit(&self) -> Option<String> {
        self.units
            .iter()
            .find(|(_, factor)| (**factor - 1.0).abs() < f64::EPSILON)
            .map(|(unit, _)| unit.clone())
            .or_else(|| self.unit.clone())
            .or_else(|| self.units.keys().next().cloned())
    }
}

/// Resolves every shape-typed string literal in the application closure.
pub fn resolve(
    workspace: &mut Workspace,
    symbols: &Symbols,
    closure: &Closure,
) -> Vec<(String, Diagnostic)> {
    let catalog = catalog(workspace);
    let mut diagnostics = Vec::new();

    for file in workspace
        .files
        .iter_mut()
        .filter(|file| closure.contains_file(&file.path))
    {
        let path = file.path.clone();
        let service = file.service.clone();
        for artifact in &mut file.ast.items {
            let context = if matches!(artifact.artifact_kind(), "Parameter" | "Secret") {
                property(&artifact.body, "type").and_then(text_value)
            } else {
                None
            };
            resolve_members(
                &mut artifact.body,
                context.as_deref(),
                &catalog,
                symbols,
                service.as_deref(),
                &path,
                &mut diagnostics,
            );
        }
    }
    diagnostics
}

fn resolve_members(
    members: &mut [Member],
    context_type: Option<&str>,
    catalog: &BTreeMap<(Option<String>, String), ShapeInfo>,
    symbols: &Symbols,
    service: Option<&str>,
    path: &str,
    diagnostics: &mut Vec<(String, Diagnostic)>,
) {
    for member in members {
        match member {
            Member::PropertyAssignment {
                name,
                type_ref,
                value,
                ..
            } => {
                let declared = type_ref.clone().or_else(|| {
                    matches!(name.as_str(), "default" | "min" | "max")
                        .then(|| context_type.map(str::to_string))
                        .flatten()
                });
                resolve_value(
                    value,
                    declared.as_deref(),
                    catalog,
                    symbols,
                    service,
                    path,
                    diagnostics,
                )
            }
            Member::Block { body, .. } | Member::EnumValueDeclaration { body, .. } => {
                resolve_members(body, None, catalog, symbols, service, path, diagnostics)
            }
            Member::Statement { statement, .. } => {
                resolve_statement(statement, catalog, symbols, service, path, diagnostics)
            }
            _ => {}
        }
    }
}

fn resolve_statement(
    statement: &mut Stmt,
    catalog: &BTreeMap<(Option<String>, String), ShapeInfo>,
    symbols: &Symbols,
    service: Option<&str>,
    path: &str,
    diagnostics: &mut Vec<(String, Diagnostic)>,
) {
    match statement {
        Stmt::ConstDeclaration { type_ref, init, .. }
        | Stmt::LetDeclaration { type_ref, init, .. } => {
            if let Some(init) = init {
                resolve_value(
                    init,
                    type_ref.as_deref(),
                    catalog,
                    symbols,
                    service,
                    path,
                    diagnostics,
                );
            }
        }
        Stmt::IfStatement {
            then_branch,
            else_branch,
            ..
        } => {
            resolve_members(
                then_branch,
                None,
                catalog,
                symbols,
                service,
                path,
                diagnostics,
            );
            if let Some(else_branch) = else_branch {
                resolve_members(
                    else_branch,
                    None,
                    catalog,
                    symbols,
                    service,
                    path,
                    diagnostics,
                );
            }
        }
        Stmt::TryStatement { body, catches, .. } => {
            resolve_members(body, None, catalog, symbols, service, path, diagnostics);
            for catch in catches {
                resolve_members(
                    &mut catch.body,
                    None,
                    catalog,
                    symbols,
                    service,
                    path,
                    diagnostics,
                );
            }
        }
        _ => {}
    }
}

fn resolve_value(
    value: &mut Expr,
    declared_type: Option<&str>,
    catalog: &BTreeMap<(Option<String>, String), ShapeInfo>,
    symbols: &Symbols,
    service: Option<&str>,
    path: &str,
    diagnostics: &mut Vec<(String, Diagnostic)>,
) {
    let shape = declared_type.and_then(|ty| resolve_shape(catalog, symbols, service, ty));

    if let Some(info) = shape {
        if !is_shape_literal(info) && !matches!(value, Expr::Object { .. }) {
            return;
        }
        match value {
            Expr::StringLiteral {
                value: raw, span, ..
            } => match parse_literal(raw, info) {
                Ok((parsed, unit)) => {
                    *value = Expr::ShapeLiteral {
                        shape: declared_type.unwrap_or_default().to_string(),
                        raw: raw.clone(),
                        value: parsed,
                        unit,
                        span: *span,
                    };
                }
                Err(message) => diagnostics.push((
                    path.to_string(),
                    Diagnostic::error("E_SHAPE_LITERAL", message, *span),
                )),
            },
            Expr::NumberLiteral { span, value } => diagnostics.push((
                path.to_string(),
                Diagnostic::error(
                    "E_SHAPE_LITERAL_QUOTED",
                    format!("Shape literal `{value}` must be quoted; use \"{value}\""),
                    *span,
                ),
            )),
            Expr::Object { members, .. } => {
                for member in members {
                    if let Member::PropertyAssignment {
                        name,
                        type_ref,
                        value,
                        ..
                    } = member
                    {
                        let field_type =
                            type_ref.clone().or_else(|| info.fields.get(name).cloned());
                        resolve_value(
                            value,
                            field_type.as_deref(),
                            catalog,
                            symbols,
                            service,
                            path,
                            diagnostics,
                        );
                    }
                }
            }
            _ => {}
        }
        return;
    }

    if let Expr::Object {
        type_ref: Some(object_type),
        members,
        ..
    } = value
    {
        let object_type = object_type.clone();
        for member in members {
            if let Member::PropertyAssignment {
                name,
                type_ref,
                value,
                ..
            } = member
            {
                let member_type = if matches!(name.as_str(), "default" | "min" | "max") {
                    Some(object_type.as_str())
                } else {
                    type_ref.as_deref()
                };
                resolve_value(
                    value,
                    member_type,
                    catalog,
                    symbols,
                    service,
                    path,
                    diagnostics,
                );
            }
        }
        return;
    }

    if let Expr::Object { members, .. } = value {
        for member in members {
            if let Member::PropertyAssignment {
                type_ref, value, ..
            } = member
            {
                resolve_value(
                    value,
                    type_ref.as_deref(),
                    catalog,
                    symbols,
                    service,
                    path,
                    diagnostics,
                );
            }
        }
    }
}

fn resolve_shape<'a>(
    catalog: &'a BTreeMap<(Option<String>, String), ShapeInfo>,
    symbols: &Symbols,
    service: Option<&str>,
    type_ref: &str,
) -> Option<&'a ShapeInfo> {
    let base = type_ref
        .split('|')
        .next()
        .unwrap_or(type_ref)
        .trim()
        .trim_end_matches("[]")
        .trim();
    if let Some((alias, local)) = base.split_once('.') {
        let target = service
            .and_then(|service| symbols.imports.get(service))
            .and_then(|imports| imports.get(alias))?;
        return catalog.get(&(Some(target.clone()), local.to_string()));
    }
    if let Some(service) = service {
        if let Some(info) = catalog.get(&(Some(service.to_string()), base.to_string())) {
            return Some(info);
        }
    }
    catalog
        .iter()
        .find(|((_, name), _)| name == base)
        .map(|(_, info)| info)
}

fn parse_literal(raw: &str, info: &ShapeInfo) -> Result<(ShapeData, Option<String>), String> {
    let text = raw.trim();
    if text.is_empty() {
        return Err("Shape literal cannot be empty".to_string());
    }
    if info.name == "Money" {
        return parse_money(text);
    }
    let is_quantity =
        info.parse.as_deref() == Some("quantity") || !info.units.is_empty() || info.unit.is_some();
    if is_quantity {
        return parse_quantity(text, info);
    }
    let mut text = text.to_string();
    match info.normalize.as_deref() {
        Some("lower-trim") => text = text.trim().to_lowercase(),
        Some("upper-trim") => text = text.trim().to_uppercase(),
        _ => {}
    }
    if let Some(validate) = info.validate.as_deref() {
        if !valid_text(&text, validate) {
            return Err(format!(
                "Invalid `{validate}` value in shape literal `{raw}`"
            ));
        }
    }
    Ok((ShapeData::Text(text), None))
}

fn valid_text(value: &str, validate: &str) -> bool {
    match validate {
        "email" => {
            let mut parts = value.split('@');
            let local = parts.next().unwrap_or_default();
            let domain = parts.next().unwrap_or_default();
            !local.is_empty()
                && !domain.is_empty()
                && domain.contains('.')
                && !value.chars().any(char::is_whitespace)
        }
        "phone" => {
            let digits = value.chars().filter(|ch| ch.is_ascii_digit()).count();
            digits >= 7
                && value
                    .chars()
                    .all(|ch| ch.is_ascii_digit() || matches!(ch, '+' | '-' | ' ' | '(' | ')'))
        }
        "digits" => !value.is_empty() && value.chars().all(|ch| ch.is_ascii_digit()),
        "year-month" => {
            let bytes = value.as_bytes();
            bytes.len() == 7
                && bytes[4] == b'-'
                && value[..4].chars().all(|ch| ch.is_ascii_digit())
                && value[5..].chars().all(|ch| ch.is_ascii_digit())
        }
        _ => true,
    }
}

fn parse_quantity(raw: &str, info: &ShapeInfo) -> Result<(ShapeData, Option<String>), String> {
    let (number_text, unit_text) = split_quantity(raw)?;
    let number: f64 = number_text
        .parse()
        .map_err(|_| format!("Invalid number in shape literal `{raw}`"))?;
    let unit_text = unit_text.trim();
    let (factor, base_unit) = if unit_text.is_empty() {
        let unit = info
            .base_unit()
            .ok_or_else(|| format!("Shape literal `{raw}` needs a unit"))?;
        (1.0, unit)
    } else if let Some(factor) = info.units.get(unit_text) {
        (
            *factor,
            info.base_unit().unwrap_or_else(|| unit_text.to_string()),
        )
    } else if info.unit.as_deref() == Some(unit_text) {
        (1.0, unit_text.to_string())
    } else {
        let units: Vec<&str> = info.units.keys().map(String::as_str).collect();
        return Err(if units.is_empty() {
            format!("Unknown unit `{unit_text}` in shape literal `{raw}`")
        } else {
            format!(
                "Unknown unit `{unit_text}` in shape literal `{raw}` (units: {})",
                units.join(", ")
            )
        });
    };
    let value = number * factor;
    Ok((number_value_from_f64(value), Some(base_unit)))
}

fn split_quantity(raw: &str) -> Result<(&str, &str), String> {
    let bytes = raw.as_bytes();
    let mut end = 0;
    if matches!(bytes.first(), Some(b'+' | b'-')) {
        end = 1;
    }
    let mut digits = 0;
    let mut dot = false;
    for (index, byte) in bytes.iter().enumerate().skip(end) {
        match byte {
            b'0'..=b'9' => {
                digits += 1;
                end = index + 1;
            }
            b'.' if !dot => {
                dot = true;
                end = index + 1;
            }
            _ => break,
        }
    }
    if digits == 0 {
        return Err(format!("Invalid shape literal `{raw}`: expected a number"));
    }
    Ok((&raw[..end], &raw[end..]))
}

fn parse_money(raw: &str) -> Result<(ShapeData, Option<String>), String> {
    let (number_text, currency_text) = split_quantity(raw)?;
    let number: f64 = number_text
        .parse()
        .map_err(|_| format!("Invalid number in Money literal `{raw}`"))?;
    let currency = currency_text.trim();
    if currency.is_empty() {
        return Err(format!("Money literal `{raw}` needs a currency"));
    }
    let mut value = BTreeMap::new();
    value.insert("raw".to_string(), number_value_from_f64(number));
    value.insert(
        "currency".to_string(),
        ShapeData::Text(currency.to_string()),
    );
    Ok((ShapeData::Object(value), None))
}

fn number_value_from_f64(value: f64) -> ShapeData {
    if value.fract() == 0.0 && value.abs() <= i64::MAX as f64 {
        ShapeData::Integer(value as i64)
    } else {
        ShapeData::Decimal(value)
    }
}

fn is_shape_literal(info: &ShapeInfo) -> bool {
    info.name == "Money"
        || info.parse.as_deref() == Some("quantity")
        || info.validate.is_some()
        || info.normalize.is_some()
        || !info.units.is_empty()
        || info.unit.is_some()
        || matches!(
            info.base.as_deref(),
            Some("video" | "audio" | "image" | "attachment")
        )
}

fn catalog(workspace: &Workspace) -> BTreeMap<(Option<String>, String), ShapeInfo> {
    let mut catalog = BTreeMap::new();
    for (file, artifact) in workspace.artifacts() {
        if artifact.artifact_kind() != "Shape" {
            continue;
        }
        let info = shape_info(artifact);
        catalog.insert((file.service.clone(), artifact.name.clone()), info);
    }
    catalog
}

fn shape_info(artifact: &Artifact) -> ShapeInfo {
    let mut info = ShapeInfo {
        name: artifact.name.clone(),
        ..ShapeInfo::default()
    };
    for member in &artifact.body {
        match member {
            Member::PropertyAssignment { name, value, .. } if name == "base" => {
                info.base = text_value(value);
            }
            Member::PropertyAssignment { name, value, .. } if name == "parse" => {
                info.parse = text_value(value);
            }
            Member::PropertyAssignment { name, value, .. } if name == "validate" => {
                info.validate = text_value(value);
            }
            Member::PropertyAssignment { name, value, .. } if name == "normalize" => {
                info.normalize = text_value(value);
            }
            Member::PropertyAssignment { name, value, .. } if name == "unit" => {
                info.unit = text_value(value);
            }
            Member::Block { name, body, .. } if name == "units" => {
                for unit in body {
                    let Member::PropertyAssignment { name, value, .. } = unit else {
                        continue;
                    };
                    if let Expr::Object { members, .. } = value {
                        if let Some(factor) = property(members, "factor").and_then(number_value) {
                            info.units.insert(name.clone(), factor);
                        }
                    }
                }
            }
            Member::Block { name, body, .. } if name == "fields" => {
                for field in body {
                    let Member::PropertyAssignment { name, value, .. } = field else {
                        continue;
                    };
                    if let Expr::Object {
                        type_ref: Some(type_ref),
                        ..
                    } = value
                    {
                        info.fields.insert(name.clone(), type_ref.clone());
                    }
                }
            }
            _ => {}
        }
    }
    info
}

fn property<'a>(members: &'a [Member], name: &str) -> Option<&'a Expr> {
    members.iter().find_map(|member| match member {
        Member::PropertyAssignment {
            name: key, value, ..
        } if key == name => Some(value),
        _ => None,
    })
}

fn text_value(value: &Expr) -> Option<String> {
    match value {
        Expr::StringLiteral { value, .. } => Some(value.clone()),
        Expr::Identifier { name, .. } => Some(name.clone()),
        Expr::MemberAccess { object, name, .. } => {
            Some(format!("{}.{}", text_value(object)?, name))
        }
        Expr::Binary {
            op, left, right, ..
        } if op == "-" => Some(format!("{}-{}", text_value(left)?, text_value(right)?)),
        _ => None,
    }
}

fn number_value(value: &Expr) -> Option<f64> {
    match value {
        Expr::NumberLiteral { value, .. } => value
            .split_whitespace()
            .next()
            .and_then(|number| number.parse().ok()),
        _ => None,
    }
}
