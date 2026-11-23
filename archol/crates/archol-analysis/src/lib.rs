//! Semantic check and link for the Archol language.

pub mod annotate;
pub mod application;
pub mod check;
pub mod i18n;
pub mod ids;
pub mod index;
pub mod link;
pub mod organogram;
pub mod service;
pub mod shape_literal;
pub mod workspace;

pub use check::check;
pub use index::{Symbols, PRIMITIVES};
pub use link::{build as build_links, LinkIndex};
pub use workspace::{find_services_root, parse_workspace, FileInput, ParsedFile, Workspace};

use archol_core::Diagnostic;

pub use annotate::{DeclInfo, SemanticIndex, Usage};
pub use application::{ApplicationInfo, Closure, ServiceRef};
pub use ids::{Lock, LockEntry};

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
    /// Applications discovered in the workspace.
    pub applications: Vec<ApplicationInfo>,
    /// Present when the analysis started from an application.
    pub closure: Option<Closure>,
    /// Stable ids and typed references (present with an application).
    pub semantic: Option<SemanticIndex>,
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
        applications: Vec::new(),
        closure: None,
        semantic: None,
    }
}

/// Compiles from an application: closure, diagnostics, links and ids.
///
/// `write_lock` persists the application lock file next to the application file (creating or
/// pruning ids). Read-only analyses keep the lock untouched.
pub fn analyze_application(
    files: Vec<FileInput>,
    services_root: Option<&str>,
    application: &str,
    write_lock: bool,
) -> Result<Analysis, String> {
    let mut workspace = parse_workspace(files, services_root);
    let applications = application::discover(&workspace);
    let (app, closure) = application::resolve_application(&workspace, application)?;
    i18n::resolve(&mut workspace, &closure, &app.locales);
    let symbols = Symbols::build(&workspace);
    let shape_diagnostics = shape_literal::resolve(&mut workspace, &symbols, &closure);

    let mut diagnostics: Vec<FileDiagnostic> = workspace
        .files
        .iter()
        .filter(|file| closure.contains_file(&file.path))
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
        shape_diagnostics
            .into_iter()
            .filter(|(path, _)| closure.contains_file(path))
            .map(|(path, diagnostic)| FileDiagnostic { path, diagnostic }),
    );
    diagnostics.extend(
        check::check(&workspace, &symbols)
            .into_iter()
            .filter(|(path, _)| closure.contains_file(path))
            .map(|(path, diagnostic)| FileDiagnostic { path, diagnostic }),
    );

    let mut links = link::build(&workspace, &symbols);
    link::retain_closure(&mut links, &closure);

    let lock_path = ids::lock_path_for_application(std::path::Path::new(&app.path));
    let lock = ids::Lock::load(&lock_path, &app.uri)?;
    let semantic = annotate::annotate(&mut workspace, &closure, &symbols, lock);
    organogram::attach(&mut workspace, &semantic, &closure);
    service::attach(&mut workspace, &semantic, &closure);
    if write_lock && semantic.lock.dirty {
        semantic.lock.save(&lock_path)?;
    }

    Ok(Analysis {
        workspace,
        symbols,
        diagnostics,
        links,
        applications,
        closure: Some(closure),
        semantic: Some(semantic),
    })
}
