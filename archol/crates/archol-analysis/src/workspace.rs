//! Carrega arquivos, mantém o AST e associa cada arquivo ao serviço dono.

use std::collections::BTreeMap;

use archol_core::{parse, Artifact, Diagnostic, LineIndex, SourceFile};

pub struct FileInput {
    pub path: String,
    pub source: String,
}

pub struct ParsedFile {
    pub path: String,
    pub source: String,
    pub ast: SourceFile,
    pub diagnostics: Vec<Diagnostic>,
    pub lines: LineIndex,
    pub service: Option<String>,
}

impl ParsedFile {
    pub fn line_text(&self, offset: usize) -> &str {
        let (line, _) = self.lines.line_col(offset);
        self.source.lines().nth(line as usize).unwrap_or("")
    }
}

#[derive(Default)]
pub struct Workspace {
    pub files: Vec<ParsedFile>,
    /// URI do serviço -> arquivos do serviço.
    pub services: BTreeMap<String, Vec<usize>>,
}

impl Workspace {
    pub fn artifacts(&self) -> impl Iterator<Item = (&ParsedFile, &Artifact)> {
        self.files
            .iter()
            .flat_map(|file| file.ast.items.iter().map(move |item| (file, item)))
    }

    pub fn diagnostics(&self) -> impl Iterator<Item = &Diagnostic> {
        self.files.iter().flat_map(|file| file.diagnostics.iter())
    }

    pub fn files_of_service(&self, uri: &str) -> impl Iterator<Item = &ParsedFile> {
        self.services
            .get(uri)
            .into_iter()
            .flatten()
            .map(move |index| &self.files[*index])
    }
}

pub fn parse_workspace(files: Vec<FileInput>, services_root: Option<&str>) -> Workspace {
    let mut parsed: Vec<ParsedFile> = files
        .into_iter()
        .map(|input| {
            let result = parse(&input.source, input.path.clone());
            ParsedFile {
                path: normalize(&input.path),
                source: input.source,
                ast: result.ast,
                diagnostics: result.diagnostics,
                lines: result.line_index,
                service: None,
            }
        })
        .collect();

    let base = services_root.map(|s| normalize(s).trim_end_matches('/').to_string());
    let roots = service_roots(&parsed, base.as_deref());

    for file in &mut parsed {
        file.service = roots
            .iter()
            .filter(|(root, _)| file.path.starts_with(&format!("{root}/")))
            .max_by_key(|(root, _)| root.len())
            .map(|(_, uri)| uri.clone());
    }

    let mut services: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for (index, file) in parsed.iter().enumerate() {
        if let Some(uri) = &file.service {
            services.entry(uri.clone()).or_default().push(index);
        }
    }

    Workspace {
        files: parsed,
        services,
    }
}

fn service_roots(files: &[ParsedFile], base: Option<&str>) -> Vec<(String, String)> {
    let mut roots = Vec::new();
    for file in files {
        let Some(name) = file.path.rsplit('/').next() else {
            continue;
        };
        let Some(slug) = name.strip_suffix("-service.archol") else {
            continue;
        };
        let Some(dir) = file.path.strip_suffix(&format!("/{name}")) else {
            continue;
        };
        let uri = if let Some(base) = base {
            dir.strip_prefix(&format!("{base}/"))
                .unwrap_or(dir)
                .to_string()
        } else {
            match dir.split_once("/services/") {
                Some((_, rest)) => rest.to_string(),
                None => slug.to_string(),
            }
        };
        roots.push((dir.to_string(), uri));
    }
    roots.sort_by_key(|(root, _)| std::cmp::Reverse(root.len()));
    roots
}

/// Descobre a raiz de serviços a partir da raiz do workspace.
pub fn find_services_root(root: &std::path::Path) -> Option<std::path::PathBuf> {
    [
        root.join("services"),
        root.join("examples").join("services"),
    ]
    .into_iter()
    .find(|candidate| candidate.is_dir())
}

pub fn normalize(path: &str) -> String {
    path.replace('\\', "/")
}
