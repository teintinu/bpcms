# Compiler

Pipeline: **application → lexer → parser (+ pending comments) → check → link → resolve references → assign used ids**.

```text
application (.archol)
  │
  ├── archol_analysis::application  entry services + transitive imports (closure)
  ├── archol_core::lex              tokens + lexical diagnostics
  ├── archol_core::parse            SourceFile (JSON AST)
  ├── archol_analysis::i18n         I18N type + application locale filtering
  ├── archol_analysis::Symbols      closure artifacts, enums, and roles
  ├── archol_analysis::check        E_DUP_ARTIFACT, E_UNKNOWN_TYPE, E_ENUM_VALUE, ...
  ├── archol_analysis::build_links  role → artifacts, enum → values, refs
  └── archol_analysis::annotate     reference paths, then ids for used paths
```

**There is no compilation without an application.** `check`, `lock`, `links`
and the LSP require one; without `--app` the CLI lists the applications and
exits 2. Files outside the closure are inert (the LSP treats them as commented
out).

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
- Every `service` must declare `locales` with at least one locale; otherwise
  `E_LOCALES_REQUIRED`.
- `title`, `goal`, `instructions`, `short` and `tooltip` are marked
  `typeRef: "I18N"`; string shorthand is expanded to the service's first locale and
  locale map entries not supported by the application are dropped.
- `sla` is marked `typeRef: "std.Duration"`; `sla: "30d"` becomes a
  `ShapeLiteral` with the normalized value and base unit.
- Any string assigned to a Shape type is parsed by the shape's declarative
  `parse`/`validate`/`normalize` (`E_SHAPE_LITERAL` on failure); unquoted
  Duration text is `E_SHAPE_LITERAL_QUOTED`. This covers `sla`, typed
  `let`/`const`, parameter `default`/`min`/`max`, and field defaults.
  Quantities are normalized to the shape's base unit; `std.Money` becomes
  `{ raw, currency }`.
- Single-quoted strings are accepted; `W_STRING_QUOTES` asks for double quotes
  and `archol format` converts them when possible.

Pending comments (`TODO`, `FIXME`, `REVIEW`, `OPTIMIZE`, `HACK`) are parsed into
`SourceFile.pending`. A missing or past due date is an error
(`E_PENDING_NO_DATE`/`E_PENDING_EXPIRED`); a due date within 15 days is a
warning (`W_PENDING_SOON`); malformed dates are `E_PENDING_DATE`.

Diagnostics carry a stable code, message, and `span`. The CLI prints
`file:line:column: severity code: message`.

## Format and lint

`archol format [dir] [--check]` uppercases pending comment keywords and reports
files that would change; `archol lint` is the same diagnostics pass as
`archol check`. The LSP exposes `textDocument/formatting` (format-on-save by
default for Archol) and publishes the lint diagnostics continuously.

## Identifiers and references

`archol lock --app <app>` writes `<app>.lock` next to the application file
(same basename, extension changed): CSV `id,path`, one identifier per line.
Ids are assigned only for used paths, sequentially `a`, `b`, …, `z`, `aa`, ….
A `*` at the end of a path pins the entry (top of file, never pruned or
renumbered). References in the AST
use those ids (`ref`, `fieldRef`, `typeRefId`), including application/import
service aliases, so the JSON is acyclic and small. See [`identifiers.md`](identifiers.md).

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
