//! Archol CLI: check, ast, links, tokens and lsp.

use std::path::{Path, PathBuf};
use std::process::exit;

use archol_analysis::{analyze, FileInput};
use archol_core::{lex, parse};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(command) = args.first() else {
        usage();
        exit(2);
    };
    let rest = &args[1..];
    let code = match command.as_str() {
        "check" => check(rest),
        "ast" => ast(rest),
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
         \x20 check [dir] [--json]   validates the workspace\n\
         \x20 ast <file> [--json]    prints the AST\n\
         \x20 tokens <file>          prints the tokens\n\
         \x20 links [dir] [--json]   inverted index for the builder\n\
         \x20 lsp                    LSP server on stdio"
    );
}

fn root_arg(args: &[String]) -> PathBuf {
    args.iter()
        .find(|arg| !arg.starts_with("--"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn has_flag(args: &[String], flag: &str) -> bool {
    args.iter().any(|arg| arg == flag)
}

fn check(args: &[String]) -> i32 {
    let root = root_arg(args);
    let (files, read_errors) = load_files(&root);
    let services_root = archol_analysis::find_services_root(&root);
    let analysis = analyze(
        files,
        services_root
            .as_ref()
            .map(|p| p.to_string_lossy().to_string())
            .as_deref(),
    );
    let mut errors = 0;
    let mut warnings = 0;
    for error in &read_errors {
        errors += 1;
        if has_flag(args, "--json") {
            println!(
                "{}",
                serde_json::json!({
                    "file": error.0,
                    "line": 1,
                    "column": 1,
                    "severity": "error",
                    "code": "E_FILE_READ",
                    "message": error.1,
                })
            );
        } else {
            println!("{}:1:1: error E_FILE_READ: {}", error.0, error.1);
        }
    }
    for entry in &analysis.diagnostics {
        let diagnostic = &entry.diagnostic;
        let file = analysis
            .workspace
            .files
            .iter()
            .find(|file| file.path == entry.path);
        if let Some(file) = file {
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
            if has_flag(args, "--json") {
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
    }
    if !has_flag(args, "--json") {
        println!(
            "{} file(s), {} error(s), {} warning(s)",
            analysis.workspace.files.len(),
            errors,
            warnings
        );
    }
    if errors > 0 {
        1
    } else {
        0
    }
}

fn ast(args: &[String]) -> i32 {
    let path = root_arg(args);
    let source = match std::fs::read_to_string(&path) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("could not read {}: {error}", path.display());
            return 2;
        }
    };
    let result = parse(&source, path.to_string_lossy().to_string());
    println!("{}", serde_json::to_string_pretty(&result.ast).unwrap());
    if !result.diagnostics.is_empty() {
        for diagnostic in &result.diagnostics {
            eprintln!("{}: {}", diagnostic.code, diagnostic.message);
        }
        return 1;
    }
    0
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
    let root = root_arg(args);
    let (files, _) = load_files(&root);
    let services_root = archol_analysis::find_services_root(&root);
    let analysis = analyze(
        files,
        services_root
            .as_ref()
            .map(|p| p.to_string_lossy().to_string())
            .as_deref(),
    );
    println!(
        "{}",
        serde_json::to_string_pretty(&analysis.links).unwrap_or_else(|error| error.to_string())
    );
    0
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
