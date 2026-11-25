//! LSP server: document state, application analysis and handlers.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use archol_analysis::application::discover;
use archol_analysis::{
    analyze, analyze_application, parse_workspace, Analysis, ApplicationInfo, FileInput,
};
use archol_core::Severity;
use serde_json::{json, Value};

use crate::app;
use crate::features::{self, range};
use crate::rpc::Rpc;
use crate::util::{path_to_uri, same_path, uri_to_path};

mod handlers;
mod navigation;
mod semantic;
#[cfg(test)]
mod tests;

const NO_APPLICATION_MESSAGE: &str =
    "No application selected. Pick one in VS Code (Archol: Select Application).";

pub struct Server {
    rpc: Rpc,
    root: PathBuf,
    docs: BTreeMap<String, String>,
    cache: Option<Analysis>,
    /// Raw selector from `initializationOptions.application` or
    /// `archol/setApplication` (application URI or application file path).
    application: Option<String>,
    /// Applications discovered from the current workspace.
    applications: Option<Vec<ApplicationInfo>>,
    /// URIs that currently have diagnostics, so stale entries can be cleared.
    published: BTreeSet<String>,
    alive: bool,
}

impl Server {
    pub fn new() -> Self {
        let root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self {
            rpc: Rpc::stdio(),
            root,
            docs: BTreeMap::new(),
            cache: None,
            application: None,
            applications: None,
            published: BTreeSet::new(),
            alive: true,
        }
    }

    pub fn run(&mut self) -> i32 {
        while self.alive {
            let Some(message) = self.rpc.read() else {
                break;
            };
            let method = message.get("method").and_then(Value::as_str).unwrap_or("");
            let id = message.get("id").cloned();
            let params = message.get("params").cloned().unwrap_or(Value::Null);
            if let Some(id) = id {
                self.request(id, method, params);
            } else {
                self.notification(method, params);
            }
        }
        0
    }

    fn request(&mut self, id: Value, method: &str, params: Value) {
        match method {
            "initialize" => {
                if let Some(root) = initialize_root(&params) {
                    self.root = root;
                }
                self.application = params
                    .pointer("/initializationOptions/application")
                    .and_then(Value::as_str)
                    .map(app::selector_text)
                    .filter(|selector| !selector.is_empty());
                self.cache = None;
                self.applications = None;
                self.rpc.respond(
                    id,
                    json!({
                        "capabilities": capabilities(),
                        "serverInfo": { "name": "archol-lsp", "version": "0.1.0" }
                    }),
                );
            }
            "shutdown" => {
                self.rpc.respond(id, Value::Null);
                self.alive = false;
            }
            "textDocument/formatting" => {
                let result = self.formatting(&params);
                self.rpc.respond(id, result);
            }
            "textDocument/hover" => {
                let result = self.hover(&params);
                self.rpc.respond(id, result);
            }
            "textDocument/completion" => {
                let result = self.completion(&params);
                self.rpc.respond(id, result);
            }
            "textDocument/definition" => {
                let result = self.definition(&params);
                self.rpc.respond(id, result);
            }
            "textDocument/documentSymbol" => {
                let result = self.document_symbols(&params);
                self.rpc.respond(id, result);
            }
            "textDocument/references" => {
                let result = self.references(&params, false);
                self.rpc.respond(id, result);
            }
            "textDocument/documentHighlight" => {
                let result = self.references(&params, true);
                self.rpc.respond(id, result);
            }
            "workspace/symbol" => {
                let result = self.workspace_symbols(&params);
                self.rpc.respond(id, result);
            }
            "textDocument/foldingRange" => {
                let result = self.folding(&params);
                self.rpc.respond(id, result);
            }
            "archol/applicationAst" => {
                let result = self.application_ast(&params);
                self.rpc.respond(id, result);
            }
            "archol/inApplication" => {
                let uri = params.get("uri").and_then(Value::as_str).unwrap_or("");
                let inside = uri_to_path(uri)
                    .map(|path| normalize(&path))
                    .map(|path| app::in_application(self.analysis(), &path))
                    .unwrap_or(false);
                self.rpc.respond(id, json!({ "inApplication": inside }));
            }
            "archol/links" => {
                let result = self.links();
                self.rpc.respond(id, result);
            }
            "archol/applications" => {
                let result = self.applications_response();
                self.rpc.respond(id, result);
            }
            "archol/setApplication" => {
                let selector = params.get("uri").and_then(Value::as_str).unwrap_or("");
                match self.set_application(selector) {
                    Ok(result) => self.rpc.respond(id, result),
                    Err(error) => self.rpc.respond_error(id, -32602, &error),
                }
            }
            "archol/status" => {
                let result = self.status();
                self.rpc.respond(id, result);
            }
            "archol/declaration" => {
                let result = self.declaration(&params);
                self.rpc.respond(id, result);
            }
            "archol/usages" => {
                let result = self.usages(&params);
                self.rpc.respond(id, result);
            }
            "archol/lock" => match self.update_lock() {
                Ok(result) => self.rpc.respond(id, result),
                Err(error) => self.rpc.respond_error(id, -32602, &error),
            },
            "workspace/configuration" => self.rpc.respond(id, json!([])),
            _ => self.rpc.respond_error(id, -32601, "method not found"),
        }
    }

