//! Archol CLI: applications, check, lock, ast, view, links, tokens and lsp.

mod view;
mod yaml;

use std::path::{Path, PathBuf};
use std::process::exit;

use archol_analysis::application;
use archol_analysis::{analyze_application, find_services_root, parse_workspace, FileInput};
use archol_core::{lex, parse, Severity};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(command) = args.first() else {
        usage();
        exit(2);
    };
    let rest = &args[1..];
    let code = match command.as_str() {
        "check" => check(rest),
        "lint" => check(rest),
        "format" => format(rest),
        "lock" => lock(rest),
        "applications" => applications(rest),
        "application" | "app" | "ast" => application(rest),
        "view" => view(rest),
        "tokens" => tokens(rest),
        "links" => links(rest),
        "lsp" => archol_lsp::run_stdio(),
        "help" | "--help" | "-h" => {
            usage();
            0
        }
        other => {
            eprintln!("unknown command: {other}");
            usage();
            2
        }
    };
    exit(code);
}

fn usage() {
    eprintln!(
        "archol <command> [args]\n\n\
         \x20 check [dir] --app <uri-or-path> [--json]   compiles the application closure\n\
         \x20 lint [dir] --app <uri-or-path> [--json]    same diagnostics as check\n\
         \x20 format [dir] [--check]                     uppercases pending comment keywords\n\
         \x20 lock [dir] --app <uri-or-path>              creates or prunes <app>.lock\n\
         \x20 applications [dir] [--json]                 lists discovered applications\n\
         \x20 application [dir] --app <uri-or-path> [--json|--yaml] [--no-loc|--no-body]  resolved application AST\n\
         \x20 ast [dir] --app <uri-or-path> [--json|--yaml] [--no-loc|--no-body]  alias for application\n\
         \x20 view <file> [--app <uri-or-path>] [--json] [--pretty]  compact preview payload for editors\n\
         \x20 tokens <file>                               prints the tokens\n\
         \x20 links [dir] --app <uri-or-path> [--json]    inverted index of the closure\n\
         \x20 lsp                                         LSP server on stdio"
    );
}

struct Options {
    positional: Vec<String>,
    app: Option<String>,
    json: bool,
    yaml: bool,
    no_loc: bool,
    no_body: bool,
    pretty: bool,
    check: bool,
}

impl Options {
    fn parse(args: &[String]) -> Self {
        let mut positional = Vec::new();
        let mut app = None;
        let mut json = false;
        let mut yaml = false;
        let mut no_loc = false;
        let mut no_body = false;
        let mut pretty = false;
        let mut check = false;
        let mut iter = args.iter();
        while let Some(arg) = iter.next() {
            match arg.as_str() {
                "--app" => app = iter.next().cloned(),
                "--json" => json = true,
                "--yaml" => yaml = true,
                "--no-loc" => no_loc = true,
                "--no-body" => no_body = true,
                "--pretty" => pretty = true,
                "--check" => check = true,
                _ => positional.push(arg.clone()),
            }
        }
        Options {
            positional,
            app,
            json,
            yaml,
            no_loc,
            no_body,
            pretty,
            check,
        }
    }

    fn root(&self) -> PathBuf {
        self.positional
            .first()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."))
    }
}

