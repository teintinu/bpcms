//! Semantic check and link for the Archol language.

pub mod check;
pub mod index;
pub mod link;
pub mod workspace;

pub use check::check;
pub use index::{Symbols, PRIMITIVES};
pub use link::{build as build_links, LinkIndex};
pub use workspace::{find_services_root, parse_workspace, FileInput, ParsedFile, Workspace};

use archol_core::Diagnostic;

#[derive(Clone)]
pub struct FileDiagnostic {
    pub path: String,
    pub diagnostic: Diagnostic,
}

pub struct Analysis {
    pub workspace: Workspace,
    pub symbols: Symbols,
    pub diagnostics: Vec<FileDiagnostic>,
    pub links: LinkIndex,
}

pub fn analyze(files: Vec<FileInput>, services_root: Option<&str>) -> Analysis {
    let workspace = parse_workspace(files, services_root);
    let symbols = Symbols::build(&workspace);
    let mut diagnostics: Vec<FileDiagnostic> = workspace
        .files
        .iter()
        .flat_map(|file| {
            file.diagnostics
                .iter()
                .cloned()
                .map(|diagnostic| FileDiagnostic {
                    path: file.path.clone(),
                    diagnostic,
                })
        })
        .collect();
    diagnostics.extend(
        check::check(&workspace, &symbols)
            .into_iter()
            .map(|(path, diagnostic)| FileDiagnostic { path, diagnostic }),
    );
    let links = link::build(&workspace, &symbols);
    Analysis {
        workspace,
        symbols,
        diagnostics,
        links,
    }
}
