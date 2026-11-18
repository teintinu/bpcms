//! LSP server: document state, analysis and handlers.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use archol_analysis::{analyze, Analysis, FileInput};
use archol_core::Severity;
use serde_json::{json, Value};

use crate::features::{self, range};

mod handlers;
mod navigation;
use crate::rpc::Rpc;
use crate::util::{path_to_uri, uri_to_path};

pub struct Server {
    rpc: Rpc,
    root: PathBuf,
    docs: BTreeMap<String, String>,
    cache: Option<Analysis>,
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
                let document = self.document_text(&params);
                let result = document
                    .and_then(|(path, text)| {
                        self.analysis()
                            .workspace
                            .files
                            .iter()
                            .find(|file| file.path == path)
                            .map(|file| {
                                serde_json::Value::Array(features::document_symbols(
                                    &file.ast, &text,
                                ))
                            })
                    })
                    .unwrap_or_else(|| json!([]));
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
                let query = params
                    .get("query")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_lowercase();
                let mut items: Vec<Value> = Vec::new();
                for file in &self.analysis().workspace.files {
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
                self.rpc.respond(id, Value::Array(items));
            }
            "textDocument/foldingRange" => {
                let result = self.folding(&params);
                self.rpc.respond(id, result);
            }
            "archol/ast" => {
                let uri = params.get("uri").and_then(Value::as_str).unwrap_or("");
                let path = uri_to_path(uri)
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_default();
                let ast = self
                    .analysis()
                    .workspace
                    .files
                    .iter()
                    .find(|file| file.path == path || file.path.ends_with(&path))
                    .map(|file| serde_json::to_value(&file.ast).unwrap_or(Value::Null));
                self.rpc
                    .respond(id, json!({ "ast": ast.unwrap_or(Value::Null) }));
            }
            "archol/links" => {
                let links = serde_json::to_value(&self.analysis().links).unwrap_or(Value::Null);
                self.rpc.respond(id, json!({ "links": links }));
            }
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
                        self.cache = None;
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
                            self.cache = None;
                            self.publish_diagnostics();
                        }
                    }
                }
            }
            "textDocument/didSave" => {
                self.cache = None;
                self.publish_diagnostics();
            }
            "textDocument/didClose" => {
                if let Some(uri) = params.pointer("/textDocument/uri").and_then(Value::as_str) {
                    if let Some(path) = uri_to_path(uri) {
                        self.docs.remove(&normalize(&path));
                        self.cache = None;
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

    fn analysis(&mut self) -> &Analysis {
        if self.cache.is_none() {
            self.cache = Some(analyze(self.collect_files(), Some(&self.services_root())));
        }
        self.cache.as_ref().unwrap()
    }

    fn services_root(&self) -> String {
        archol_analysis::find_services_root(&self.root)
            .unwrap_or_else(|| self.root.join("services"))
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
            inputs.push(FileInput {
                path: path.clone(),
                source: source.clone(),
            });
        }
        inputs
    }

    fn document_text(&mut self, params: &Value) -> Option<(String, String)> {
        let uri = params.pointer("/textDocument/uri")?.as_str()?;
        let path = normalize(&uri_to_path(uri)?);
        let display = self
            .analysis()
            .workspace
            .files
            .iter()
            .find(|file| file.path == path)
            .map(|file| file.path.clone())
            .unwrap_or(path);
        let text = self
            .docs
            .get(&display)
            .cloned()
            .or_else(|| std::fs::read_to_string(&display).ok())?;
        Some((display, text))
    }

    fn publish_diagnostics(&mut self) {
        let analysis = self.analysis();
        let mut by_uri: BTreeMap<String, Vec<Value>> = BTreeMap::new();
        for entry in &analysis.diagnostics {
            let Some(file) = analysis
                .workspace
                .files
                .iter()
                .find(|file| file.path == entry.path)
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
        for (uri, diagnostics) in by_uri {
            self.rpc.notify(
                "textDocument/publishDiagnostics",
                json!({ "uri": uri, "diagnostics": diagnostics }),
            );
        }
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
        "documentSymbolProvider": true,
        "referencesProvider": true,
        "documentHighlightProvider": true,
        "foldingRangeProvider": true,
        "workspaceSymbolProvider": true
    })
}
