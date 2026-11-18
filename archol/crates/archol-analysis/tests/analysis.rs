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
fn const_com_valor_de_enum_invalido() {
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
fn comparacao_com_valor_de_enum_invalido() {
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
fn comparacao_valida_nao_gera_erro() {
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
fn role_desconhecida() {
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
fn link_inverte_role_para_artefatos() {
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
        .expect("role no link");
    assert!(clerk.used_by.iter().any(|usage| usage.artifact == "V"));
}

#[test]
fn link_expoe_valores_de_enum() {
    let analysis = workspace(&base_files());
    let regime = analysis
        .links
        .enums
        .iter()
        .find(|e| e.name == "TaxRegimeCode")
        .expect("enum no link");
    let values: Vec<_> = regime.values.iter().map(|v| v.name.as_str()).collect();
    assert_eq!(values, vec!["SIMPLES", "LUCRO_REAL"]);
}

#[test]
fn duplicidade_no_mesmo_servico() {
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
fn enum_de_outro_servico_com_alias() {
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
fn enum_homonimo_nao_gera_falso_positivo() {
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
fn literal_na_esquerda_aponta_para_o_literal() {
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
        .expect("diagnóstico");
    let expected = source.find("INVALIDO").unwrap();
    assert_eq!(diagnostic.diagnostic.span.start, expected);
}
