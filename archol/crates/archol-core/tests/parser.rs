use archol_core::{parse, Expr, Member, Stmt, TokenKind};

fn ast(source: &str) -> archol_core::SourceFile {
    let result = parse(source, "teste.archol");
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    result.ast
}

#[test]
fn le_artefato_com_nome_string() {
    let file = ast("service \"example.com/services/demo\" {\n  title: { pt-BR: \"Demo\" }\n}");
    assert_eq!(file.items.len(), 1);
    assert_eq!(file.items[0].artifact_kind, "Service");
    assert_eq!(file.items[0].name, "example.com/services/demo");
}

#[test]
fn le_enum_com_titulo() {
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
        .expect("valor de enum");
    assert_eq!(value.0, "SIMPLES");
    assert_eq!(value.1.len(), 1);
}

#[test]
fn le_processo_com_const_e_if() {
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
        panic!("esperado if");
    };
    let Expr::Binary { op, .. } = condition else {
        panic!("esperado comparação")
    };
    assert_eq!(op, "==");
}

#[test]
fn le_lista_e_objeto_aninhado() {
    let file = ast(
        "document Invoice {\n  fields {\n    items: InvoiceItem[] { title: { pt-BR: \"Itens\" } }\n  }\n  roles: [Clerk, Manager]\n}",
    );
    let members = &file.items[0].body;
    assert!(matches!(members[1], Member::PropertyAssignment { .. }));
}

#[test]
fn ignora_comentarios_no_ast() {
    let file = ast("// comentário\ndocument A {\n  // outro\n  title: { pt-BR: \"A\" }\n}");
    assert_eq!(file.items.len(), 1);
}

#[test]
fn erro_em_bloco_nao_fechado() {
    let result = parse("document A {\n  title: { pt-BR: \"A\" }\n", "x.archol");
    assert!(result
        .diagnostics
        .iter()
        .any(|d| d.code == "E_PARSE_UNCLOSED"));
}

#[test]
fn tokens_de_duracao_e_percentual() {
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
fn barra_invertida_no_fim_da_string_nao_panica() {
    let source = "document A { note: \"abc\\";
    let lexed = archol_core::lex(source);
    assert!(lexed
        .tokens
        .iter()
        .all(|token| token.span.end <= source.len()));
    assert!(!lexed.diagnostics.is_empty());
}

#[test]
fn aninhamento_profundo_vira_diagnostico_e_nao_stack_overflow() {
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
