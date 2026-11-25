//! Resolved AST for the step-oriented process syntax: service alias inference,
//! `vars`/`lane` blocks, step commands, typed guards, icon literals and catch.

use archol_analysis::{analyze_application, application_ast, FileInput};

fn files() -> Vec<FileInput> {
    vec![
        (
            "ws/applications/example.com/app/arithmetic/arithmetic-application.archol",
            r#"application "example.com/app/arithmetic" {
  services {
    "example.com/services/arithmetic"
  }
  theme: Nuvula
  locales: [pt-BR]
  environments: [dev]
}"#,
        ),
        (
            "ws/services/example.com/services/arithmetic/arithmetic-service.archol",
            r#"service "example.com/services/arithmetic" {
  title: { pt-BR: "Aritmética" }
  locales: [ "pt-BR" ]
}"#,
        ),
        (
            "ws/services/example.com/services/arithmetic/arithmetic-organograms.archol",
            r#"organogram Arithmetic {
  title: { pt-BR: "Aritmética" }
}
role Operator {
  title: { pt-BR: "Operador" }
}"#,
        ),
        (
            "ws/services/example.com/services/arithmetic/arithmetic-shapes.archol",
            r#"shape Operation {
  base: classification
  value Sum { title: { pt-BR: "Soma" }, icon: "+" }
  value Subtract { title: { pt-BR: "Subtração" }, icon: "-" }
}"#,
        ),
        (
            "ws/services/example.com/services/arithmetic/processes/arithmetic-operation-process.archol",
            r#"process Operation() {
  title: { pt-BR: "Somar" }
  vars {
    value1: number
    operation: Operation
    result: number
  }
  lane Operator for [Operator] {
    Ask
    SumJob if operation === Sum
    Show
    catch (error) {
      allow retry
      report ShowError {
        title: { pt-BR: "Erro" }
        show error
      }
    }
  }
}

form Ask {
  title: { pt-BR: "Perguntar" }
  edit value1
  edit operation
}

compute SumJob {
  title: { pt-BR: "Somar" }
  result = value1 + 1
}

report Show {
  title: { pt-BR: "Mostrar" }
  show value1
  show result
}"#,
        ),
        (
            "ws/services/example.com/services/arithmetic/views/arithmetic-progress-view.archol",
            r#"view Progress {
  title: { pt-BR: "Andamento" }
  type: process
  process: Operation
  roles: [Operator]
}"#,
        ),
    ]
    .into_iter()
    .map(|(path, source)| FileInput {
        path: path.to_string(),
        source: source.to_string(),
    })
    .collect()
}

fn tree() -> serde_json::Value {
    let analysis = analyze_application(
        files(),
        Some("ws/services"),
        "example.com/app/arithmetic",
        false,
    )
    .expect("analysis");
    application_ast::build(&analysis, "ws").expect("application AST")
}

#[test]
fn service_alias_is_inferred_from_the_uri() {
    let tree = tree();
    let service = tree["artifact"]
        .as_object()
        .unwrap()
        .values()
        .find(|artifact| artifact["kind"] == "ServiceDef")
        .expect("service");
    assert_eq!(service["aliases"][0]["name"], "Arithmetic");
    assert_eq!(service["entryPoint"], true);
}

#[test]
fn process_variables_lanes_and_step_commands_are_emitted() {
    let tree = tree();
    let process = tree["artifact"]
        .as_object()
        .unwrap()
        .values()
        .find(|artifact| artifact["kind"] == "ProcessDef")
        .expect("process");
    let variables: Vec<_> = process["variables"]
        .as_array()
        .unwrap()
        .iter()
        .map(|variable| variable["name"].as_str().unwrap())
        .collect();
    assert_eq!(variables, ["value1", "operation", "result"]);
    assert_eq!(process["variables"][1]["type"]["kind"], "ShapeRef");
    assert_eq!(process["lanes"][0]["name"], "Operator");
    assert_eq!(process["steps"][0]["stepKind"], "form");
    assert_eq!(process["steps"][0]["commands"][0]["kind"], "EditCommand");
    assert_eq!(process["steps"][1]["stepKind"], "compute");
    assert_eq!(process["steps"][1]["guard"]["kind"], "Binary");
    assert_eq!(process["steps"][1]["guard"]["left"]["kind"], "VariableRef");
    assert_eq!(process["steps"][1]["guard"]["left"]["name"], "operation");
    assert_eq!(
        process["steps"][1]["guard"]["right"]["kind"],
        "ShapeValueRef"
    );
    assert_eq!(process["steps"][1]["guard"]["right"]["valuePath"], "Sum");
    assert_eq!(process["steps"][1]["commands"][0]["kind"], "AssignCommand");
    assert_eq!(process["steps"][2]["stepKind"], "report");
    assert_eq!(process["steps"][2]["commands"][1]["kind"], "ShowCommand");
    assert_eq!(process["lanes"][0]["catches"][0]["kind"], "CatchDef");
    assert_eq!(
        process["lanes"][0]["catches"][0]["steps"][0]["name"],
        "ShowError"
    );
}

#[test]
fn one_character_icon_is_an_icon_literal() {
    let tree = tree();
    let shape = tree["artifact"]
        .as_object()
        .unwrap()
        .values()
        .find(|artifact| artifact["kind"] == "ShapeDef")
        .expect("shape");
    assert_eq!(shape["values"][0]["icon"]["kind"], "IconLiteral");
    assert_eq!(shape["values"][0]["icon"]["value"], "+");
}

#[test]
fn process_binding_prefers_the_process_over_a_same_named_shape() {
    let tree = tree();
    let view = tree["artifact"]
        .as_object()
        .unwrap()
        .values()
        .find(|artifact| artifact["kind"] == "ViewDef")
        .expect("view");
    assert_eq!(view["binds"][0]["kind"], "ProcessRef");
}