fn check(args: &[String]) -> i32 {
    let options = Options::parse(args);
    let root = options.root();
    let (files, read_errors) = load_files(&root);
    let services_root = find_services_root(&root);
    let services = services_root
        .as_ref()
        .map(|path| path.to_string_lossy().to_string());
    let mut errors = print_read_errors(&read_errors, options.json);
    let Some(selector) = options.app.as_deref() else {
        return missing_app(files, services.as_deref());
    };
    let analysis = match analyze_application(files, services.as_deref(), selector, false) {
        Ok(analysis) => analysis,
        Err(error) => {
            eprintln!("error: {error}");
            return 1;
        }
    };
    let mut warnings = 0;
    for entry in &analysis.diagnostics {
        let diagnostic = &entry.diagnostic;
        let Some(file) = analysis
            .workspace
            .files
            .iter()
            .find(|file| file.path == entry.path)
        else {
            continue;
        };
        let (line, col) = file.lines.line_col(diagnostic.span.start);
        let severity = match diagnostic.severity {
            archol_core::Severity::Error => "error",
            archol_core::Severity::Warning => "warning",
            archol_core::Severity::Info => "info",
            archol_core::Severity::Hint => "hint",
        };
        if matches!(diagnostic.severity, archol_core::Severity::Error) {
            errors += 1;
        } else if matches!(diagnostic.severity, archol_core::Severity::Warning) {
            warnings += 1;
        }
        if options.json {
            println!(
                "{}",
                serde_json::json!({
                    "file": file.path,
                    "line": line + 1,
                    "column": col + 1,
                    "severity": severity,
                    "code": diagnostic.code,
                    "message": diagnostic.message,
                })
            );
        } else {
            println!(
                "{}:{}:{}: {} {}: {}",
                file.path,
                line + 1,
                col + 1,
                severity,
                diagnostic.code,
                diagnostic.message
            );
        }
    }
    if !options.json {
        let file_count = analysis
            .closure
            .as_ref()
            .map(|closure| closure.files.len())
            .unwrap_or(analysis.workspace.files.len());
        println!(
            "{} file(s), {} error(s), {} warning(s)",
            file_count, errors, warnings
        );
    }
    if errors > 0 {
        1
    } else {
        0
    }
}

fn format(args: &[String]) -> i32 {
    let options = Options::parse(args);
    let root = options.root();
    let (files, read_errors) = load_files(&root);
    for (path, message) in &read_errors {
        eprintln!("{path}: {message}");
    }
    let mut changed = 0;
    for file in &files {
        let parsed = parse(&file.source, file.path.clone());
        let formatted = archol_core::pending::format_source(&file.source, &parsed.ast.pendings);
        if formatted == file.source {
            continue;
        }
        changed += 1;
        if options.check {
            println!("{}", file.path);
        } else if let Err(error) = std::fs::write(&file.path, formatted) {
            eprintln!("{}: {error}", file.path);
            return 2;
        } else {
            println!("{}", file.path);
        }
    }
    if options.check {
        i32::from(changed > 0 || !read_errors.is_empty())
    } else {
        i32::from(!read_errors.is_empty())
    }
}

fn lock(args: &[String]) -> i32 {
    let options = Options::parse(args);
    let root = options.root();
    let (files, read_errors) = load_files(&root);
    for (path, message) in &read_errors {
        eprintln!("{path}: {message}");
    }
    let services_root = find_services_root(&root);
    let services = services_root
        .as_ref()
        .map(|path| path.to_string_lossy().to_string());
    let Some(selector) = options.app.as_deref() else {
        return missing_app(files, services.as_deref());
    };
    let analysis = match analyze_application(files, services.as_deref(), selector, true) {
        Ok(analysis) => analysis,
        Err(error) => {
            eprintln!("error: {error}");
            return 1;
        }
    };
    let Some(closure) = analysis.closure.as_ref() else {
        eprintln!("error: analysis did not return a closure");
        return 1;
    };
    let lock_path =
        archol_analysis::ids::lock_path_for_application(Path::new(&closure.application.path));
    let identifiers = analysis
        .semantic
        .as_ref()
        .map(|semantic| {
            semantic.lock.entries.len()
                + semantic.lock.file_entries.len()
                + semantic.lock.pending_entries.len()
        })
        .unwrap_or(0);
    println!("lock: {}", lock_path.display());
    println!("identifiers: {identifiers}");
    println!("application: {}", closure.application.uri);
    0
}

