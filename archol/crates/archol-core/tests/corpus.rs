//! Acceptance test: every real `.archol` in the repository must parse.

use std::path::{Path, PathBuf};

#[test]
fn real_examples_parse_without_errors() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../examples");
    if !root.exists() {
        return;
    }
    let mut files = Vec::new();
    walk(&root, &mut files);
    assert!(files.len() >= 300, "too few files: {}", files.len());
    let mut problems = Vec::new();
    for path in &files {
        let source = std::fs::read_to_string(path).unwrap();
        let result = archol_core::parse(&source, path.display().to_string());
        for diagnostic in &result.diagnostics {
            if matches!(diagnostic.severity, archol_core::Severity::Error) {
                problems.push(format!(
                    "{}: {} {}",
                    path.display(),
                    diagnostic.code,
                    diagnostic.message
                ));
            }
        }
    }
    assert!(problems.is_empty(), "failures:\n{}", problems.join("\n"));
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
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
