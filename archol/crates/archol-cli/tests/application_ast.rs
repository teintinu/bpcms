//! The resolved `ArcholApplication` tree: closed references, typed ids and
//! `reintroduce` merged into the owner artifact.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::{Map, Value};

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

fn def_kind_for_ref(kind: &str) -> Option<String> {
    if !kind.ends_with("Ref") {
        return None;
    }
    if matches!(
        kind,
        "StdRef"
            | "PrimitiveRef"
            | "ThemeRef"
            | "LocaleRef"
            | "FieldRef"
            | "ShapeValueRef"
            | "StepRef"
            | "ActionRef"
    ) {
        return None;
    }
    Some(format!("{}Def", kind.trim_end_matches("Ref")))
}

fn walk(
    value: &Value,
    artifacts: &Map<String, Value>,
    inside_body: bool,
    problems: &mut Vec<String>,
) {
    match value {
        Value::Object(object) => {
            if inside_body {
                return;
            }
            if let Some(kind) = object.get("kind").and_then(Value::as_str) {
                if let Some(expected) = def_kind_for_ref(kind) {
                    let id = object.get("id").and_then(Value::as_str).unwrap_or("");
                    match artifacts.get(id) {
                        Some(target)
                            if target.get("kind").and_then(Value::as_str) == Some(&expected) => {}
                        _ => problems.push(format!("{kind} `{id}` does not resolve to {expected}")),
                    }
                }
                if kind == "FieldRef" {
                    let document = object
                        .get("documentId")
                        .and_then(Value::as_str)
                        .unwrap_or("");
                    let field = object
                        .get("fieldPath")
                        .and_then(Value::as_str)
                        .unwrap_or("");
                    let ok = artifacts.get(document).is_some_and(|doc| {
                        doc.get("fields")
                            .and_then(Value::as_array)
                            .is_some_and(|fields| {
                                fields.iter().any(|field_def| {
                                    field_def.get("name").and_then(Value::as_str) == Some(field)
                                })
                            })
                    });
                    if !ok {
                        problems.push(format!("FieldRef `{document}.{field}` does not resolve"));
                    }
                }
                if kind == "ShapeValueRef" {
                    let shape = object.get("shapeId").and_then(Value::as_str).unwrap_or("");
                    let value = object
                        .get("valuePath")
                        .and_then(Value::as_str)
                        .unwrap_or("");
                    let ok = artifacts.get(shape).is_some_and(|def| {
                        def.get("values")
                            .and_then(Value::as_array)
                            .is_some_and(|values| {
                                values.iter().any(|entry| {
                                    entry.get("name").and_then(Value::as_str) == Some(value)
                                })
                            })
                    });
                    if !ok {
                        problems.push(format!("ShapeValueRef `{shape}.{value}` does not resolve"));
                    }
                }
                if kind == "ActionRef" {
                    let document = object
                        .get("documentId")
                        .and_then(Value::as_str)
                        .unwrap_or("");
                    let action = object
                        .get("actionPath")
                        .and_then(Value::as_str)
                        .unwrap_or("");
                    let ok = artifacts.get(document).is_some_and(|def| {
                        def.get("actions")
                            .and_then(Value::as_array)
                            .is_some_and(|actions| {
                                actions.iter().any(|entry| {
                                    entry.get("name").and_then(Value::as_str) == Some(action)
                                })
                            })
                    });
                    if !ok {
                        problems.push(format!("ActionRef `{document}.{action}` does not resolve"));
                    }
                }
            }
            for (key, item) in object {
                if key != "loc" {
                    walk(item, artifacts, inside_body || key == "body", problems);
                }
            }
        }
        Value::Array(items) => {
            for item in items {
                walk(item, artifacts, inside_body, problems);
            }
        }
        _ => {}
    }
}