fn applications(args: &[String]) -> i32 {
    let options = Options::parse(args);
    let root = options.root();
    let (files, read_errors) = load_files(&root);
    for (path, message) in &read_errors {
        eprintln!("{path}: {message}");
    }
    let services_root = find_services_root(&root);
    let services = services_root
        .as_ref()
        .map(|path| path.to_string_lossy().to_string());
    let workspace = parse_workspace(files, services.as_deref());
    let apps = application::discover(&workspace);
    if options.json {
        let list: Vec<_> = apps
            .iter()
            .map(|app| serde_json::json!({ "uri": app.uri, "path": app.path }))
            .collect();
        println!(
            "{}",
            serde_json::to_string_pretty(&list).unwrap_or_else(|_| "[]".to_string())
        );
    } else {
        for app in &apps {
            println!("{}\t{}", app.uri, app.path);
        }
    }
    i32::from(!read_errors.is_empty())
}

fn application(args: &[String]) -> i32 {
    let options = Options::parse(args);
    let root = options.root();
    let root = if root.is_file() {
        workspace_roots(&root).into_iter().next().unwrap_or(root)
    } else {
        root
    };
    let (files, read_errors) = load_files(&root);
    for (path, message) in &read_errors {
        eprintln!("{path}: {message}");
    }
    let services_root = find_services_root(&root);
    let services = services_root
        .as_ref()
        .map(|path| path.to_string_lossy().to_string());
    let Some(selector) = options.app.as_deref() else {
        return missing_app(files, services.as_deref());
    };
    let analysis = match analyze_application(files, services.as_deref(), selector, false) {
        Ok(analysis) => analysis,
        Err(error) => {
            eprintln!("error: {error}");
            return 1;
        }
    };
    if analysis
        .diagnostics
        .iter()
        .any(|entry| matches!(entry.diagnostic.severity, Severity::Error))
    {
        eprintln!("error: application has diagnostics; run `archol check` first");
        return 1;
    }
    match archol_analysis::application_ast::build(&analysis, &root.to_string_lossy()) {
        Ok(mut value) => {
            if options.no_body {
                strip_bodies(&mut value);
            }
            print_ast(&value, options.yaml, options.no_loc);
            0
        }
        Err(error) => {
            eprintln!("error: {error}");
            1
        }
    }
}

fn print_ast<T: serde::Serialize>(value: &T, yaml: bool, no_loc: bool) {
    let mut json = serde_json::to_value(value).unwrap();
    archol_core::order::order_value(&mut json);
    if no_loc {
        strip_locs(&mut json);
    }
    if yaml {
        print!("{}", yaml::to_yaml(&json));
    } else {
        println!("{}", serde_json::to_string_pretty(&json).unwrap());
    }
}

/// Removes every `loc`-suffixed property recursively.
fn strip_locs(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(map) => {
            map.retain(|key, _| !key.to_ascii_lowercase().ends_with("loc"));
            for value in map.values_mut() {
                strip_locs(value);
            }
        }
        serde_json::Value::Array(items) => {
            for item in items {
                strip_locs(item);
            }
        }
        _ => {}
    }
}

/// Removes every `body` property recursively; the projections remain.
fn strip_bodies(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(map) => {
            map.remove("body");
            for value in map.values_mut() {
                strip_bodies(value);
            }
        }
        serde_json::Value::Array(items) => {
            for item in items {
                strip_bodies(item);
            }
        }
        _ => {}
    }
}

/// Workspace roots above `path`, nearest first. Directories that also carry an
/// `applications/` folder are preferred, so URI segments named `services` do
/// not stop the search too early.
fn workspace_roots(path: &Path) -> Vec<PathBuf> {
    let mut dir = if path.is_dir() {
        path.to_path_buf()
    } else {
        path.parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."))
    };
    let mut roots = Vec::new();
    loop {
        if find_services_root(&dir).is_some() {
            roots.push(dir.clone());
        }
        match dir.parent() {
            Some(parent) if !parent.as_os_str().is_empty() => dir = parent.to_path_buf(),
            _ => break,
        }
    }
    let with_applications: Vec<PathBuf> = roots
        .iter()
        .filter(|root| root.join("applications").is_dir())
        .cloned()
        .collect();
    if with_applications.is_empty() {
        roots
    } else {
        with_applications
    }
}

