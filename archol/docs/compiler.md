# Compiler

Pipeline: **lexer → parser → check → link**.

```text
.archol source
  │
  ├── archol_core::lex            tokens + lexical diagnostics
  ├── archol_core::parse          SourceFile (JSON AST)
  ├── archol_analysis::Symbols    workspace artifacts, enums, and roles
  ├── archol_analysis::check      E_DUP_ARTIFACT, E_UNKNOWN_TYPE, E_ENUM_VALUE, ...
  └── archol_analysis::build_links  role → artifacts, enum → values, refs
```

## Check

The check is more than type inference:

- `const a: TaxRegimeCode = SIMPLES` verifies that `SIMPLES` is among the
  `value`s declared in the `TaxRegimeCode` shape (`E_ENUM_VALUE`).
- The enum is resolved **in the file's service**, with `Alias.Type` via
  `imports { service "..." as Alias }`; same-named enums in different services
  are not confused (the corpus has three `PaymentMethod`s).
- `if (regime == SIMPLES)` resolves the type of `regime` (a process parameter,
  `const`, or `let`) and validates the compared value.
- Unknown PascalCase types in fields (`x: Missing { ... }`) produce
  `E_UNKNOWN_TYPE`.
- `roles: [Ghost]` produces `E_UNKNOWN_ROLE` if the role does not exist.
- `about`, `document`, `source`, `sources`, `appliesTo`, and `fanOut` must point
  to existing artifacts (`E_UNKNOWN_REF`).
- Process parameters also have their type validated; an unreadable file becomes
  `E_FILE_READ` (it is not silently discarded).
- The same artifact name, in the same service and type, is `E_DUP_ARTIFACT`.

Diagnostics carry a stable code, message, and `span`. The CLI prints
`file:line:column: severity code: message`.

## Link

The link indexes what the builder needs and the source does not answer
ready-made:

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

Usage: `archol links ../examples > links.json`.

## Design rules

- No dependencies beyond `serde` (AST JSON) — easy to port.
- Maximum nesting depth (200) produces `E_PARSE_DEPTH` instead of overflowing
  the stack; spans never extend past the end of the source.
- No async: the LSP uses a synchronous loop with manual JSON-RPC.
- One test file per layer (inline tests in the lexer, `tests/parser.rs`,
  `crates/archol-analysis/tests/analysis.rs`, `tests/corpus.rs`,
  `crates/archol-cli/tests/cli.rs`).
- The real corpus (`../examples`, 330 files) must continue with **zero** parse
  errors; the `corpus` test fails if a new file breaks it.