#[test]
fn application_ast_is_closed_typed_and_merged() {
    let root = repo_root();
    let mut apps = Vec::new();
    application_files(&root.join("examples/applications"), &mut apps);
    apps.sort();
    assert!(!apps.is_empty(), "no example applications found");

    for app in &apps {
        let source = std::fs::read_to_string(app).unwrap();
        let uri = application_uri(&source);
        let output = Command::new(env!("CARGO_BIN_EXE_archol"))
            .current_dir(&root)
            .args(["application", "examples", "--app", &uri, "--no-loc"])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}: application AST failed: {}",
            app.display(),
            String::from_utf8_lossy(&output.stderr)
        );
        let tree: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(tree["kind"], "ArcholApplication");
        assert_eq!(tree["schemaVersion"], 1);
        assert_eq!(tree["uri"], uri);

        let artifacts = tree["artifact"].as_object().expect("artifact map");
        assert!(
            !artifacts.is_empty(),
            "{}: empty artifact map",
            app.display()
        );
        for (id, artifact) in artifacts {
            assert_eq!(
                artifact["id"].as_str(),
                Some(id.as_str()),
                "{}: artifact key != id",
                app.display()
            );
            if let Some(service) = artifact["serviceId"].as_str() {
                assert_eq!(
                    artifacts
                        .get(service)
                        .and_then(|entry| entry["kind"].as_str()),
                    Some("ServiceDef"),
                    "{}: {id}: serviceId `{service}` does not resolve",
                    app.display()
                );
            }
        }

        let mut problems = Vec::new();
        walk(&tree, artifacts, false, &mut problems);
        assert!(
            problems.is_empty(),
            "{}:\n{}",
            app.display(),
            problems.join("\n")
        );

        // Application role selection and hierarchy resolve to RoleDefs.
        for role in tree["application"]["roles"]
            .as_array()
            .into_iter()
            .flatten()
        {
            let id = role["id"].as_str().unwrap_or("");
            assert_eq!(
                artifacts.get(id).and_then(|entry| entry["kind"].as_str()),
                Some("RoleDef"),
                "{}: app role `{id}`",
                app.display()
            );
        }

        assert_eq!(
            tree["title"]["kind"],
            "ShapeValue",
            "{}: application title",
            app.display()
        );
        assert_eq!(
            tree["theme"]["kind"],
            "ThemeRef",
            "{}: application theme",
            app.display()
        );
        let locales = tree["locales"].as_array().expect("application locales");
        assert!(!locales.is_empty(), "{}: empty locales", app.display());
        assert_eq!(
            locales[0]["default"],
            true,
            "{}: default locale",
            app.display()
        );
        for locale in locales {
            assert_eq!(locale["kind"], "LocaleRef");
            assert!(locale["tag"].as_str().is_some_and(|tag| !tag.is_empty()));
        }
        let environments = tree["environments"]
            .as_array()
            .expect("application environments");
        assert!(
            !environments.is_empty(),
            "{}: empty environments",
            app.display()
        );
        for environment in environments {
            assert_eq!(environment["kind"], "EnvironmentEntry");
            assert!(environment["name"]
                .as_str()
                .is_some_and(|name| !name.is_empty()));
        }

        // Role <-> permission inverse is consistent.
        let mut roles: BTreeMap<&str, &Value> = BTreeMap::new();
        let mut permissions: BTreeMap<&str, &Value> = BTreeMap::new();
        for (id, artifact) in artifacts {
            match artifact["kind"].as_str() {
                Some("RoleDef") => {
                    roles.insert(id.as_str(), artifact);
                }
                Some("PermissionDef") => {
                    permissions.insert(id.as_str(), artifact);
                }
                _ => {}
            }
        }
        for (role_id, role) in &roles {
            for permission in role["permissions"].as_array().into_iter().flatten() {
                let permission_id = permission["id"].as_str().unwrap_or("");
                let inverse = permissions
                    .get(permission_id)
                    .and_then(|permission| permission["roles"].as_array())
                    .is_some_and(|roles| {
                        roles
                            .iter()
                            .any(|role| role["id"].as_str() == Some(role_id))
                    });
                assert!(
                    inverse,
                    "{}: role `{role_id}` -> permission `{permission_id}` lacks inverse",
                    app.display()
                );
            }
        }
    }

    // `reintroduce` is merged into the owner artifact, not duplicated.
    let output = Command::new(env!("CARGO_BIN_EXE_archol"))
        .current_dir(&root)
        .args([
            "application",
            "examples",
            "--app",
            "example.com/app/loja-com-antifraude",
            "--no-loc",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let tree: Value = serde_json::from_slice(&output.stdout).unwrap();
    let artifacts = tree["artifact"].as_object().unwrap();
    let status = artifacts
        .values()
        .find(|artifact| artifact["kind"] == "ShapeDef" && artifact["name"] == "CheckoutStatus")
        .expect("CheckoutStatus shape");
    assert!(
        status["values"]
            .as_array()
            .unwrap()
            .iter()
            .any(|value| value["name"] == "FraudChecked"),
        "reintroduced FraudChecked value missing"
    );
    let session = artifacts
        .values()
        .find(|artifact| artifact["kind"] == "DocumentDef" && artifact["name"] == "CheckoutSession")
        .expect("CheckoutSession document");
    let action = session["actions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|action| action["name"] == "checkFraud")
        .expect("reintroduced checkFraud action");
    assert_eq!(action["to"]["valuePath"], "FraudChecked");
    assert!(
        !action["roles"].as_array().unwrap().is_empty(),
        "checkFraud role reference missing"
    );
}