    fn notification(&mut self, method: &str, params: Value) {
        match method {
            "initialized" => {}
            "exit" => self.alive = false,
            "textDocument/didOpen" => {
                if let Some(uri) = params.pointer("/textDocument/uri").and_then(Value::as_str) {
                    if let Some(path) = uri_to_path(uri) {
                        let text = params
                            .pointer("/textDocument/text")
                            .and_then(Value::as_str)
                            .unwrap_or("");
                        self.docs.insert(normalize(&path), text.to_string());
                        self.invalidate();
                        self.publish_diagnostics();
                    }
                }
            }
            "textDocument/didChange" => {
                if let Some(uri) = params.pointer("/textDocument/uri").and_then(Value::as_str) {
                    if let Some(path) = uri_to_path(uri) {
                        if let Some(text) = params
                            .pointer("/contentChanges/0/text")
                            .and_then(Value::as_str)
                        {
                            self.docs.insert(normalize(&path), text.to_string());
                            self.invalidate();
                            self.publish_diagnostics();
                        }
                    }
                }
            }
            "textDocument/didSave" => {
                self.invalidate();
                self.publish_diagnostics();
            }
            "textDocument/didClose" => {
                if let Some(uri) = params.pointer("/textDocument/uri").and_then(Value::as_str) {
                    if let Some(path) = uri_to_path(uri) {
                        let path = normalize(&path);
                        self.docs.remove(&path);
                        self.published.remove(&path_to_uri(Path::new(&path)));
                        self.invalidate();
                        self.rpc.notify(
                            "textDocument/publishDiagnostics",
                            json!({ "uri": uri, "diagnostics": [] }),
                        );
                    }
                }
            }
            _ => {}
        }
    }

    /// Forgets every derived result; document contents affect both the
    /// workspace and the list of available applications.
    fn invalidate(&mut self) {
        self.cache = None;
        self.applications = None;
    }

    fn analysis(&mut self) -> &Analysis {
        if self.cache.is_none() {
            let root = self.services_root();
            let files = self.collect_files();
            let analysis = match self.application.as_deref() {
                Some(application) => {
                    match analyze_application(files, Some(&root), application, false) {
                        Ok(analysis) => analysis,
                        // Keep the server responsive even if the application
                        // disappeared from the edited workspace.
                        Err(_) => analyze(self.collect_files(), Some(&root)),
                    }
                }
                None => analyze(files, Some(&root)),
            };
            self.cache = Some(analysis);
        }
        self.cache.as_ref().expect("analysis cache")
    }