/// Compact preview payload for editor integrations: tokens, artifact outline,
/// pending comments and this file's diagnostics. Syntax-only unless `--app`
/// selects an application, in which case annotations and semantic diagnostics
/// come from the closure.
fn view(args: &[String]) -> i32 {
    let options = Options::parse(args);
    let Some(file) = options.positional.first() else {
        eprintln!("error: view requires a file");
        return 2;
    };
    let path = PathBuf::from(file);
    let result = match options.app.as_deref() {
        Some(selector) => annotated_view(&path, selector),
        None => syntax_view(&path),
    };
    let payload = match result {
        Ok(payload) => payload,
        Err(error) => {
            eprintln!("error: {error}");
            return 1;
        }
    };
    if options.pretty {
        println!(
            "{}",
            serde_json::to_string_pretty(&payload).unwrap_or_default()
        );
    } else if options.json {
        println!("{}", serde_json::to_string(&payload).unwrap_or_default());
    } else {
        print_view_summary(&payload);
    }
    0
}

fn syntax_view(path: &Path) -> Result<serde_json::Value, String> {
    let source = std::fs::read_to_string(path)
        .map_err(|error| format!("could not read {}: {error}", path.display()))?;
    let text = path.to_string_lossy().to_string();
    let parsed = parse(&source, text.clone());
    Ok(view::payload(view::ViewInput {
        path: &text,
        source: &source,
        ast: &parsed.ast,
        diagnostics: &parsed.diagnostics,
        application: None,
    }))
}

fn annotated_view(path: &Path, selector: &str) -> Result<serde_json::Value, String> {
    let (wanted, text) = (
        std::fs::canonicalize(path).ok(),
        path.to_string_lossy().to_string(),
    );
    let mut error = format!(
        "application `{selector}` was not found from {}",
        path.display()
    );
    for root in workspace_roots(path) {
        let (files, _read_errors) = load_files(&root);
        let services_root = find_services_root(&root);
        let services = services_root
            .as_ref()
            .map(|path| path.to_string_lossy().to_string());
        let analysis = match analyze_application(files, services.as_deref(), selector, false) {
            Ok(analysis) => analysis,
            Err(candidate_error) => {
                error = candidate_error;
                continue;
            }
        };
        let Some(file) = analysis.workspace.files.iter().find(|file| {
            file.path == text
                || wanted.as_ref().is_some_and(|wanted| {
                    std::fs::canonicalize(&file.path).ok().as_ref() == Some(wanted)
                })
        }) else {
            error = format!("{} is not part of the application closure", path.display());
            continue;
        };
        let diagnostics: Vec<archol_core::Diagnostic> = analysis
            .diagnostics
            .iter()
            .filter(|entry| entry.path == file.path)
            .map(|entry| entry.diagnostic.clone())
            .collect();
        let application = analysis.closure.as_ref().map(|closure| {
            (
                closure.application.uri.as_str(),
                closure.application.path.as_str(),
            )
        });
        return Ok(view::payload(view::ViewInput {
            path: &file.path,
            source: &file.source,
            ast: &file.ast,
            diagnostics: &diagnostics,
            application,
        }));
    }
    Err(error)
}

fn print_view_summary(payload: &serde_json::Value) {
    let count = |field: &str| {
        payload
            .get(field)
            .and_then(serde_json::Value::as_array)
            .map(Vec::len)
            .unwrap_or(0)
    };
    if let Some(application) = payload.get("application") {
        println!(
            "application: {}",
            application
                .get("uri")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("")
        );
    }
    println!(
        "{}: {} token(s), {} artifact(s), {} diagnostic(s)",
        payload
            .get("path")
            .and_then(serde_json::Value::as_str)
            .unwrap_or(""),
        count("tokens"),
        count("outline"),
        count("diagnostics"),
    );
    if let Some(diagnostics) = payload
        .get("diagnostics")
        .and_then(serde_json::Value::as_array)
    {
        for diagnostic in diagnostics {
            println!(
                "{}:{}: {} {}: {}",
                diagnostic
                    .get("line")
                    .and_then(serde_json::Value::as_u64)
                    .unwrap_or(1),
                diagnostic
                    .get("column")
                    .and_then(serde_json::Value::as_u64)
                    .unwrap_or(1),
                diagnostic
                    .get("severity")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("error"),
                diagnostic
                    .get("code")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or(""),
                diagnostic
                    .get("message")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or(""),
            );
        }
    }
}

