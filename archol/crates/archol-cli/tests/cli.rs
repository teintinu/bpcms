//! CLI integration tests: exit codes, application selection and locks.

use std::path::{Path, PathBuf};
use std::process::Command;

const APP: &str = "example.com/app/demo";
const SERVICE: &str = "example.com/services/demo";

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("archol-cli-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn write(path: &Path, content: &str) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_archol"))
        .args(args)
        .output()
        .unwrap()
}

/// Writes a minimal application fixture and returns its file path.
fn app_fixture(dir: &Path, app_uri: &str, service_uri: &str) -> PathBuf {
    let slug = app_uri.rsplit('/').next().unwrap();
    let path = dir
        .join("applications")
        .join(app_uri)
        .join(format!("{slug}-application.archol"));
    write(
        &path,
        &format!(
            "application \"{app_uri}\" {{\n  services {{\n    \"{service_uri}\" as Demo\n  }}\n}}"
        ),
    );
    path
}

fn service_fixture(dir: &Path) -> PathBuf {
    let path = dir
        .join("services")
        .join(SERVICE)
        .join("demo-service.archol");
    write(
        &path,
        "service \"example.com/services/demo\" { title: { pt-BR: \"Demo\" } }",
    );
    path
}

#[test]
fn check_reports_invalid_enum_and_exits_with_1() {
    let dir = temp_dir("enum");
    write(
        &dir.join("services/example.com/services/demo/demo-service.archol"),
        "service \"example.com/services/demo\" { title: { pt-BR: \"Demo\" } }",
    );
    write(
        &dir.join("services/example.com/services/demo/demo-shapes.archol"),
        "shape Regime {\n  base: classification\n  value SIMPLES { title: { pt-BR: \"S\" } }\n}",
    );
    write(
        &dir.join("services/example.com/services/demo/processes/p.archol"),
        "process P(regime: Regime) {\n  const a: Regime = OUTRO\n}",
    );
    app_fixture(&dir, APP, SERVICE);
    let output = run(&["check", dir.to_str().unwrap(), "--app", APP]);
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stdout).contains("E_ENUM_VALUE"));
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn check_without_app_exits_2_and_mentions_app() {
    let dir = temp_dir("missing-app");
    service_fixture(&dir);
    app_fixture(&dir, APP, SERVICE);
    let output = run(&["check", dir.to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(2));
    let mut text = String::from_utf8_lossy(&output.stdout).to_string();
    text.push_str(&String::from_utf8_lossy(&output.stderr));
    assert!(text.contains("--app"), "{text}");
    assert!(text.contains(APP), "{text}");
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn tokens_with_trailing_backslash_does_not_panic() {
    let dir = temp_dir("escape");
    let file = dir.join("a.archol");
    write(&file, "document A { note: \"abc\\");
    let output = run(&["tokens", file.to_str().unwrap()]);
    assert_ne!(output.status.code(), Some(101));
    assert!(!String::from_utf8_lossy(&output.stderr).contains("panicked"));
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn check_reports_invalid_utf8() {
    let dir = temp_dir("utf8");
    service_fixture(&dir);
    app_fixture(&dir, APP, SERVICE);
    let file = dir.join("services/example.com/services/demo/bad.archol");
    std::fs::write(&file, b"document A {\n  x: \xff\n}\n").unwrap();
    let output = run(&["check", dir.to_str().unwrap(), "--app", APP]);
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stdout).contains("E_FILE_READ"));
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn deep_nesting_does_not_overflow_stack() {
    let dir = temp_dir("deep");
    let file = dir.join("deep.archol");
    let mut source = String::from("document D {\n");
    for _ in 0..400 {
        source.push_str("a: { ");
    }
    source.push_str("b: 1");
    for _ in 0..400 {
        source.push_str(" }");
    }
    source.push_str("\n}\n");
    write(&file, &source);
    let output = run(&["ast", file.to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(1));
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn lock_creates_stable_jsonl_and_prunes_removed_identifiers() {
    let dir = temp_dir("lock");
    service_fixture(&dir);
    let shapes = dir
        .join("services")
        .join(SERVICE)
        .join("demo-shapes.archol");
    write(
        &shapes,
        "shape Regime {\n  base: classification\n  value SIMPLES { title: { pt-BR: \"S\" } }\n  value OUTRO { title: { pt-BR: \"O\" } }\n}",
    );
    write(
        &dir.join("services")
            .join(SERVICE)
            .join("demo-document.archol"),
        "document Invoice {\n  title: { pt-BR: \"Nota\" }\n  fields {\n    regime: Regime { title: { pt-BR: \"Regime\" } }\n  }\n}",
    );
    let app_path = app_fixture(&dir, APP, SERVICE);
    let lock_path = app_path.parent().unwrap().join("bpcms.lock");

    let first = run(&["lock", dir.to_str().unwrap(), "--app", APP]);
    assert_eq!(first.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&first.stdout);
    assert!(
        stdout.contains("application: example.com/app/demo"),
        "{stdout}"
    );
    assert!(stdout.contains("identifiers:"), "{stdout}");

    let text = std::fs::read_to_string(&lock_path).unwrap();
    let entries: Vec<serde_json::Value> = text
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(
        entries.len(),
        text.lines().filter(|line| !line.is_empty()).count()
    );
    assert!(!entries.is_empty());
    let mut paths = Vec::new();
    for entry in &entries {
        let id = entry["id"].as_str().unwrap();
        assert!(!id.chars().next().unwrap().is_ascii_digit(), "{id}");
        paths.push(entry["path"].as_str().unwrap().to_string());
    }
    assert!(paths.contains(&format!("{SERVICE}#service")), "{paths:?}");
    assert!(
        paths.contains(&format!("{SERVICE}#shape/Regime/value/SIMPLES")),
        "{paths:?}"
    );
    assert!(
        paths.contains(&format!("{SERVICE}#document/Invoice/field/regime")),
        "{paths:?}"
    );

    let second = run(&["lock", dir.to_str().unwrap(), "--app", APP]);
    assert_eq!(second.status.code(), Some(0));
    let stable = std::fs::read_to_string(&lock_path).unwrap();
    assert_eq!(text, stable, "ids must stay stable on a second run");

    write(
        &shapes,
        "shape Regime {\n  base: classification\n  value OUTRO { title: { pt-BR: \"O\" } }\n}",
    );
    let third = run(&["lock", dir.to_str().unwrap(), "--app", APP]);
    assert_eq!(third.status.code(), Some(0));
    let pruned = std::fs::read_to_string(&lock_path).unwrap();
    assert!(!pruned.contains("#shape/Regime/value/SIMPLES"), "{pruned}");
    assert!(pruned.contains("#shape/Regime/value/OUTRO"), "{pruned}");
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn applications_lists_fixture_app() {
    let dir = temp_dir("applications");
    service_fixture(&dir);
    let app_path = app_fixture(&dir, APP, SERVICE);
    let output = run(&["applications", dir.to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    let expected = format!("{APP}\t{}", app_path.to_string_lossy());
    assert!(stdout.lines().any(|line| line == expected), "{stdout}");

    let json = run(&["applications", dir.to_str().unwrap(), "--json"]);
    assert_eq!(json.status.code(), Some(0));
    let apps: Vec<serde_json::Value> = serde_json::from_slice(&json.stdout).unwrap();
    assert!(apps.iter().any(|app| app["uri"] == APP), "{apps:?}");
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn ast_with_app_contains_typed_references() {
    let dir = temp_dir("ast-app");
    service_fixture(&dir);
    write(
        &dir.join("services")
            .join(SERVICE)
            .join("demo-shapes.archol"),
        "shape Regime {\n  base: classification\n  value SIMPLES { title: { pt-BR: \"S\" } }\n}",
    );
    write(
        &dir.join("services")
            .join(SERVICE)
            .join("demo-document.archol"),
        "document Invoice {\n  title: { pt-BR: \"Nota\" }\n  fields {\n    regime: Regime { title: { pt-BR: \"Regime\" } }\n  }\n}",
    );
    let process = dir
        .join("services")
        .join(SERVICE)
        .join("demo-process.archol");
    write(
        &process,
        "process P(invoice: Invoice) {\n  if (invoice.regime == SIMPLES) {\n    return Done\n  }\n}",
    );
    app_fixture(&dir, APP, SERVICE);

    let output = run(&["ast", process.to_str().unwrap(), "--app", APP]);
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    let ast: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(ast["kind"], "SourceFile");
    assert!(stdout.contains("\"ref\":"), "{stdout}");
    assert!(stdout.contains("\"fieldRef\":"), "{stdout}");
    assert!(stdout.contains("\"typeRefId\":"), "{stdout}");
    let _ = std::fs::remove_dir_all(dir);
}
