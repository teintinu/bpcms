//! I18N properties: type marking, string shorthand and locale filtering.
//!
//! `title`, `goal`, `instructions`, `short` and `tooltip` are `I18N` values: a
//! localized string, or a map from locale to string/plural forms. The compiler
//! marks the property with `typeRef: "std.bpcms.i18n"` and keeps only the locales
//! supported by the application (`locales`), so the builder never receives
//! translations for languages the app does not use.
//!
//! A string is the shorthand for the first locale declared by the owning
//! service (`locales`). The compiler expands it into the canonical locale map
//! (`{ "pt-BR": "..." }`), so the AST always tells the builder which locale the
//! text belongs to.

use std::collections::BTreeMap;

use archol_core::{Artifact, Expr, Member, Span, Stmt};

use crate::application::Closure;
use crate::workspace::Workspace;

/// Property names whose value has the `I18N` type.
const I18N_PROPERTIES: &[&str] = &["title", "goal", "instructions", "short", "tooltip"];

/// Locales declared by an artifact (`locales: [ "pt-BR", "en" ]`).
pub fn locales_of(artifact: &Artifact) -> Vec<String> {
    artifact
        .body
        .iter()
        .find_map(|member| match member {
            Member::PropertyAssignment { name, value, .. } if name == "locales" => {
                locale_list(value)
            }
            _ => None,
        })
        .unwrap_or_default()
}

/// Service URI -> supported locales, from every `service` declaration.
fn service_locales(workspace: &Workspace) -> BTreeMap<String, Vec<String>> {
    let mut locales = BTreeMap::new();
    for (_, artifact) in workspace.artifacts() {
        if artifact.artifact_kind != "Service" {
            continue;
        }
        locales.insert(artifact.name.clone(), locales_of(artifact));
    }
    locales
}

/// Marks every `I18N` property and drops locales the application does not use.
///
/// `app_locales` are the application's `locales`; an empty list disables
/// filtering (there is nothing to filter by).
pub fn resolve(workspace: &mut Workspace, closure: &Closure, app_locales: &[String]) {
    let services = service_locales(workspace);
    let app_default = app_locales.first().cloned();

    for file in workspace
        .files
        .iter_mut()
        .filter(|file| closure.contains_file(&file.path))
    {
        let supported = file
            .service
            .as_ref()
            .and_then(|service| services.get(service))
            .cloned()
            .unwrap_or_default();
        // A service always declares `locales`; the first one is the default
        // for the string shorthand. Only the application itself (no owning
        // service) uses the first app locale.
        let default_locale = if file.service.is_some() {
            supported.first().cloned()
        } else {
            app_default.clone()
        };

        for artifact in &mut file.ast.items {
            resolve_members(&mut artifact.body, app_locales, default_locale.as_deref());
        }
    }
}

fn resolve_members(members: &mut [Member], app_locales: &[String], default_locale: Option<&str>) {
    for member in members {
        match member {
            Member::PropertyAssignment {
                name,
                type_ref,
                value,
                ..
            } => {
                if I18N_PROPERTIES.contains(&name.as_str()) {
                    *type_ref = Some("std.bpcms.i18n".to_string());
                    resolve_i18n_value(value, app_locales, default_locale);
                } else {
                    resolve_nested(value, app_locales, default_locale);
                }
            }
            Member::Block { body, .. } | Member::EnumValueDeclaration { body, .. } => {
                resolve_members(body, app_locales, default_locale);
            }
            Member::Statement { statement, .. } => {
                resolve_statement(statement, app_locales, default_locale)
            }
            _ => {}
        }
    }
}

fn resolve_statement(statement: &mut Stmt, app_locales: &[String], default_locale: Option<&str>) {
    match statement {
        Stmt::IfStatement {
            then_branch,
            else_branch,
            ..
        } => {
            resolve_members(then_branch, app_locales, default_locale);
            if let Some(else_branch) = else_branch {
                resolve_members(else_branch, app_locales, default_locale);
            }
        }
        Stmt::TryStatement { body, catches, .. } => {
            resolve_members(body, app_locales, default_locale);
            for catch in catches {
                resolve_members(&mut catch.body, app_locales, default_locale);
            }
        }
        Stmt::ConstDeclaration {
            init: Some(init), ..
        }
        | Stmt::LetDeclaration {
            init: Some(init), ..
        } => resolve_nested(init, app_locales, default_locale),
        _ => {}
    }
}

