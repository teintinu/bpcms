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
    let (service_ty, service_title) =
        property(&service.ast.items[0].body, "title").expect("service title");
    assert_eq!(service_ty, None);
    let Expr::ShapeValue {
        type_ref, value, ..
    } = service_title
    else {
        panic!("string shorthand must become a ShapeValue");
    };
    assert_eq!(type_ref, "std.bpcms.i18n");
    assert_eq!(value.keys().collect::<Vec<_>>(), vec!["pt-BR"]);

    let process = analysis
        .workspace
        .files
        .iter()
        .find(|file| file.path.ends_with("demo-process.archol"))
        .expect("process file");
    let (title_ty, title) = property(&process.ast.items[0].body, "title").expect("title");
    assert_eq!(title_ty, None);
    let Expr::ShapeValue {
        type_ref, value, ..
    } = title
    else {
        panic!("title must become a ShapeValue");
    };
    assert_eq!(type_ref, "std.bpcms.i18n");
    assert_eq!(
        value.keys().map(String::as_str).collect::<Vec<_>>(),
        vec!["en", "pt-BR"]
    );

    let (goal_ty, goal) = property(&process.ast.items[0].body, "goal").expect("goal");
    assert_eq!(goal_ty, None);
    let Expr::ShapeValue { value, .. } = goal else {
        panic!("goal string shorthand must become a ShapeValue");
    };
    assert_eq!(value.keys().collect::<Vec<_>>(), vec!["pt-BR"]);
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

fn shape_files(process: &str) -> Vec<FileInput> {
    [
        (
            "ws/applications/example.com/app/demo/demo-application.archol",
            "application \"example.com/app/demo\" {\n  services {\n    \"example.com/services/demo\" as Demo\n  }\n  locales: [pt-BR, en-US]\n}",
        ),
        (
            "ws/services/example.com/services/demo/demo-service.archol",
            "service \"example.com/services/demo\" {\n  title: { pt-BR: \"Demo\" }\n  locales: [ \"pt-BR\", \"en\" ]\n  imports { service \"bpcms.org/std\" as std }\n}",
        ),
        (
            "ws/services/example.com/services/demo/demo-shapes.archol",
            "shape ProductVideo {\n  title: { pt-BR: \"Vídeo\" }\n  base: video\n}",
        ),
        (
            "ws/services/example.com/services/demo/demo-process.archol",
            process,
        ),
        (
            "ws/services/bpcms.org/std/std-service.archol",
            "service \"bpcms.org/std\" {\n  title: { pt-BR: \"Std\" }\n  locales: [ \"pt-BR\" ]\n}",
        ),
        (
            "ws/services/bpcms.org/std/std-shapes.archol",
            "shape Duration {\n  title: { pt-BR: \"Duração\" }\n  base: decimal\n  units {\n    s: { factor: 1 }\n    h: { factor: 3600 }\n  }\n  parse: quantity\n}\nshape Money {\n  title: { pt-BR: \"Dinheiro\" }\n  base: decimal\n}\nshape Email {\n  title: { pt-BR: \"E-mail\" }\n  base: string\n  normalize: lower-trim\n  validate: email\n}",
        ),
    ]
    .into_iter()
    .map(|(path, source)| FileInput {
        path: path.to_string(),
        source: source.to_string(),
    })
    .collect()
}