    fn discovered_applications(&mut self) -> Vec<ApplicationInfo> {
        if let Some(applications) = &self.applications {
            return applications.clone();
        }
        let found = if self.application.is_some() {
            self.analysis().applications.clone()
        } else {
            Vec::new()
        };
        let applications = if found.is_empty() {
            let root = self.services_root();
            let workspace = parse_workspace(self.collect_files(), Some(&root));
            discover(&workspace)
        } else {
            found
        };
        self.applications = Some(applications.clone());
        applications
    }

    fn current_application(&mut self) -> Option<String> {
        let selected = self.application.clone()?;
        if let Some(uri) = self
            .cache
            .as_ref()
            .and_then(|analysis| analysis.closure.as_ref())
            .map(|closure| closure.application.uri.clone())
        {
            return Some(uri);
        }
        let applications = self.discovered_applications();
        app::find_application(&applications, &selected).map(|application| application.uri.clone())
    }

    fn applications_response(&mut self) -> Value {
        let applications = self.discovered_applications();
        let current = self.current_application();
        let list: Vec<Value> = applications
            .iter()
            .map(|application| {
                json!({
                    "uri": application.uri,
                    "path": application.path,
                    "name": app::name_of(&application.uri),
                })
            })
            .collect();
        json!({ "applications": list, "current": current })
    }

    fn set_application(&mut self, selector: &str) -> Result<Value, String> {
        let selector = app::selector_text(selector);
        if selector.is_empty() {
            self.application = None;
            self.invalidate();
            self.publish_diagnostics();
            return Ok(json!({ "application": Value::Null, "files": 0, "services": 0 }));
        }
        let applications = self.discovered_applications();
        let application = app::find_application(&applications, &selector)
            .ok_or_else(|| format!("application `{selector}` not found in the workspace"))?
            .clone();
        let root = self.services_root();
        let files = self.collect_files();
        let analysis = analyze_application(files, Some(&root), &application.uri, true)?;
        let closure = analysis
            .closure
            .as_ref()
            .ok_or_else(|| format!("application `{}` has no closure", application.uri))?;
        let lock_path = archol_analysis::ids::lock_path_for_application(std::path::Path::new(
            &application.path,
        ));
        let identifiers = analysis
            .semantic
            .as_ref()
            .map(|semantic| {
                semantic.lock.entries.len()
                    + semantic.lock.file_entries.len()
                    + semantic.lock.pending_entries.len()
            })
            .unwrap_or(0);
        let result = json!({
            "application": closure.application.uri,
            "files": closure.files.len(),
            "services": closure.services.len(),
            "lock": lock_path.to_string_lossy(),
            "identifiers": identifiers,
        });
        self.application = Some(application.uri);
        self.applications = Some(applications);
        self.cache = Some(analysis);
        self.publish_diagnostics();
        Ok(result)
    }

    /// Creates/prunes the application lock file for the current application.
    fn update_lock(&mut self) -> Result<Value, String> {
        let Some(selector) = self.application.clone() else {
            return Err("no application selected".to_string());
        };
        let applications = self.discovered_applications();
        let application = app::find_application(&applications, &app::selector_text(&selector))
            .cloned()
            .ok_or_else(|| format!("application `{selector}` not found in the workspace"))?;
        let root = self.services_root();
        let files = self.collect_files();
        let analysis = analyze_application(files, Some(&root), &application.uri, true)?;
        let lock_path = archol_analysis::ids::lock_path_for_application(std::path::Path::new(
            &application.path,
        ));
        let identifiers = analysis
            .semantic
            .as_ref()
            .map(|semantic| {
                semantic.lock.entries.len()
                    + semantic.lock.file_entries.len()
                    + semantic.lock.pending_entries.len()
            })
            .unwrap_or(0);
        self.cache = Some(analysis);
        self.publish_diagnostics();
        Ok(json!({
            "application": application.uri,
            "path": lock_path.to_string_lossy(),
            "identifiers": identifiers,
        }))
    }

    fn status(&mut self) -> Value {
        if self.application.is_none() {
            return json!({ "application": Value::Null, "files": Value::Null, "services": Value::Null });
        }
        let analysis = self.analysis();
        match analysis.closure.as_ref() {
            Some(closure) => json!({
                "application": closure.application.uri,
                "files": closure.files.len(),
                "services": closure.services.len(),
            }),
            None => {
                json!({ "application": Value::Null, "files": Value::Null, "services": Value::Null })
            }
        }
    }

