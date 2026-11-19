# AST

The AST is serializable to JSON, in the style of the TypeScript AST: every node
has `kind` (the node name) and `loc` (`start`/`end` in bytes). It is the
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
      "loc": { "start": 60, "end": 640 },
      "body": [
        { "kind": "PropertyAssignment", "name": "base",
          "value": { "kind": "Identifier", "name": "classification", "loc": {"start": 170, "end": 184} },
          "loc": { "start": 164, "end": 184 } },
        { "kind": "EnumValueDeclaration", "name": "SIMPLES",
          "body": [ { "kind": "PropertyAssignment", "name": "title", "value": {...} } ],
          "loc": { "start": 190, "end": 320 } }
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
| `PropertyAssignment` | `name`, `value`, `typeRef?` (`I18N`, `std.Duration`, ...) |
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
`NullLiteral`, `List`, `Object` (`typeRef`, `members`), `ShapeLiteral`,
`Call` (callee + args), `NamedArgument`, `MemberAccess`, `Binary`, `Unary`,
`Raw`.

## I18N

`title`, `goal`, `instructions`, `short` and `tooltip` carry `"typeRef": "I18N"` on the `PropertyAssignment`.
The value is always a locale map in the compiled AST: a string shorthand is
expanded to the service's first locale, and locales unsupported by the
application are removed. The locale object itself does not repeat the type.

```json
{
  "kind": "PropertyAssignment",
  "name": "goal",
  "typeRef": "I18N",
  "value": {
    "kind": "Object",
    "members": [
      { "kind": "PropertyAssignment", "name": "pt-BR",
        "value": { "kind": "StringLiteral", "value": "Cobrar faturas vencidas" } }
    ]
  }
}
```

`short`/`tooltip` are nested `I18N` entries and are not locales. The syntax
AST (`archol ast` without `--app`) keeps values as written; locale resolution
needs an application.

Other semantic types use the same `PropertyAssignment.typeRef`: `sla` is
`std.Duration` (`"typeRef": "std.Duration"`).

## Shape literals

A string assigned to a Shape type is parsed at compile time and becomes a
`ShapeLiteral`. The node carries the shape reference, the raw text, the parsed
value and, for quantities, the base unit:

```json
{
  "kind": "ShapeLiteral",
  "shape": "std.Duration",
  "raw": "30d",
  "value": 2592000,
  "unit": "s",
  "loc": { "start": 0, "end": 0 }
}
```

`value` is shape-dependent: a number for quantities, text for media shapes, or
an object when the shape's parse returns fields. Strings accept single or
double quotes; `archol format` prefers double quotes.

## Pending comments

`SourceFile.pending` is an array of `Pending` nodes for comments beginning with
`TODO`, `FIXME`, `REVIEW`, `OPTIMIZE` or `HACK` (case-insensitive). A `Date`
value is the due date. Pendings are **metadata, not artifacts**: they have no
`id` and never appear in the application lock. The AST explorer merges them
into the tree at their source position.

```json
{
  "kind": "Pending",
  "keyword": "REVIEW",
  "dueDate": "2026-12-15",
  "dateText": "2026-12-15",
  "message": "revisar depois do carnaval",
  "loc": { "start": 0, "end": 63 },
  "keywordLoc": { "start": 3, "end": 9 }
}
```

## Identifier ids and references

Declarations may carry `id`; identifier references may carry `ref` plus
`refKind`; member access may carry `fieldRef`; type references may carry
`typeRefId`. All values come from the application lock (`<app>.lock`). See
[`identifiers.md`](identifiers.md) for the lock format and traversal examples.

## Compatibility with TypeScript

Node semantics follow TS: `PropertyAssignment` corresponds to
`PropertyAssignment`, `Call` to `CallExpression`, `IfStatement` to
`IfStatement`, `Object` to `ObjectLiteralExpression` (with `typeRef` when there
is a `Store { ... }`), `EnumValueDeclaration` to `EnumMember`. camelCase fields
and byte locations keep export to TypeScript, Python, Go, or Dart trivial.

## Tolerance

The parser always returns an AST, even for code it does not model (e.g., `std`
methods). Unrecognized sections become `RawStatement` with a `loc`; the check
only reports what it understands. This keeps the explorer and the LSP useful in
large files without turning the parser into a TypeScript compiler.
