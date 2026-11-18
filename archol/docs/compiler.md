# Compilador

Pipeline: **lexer → parser → check → link**.

```text
fonte .archol
  │
  ├── archol_core::lex            tokens + diagnósticos léxicos
  ├── archol_core::parse          SourceFile (AST JSON)
  ├── archol_analysis::Symbols    artefatos, enums e papéis do workspace
  ├── archol_analysis::check      E_DUP_ARTIFACT, E_UNKNOWN_TYPE, E_ENUM_VALUE, ...
  └── archol_analysis::build_links  role → artefatos, enum → valores, refs
```

## Check

O check é mais do que inferência de tipo:

- `const a: TaxRegimeCode = SIMPLES` confere se `SIMPLES` está entre os `value`
  declarados no shape `TaxRegimeCode` (`E_ENUM_VALUE`).
- O enum é resolvido **no serviço do arquivo**, com `Alias.Tipo` via
  `imports { service "..." as Alias }`; enums homônimos em serviços diferentes
  não se confundem (o corpus tem três `PaymentMethod`).
- `if (regime == SIMPLES)` resolve o tipo de `regime` (parâmetro do processo,
  `const` ou `let`) e valida o valor comparado.
- Tipos PascalCase desconhecidos em campos (`x: Missing { ... }`) geram
  `E_UNKNOWN_TYPE`.
- `roles: [Fantasma]` gera `E_UNKNOWN_ROLE` se o papel não existe.
- `about`, `document`, `source`, `sources`, `appliesTo` e `fanOut` precisam
  apontar para artefatos existentes (`E_UNKNOWN_REF`).
- Parâmetros de processo também têm o tipo validado; arquivo ilegível vira
  `E_FILE_READ` (não é descartado em silêncio).
- O mesmo nome de artefato, no mesmo serviço e tipo, é `E_DUP_ARTIFACT`.

Diagnósticos trazem código estável, mensagem e `span`. A CLI imprime
`arquivo:linha:coluna: severidade código: mensagem`.

## Link

O link indexa o que o construtor precisa e que o fonte não responde pronto:

```json
{
  "roles": [
    { "name": "Clerk",
      "usedBy": [
        { "artifact": "Invoice", "artifactKind": "Document", "usage": "roles",
          "path": "billing-invoice-document.archol", "line": 120 }
      ] }
  ],
  "enums": [
    { "name": "TaxRegimeCode", "service": "example.com/services/billing",
      "values": [{ "name": "SIMPLES", "title": "Simples Nacional" }] }
  ],
  "artifacts": [
    { "name": "Invoice", "kind": "Document", "path": "...", "span": [0, 0],
      "refs": [{ "kind": "about", "name": "Invoice" }] }
  ]
}
```

Uso: `archol links ../examples > links.json`.

## Regras de projeto

- Sem dependências além de `serde` (JSON do AST) — fácil de portar.
- Profundidade máxima de aninhamento (200) gera `E_PARSE_DEPTH` em vez de
  estourar a pilha; spans nunca passam do fim do fonte.
- Sem async: o LSP usa um laço síncrono com JSON-RPC manual.
- Um arquivo de teste por camada (testes inline no lexer, `tests/parser.rs`,
  `crates/archol-analysis/tests/analysis.rs`, `tests/corpus.rs`,
  `crates/archol-cli/tests/cli.rs`).
- O corpus real (`../examples`, 330 arquivos) precisa continuar com **zero**
  erros de parse; o teste `corpus` falha se um arquivo novo quebrar.