    fn declaration(&mut self, params: &Value) -> Value {
        let Some(id) = params.get("id").and_then(Value::as_str) else {
            return Value::Null;
        };
        if self.application.is_none() {
            return Value::Null;
        }
        let analysis = self.analysis();
        let Some(target) = semantic::declaration(analysis, id) else {
            return Value::Null;
        };
        let source = analysis
            .workspace
            .files
            .iter()
            .find(|file| same_path(&file.path, &target.path))
            .map(|file| file.source.as_str())
            .unwrap_or("");
        json!({
            "uri": path_to_uri(Path::new(&target.path)),
            "range": range(source, target.span),
        })
    }

    fn usages(&mut self, params: &Value) -> Value {
        let Some(id) = params.get("id").and_then(Value::as_str) else {
            return json!({ "count": 0, "locations": [] });
        };
        if self.application.is_none() {
            return json!({ "count": 0, "locations": [] });
        }
        let analysis = self.analysis();
        let locations: Vec<Value> = semantic::usages(analysis, id)
            .into_iter()
            .map(|usage| {
                let source = analysis
                    .workspace
                    .files
                    .iter()
                    .find(|file| same_path(&file.path, &usage.path))
                    .map(|file| file.source.as_str())
                    .unwrap_or("");
                json!({
                    "uri": path_to_uri(Path::new(&usage.path)),
                    "range": range(source, usage.span),
                    "kind": usage.kind,
                })
            })
            .collect();
        json!({ "count": locations.len(), "locations": locations })
    }

    fn links(&mut self) -> Value {
        if self.application.is_none() {
            return json!({ "links": Value::Null });
        }
        let analysis = self.analysis();
        let links = serde_json::to_value(&analysis.links).unwrap_or(Value::Null);
        json!({ "links": links })
    }

    fn formatting(&mut self, params: &Value) -> Value {
        let Some(uri) = params.pointer("/textDocument/uri").and_then(Value::as_str) else {
            return Value::Null;
        };
        let Some(path) = self.resolve_path(uri) else {
            return Value::Null;
        };
        let Some(text) = self.document_source(&path) else {
            return Value::Null;
        };
        let parsed = archol_core::parse(&text, path);
        let edits: Vec<Value> = archol_core::pending::format_edits(&text, &parsed.ast.pendings)
            .into_iter()
            .map(|(span, new_text)| json!({ "range": range(&text, span), "newText": new_text }))
            .collect();
        Value::Array(edits)
    }

    fn application_ast(&mut self, params: &Value) -> Value {
        if self.application.is_none() {
            return json!({ "ast": Value::Null, "error": NO_APPLICATION_MESSAGE });
        }
        let workspace_root = self.workspace_root();
        let analysis = self.analysis();
        if analysis.closure.is_none() || analysis.semantic.is_none() {
            return json!({
                "ast": Value::Null,
                "error": "The selected application has no resolved closure.",
            });
        }
        let application = analysis.closure.as_ref().map(|closure| {
            json!({
                "uri": closure.application.uri,
                "path": closure.application.path,
            })
        });
        match archol_analysis::application_ast::build(analysis, &workspace_root) {
            Ok(mut ast) => {
                if params
                    .get("noBody")
                    .and_then(Value::as_bool)
                    .unwrap_or(true)
                {
                    strip_bodies(&mut ast);
                }
                archol_core::order::order_value(&mut ast);
                json!({
                    "ast": ast,
                    "application": application,
                    "inApplication": true,
                })
            }
            Err(error) => json!({ "ast": Value::Null, "error": error }),
        }
    }

