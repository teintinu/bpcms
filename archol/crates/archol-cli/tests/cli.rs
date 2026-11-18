//! CLI integration tests: exit codes and robustness.

use std::path::PathBuf;
use std::process::Command;

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("archol-cli-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn write(path: &PathBuf, content: &str) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_archol"))
        .args(args)
        .output()
        .unwrap()
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
    let output = run(&["check", dir.to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stdout).contains("E_ENUM_VALUE"));
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
    let file = dir.join("bad.archol");
    std::fs::write(&file, b"document A {\n  x: \xff\n}\n").unwrap();
    let output = run(&["check", file.to_str().unwrap()]);
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
