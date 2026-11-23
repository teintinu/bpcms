//! Application AST snapshots (`ArcholApplication`) for every example app.
//!
//! Runs `archol application <app> --app <uri> --yaml --no-loc --no-body`,
//! writes `<app>.result-ast.yml` and compares it with
//! `<app>.expected-ast.yml`. The result file is kept so a failure can be
//! diffed. Use `archol/scripts/test-update-snapshots.sh` to refresh the
//! expected files after an intentional change.

use std::path::{Path, PathBuf};
use std::process::Command;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .unwrap()
}

fn application_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            application_files(&path, out);
        } else if path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.ends_with("-application.archol"))
        {
            out.push(path);
        }
    }
}

fn application_uri(source: &str) -> String {
    let marker = "application \"";
    let start = source.find(marker).expect("application declaration") + marker.len();
    let end = source[start..].find('"').expect("application uri") + start;
    source[start..end].to_string()
}

#[test]
fn every_example_application_has_a_matching_application_ast_snapshot() {
    let root = repo_root();
    let mut apps = Vec::new();
    application_files(&root.join("examples/applications"), &mut apps);
    apps.sort();
    assert!(!apps.is_empty(), "no example applications found");

    let mut failures = Vec::new();
    for app in &apps {
        let source = std::fs::read_to_string(app).unwrap();
        let uri = application_uri(&source);
        let slug = app
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .strip_suffix("-application.archol")
            .unwrap()
            .to_string();
        let output = Command::new(env!("CARGO_BIN_EXE_archol"))
            .current_dir(&root)
            .args([
                "application",
                "examples",
                "--app",
                &uri,
                "--yaml",
                "--no-loc",
                "--no-body",
            ])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}: application AST failed: {}",
            app.display(),
            String::from_utf8_lossy(&output.stderr)
        );

        let result = app.with_file_name(format!("{slug}.result-ast.yml"));
        let expected = app.with_file_name(format!("{slug}.expected-ast.yml"));
        std::fs::write(&result, &output.stdout).unwrap();
        let Some(expected_text) = std::fs::read_to_string(&expected).ok() else {
            failures.push(format!(
                "{}: missing expected snapshot {}",
                app.display(),
                expected.display()
            ));
            continue;
        };
        let actual_text = String::from_utf8_lossy(&output.stdout);
        if actual_text != expected_text {
            let first = actual_text
                .lines()
                .zip(expected_text.lines())
                .position(|(actual, expected)| actual != expected)
                .map(|index| index + 1)
                .unwrap_or(
                    actual_text
                        .lines()
                        .count()
                        .min(expected_text.lines().count())
                        + 1,
                );
            failures.push(format!(
                "{}: application AST differs from {} at line {first}; result kept at {}",
                app.display(),
                expected.display(),
                result.display()
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