    fn workspace_symbols(&mut self, params: &Value) -> Value {
        if self.application.is_none() {
            return json!([]);
        }
        let query = params
            .get("query")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_lowercase();
        let analysis = self.analysis();
        let mut items: Vec<Value> = Vec::new();
        for file in &analysis.workspace.files {
            if !app::in_application(analysis, &file.path) {
                continue;
            }
            for item in &file.ast.items {
                if !item.name.to_lowercase().contains(&query) {
                    continue;
                }
                items.push(json!({
                    "name": item.name,
                    "kind": 5,
                    "location": {
                        "uri": path_to_uri(Path::new(&file.path)),
                        "range": range(&file.source, item.span)
                    }
                }));
            }
        }
        Value::Array(items)
    }

    fn document_symbols(&mut self, params: &Value) -> Value {
        let Some((path, text)) = self.document_text(params) else {
            return json!([]);
        };
        if !self.in_current_application(&path) {
            return json!([]);
        }
        let analysis = self.analysis();
        analysis
            .workspace
            .files
            .iter()
            .find(|file| same_path(&file.path, &path))
            .map(|file| Value::Array(features::document_symbols(&file.ast, &text)))
            .unwrap_or_else(|| json!([]))
    }

    fn services_root(&self) -> String {
        archol_analysis::find_services_root(&self.root)
            .unwrap_or_else(|| self.root.join("services"))
            .to_string_lossy()
            .to_string()
    }

    /// Root that contains `applications/` and `services/` (future `externalServices/`).
    fn workspace_root(&self) -> String {
        archol_analysis::find_services_root(&self.root)
            .and_then(|services| services.parent().map(Path::to_path_buf))
            .unwrap_or_else(|| self.root.clone())
            .to_string_lossy()
            .to_string()
    }

    fn collect_files(&self) -> Vec<FileInput> {
        let mut files = Vec::new();
        collect(&self.root, &mut files);
        let mut inputs: Vec<FileInput> = files
            .into_iter()
            .filter(|path| !self.docs.contains_key(&normalize(path)))
            .filter_map(|path| {
                std::fs::read_to_string(&path).ok().map(|source| FileInput {
                    path: path.to_string_lossy().to_string(),
                    source,
                })
            })
            .collect();
        for (path, source) in &self.docs {
            // A renamed/deleted file can stay in `docs` until the client sends
            // didClose. Do not resurrect it and create a duplicate declaration.
            if !std::path::Path::new(path).exists() {
                continue;
            }
            inputs.push(FileInput {
                path: path.clone(),
                source: source.clone(),
            });
        }
        inputs
    }

    fn document_source(&self, path: &str) -> Option<String> {
        self.docs
            .get(path)
            .cloned()
            .or_else(|| std::fs::read_to_string(path).ok())
    }

    /// Resolves a document URI to the path used by the workspace walk,
    /// preferring an open document and then a workspace file.
    fn resolve_path(&mut self, uri: &str) -> Option<String> {
        let path = normalize(&uri_to_path(uri)?);
        if self.docs.contains_key(&path) {
            return Some(path);
        }
        if let Some(file) = self.cache.as_ref().and_then(|analysis| {
            analysis
                .workspace
                .files
                .iter()
                .find(|file| same_path(&file.path, &path))
        }) {
            return Some(file.path.clone());
        }
        if let Some(key) = self.docs.keys().find(|key| same_path(key, &path)).cloned() {
            return Some(key);
        }
        if Path::new(&path).is_file() {
            return Some(path);
        }
        None
    }

    fn document_text(&mut self, params: &Value) -> Option<(String, String)> {
        let uri = params.pointer("/textDocument/uri")?.as_str()?;
        let path = self.resolve_path(uri)?;
        let text = self.document_source(&path)?;
        Some((path, text))
    }

    fn in_current_application(&mut self, path: &str) -> bool {
        if self.application.is_none() {
            return false;
        }
        app::in_application(self.analysis(), path)
    }

