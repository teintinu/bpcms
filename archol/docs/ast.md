# AST

The AST is serializable to JSON, in the style of the TypeScript AST: every node
has `kind` (the node name) and `span` (`start`/`end` in bytes). It is the
structure the AST explorer, the LSP, and the builder consume.

```json
{
  "kind": "SourceFile",
  "path": "billing-shapes.archol",
  "items": [
    {
      "kind": "Artifact",
      "artifactKind": "Shape",
      "name": "TaxRegimeCode",
      "params": [],
      "span": { "start": 60, "end": 640 },
      "body": [
        { "kind": "PropertyAssignment", "name": "base",
          "value": { "kind": "Identifier", "name": "classification", "span": {"start": 170, "end": 184} },
          "span": { "start": 164, "end": 184 } },
        { "kind": "EnumValueDeclaration", "name": "SIMPLES",
          "body": [ { "kind": "PropertyAssignment", "name": "title", "value": {...} } ],
          "span": { "start": 190, "end": 320 } }
      ]
    }
  ]
}
```

## Nodes

| `kind` | Main fields |
| :--- | :--- |
| `SourceFile` | `path`, `items` |
| `Artifact` | `artifactKind`, `name`, `params`, `body` |
| `PropertyAssignment` | `name`, `value` |
| `Block` | `keyword?`, `name`, `body` (`fields`, `actions`, `on ...`, `end ...`) |
| `EnumValueDeclaration` | `name`, `body` |
| `ShorthandProperty` | `name` (`{ fiscalPeriod }`, `required`) |
| `ImportDeclaration` | `uri`, `alias` |
| `TriggerDeclaration` | `keyword`, `targets` |
| `Statement` | `statement` |

Statements: `ExpressionStatement`, `ConstDeclaration`, `LetDeclaration`,
`IfStatement`, `TryStatement`, `ReturnStatement`, `RawStatement` (tolerance for
code that is not modeled).

Expressions: `Identifier`, `StringLiteral`, `NumberLiteral`, `BooleanLiteral`,
`NullLiteral`, `List`, `Object` (`typeRef`, `members`), `Call` (callee + args),
`NamedArgument`, `MemberAccess`, `Binary`, `Unary`, `Raw`.

## Compatibility with TypeScript

Node semantics follow TS: `PropertyAssignment` corresponds to
`PropertyAssignment`, `Call` to `CallExpression`, `IfStatement` to
`IfStatement`, `Object` to `ObjectLiteralExpression` (with `typeRef` when there
is a `Store { ... }`), `EnumValueDeclaration` to `EnumMember`. camelCase fields
and byte spans keep export to TypeScript, Python, Go, or Dart trivial.

## Tolerance

The parser always returns an AST, even for code it does not model (e.g., `std`
methods). Unrecognized sections become `RawStatement` with a `span`; the check
only reports what it understands. This keeps the explorer and the LSP useful in
large files without turning the parser into a TypeScript compiler.
