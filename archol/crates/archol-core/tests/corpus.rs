//! Teste de aceitação: todo `.archol` real do repositório precisa parsear.

use std::path::{Path, PathBuf};

#[test]
fn exemplos_reais_parseiam_sem_erro() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../examples");
    if !root.exists() {
        return;
    }
    let mut files = Vec::new();
    walk(&root, &mut files);
    assert!(files.len() >= 300, "poucos arquivos: {}", files.len());
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
    assert!(problems.is_empty(), "falhas:\n{}", problems.join("\n"));
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