    fn publish_diagnostics(&mut self) {
        let open_paths: Vec<String> = self.docs.keys().cloned().collect();
        let selected = self.application.clone();
        let mut by_uri: BTreeMap<String, Vec<Value>> = BTreeMap::new();
        if selected.is_none() {
            for path in &open_paths {
                by_uri
                    .entry(path_to_uri(Path::new(path)))
                    .or_default()
                    .push(info_diagnostic(NO_APPLICATION_MESSAGE, "W_NO_APPLICATION"));
            }
        } else {
            let analysis = self.analysis();
            for entry in &analysis.diagnostics {
                if !app::in_application(analysis, &entry.path) {
                    continue;
                }
                let Some(file) = analysis
                    .workspace
                    .files
                    .iter()
                    .find(|file| same_path(&file.path, &entry.path))
                else {
                    continue;
                };
                let diagnostic = &entry.diagnostic;
                let severity = match diagnostic.severity {
                    Severity::Error => 1,
                    Severity::Warning => 2,
                    Severity::Info => 3,
                    Severity::Hint => 4,
                };
                by_uri
                    .entry(path_to_uri(Path::new(&file.path)))
                    .or_default()
                    .push(json!({
                        "range": range(&file.source, diagnostic.span),
                        "severity": severity,
                        "code": diagnostic.code,
                        "source": "archol",
                        "message": diagnostic.message,
                    }));
            }
            let application = analysis
                .closure
                .as_ref()
                .map(|closure| closure.application.uri.clone())
                .or(selected)
                .unwrap_or_default();
            for path in &open_paths {
                if !app::in_application(analysis, path) {
                    by_uri
                        .entry(path_to_uri(Path::new(path)))
                        .or_default()
                        .push(info_diagnostic(
                            &format!(
                                "This file is outside application {application} and is treated as commented out."
                            ),
                            "W_OUTSIDE_APPLICATION",
                        ));
                }
            }
        }

        let current: BTreeSet<String> = by_uri.keys().cloned().collect();
        for uri in self.published.difference(&current) {
            self.rpc.notify(
                "textDocument/publishDiagnostics",
                json!({ "uri": uri, "diagnostics": [] }),
            );
        }
        self.published = current;
        for (uri, diagnostics) in by_uri {
            self.rpc.notify(
                "textDocument/publishDiagnostics",
                json!({ "uri": uri, "diagnostics": diagnostics }),
            );
        }
    }
}

fn info_diagnostic(message: &str, code: &str) -> Value {
    json!({
        "range": {
            "start": { "line": 0, "character": 0 },
            "end": { "line": 0, "character": 0 },
        },
        "severity": 3,
        "source": "archol",
        "code": code,
        "message": message,
    })
}

fn strip_bodies(value: &mut Value) {
    match value {
        Value::Object(map) => {
            map.remove("body");
            for value in map.values_mut() {
                strip_bodies(value);
            }
        }
        Value::Array(items) => {
            for item in items {
                strip_bodies(item);
            }
        }
        _ => {}
    }
}

fn normalize(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn collect(dir: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name == "target" || name == "node_modules" || name == ".git" || name.starts_with('.')
            {
                continue;
            }
            collect(&path, files);
        } else if path.extension().map(|e| e == "archol").unwrap_or(false) {
            files.push(path);
        }
    }
}

fn initialize_root(params: &Value) -> Option<PathBuf> {
    if let Some(uri) = params.pointer("/rootUri").and_then(Value::as_str) {
        return uri_to_path(uri);
    }
    params
        .pointer("/rootPath")
        .and_then(Value::as_str)
        .map(PathBuf::from)
        .or_else(|| {
            params
                .pointer("/workspaceFolders/0/uri")
                .and_then(Value::as_str)
                .and_then(uri_to_path)
        })
}

fn capabilities() -> Value {
    json!({
        "textDocumentSync": { "openClose": true, "change": 1, "save": { "includeText": true } },
        "hoverProvider": true,
        "completionProvider": {
            "triggerCharacters": [":", "=", " ", "[", ".", "!"],
            "resolveProvider": false
        },
        "definitionProvider": true,
        "documentFormattingProvider": true,
        "documentSymbolProvider": true,
        "referencesProvider": true,
        "documentHighlightProvider": true,
        "foldingRangeProvider": true,
        "workspaceSymbolProvider": true
    })
}
