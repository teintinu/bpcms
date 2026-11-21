use archol_core::{parse, ArtifactKind, Expr, Member, Stmt, TokenKind};

fn ast(source: &str) -> archol_core::SourceFile {
    let result = parse(source, "test.archol");
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    result.ast
}

#[test]
fn reads_artifact_with_string_name() {
    let file = ast("service \"example.com/services/demo\" {\n  title: { pt-BR: \"Demo\" }\n  locales: [ \"pt-BR\" ]\n}");
    assert_eq!(file.items.len(), 1);
    assert_eq!(file.items[0].kind, ArtifactKind::Service);
    assert_eq!(file.items[0].artifact_kind(), "Service");
    assert_eq!(file.items[0].name, "example.com/services/demo");
}

#[test]
fn reads_enum_with_title() {
    let file = ast(
        "shape TaxRegimeCode {\n  base: classification\n  value SIMPLES {\n    title: { pt-BR: \"Simples\" }\n  }\n}",
    );
    let body = &file.items[0].body;
    let value = body
        .iter()
        .find_map(|member| match member {
            Member::EnumValueDeclaration { name, body, .. } => Some((name, body)),
            _ => None,
        })
        .expect("enum value");
    assert_eq!(value.0, "SIMPLES");
    assert_eq!(value.1.len(), 1);
}

#[test]
fn reads_process_with_const_and_if() {
    let file = ast(
        "process Check(regime: TaxRegimeCode) {\n  const a: TaxRegimeCode = SIMPLES\n  if (regime == SIMPLES) {\n    return Done\n  }\n}",
    );
    let body = &file.items[0].body;
    assert!(matches!(
        body[0],
        Member::Statement { statement: Stmt::ConstDeclaration { ref name, .. }, .. } if name == "a"
    ));
    let Member::Statement {
        statement: Stmt::IfStatement { condition, .. },
        ..
    } = &body[1]
    else {
        panic!("expected if");
    };
    let Expr::Binary { op, .. } = condition else {
        panic!("expected comparison")
    };
    assert_eq!(op, "==");
}

#[test]
fn reads_list_and_nested_object() {
    let file = ast(
        "document Invoice {\n  fields {\n    items: InvoiceItem[] { title: { pt-BR: \"Itens\" } }\n  }\n  roles: [Clerk, Manager]\n}",
    );
    let members = &file.items[0].body;
    assert!(matches!(members[1], Member::PropertyAssignment { .. }));
}

#[test]
fn ignores_comments_in_ast() {
    let file = ast("// comment\ndocument A {\n  // another\n  title: { pt-BR: \"A\" }\n}");
    assert_eq!(file.items.len(), 1);
}

#[test]
fn error_on_unclosed_block() {
    let result = parse("document A {\n  title: { pt-BR: \"A\" }\n", "x.archol");
    assert!(result
        .diagnostics
        .iter()
        .any(|d| d.code == "E_PARSE_UNCLOSED"));
}

#[test]
fn duration_and_percentage_tokens() {
    let lexed = archol_core::lex("5d 15m 30%");
    let kinds: Vec<TokenKind> = lexed
        .tokens
        .iter()
        .map(|t| t.kind)
        .filter(|k| !k.is_trivia())
        .collect();
    assert_eq!(kinds[0], TokenKind::Number);
    assert_eq!(kinds[2], TokenKind::Number);
    assert_eq!(kinds[3], TokenKind::Percent);
}

#[test]
fn trailing_backslash_in_string_does_not_panic() {
    let source = "document A { note: \"abc\\";
    let lexed = archol_core::lex(source);
    assert!(lexed
        .tokens
        .iter()
        .all(|token| token.span.end <= source.len()));
    assert!(!lexed.diagnostics.is_empty());
}

#[test]
fn deep_nesting_becomes_diagnostic_not_stack_overflow() {
    let mut source = String::from("document D {\n");
    for _ in 0..400 {
        source.push_str("a: { ");
    }
    source.push_str("b: 1");
    for _ in 0..400 {
        source.push_str(" }");
    }
    source.push_str("\n}\n");
    let result = parse(&source, "deep.archol");
    assert!(result
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == "E_PARSE_DEPTH"));
}

#[test]
fn marks_i18n_properties_with_their_type() {
    let file = ast(
        "process P {\n  title: \"Processo\"\n  goal: \"Fazer\"\n  instructions: { pt-BR: \"Instrução\" }\n  sla: 1d\n}",
    );
    let types: Vec<_> = file.items[0]
        .body
        .iter()
        .filter_map(|member| match member {
            Member::PropertyAssignment { name, type_ref, .. } => {
                Some((name.as_str(), type_ref.as_deref()))
            }
            _ => None,
        })
        .collect();
    assert!(types.contains(&("title", None)));
    assert!(types.contains(&("goal", None)));
    assert!(types.contains(&("instructions", None)));
    assert!(types.contains(&("sla", Some("std.Duration"))));
}

#[test]
fn accepts_single_quoted_strings_and_prefers_double_quotes() {
    let result = parse("document A {\n  title: 'Nome'\n}", "x.archol");
    assert!(result
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == "W_STRING_QUOTES"));
    let Member::PropertyAssignment { value, .. } = &result.ast.items[0].body[0] else {
        panic!("title property");
    };
    assert!(matches!(value, Expr::StringLiteral { value, .. } if value == "Nome"));

    let clean = parse("document A {\n  title: \"Nome\"\n}", "x.archol");
    assert!(!clean
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == "W_STRING_QUOTES"));
}

#[test]
fn reads_typed_artifact_kinds_and_hierarchy_chains() {
    let file = ast(
        "organogram Finance {\n  title: { pt-BR: \"Financeiro\" }\n  hierarchy {\n    FinanceManager > CollectionAgent > Clerk\n  }\n}\nrole Clerk { title: { pt-BR: \"Faturista\" } }",
    );
    assert_eq!(file.items[0].kind, ArtifactKind::Organogram);
    assert_eq!(file.items[0].kind.node_kind(), "ArtifactOrganogram");
    assert_eq!(file.items[1].kind, ArtifactKind::Role);
    assert_eq!(file.items[1].kind.node_kind(), "ArtifactRole");
    let Member::Hierarchy { chains, .. } = &file.items[0].body[1] else {
        panic!("expected hierarchy member");
    };
    assert_eq!(chains.len(), 1);
    let names: Vec<_> = chains[0]
        .roles
        .iter()
        .map(|role| role.name.as_str())
        .collect();
    assert_eq!(names, vec!["FinanceManager", "CollectionAgent", "Clerk"]);
    assert!(chains[0].roles.iter().all(|role| role.role_ref.is_none()));
}

#[test]
fn hierarchy_rejects_single_role_lines() {
    let result = parse(
        "organogram O {\n  hierarchy {\n    Clerk\n  }\n}",
        "x.archol",
    );
    assert!(
        result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "E_HIERARCHY_CHAIN"),
        "{:?}",
        result.diagnostics
    );
}