#[test]
fn duration_and_shape_strings_become_shape_literals() {
    let files = shape_files(
        "process P {\n  let video: ProductVideo = \"https://example.com/v.mp4\"\n  title: \"Processo\"\n  sla: \"2h\"\n}",
    );
    let analysis = analyze_application(files, Some("ws/services"), "example.com/app/demo", false)
        .expect("analysis");
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis
            .diagnostics
            .iter()
            .map(|d| (&d.diagnostic.code, &d.diagnostic.message))
            .collect::<Vec<_>>()
    );
    let process = analysis
        .workspace
        .files
        .iter()
        .find(|file| file.path.ends_with("demo-process.archol"))
        .expect("process");
    let (_, sla) = property(&process.ast.items[0].body, "sla").expect("sla");
    let Expr::ShapeLiteral {
        shape,
        raw,
        value,
        unit,
        ..
    } = sla
    else {
        panic!("sla must be a ShapeLiteral");
    };
    assert_eq!(shape, "std.Duration");
    assert_eq!(raw, "2h");
    assert!(matches!(value, archol_core::ShapeData::Integer(7200)));
    assert_eq!(unit.as_deref(), Some("s"));

    let mut video = None;
    walk_members(&process.ast.items[0].body, &mut |member| {
        if let Member::Statement {
            statement: archol_core::Stmt::LetDeclaration { name, init, .. },
            ..
        } = member
        {
            if name == "video" {
                video = init.clone();
            }
        }
    });
    let Some(Expr::ShapeLiteral {
        shape, value, unit, ..
    }) = video
    else {
        panic!("video must be a ShapeLiteral");
    };
    assert_eq!(shape, "ProductVideo");
    assert!(
        matches!(value, archol_core::ShapeData::Text(text) if text == "https://example.com/v.mp4")
    );
    assert_eq!(unit, None);
}

#[test]
fn invalid_shape_literals_are_compile_errors() {
    let files = shape_files("process P {\n  title: \"P\"\n  sla: \"2x\"\n}");
    let analysis = analyze_application(files, Some("ws/services"), "example.com/app/demo", false)
        .expect("analysis");
    assert!(analysis
        .diagnostics
        .iter()
        .any(|d| d.diagnostic.code == "E_SHAPE_LITERAL"));

    let files = shape_files("process P {\n  title: \"P\"\n  sla: 2h\n}");
    let analysis = analyze_application(files, Some("ws/services"), "example.com/app/demo", false)
        .expect("analysis");
    assert!(analysis
        .diagnostics
        .iter()
        .any(|d| d.diagnostic.code == "E_SHAPE_LITERAL_QUOTED"));
}

fn walk_members(members: &[Member], visit: &mut impl FnMut(&Member)) {
    for member in members {
        visit(member);
        match member {
            Member::Block { body, .. } | Member::EnumValueDeclaration { body, .. } => {
                walk_members(body, visit)
            }
            _ => {}
        }
    }
}

#[test]
fn parameter_shape_literals_use_declared_type() {
    let mut files = shape_files("process P {\n  title: \"P\"\n}");
    files.push(FileInput {
        path: "ws/services/example.com/services/demo/demo-parameters.archol".to_string(),
        source: "parameter Wait {\n  title: { pt-BR: \"Espera\" }\n  type: std.Duration\n  default: \"2h\"\n}\nparameter Price {\n  title: { pt-BR: \"Preço\" }\n  type: std.Money\n  default: \"500 BRL\"\n}\nparameter Mail {\n  title: { pt-BR: \"E-mail\" }\n  type: std.Email\n  default: \"  USER@Example.COM \"\n}".to_string(),
    });
    let analysis = analyze_application(files, Some("ws/services"), "example.com/app/demo", false)
        .expect("analysis");
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis
            .diagnostics
            .iter()
            .map(|d| (&d.diagnostic.code, &d.diagnostic.message))
            .collect::<Vec<_>>()
    );
    let file = analysis
        .workspace
        .files
        .iter()
        .find(|file| file.path.ends_with("demo-parameters.archol"))
        .expect("parameters");
    let (_, duration) = property(&file.ast.items[0].body, "default").expect("duration default");
    assert!(matches!(
        duration,
        Expr::ShapeLiteral {
            value: archol_core::ShapeData::Integer(7200),
            unit: Some(unit),
            ..
        } if unit == "s"
    ));
    let (_, money) = property(&file.ast.items[1].body, "default").expect("money default");
    assert!(matches!(
        money,
        Expr::ShapeLiteral {
            value: archol_core::ShapeData::Object(value),
            ..
        } if matches!(value.get("currency"), Some(archol_core::ShapeData::Text(code)) if code == "BRL")
    ));
    let (_, email) = property(&file.ast.items[2].body, "default").expect("email default");
    assert!(
        matches!(
            email,
            Expr::ShapeLiteral {
                value: archol_core::ShapeData::Text(text),
                ..
            } if text == "user@example.com"
        ),
        "{email:?}"
    );
}

