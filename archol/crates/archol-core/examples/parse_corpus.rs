use std::path::Path;
use std::process::exit;

fn main() {
    let root = std::env::args().nth(1).expect("usage: parse_corpus <dir>");
    let mut files = Vec::new();
    walk(Path::new(&root), &mut files);
    files.sort();
    let mut total = 0;
    let mut errors = 0;
    for path in &files {
        let source = std::fs::read_to_string(path).unwrap();
        let result = archol_core::parse(&source, path.display().to_string());
        let errs: Vec<_> = result
            .diagnostics
            .iter()
            .filter(|d| matches!(d.severity, archol_core::Severity::Error))
            .collect();
        total += result.ast.items.len();
        if !errs.is_empty() {
            errors += 1;
            println!("{}: {} error(s)", path.display(), errs.len());
            let index = archol_core::LineIndex::new(&source);
            for d in errs.iter().take(3) {
                let (line, col) = index.line_col(d.span.start);
                let text = source.lines().nth(line as usize).unwrap_or("").trim();
                println!(
                    "  {}:{} {} {} | {}",
                    line + 1,
                    col,
                    d.code,
                    d.message,
                    text.chars().take(90).collect::<String>()
                );
            }
        }
    }
    println!(
        "files={} artifacts={} with_errors={}",
        files.len(),
        total,
        errors
    );
    if errors > 0 {
        exit(1);
    }
}

fn walk(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            walk(&path, out);
        } else if path.extension().map(|e| e == "archol").unwrap_or(false) {
            out.push(path);
        }
    }
}
