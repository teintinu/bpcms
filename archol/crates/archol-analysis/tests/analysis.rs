use archol_analysis::{analyze, analyze_application, FileInput};
use archol_core::{Expr, Member};

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
            "service \"example.com/services/demo\" {\n  title: { pt-BR: \"Demo\" }\n  locales: [ \"pt-BR\" ]\n}",
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
            "service \"example.com/services/alpha\" { title: { pt-BR: \"Alpha\" } locales: [ \"pt-BR\" ] }",
        ),
        (
            "ws/services/example.com/services/alpha/alpha-shapes.archol",
            "shape TaxRegimeCode {\n  base: classification\n  value SIMPLES { title: { pt-BR: \"S\" } }\n}",
        ),
        (
            "ws/services/example.com/services/beta/beta-service.archol",
            "service \"example.com/services/beta\" {\n  locales: [ \"pt-BR\" ]\n  imports { service \"example.com/services/alpha\" as Alpha }\n}",
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
            "service \"example.com/services/a\" { title: { pt-BR: \"A\" } locales: [ \"pt-BR\" ] }",
        ),
        (
            "ws/services/example.com/services/a/a-shapes.archol",
            "shape Pay { base: classification\n  value A { title: { pt-BR: \"A\" } }\n}",
        ),
        (
            "ws/services/example.com/services/b/b-service.archol",
            "service \"example.com/services/b\" { title: { pt-BR: \"B\" } locales: [ \"pt-BR\" ] }",
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

#[test]
fn i18n_properties_are_typed_and_filtered_by_application_locales() {
    let files = vec![
        (
            "ws/applications/example.com/app/demo/demo-application.archol",
            "application \"example.com/app/demo\" {\n  services {\n    \"example.com/services/demo\" as Demo\n  }\n  locales: [pt-BR, en-US]\n}",
        ),
        (
            "ws/services/example.com/services/demo/demo-service.archol",
            "service \"example.com/services/demo\" {\n  title: \"Demonstração\"\n  locales: [ \"pt-BR\", \"en\" ]\n}",
        ),
        (
            "ws/services/example.com/services/demo/processes/demo-process.archol",
            "process P {\n  title: { pt-BR: \"Processo\", en: \"Process\", fr: \"Processus\" }\n  goal: \"Fazer a demonstração\"\n}",
        ),
    ]
    .into_iter()
    .map(|(path, source)| FileInput {
        path: path.to_string(),
        source: source.to_string(),
    })
    .collect();
    let analysis = analyze_application(files, Some("ws/services"), "example.com/app/demo", false)
        .expect("analysis");

    let service = analysis
        .workspace
        .files
        .iter()
        .find(|file| file.path.ends_with("demo-service.archol"))
        .expect("service file");
    let service_title = property(&service.ast.items[0].body, "title").expect("service title");
    assert_eq!(service_title.0.as_deref(), Some("I18N"));
    let Expr::Object { members, .. } = service_title.1 else {
        panic!("string shorthand must expand to a locale map");
    };
    assert_eq!(members.len(), 1);
    assert_eq!(members[0].name(), Some("pt-BR"));

    let process = analysis
        .workspace
        .files
        .iter()
        .find(|file| file.path.ends_with("demo-process.archol"))
        .expect("process file");
    let (title_ty, title) = property(&process.ast.items[0].body, "title").expect("title");
    assert_eq!(title_ty.as_deref(), Some("I18N"));
    let Expr::Object { members, .. } = title else {
        panic!("title must be an object");
    };
    let locales: Vec<_> = members.iter().filter_map(|member| member.name()).collect();
    assert_eq!(locales, vec!["pt-BR", "en"]);

    let (goal_ty, goal) = property(&process.ast.items[0].body, "goal").expect("goal");
    assert_eq!(goal_ty.as_deref(), Some("I18N"));
    let Expr::Object { members, .. } = goal else {
        panic!("goal string shorthand must expand to a locale map");
    };
    assert_eq!(members[0].name(), Some("pt-BR"));
}

fn property<'a>(members: &'a [Member], wanted: &str) -> Option<(Option<String>, &'a Expr)> {
    for member in members {
        match member {
            Member::PropertyAssignment {
                name,
                type_ref,
                value,
                ..
            } if name == wanted => return Some((type_ref.clone(), value)),
            Member::Block { body, .. } | Member::EnumValueDeclaration { body, .. } => {
                if let Some(found) = property(body, wanted) {
                    return Some(found);
                }
            }
            _ => {}
        }
    }
    None
}

#[test]
fn service_must_declare_at_least_one_locale() {
    let no_locales = workspace(&[(
        "ws/services/example.com/services/demo/demo-service.archol",
        "service \"example.com/services/demo\" {\n  title: \"Demo\"\n}",
    )]);
    assert!(no_locales
        .diagnostics
        .iter()
        .any(|d| d.diagnostic.code == "E_LOCALES_REQUIRED"));

    let empty = workspace(&[(
        "ws/services/example.com/services/demo/demo-service.archol",
        "service \"example.com/services/demo\" {\n  title: \"Demo\"\n  locales: [ ]\n}",
    )]);
    assert!(empty
        .diagnostics
        .iter()
        .any(|d| d.diagnostic.code == "E_LOCALES_REQUIRED"));

    let valid = workspace(&[(
        "ws/services/example.com/services/demo/demo-service.archol",
        "service \"example.com/services/demo\" {\n  title: \"Demo\"\n  locales: [ \"pt-BR\" ]\n}",
    )]);
    assert!(!valid
        .diagnostics
        .iter()
        .any(|d| d.diagnostic.code == "E_LOCALES_REQUIRED"));
}
