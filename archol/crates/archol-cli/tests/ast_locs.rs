//! Loc coverage: the AST carries byte locs by default and `--no-loc`
//! removes every loc-shaped property from JSON and YAML.

use std::path::PathBuf;
use std::process::Command;

use serde_json::Value;

fn fixture() -> (PathBuf, String) {
    let dir = std::env::temp_dir().join(format!("archol-locs-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("sample.archol");
    let source = "document Sample {\n  title: \"Amostra\"\n  fields {\n    name: string\n  }\n}\n";
    std::fs::write(&path, source).unwrap();
    (path, source.to_string())
}

fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_archol"))
        .args(args)
        .output()
        .unwrap()
}

fn collect_locs(value: &Value, locs: &mut Vec<Value>) {
    match value {
        Value::Object(map) => {
            for (key, item) in map {
                if key == "loc" {
                    locs.push(item.clone());
                } else {
                    collect_locs(item, locs);
                }
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_locs(item, locs);
            }
        }
        _ => {}
    }
}

fn find_string_literal<'a>(value: &'a Value, text: &str) -> Option<&'a Value> {
    match value {
        Value::Object(map) => {
            if map.get("kind").and_then(Value::as_str) == Some("StringLiteral")
                && map.get("value").and_then(Value::as_str) == Some(text)
            {
                return Some(value);
            }
            map.values()
                .find_map(|item| find_string_literal(item, text))
        }
        Value::Array(items) => items
            .iter()
            .find_map(|item| find_string_literal(item, text)),
        _ => None,
    }
}

fn contains_loc_key(value: &Value) -> bool {
    match value {
        Value::Object(map) => map
            .iter()
            .any(|(key, item)| key.to_ascii_lowercase().ends_with("loc") || contains_loc_key(item)),
        Value::Array(items) => items.iter().any(contains_loc_key),
        _ => false,
    }
}

#[test]
fn locs_exist_by_default_and_are_removed_with_no_loc() {
    let (path, source) = fixture();
    let file = path.to_str().unwrap();

    let output = run(&["ast", file]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let ast: Value = serde_json::from_slice(&output.stdout).unwrap();
    let mut locs = Vec::new();
    collect_locs(&ast, &mut locs);
    assert!(!locs.is_empty(), "AST has no loc properties");
    for loc in &locs {
        let start = loc["start"].as_u64().expect("loc.start") as usize;
        let end = loc["end"].as_u64().expect("loc.end") as usize;
        assert!(start <= end, "loc start {start} > end {end}");
        assert!(end <= source.len(), "loc end {end} past source");
    }

    let literal = find_string_literal(&ast, "Amostra").expect("StringLiteral Amostra");
    let loc = &literal["loc"];
    let start = loc["start"].as_u64().unwrap() as usize;
    let end = loc["end"].as_u64().unwrap() as usize;
    assert_eq!(&source[start..end], "\"Amostra\"");

    let output = run(&["ast", file, "--no-loc"]);
    assert!(output.status.success());
    let stripped: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(!contains_loc_key(&stripped), "--no-loc left loc keys");

    let output = run(&["ast", file, "--yaml", "--no-loc"]);
    assert!(output.status.success());
    let yaml = String::from_utf8_lossy(&output.stdout);
    assert!(!yaml.contains("\"loc\""), "--no-loc left loc in YAML");
    assert!(
        !yaml.contains("\"keywordLoc\""),
        "--no-loc left keywordLoc in YAML"
    );

    let _ = std::fs::remove_dir_all(path.parent().unwrap());
}
