//! Identifier navigation over the semantic index.

use archol_analysis::Analysis;
use archol_core::Span;

#[derive(Clone, Debug)]
pub struct Declaration {
    pub path: String,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct UsageLocation {
    pub path: String,
    pub span: Span,
    pub kind: String,
}

/// Declaration target for a lock id, or `None` when the id is unknown/outside
/// the application closure.
pub fn declaration(analysis: &Analysis, id: &str) -> Option<Declaration> {
    let declaration = analysis.semantic.as_ref()?.declarations.get(id)?;
    Some(Declaration {
        path: declaration.file.clone(),
        span: declaration.span,
    })
}

/// Usages of a lock id, sorted by file and byte offset.
pub fn usages(analysis: &Analysis, id: &str) -> Vec<UsageLocation> {
    let Some(semantic) = analysis.semantic.as_ref() else {
        return Vec::new();
    };
    let mut locations: Vec<UsageLocation> = semantic
        .usages
        .iter()
        .filter(|usage| usage.id == id)
        .map(|usage| UsageLocation {
            path: usage.file.clone(),
            span: usage.span,
            kind: usage.kind.clone(),
        })
        .collect();
    locations.sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then(left.span.start.cmp(&right.span.start))
            .then(left.span.end.cmp(&right.span.end))
    });
    locations
}
