//! Resolução de definição: variável, campo de documento, valor de enum e ícone.
//!
//! É semântica o suficiente para responder `invoice.status` (campo do documento
//! `Invoice`) e `Overdue` (valor de `InvoiceStatus`, não o ícone homônimo).

use archol_analysis::Analysis;
use archol_core::{Artifact, Expr, Member, Span, Stmt};

use crate::features;

pub struct Target {
    pub path: String,
    pub span: Span,
}

pub fn definition(analysis: &Analysis, path: &str, offset: usize) -> Option<Target> {
    let file = analysis.workspace.files.iter().find(|f| f.path == path)?;
    let (word, _) = features::word_at(&file.source, offset)?;
    let symbols = &analysis.symbols;
    let service = file.service.as_deref();
    let artifact = features::artifact_at(&file.ast, offset)?;

    // 0. O próprio parâmetro ou `const`/`let` -> declaração da variável.
    if let Some(param) = artifact
        .params
        .iter()
        .find(|param| features::contains(param.span, offset))
    {
        return Some(Target {
            path: file.path.clone(),
            span: param.span,
        });
    }
    if let Some((name, _)) = features::const_at(&file.ast, offset) {
        if name == word {
            if let Some(span) = local_target(artifact, name) {
                return Some(Target {
                    path: file.path.clone(),
                    span,
                });
            }
        }
    }

    // 1. `icon: Overdue` -> artefato Icon/Overdue (não o valor de enum).
    if let Some((name, value)) = features::property_at(&file.ast, offset) {
        if name == "icon" && features::contains(value.span(), offset) {
            if let Some(target) = artifact_target(analysis, &word, service, Some("Icon")) {
                return Some(target);
            }
        }
    }

    // 2. `invoice.status == Overdue` -> valor Overdue em InvoiceStatus.
    if let Some(Expr::Binary { left, right, .. }) = features::comparison_at(&file.ast, offset) {
        if let Some(target) = enum_literal_target(
            analysis,
            artifact,
            left.as_ref(),
            right.as_ref(),
            offset,
            &word,
        ) {
            return Some(target);
        }
    }

    // 3. `const a: Regime = SIMPLES` -> valor SIMPLES declarado no enum.
    if let Some((name, Some(type_ref))) = features::const_at(&file.ast, offset) {
        if name != word {
            if let Some(info) = symbols.resolve_enum(type_ref, service) {
                if let Some(value) = info.values.iter().find(|value| value.name == word) {
                    return Some(Target {
                        path: info.path.clone(),
                        span: Span::new(value.span.0, value.span.1),
                    });
                }
            }
        }
    }

    // 4. `invoice.status` -> campo `status` do documento Invoice.
    if let Some(expr) = features::expr_at(&file.ast, offset) {
        if let Expr::MemberAccess { object, name, span } = expr {
            let name_start = span.end.saturating_sub(name.len());
            if offset >= name_start {
                if let Some(base) = expr_type(analysis, artifact, object.as_ref()) {
                    if let Some(target) = field_target(analysis, service, &base, name.as_str()) {
                        return Some(target);
                    }
                }
                return None;
            }
        }
        if let Expr::Identifier { name, .. } = expr {
            if let Some(target) = local_target(artifact, name.as_str()) {
                return Some(Target {
                    path: file.path.clone(),
                    span: target,
                });
            }
            if symbols.artifacts.contains_key(name.as_str()) {
                return artifact_target(analysis, name.as_str(), service, None);
            }
            // Uso direto de valor de enum (ex.: argumento nomeado).
            if let Some((info, value)) = symbols.find_value(name.as_str()) {
                return Some(Target {
                    path: info.path.clone(),
                    span: Span::new(value.span.0, value.span.1),
                });
            }
        }
    }

    // 5. Fallback: artefato (documento/shape/role/icon) ou enum.
    artifact_target(analysis, &word, service, None)
}

fn enum_literal_target(
    analysis: &Analysis,
    artifact: &Artifact,
    left: &Expr,
    right: &Expr,
    offset: usize,
    word: &str,
) -> Option<Target> {
    let symbols = &analysis.symbols;
    let service = analysis
        .workspace
        .files
        .iter()
        .find(|file| file.ast.items.iter().any(|item| item.span == artifact.span))
        .and_then(|file| file.service.clone());
    let service = service.as_deref();
    let pairs = [(left, right), (right, left)];
    for (variable_side, literal_side) in pairs {
        if !features::contains(literal_side.span(), offset) {
            continue;
        }
        if literal_side.as_ident() != Some(word) {
            continue;
        }
        let ty = expr_type(analysis, artifact, variable_side).or_else(|| {
            // `fanOut: Invoice where status == Overdue`: o sujeito implícito é
            // o documento Invoice e `status` é um campo dele.
            let name = variable_side.as_ident()?;
            let (property, value) = features::property_at(file_ast(analysis, artifact)?, offset)?;
            if property != "fanOut" {
                return None;
            }
            let Expr::Binary { left, .. } = value else {
                return None;
            };
            let subject = left.as_ident()?;
            field_type(analysis, subject, name)
        })?;
        let info = symbols.resolve_enum(&ty, service)?;
        let value = info.values.iter().find(|value| value.name == word)?;
        return Some(Target {
            path: info.path.clone(),
            span: Span::new(value.span.0, value.span.1),
        });
    }
    None
}