/// An `I18N` value: a string (first service locale) or a locale map.
fn resolve_i18n_value(value: &mut Expr, app_locales: &[String], default_locale: Option<&str>) {
    if let Expr::StringLiteral { .. } = value {
        let Some(locale) = default_locale else {
            // Without a service/app locale there is no key to expand into; the
            // property still carries `typeRef: "std.bpcms.i18n"`.
            return;
        };
        let span = value.span();
        let literal = std::mem::replace(
            value,
            Expr::NullLiteral {
                span: Span::new(span.start, span.start),
            },
        );
        *value = Expr::Object {
            type_ref: None,
            type_ref_id: None,
            members: vec![Member::PropertyAssignment {
                name: locale.to_string(),
                type_ref: None,
                value: literal,
                span,
                id: None,
            }],
            span,
        };
        return;
    }

    let Expr::Object { members, .. } = value else {
        return;
    };

    members.retain(|member| match member {
        Member::PropertyAssignment { name, .. } => {
            name == "short" || name == "tooltip" || locale_supported(name, app_locales)
        }
        _ => false,
    });
    for member in members.iter_mut() {
        if let Member::PropertyAssignment {
            name,
            type_ref,
            value,
            ..
        } = member
        {
            if name == "short" || name == "tooltip" {
                *type_ref = Some("std.bpcms.i18n".to_string());
                resolve_i18n_value(value, app_locales, default_locale);
            }
        }
    }
}

/// Recurses into object literals so nested `I18N` properties are found.
fn resolve_nested(value: &mut Expr, app_locales: &[String], default_locale: Option<&str>) {
    if let Expr::Object { members, .. } = value {
        resolve_members(members, app_locales, default_locale);
    }
}

fn locale_supported(locale: &str, app_locales: &[String]) -> bool {
    if app_locales.is_empty() {
        return true;
    }
    let language = language_of(locale);
    app_locales
        .iter()
        .any(|app_locale| app_locale == locale || language_of(app_locale) == language)
}

fn language_of(locale: &str) -> &str {
    locale
        .split(['-', '_'])
        .next()
        .filter(|language| !language.is_empty())
        .unwrap_or(locale)
}

/// Locale names in a list, accepting quoted and bare hyphenated forms.
fn locale_list(value: &Expr) -> Option<Vec<String>> {
    let Expr::List { elements, .. } = value else {
        return None;
    };
    Some(
        elements
            .iter()
            .filter_map(locale_name)
            .filter(|locale| !locale.is_empty())
            .collect(),
    )
}

fn locale_name(value: &Expr) -> Option<String> {
    match value {
        Expr::StringLiteral { value, .. } => Some(value.clone()),
        Expr::Identifier { name, .. } => Some(name.clone()),
        Expr::Binary {
            op, left, right, ..
        } if op == "-" => Some(format!("{}-{}", locale_name(left)?, locale_name(right)?)),
        Expr::MemberAccess { object, name, .. } => {
            Some(format!("{}.{}", locale_name(object)?, name))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use archol_core::parse;

    fn artifact(source: &str) -> Artifact {
        parse(source, "test.archol").ast.items.remove(0)
    }

    #[test]
    fn reads_quoted_and_bare_locales() {
        let quoted = artifact("service \"a\" {\n  locales: [ \"pt-BR\", \"en\" ]\n}");
        assert_eq!(locales_of(&quoted), vec!["pt-BR", "en"]);

        let bare = artifact("application \"a\" {\n  locales: [pt-BR, en-US]\n}");
        assert_eq!(locales_of(&bare), vec!["pt-BR", "en-US"]);
    }

    #[test]
    fn language_match_keeps_fallbacks() {
        let locales = vec!["pt-BR".to_string(), "en-US".to_string()];
        assert!(locale_supported("pt-BR", &locales));
        assert!(locale_supported("pt", &locales));
        assert!(locale_supported("en", &locales));
        assert!(!locale_supported("fr", &locales));
    }
}
