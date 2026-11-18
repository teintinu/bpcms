use archol_analysis::{analyze, FileInput};

fn workspace(files: &[(&str, &str)]) -> archol_analysis::Analysis {
    analyze(
        files
            .iter()
            .map(|(path, source)| FileInput {
                path: path.to_string(),
                source: source.to_string(),
            })
            .collect(),
        Some("ws/services"),
    )
}

const SERVICE: &str = "ws/services/example.com/services/demo/demo-service.archol";
const SHAPES: &str = "ws/services/example.com/services/demo/demo-shapes.archol";

fn base_files() -> Vec<(&'static str, &'static str)> {
    vec![
        (
            SERVICE,
            "service \"example.com/services/demo\" {\n  title: { pt-BR: \"Demo\" }\n}",
        ),
        (
            SHAPES,
            "shape TaxRegimeCode {\n  base: classification\n  value SIMPLES { title: { pt-BR: \"Simples\" } }\n  value LUCRO_REAL { title: { pt-BR: \"Real\" } }\n}\nrole Clerk {\n  title: { pt-BR: \"Atendente\" }\n}",
        ),
    ]
}

#[test]
fn const_with_invalid_enum_value() {
    let mut files = base_files();
    files.push((
        "ws/services/example.com/services/demo/processes/p.archol",
        "process P(regime: TaxRegimeCode) {\n  const a: TaxRegimeCode = OUTRO\n}",
    ));
    let analysis = workspace(&files);
    let codes: Vec<_> = analysis
        .diagnostics
        .iter()
        .map(|d| d.diagnostic.code.as_str())
        .collect();
    assert!(codes.contains(&"E_ENUM_VALUE"), "{codes:?}");
}

#[test]
fn comparison_with_invalid_enum_value() {
    let mut files = base_files();
    files.push((
        "ws/services/example.com/services/demo/processes/p.archol",
        "process P(regime: TaxRegimeCode) {\n  if (regime == INVALIDO) {\n    return Done\n  }\n}",
    ));
    let analysis = workspace(&files);
    assert!(analysis
        .diagnostics
        .iter()
        .any(|d| d.diagnostic.code == "E_ENUM_VALUE"));
}

#[test]
fn valid_comparison_does_not_error() {
    let mut files = base_files();
    files.push((
        "ws/services/example.com/services/demo/processes/p.archol",
        "process P(regime: TaxRegimeCode) {\n  if (regime == SIMPLES) {\n    return Done\n  }\n}",
    ));
    let analysis = workspace(&files);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis
            .diagnostics
            .iter()
            .map(|d| (&d.diagnostic.code, &d.diagnostic.message))
            .collect::<Vec<_>>()
    );
}

#[test]
fn unknown_role() {
    let mut files = base_files();
    files.push((
        "ws/services/example.com/services/demo/views/v.archol",
        "view V {\n  roles: [Ghost]\n}",
    ));
    let analysis = workspace(&files);
    assert!(analysis
        .diagnostics
        .iter()
        .any(|d| d.diagnostic.code == "E_UNKNOWN_ROLE"));
}

#[test]
fn link_inverts_role_to_artifacts() {
    let mut files = base_files();
    files.push((
        "ws/services/example.com/services/demo/views/v.archol",
        "view V {\n  roles: [Clerk]\n}",
    ));
    let analysis = workspace(&files);
    let clerk = analysis
        .links
        .roles
        .iter()
        .find(|role| role.name == "Clerk")
        .expect("role in link");
    assert!(clerk.used_by.iter().any(|usage| usage.artifact == "V"));
}

#[test]
fn link_exposes_enum_values() {
    let analysis = workspace(&base_files());
    let regime = analysis
        .links
        .enums
        .iter()
        .find(|e| e.name == "TaxRegimeCode")
        .expect("enum in link");
    let values: Vec<_> = regime.values.iter().map(|v| v.name.as_str()).collect();
    assert_eq!(values, vec!["SIMPLES", "LUCRO_REAL"]);
}

#[test]
fn duplicate_in_same_service() {
    let mut files = base_files();
    files.push((
        "ws/services/example.com/services/demo/demo-shapes2.archol",
        "role Clerk {\n  title: { pt-BR: \"Outro\" }\n}",
    ));
    let analysis = workspace(&files);
    assert!(analysis
        .diagnostics
        .iter()
        .any(|d| d.diagnostic.code == "E_DUP_ARTIFACT"));
}

#[test]
fn enum_from_other_service_with_alias() {
    let files = vec![
        (
            "ws/services/example.com/services/alpha/alpha-service.archol",
            "service \"example.com/services/alpha\" { title: { pt-BR: \"Alpha\" } }",
        ),
        (
            "ws/services/example.com/services/alpha/alpha-shapes.archol",
            "shape TaxRegimeCode {\n  base: classification\n  value SIMPLES { title: { pt-BR: \"S\" } }\n}",
        ),
        (
            "ws/services/example.com/services/beta/beta-service.archol",
            "service \"example.com/services/beta\" {\n  imports { service \"example.com/services/alpha\" as Alpha }\n}",
        ),
        (
            "ws/services/example.com/services/beta/processes/beta.archol",
            "process Beta {\n  const a: Alpha.TaxRegimeCode = OUTRO\n}",
        ),
    ];
    let analysis = workspace(&files);
    assert!(
        analysis
            .diagnostics
            .iter()
            .any(|d| d.diagnostic.code == "E_ENUM_VALUE"),
        "{:?}",
        analysis
            .diagnostics
            .iter()
            .map(|d| &d.diagnostic.code)
            .collect::<Vec<_>>()
    );
}

#[test]
fn homonymous_enum_does_not_yield_false_positive() {
    let files = vec![
        (
            "ws/services/example.com/services/a/a-service.archol",
            "service \"example.com/services/a\" { title: { pt-BR: \"A\" } }",
        ),
        (
            "ws/services/example.com/services/a/a-shapes.archol",
            "shape Pay { base: classification\n  value A { title: { pt-BR: \"A\" } }\n}",
        ),
        (
            "ws/services/example.com/services/b/b-service.archol",
            "service \"example.com/services/b\" { title: { pt-BR: \"B\" } }",
        ),
        (
            "ws/services/example.com/services/b/b-shapes.archol",
            "shape Pay { base: classification\n  value B { title: { pt-BR: \"B\" } }\n}",
        ),
        (
            "ws/services/example.com/services/a/processes/a.archol",
            "process PA {\n  const x: Pay = A\n}",
        ),
        (
            "ws/services/example.com/services/b/processes/b.archol",
            "process PB {\n  const x: Pay = B\n}",
        ),
    ];
    let analysis = workspace(&files);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics.len()
    );
    let pays: Vec<_> = analysis
        .links
        .enums
        .iter()
        .filter(|e| e.name == "Pay")
        .collect();
    assert_eq!(pays.len(), 2);
}

#[test]
fn literal_on_left_points_to_literal() {
    let mut files = base_files();
    let source =
        "process P(regime: TaxRegimeCode) {\n  if (INVALIDO == regime) {\n    return Done\n  }\n}";
    files.push((
        "ws/services/example.com/services/demo/processes/p.archol",
        source,
    ));
    let analysis = workspace(&files);
    let diagnostic = analysis
        .diagnostics
        .iter()
        .find(|d| d.diagnostic.code == "E_ENUM_VALUE")
        .expect("diagnostic");
    let expected = source.find("INVALIDO").unwrap();
    assert_eq!(diagnostic.diagnostic.span.start, expected);
}