fn file_ast<'a>(
    analysis: &'a Analysis,
    artifact: &'a Artifact,
) -> Option<&'a archol_core::SourceFile> {
    analysis
        .workspace
        .files
        .iter()
        .find(|file| file.ast.items.iter().any(|item| item.span == artifact.span))
        .map(|file| &file.ast)
}

fn expr_type(analysis: &Analysis, artifact: &Artifact, expr: &Expr) -> Option<String> {
    match expr {
        Expr::Identifier { name, .. } => {
            if let Some(ty) = local_type(artifact, name) {
                return Some(ty);
            }
            analysis
                .symbols
                .artifacts
                .contains_key(name)
                .then(|| name.clone())
        }
        Expr::MemberAccess { object, name, .. } => {
            let base = expr_type(analysis, artifact, object)?;
            field_type(analysis, &base, name)
        }
        _ => None,
    }
}

fn local_type(artifact: &Artifact, name: &str) -> Option<String> {
    if let Some(ty) = features::param_type(artifact, name) {
        return Some(ty.to_string());
    }
    let mut found = None;
    features::walk(&artifact.body, &mut |member| {
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

fn local_target(artifact: &Artifact, name: &str) -> Option<Span> {
    if let Some(param) = artifact.params.iter().find(|param| param.name == name) {
        return Some(param.span);
    }
    let mut found = None;
    features::walk(&artifact.body, &mut |member| {
        if let Member::Statement {
            statement:
                Stmt::ConstDeclaration {
                    name: decl_name,
                    span,
                    ..
                }
                | Stmt::LetDeclaration {
                    name: decl_name,
                    span,
                    ..
                },
            ..
        } = member
        {
            if decl_name == name {
                found = Some(*span);
            }
        }
    });
    found
}

fn find_artifact_file<'a>(
    analysis: &'a Analysis,
    type_name: &str,
    service: Option<&str>,
) -> Option<&'a archol_analysis::ParsedFile> {
    let (name, target_service) = split_qualified(&analysis.symbols, type_name, service);
    let candidates = analysis.symbols.artifacts.get(&name)?;
    let chosen = candidates
        .iter()
        .find(|info| info.service.as_deref() == target_service.as_deref())
        .or_else(|| {
            candidates
                .iter()
                .find(|info| info.service.as_deref() == service)
        })
        .or_else(|| candidates.first())?;
    analysis
        .workspace
        .files
        .iter()
        .find(|file| file.path == chosen.path)
}

fn split_qualified(
    symbols: &archol_analysis::Symbols,
    type_name: &str,
    service: Option<&str>,
) -> (String, Option<String>) {
    let base = type_name.trim_end_matches("[]").trim();
    if let Some((alias, local)) = base.split_once('.') {
        if let Some(target) = service
            .and_then(|service| symbols.imports.get(service))
            .and_then(|imports| imports.get(alias))
        {
            return (local.to_string(), Some(target.clone()));
        }
        return (local.to_string(), None);
    }
    (base.to_string(), None)
}

fn field_type(analysis: &Analysis, type_name: &str, field: &str) -> Option<String> {
    let file = find_artifact_file(analysis, type_name, None)?;
    let artifact = file
        .ast
        .items
        .iter()
        .find(|item| item.name == strip_qualifier(type_name))?;
    let fields = artifact.body.iter().find_map(|member| match member {
        Member::Block { name, body, .. } if name == "fields" => Some(body),
        _ => None,
    })?;
    let value = fields.iter().find_map(|member| match member {
        Member::PropertyAssignment { name, value, .. } if name == field => Some(value),
        _ => None,
    })?;
    match value {
        Expr::Object {
            type_ref: Some(ty), ..
        } => Some(ty.clone()),
        Expr::Identifier { name, .. } => Some(name.clone()),
        _ => None,
    }
}

fn strip_qualifier(name: &str) -> &str {
    name.rsplit('.').next().unwrap_or(name)
}

fn field_target(
    analysis: &Analysis,
    service: Option<&str>,
    type_name: &str,
    field: &str,
) -> Option<Target> {
    let file = find_artifact_file(analysis, type_name, service)?;
    let artifact = file
        .ast
        .items
        .iter()
        .find(|item| item.name == strip_qualifier(type_name))?;
    let fields = artifact.body.iter().find_map(|member| match member {
        Member::Block { name, body, .. } if name == "fields" => Some(body),
        _ => None,
    })?;
    let target = fields.iter().find_map(|member| match member {
        Member::PropertyAssignment { name, span, .. } if name == field => Some(*span),
        _ => None,
    })?;
    Some(Target {
        path: file.path.clone(),
        span: target,
    })
}

fn artifact_target(
    analysis: &Analysis,
    word: &str,
    service: Option<&str>,
    kind: Option<&str>,
) -> Option<Target> {
    let candidates = analysis.symbols.artifacts.get(word)?;
    let allowed: Vec<&_> = candidates
        .iter()
        .filter(|info| kind.map(|kind| info.kind == kind).unwrap_or(true))
        .collect();
    let chosen = allowed
        .iter()
        .copied()
        .find(|info| info.service.as_deref() == service)
        .or_else(|| allowed.first().copied())?;
    Some(Target {
        path: chosen.path.clone(),
        span: Span::new(chosen.span_start, chosen.span_end),
    })
}

#[cfg(test)]
mod tests {
    use archol_analysis::{analyze, FileInput};

    use super::definition;

    const ROOT: &str = "ws/services/example.com/services/billing";

    fn workspace() -> archol_analysis::Analysis {
        let files = vec![
            (
                format!("{ROOT}/billing-service.archol"),
                "service \"example.com/services/billing\" { title: { pt-BR: \"Billing\" } }".to_string(),
            ),
            (
                format!("{ROOT}/billing-shapes.archol"),
                "shape InvoiceStatus {\n  base: classification\n  value Previous { title: { pt-BR: \"P\" } }\n  value Overdue { title: { pt-BR: \"O\" } icon: Overdue }\n  value Paid { title: { pt-BR: \"Paid\" } }\n}".to_string(),
            ),
            (
                format!("{ROOT}/billing-icons.archol"),
                "icon Overdue {\n  title: { pt-BR: \"Vencida\" }\n}".to_string(),
            ),
            (
                format!("{ROOT}/documents/billing-invoice-document.archol"),
                "document Invoice {\n  title: { pt-BR: \"Nota\" }\n  fields {\n    number: string { title: { pt-BR: \"Número\" } }\n    status: InvoiceStatus { title: { pt-BR: \"Situação\" } }\n  }\n}".to_string(),
            ),
            (
                format!("{ROOT}/processes/billing-dunning-process.archol"),
                "process Dunning(invoice: Invoice) {\n  if (invoice.status == Overdue) {\n    return Done\n  }\n}".to_string(),
            ),
        ];
        analyze(
            files
                .into_iter()
                .map(|(path, source)| FileInput { path, source })
                .collect(),
            Some("ws/services"),
        )
    }

    fn locate(
        analysis: &archol_analysis::Analysis,
        file_suffix: &str,
        needle: &str,
        within: &str,
    ) -> (String, usize) {
        let file = analysis
            .workspace
            .files
            .iter()
            .find(|file| file.path.ends_with(file_suffix))
            .expect("arquivo");
        let base = file.source.find(needle).expect("trecho");
        let delta = needle.find(within).expect("trecho interno");
        (file.path.clone(), base + delta)
    }

    #[test]
    fn resolve_campo_status_do_documento() {
        let analysis = workspace();
        let (path, offset) = locate(
            &analysis,
            "billing-dunning-process.archol",
            "invoice.status",
            "status",
        );
        let target = definition(&analysis, &path, offset).expect("definição");
        assert!(target.path.ends_with("billing-invoice-document.archol"));
        let file = analysis
            .workspace
            .files
            .iter()
            .find(|file| file.path == target.path)
            .unwrap();
        let text = &file.source[target.span.start..target.span.end];
        assert!(text.starts_with("status: InvoiceStatus"), "{text}");
    }

    #[test]
    fn resolve_valor_overdue_nao_icone() {
        let analysis = workspace();
        let (path, offset) = locate(
            &analysis,
            "billing-dunning-process.archol",
            "Overdue)",
            "Overdue",
        );
        let target = definition(&analysis, &path, offset).expect("definição");
        assert!(target.path.ends_with("billing-shapes.archol"));
        let file = analysis
            .workspace
            .files
            .iter()
            .find(|file| file.path == target.path)
            .unwrap();
        assert!(file.source[target.span.start..target.span.end].starts_with("value Overdue"));
    }

    #[test]
    fn resolve_icone_no_uso_e_nao_valor_homonimo() {
        let analysis = workspace();
        let (path, offset) = locate(
            &analysis,
            "billing-shapes.archol",
            "icon: Overdue",
            "Overdue",
        );
        let target = definition(&analysis, &path, offset).expect("definição");
        assert!(
            target.path.ends_with("billing-icons.archol"),
            "{}",
            target.path
        );
    }

    #[test]
    fn resolve_icone_quando_contexto_e_icon() {
        let analysis = workspace();
        let (path, offset) = locate(&analysis, "billing-icons.archol", "icon Overdue", "Overdue");
        let target = definition(&analysis, &path, offset).expect("definição");
        assert!(target.path.ends_with("billing-icons.archol"));
    }

    #[test]
    fn resolve_parametro_do_processo() {
        let analysis = workspace();
        let (path, offset) = locate(
            &analysis,
            "billing-dunning-process.archol",
            "invoice: Invoice",
            "invoice",
        );
        let target = definition(&analysis, &path, offset).expect("definição");
        assert!(target.path.ends_with("billing-dunning-process.archol"));
        let file = analysis
            .workspace
            .files
            .iter()
            .find(|file| file.path == target.path)
            .unwrap();
        assert!(file.source[target.span.start..target.span.end].contains("invoice"));
    }
}
