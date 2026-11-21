//! Application discovery and the service/file closure used by the compiler.
//!
//! Archol compiles from an application: its entry-point services plus every
//! service reachable through `imports`. Files outside that closure are inert.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use archol_core::{Artifact, Member};

use crate::workspace::Workspace;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServiceRef {
    pub uri: String,
    pub alias: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ApplicationInfo {
    pub uri: String,
    pub path: String,
    pub services: Vec<ServiceRef>,
    /// Application locales; the first one is the default.
    pub locales: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct Closure {
    pub application: ApplicationInfo,
    pub services: BTreeSet<String>,
    /// Paths of the application file and of every file owned by the services.
    pub files: BTreeSet<String>,
}

impl Closure {
    pub fn contains_file(&self, path: &str) -> bool {
        self.files.contains(path)
    }
}

pub fn is_application(artifact: &Artifact) -> bool {
    artifact.artifact_kind() == "Application"
}

pub fn application_from_artifact(artifact: &Artifact, path: &str) -> Option<ApplicationInfo> {
    if !is_application(artifact) {
        return None;
    }
    let mut services = Vec::new();
    for member in &artifact.body {
        let Member::Block { name, body, .. } = member else {
            continue;
        };
        if name != "services" {
            continue;
        }
        for entry in body {
            if let Member::ServiceEntry { uri, alias, .. } = entry {
                services.push(ServiceRef {
                    uri: uri.clone(),
                    alias: alias.clone(),
                });
            }
        }
    }
    Some(ApplicationInfo {
        uri: artifact.name.clone(),
        path: path.to_string(),
        services,
        locales: crate::i18n::locales_of(artifact),
    })
}

/// All applications declared in the workspace, sorted by URI.
pub fn discover(workspace: &Workspace) -> Vec<ApplicationInfo> {
    let mut apps: Vec<ApplicationInfo> = workspace
        .files
        .iter()
        .flat_map(|file| {
            file.ast
                .items
                .iter()
                .filter_map(move |item| application_from_artifact(item, &file.path))
        })
        .collect();
    apps.sort_by(|a, b| a.uri.cmp(&b.uri));
    apps
}

pub fn find_by_uri<'a>(apps: &'a [ApplicationInfo], uri: &str) -> Option<&'a ApplicationInfo> {
    apps.iter().find(|app| app.uri == uri || app.path == uri)
}

/// Service URI -> imported service URIs, from every `service` artifact.
fn import_graph(workspace: &Workspace) -> BTreeMap<String, BTreeSet<String>> {
    let mut graph: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for (_, artifact) in workspace.artifacts() {
        if artifact.artifact_kind() != "Service" {
            continue;
        }
        let mut imports = BTreeSet::new();
        collect_imports(&artifact.body, &mut imports);
        graph
            .entry(artifact.name.clone())
            .or_default()
            .extend(imports);
    }
    graph
}

fn collect_imports(members: &[Member], imports: &mut BTreeSet<String>) {
    for member in members {
        match member {
            Member::ImportDeclaration { uri, .. } => {
                imports.insert(uri.clone());
            }
            Member::Block { body, .. } | Member::EnumValueDeclaration { body, .. } => {
                collect_imports(body, imports)
            }
            _ => {}
        }
    }
}

/// Computes the services reachable from the application and the files they own.
pub fn closure(workspace: &Workspace, application: &ApplicationInfo) -> Closure {
    let graph = import_graph(workspace);
    let mut services = BTreeSet::new();
    let mut queue: VecDeque<String> = application
        .services
        .iter()
        .map(|service| service.uri.clone())
        .collect();
    while let Some(service) = queue.pop_front() {
        if !services.insert(service.clone()) {
            continue;
        }
        if let Some(imports) = graph.get(&service) {
            for import in imports {
                if !services.contains(import) {
                    queue.push_back(import.clone());
                }
            }
        }
    }

    let mut files = BTreeSet::new();
    files.insert(application.path.clone());
    for file in &workspace.files {
        if file
            .service
            .as_ref()
            .map(|service| services.contains(service))
            .unwrap_or(false)
        {
            files.insert(file.path.clone());
        }
    }

    Closure {
        application: application.clone(),
        services,
        files,
    }
}

/// Finds an application by URI or by path, returning it with its closure.
pub fn resolve_application(
    workspace: &Workspace,
    uri_or_path: &str,
) -> Result<(ApplicationInfo, Closure), String> {
    let apps = discover(workspace);
    let app = apps
        .iter()
        .find(|app| app.uri == uri_or_path || app.path == uri_or_path)
        .or_else(|| {
            apps.iter()
                .find(|app| app.path.ends_with(uri_or_path) || app.uri.ends_with(uri_or_path))
        })
        .ok_or_else(|| format!("application `{uri_or_path}` not found in the workspace"))?;
    Ok((app.clone(), closure(workspace, app)))
}

#[cfg(test)]
mod tests {
    use crate::analyze;

    use super::*;

    fn workspace() -> Workspace {
        let files = vec![
            (
                "ws/applications/example.com/app/demo/demo-application.archol",
                "application \"example.com/app/demo\" {\n  services {\n    \"example.com/services/a\" as A\n  }\n}",
            ),
            (
                "ws/services/example.com/services/a/a-service.archol",
                "service \"example.com/services/a\" {\n  locales: [ \"pt-BR\" ]\n  imports { service \"example.com/services/b\" as B }\n}",
            ),
            (
                "ws/services/example.com/services/b/b-service.archol",
                "service \"example.com/services/b\" { title: { pt-BR: \"B\" } locales: [ \"pt-BR\" ] }",
            ),
            (
                "ws/services/example.com/services/c/c-service.archol",
                "service \"example.com/services/c\" { title: { pt-BR: \"C\" } locales: [ \"pt-BR\" ] }",
            ),
            (
                "ws/services/example.com/services/c/documents/c-doc-document.archol",
                "document CDoc { title: { pt-BR: \"C\" } }",
            ),
        ];
        let analysis = analyze(
            files
                .into_iter()
                .map(|(path, source)| crate::FileInput {
                    path: path.to_string(),
                    source: source.to_string(),
                })
                .collect(),
            Some("ws/services"),
        );
        analysis.workspace
    }

    #[test]
    fn discovers_application_and_services() {
        let workspace = workspace();
        let apps = discover(&workspace);
        assert_eq!(apps.len(), 1);
        assert_eq!(apps[0].uri, "example.com/app/demo");
        assert_eq!(apps[0].services.len(), 1);
    }

    #[test]
    fn closure_follows_transitive_imports_and_excludes_unrelated_files() {
        let workspace = workspace();
        let (_, closure) = resolve_application(&workspace, "example.com/app/demo").unwrap();
        assert!(closure.services.contains("example.com/services/a"));
        assert!(closure.services.contains("example.com/services/b"));
        assert!(!closure.services.contains("example.com/services/c"));
        assert!(closure
            .files
            .iter()
            .any(|path| path.ends_with("demo-application.archol")));
        assert!(closure
            .files
            .iter()
            .any(|path| path.ends_with("a-service.archol")));
        assert!(closure
            .files
            .iter()
            .any(|path| path.ends_with("b-service.archol")));
        assert!(!closure
            .files
            .iter()
            .any(|path| path.contains("services/c/")));
    }
}