fn tokens(args: &[String]) -> i32 {
    let path = root_arg(args);
    let source = match std::fs::read_to_string(&path) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("could not read {}: {error}", path.display());
            return 2;
        }
    };
    let lexed = lex(&source);
    let index = archol_core::LineIndex::new(&source);
    for token in &lexed.tokens {
        let (line, col) = index.line_col(token.span.start);
        let text = &source[token.span.start..token.span.end];
        println!("{}:{} {:?} {:?}", line + 1, col + 1, token.kind, text);
    }
    for diagnostic in &lexed.diagnostics {
        eprintln!("{}: {}", diagnostic.code, diagnostic.message);
    }
    i32::from(!lexed.diagnostics.is_empty())
}

fn links(args: &[String]) -> i32 {
    let options = Options::parse(args);
    let root = options.root();
    let (files, read_errors) = load_files(&root);
    for (path, message) in &read_errors {
        eprintln!("{path}: {message}");
    }
    let services_root = find_services_root(&root);
    let services = services_root
        .as_ref()
        .map(|path| path.to_string_lossy().to_string());
    let Some(selector) = options.app.as_deref() else {
        return missing_app(files, services.as_deref());
    };
    let analysis = match analyze_application(files, services.as_deref(), selector, false) {
        Ok(analysis) => analysis,
        Err(error) => {
            eprintln!("error: {error}");
            return 1;
        }
    };
    println!(
        "{}",
        serde_json::to_string_pretty(&analysis.links).unwrap_or_else(|error| error.to_string())
    );
    0
}

fn missing_app(files: Vec<FileInput>, services_root: Option<&str>) -> i32 {
    let workspace = parse_workspace(files, services_root);
    let apps = application::discover(&workspace);
    eprintln!("error: --app is required; Archol compiles only from an application.");
    if apps.is_empty() {
        eprintln!("no applications were discovered in the workspace.");
    } else {
        eprintln!("available applications:");
        for app in &apps {
            eprintln!("  {}  {}", app.uri, app.path);
        }
    }
    eprintln!("usage: archol <command> [dir] --app <uri-or-path>");
    2
}

fn print_read_errors(errors: &[(String, String)], json: bool) -> u32 {
    for (path, message) in errors {
        if json {
            println!(
                "{}",
                serde_json::json!({
                    "file": path,
                    "line": 1,
                    "column": 1,
                    "severity": "error",
                    "code": "E_FILE_READ",
                    "message": message,
                })
            );
        } else {
            println!("{path}:1:1: error E_FILE_READ: {message}");
        }
    }
    errors.len() as u32
}

fn root_arg(args: &[String]) -> PathBuf {
    args.iter()
        .find(|arg| !arg.starts_with("--"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn load_files(root: &Path) -> (Vec<FileInput>, Vec<(String, String)>) {
    let mut paths = Vec::new();
    if root.is_file() {
        paths.push(root.to_path_buf());
    } else {
        walk(root, &mut paths);
    }
    paths.sort();
    let mut files = Vec::new();
    let mut errors = Vec::new();
    for path in paths {
        match std::fs::read(&path)
            .ok()
            .and_then(|bytes| String::from_utf8(bytes).ok())
        {
            Some(source) => files.push(FileInput {
                path: path.to_string_lossy().to_string(),
                source,
            }),
            None => errors.push((
                path.to_string_lossy().to_string(),
                "unreadable file or invalid UTF-8".to_string(),
            )),
        }
    }
    (files, errors)
}

fn walk(dir: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut entries: Vec<_> = entries.flatten().collect();
    entries.sort_by_key(|entry| entry.path());
    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name == "target" || name == "node_modules" || name == ".git" {
                continue;
            }
            walk(&path, files);
        } else if path.extension().map(|ext| ext == "archol").unwrap_or(false) {
            files.push(path);
        }
    }
}