#[test]
fn i18n_string_uses_service_locales_except_for_the_application() {
    let files = vec![
        (
            "ws/applications/example.com/app/demo/demo-application.archol",
            "application \"example.com/app/demo\" {\n  services {\n    \"example.com/services/demo\" as Demo\n  }\n  title: \"App\"\n  locales: [ \"pt-BR\", \"en-US\" ]\n}",
        ),
        (
            "ws/services/example.com/services/demo/demo-service.archol",
            "service \"example.com/services/demo\" {\n  title: \"Serviço\"\n  locales: [ \"en\", \"pt-BR\" ]\n}",
        ),
        (
            "ws/services/example.com/services/demo/processes/demo-process.archol",
            "process P {\n  title: \"Processo\"\n}",
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

    let default_locale = |file_suffix: &str, wanted: &str| {
        let file = analysis
            .workspace
            .files
            .iter()
            .find(|file| file.path.ends_with(file_suffix))
            .expect("file");
        let (_, value) = property(&file.ast.items[0].body, wanted).expect("property");
        let Expr::ShapeValue { value, .. } = value else {
            panic!("{wanted} must be a ShapeValue");
        };
        value.keys().next().cloned()
    };

    assert_eq!(
        default_locale("demo-application.archol", "title").as_deref(),
        Some("pt-BR")
    );
    assert_eq!(
        default_locale("demo-service.archol", "title").as_deref(),
        Some("en")
    );
    assert_eq!(
        default_locale("demo-process.archol", "title").as_deref(),
        Some("en")
    );
}

fn organogram_files(hierarchy: &str) -> Vec<FileInput> {
    [
        (
            "ws/applications/example.com/app/demo/demo-application.archol",
            "application \"example.com/app/demo\" {\n  services {\n    \"example.com/services/demo\" as Demo\n  }\n  locales: [pt-BR]\n}",
        ),
        (
            "ws/services/example.com/services/demo/demo-service.archol",
            "service \"example.com/services/demo\" {\n  title: { pt-BR: \"Demo\" }\n  locales: [ \"pt-BR\" ]\n}",
        ),
        (
            "ws/services/example.com/services/demo/demo-organograms.archol",
            &format!(
                "organogram Finance {{\n  title: {{ pt-BR: \"Financeiro\" }}\n  hierarchy {{\n{hierarchy}\n  }}\n}}\nrole Clerk {{\n  title: {{ pt-BR: \"Faturista\" }}\n  permissions: [invoice.view]\n}}\nrole CollectionAgent {{\n  title: {{ pt-BR: \"Cobrança\" }}\n  inherits: Clerk\n  permissions: [invoice.collect]\n}}\nrole FinanceManager {{\n  title: {{ pt-BR: \"Gerente\" }}\n  permissions: [invoice.approve]\n}}"
            ),
        ),
    ]
    .into_iter()
    .map(|(path, source)| FileInput {
        path: path.to_string(),
        source: source.to_string(),
    })
    .collect()
}

#[test]
fn hierarchy_roles_resolve_to_lock_refs_and_derive_manages() {
    let analysis = analyze_application(
        organogram_files("FinanceManager > CollectionAgent > Clerk"),
        Some("ws/services"),
        "example.com/app/demo",
        false,
    )
    .expect("analysis");
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis
            .diagnostics
            .iter()
            .map(|d| (&d.diagnostic.code, &d.diagnostic.message))
            .collect::<Vec<_>>()
    );
    let organogram = analysis
        .workspace
        .files
        .iter()
        .flat_map(|file| file.ast.items.iter())
        .find(|artifact| artifact.kind == archol_core::ArtifactKind::Organogram)
        .expect("organogram");
    let Member::Hierarchy { chains, .. } = &organogram.body[1] else {
        panic!("expected hierarchy");
    };
    assert!(chains[0].roles.iter().all(|role| role.role_ref.is_some()));
    assert!(chains[0].roles.iter().all(|role| role.path.is_some()));

    let by_name = |name: &str| {
        organogram
            .roles
            .iter()
            .find(|role| role.name == name)
            .unwrap_or_else(|| panic!("role {name}"))
    };
    let order: Vec<_> = organogram
        .roles
        .iter()
        .map(|role| role.name.as_str())
        .collect();
    assert_eq!(order, vec!["FinanceManager", "CollectionAgent", "Clerk"]);
    assert_eq!(
        by_name("FinanceManager")
            .manages
            .iter()
            .map(|role| role.name.as_str())
            .collect::<Vec<_>>(),
        vec!["CollectionAgent", "Clerk"]
    );
    assert!(by_name("Clerk").manages.is_empty());
    assert_eq!(
        by_name("CollectionAgent")
            .inherits
            .iter()
            .map(|role| role.name.as_str())
            .collect::<Vec<_>>(),
        vec!["Clerk"]
    );
    assert_eq!(
        by_name("FinanceManager")
            .permissions
            .iter()
            .map(|permission| permission.name.as_str())
            .collect::<Vec<_>>(),
        vec!["invoice.approve"]
    );
    assert!(by_name("FinanceManager")
        .permissions
        .iter()
        .all(|permission| !permission.permission_ref.is_empty()));
}

#[test]
fn hierarchy_reports_unknown_roles_and_cycles() {
    let unknown = analyze_application(
        organogram_files("FinanceManager > Ghost"),
        Some("ws/services"),
        "example.com/app/demo",
        false,
    )
    .expect("analysis");
    assert!(unknown
        .diagnostics
        .iter()
        .any(|d| d.diagnostic.code == "E_UNKNOWN_ROLE"));

    let cycle = analyze_application(
        organogram_files("FinanceManager > CollectionAgent\n    CollectionAgent > FinanceManager"),
        Some("ws/services"),
        "example.com/app/demo",
        false,
    )
    .expect("analysis");
    assert!(cycle
        .diagnostics
        .iter()
        .any(|d| d.diagnostic.code == "E_HIERARCHY_CYCLE"));

    let duplicate = analyze_application(
        organogram_files("FinanceManager > Clerk\n    FinanceManager > Clerk"),
        Some("ws/services"),
        "example.com/app/demo",
        false,
    )
    .expect("analysis");
    assert!(duplicate
        .diagnostics
        .iter()
        .any(|d| d.diagnostic.code == "E_HIERARCHY_DUPLICATE"));
}

#[test]
fn service_ast_indexes_its_artifacts_and_auto_permissions() {
    let analysis = analyze_application(
        organogram_files("FinanceManager > CollectionAgent > Clerk"),
        Some("ws/services"),
        "example.com/app/demo",
        false,
    )
    .expect("analysis");
    let service = analysis
        .workspace
        .files
        .iter()
        .flat_map(|file| file.ast.items.iter())
        .find(|artifact| artifact.kind == archol_core::ArtifactKind::Service)
        .expect("service");
    let names: Vec<_> = service
        .artifacts
        .iter()
        .map(|pointer| (pointer.artifact.name(), pointer.name.as_str()))
        .collect();
    assert!(names.contains(&("Organogram", "Finance")), "{names:?}");
    assert!(names.contains(&("Role", "FinanceManager")), "{names:?}");
    assert!(names.contains(&("Permission", "invoice.view")), "{names:?}");
    assert!(service
        .artifacts
        .iter()
        .all(|pointer| !pointer.artifact_ref.is_empty() && !pointer.path.is_empty()));

    let permissions: Vec<_> = service
        .permissions
        .iter()
        .map(|permission| {
            (
                permission.name.as_str(),
                permission.namespace.as_str(),
                permission.action.as_str(),
            )
        })
        .collect();
    assert_eq!(
        permissions,
        vec![
            ("invoice.approve", "invoice", "approve"),
            ("invoice.collect", "invoice", "collect"),
            ("invoice.view", "invoice", "view"),
        ]
    );
    assert!(service
        .permissions
        .iter()
        .all(|permission| !permission.permission_ref.is_empty()));

    let role = analysis
        .workspace
        .files
        .iter()
        .flat_map(|file| file.ast.items.iter())
        .find(|artifact| {
            artifact.kind == archol_core::ArtifactKind::Role && artifact.name == "Clerk"
        })
        .expect("role");
    let permissions = property(&role.body, "permissions")
        .expect("role permissions")
        .1;
    let Expr::List { elements, .. } = permissions else {
        panic!("permissions must be a List");
    };
    assert!(elements.iter().all(
        |element| matches!(element, Expr::Reference { ref_kind, .. } if ref_kind == "permission")
    ));
}
