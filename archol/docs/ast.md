# AST

This is the **internal per-file syntax tree** used by the parser, checker, LSP
features and formatter. Every node has `kind` (the node name) and `loc`
(`start`/`end` in bytes). It is never emitted as a public AST.

Every emitted AST is the resolved application tree, whose root is always
`ArcholApplication`: see [`application-ast.md`](application-ast.md) and its JSON
Schema [`application-ast.schema.json`](application-ast.schema.json).

```json
{
  "kind": "SourceFile",
  "path": "billing-shapes.archol",
  "items": [
    {
      "kind": "ArtifactShape",
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

## Property order

JSON, YAML, the LSP AST response and the AST explorer share one order file:
[`archol/ast-order.json`](../ast-order.json). The first keys keep a semantic
order (`kind`, `title`, `name`, `value`, `message`, `id`), keys ending in `loc`
go last, and every other key is alphabetical. `archol_core::order::order_value`
applies it recursively before the CLI prints JSON/YAML or the LSP sends the AST;
the extension reads the same JSON file for its tree and JSON view.

## Nodes

| `kind` | Main fields |
| :--- | :--- |
| `SourceFile` | `path`, `items` |
| `Artifact<Kind>` | one `kind` per artifact: `ArtifactApplication`, `ArtifactService`, `ArtifactDocument`, `ArtifactProcess`, `ArtifactView`, `ArtifactForm`, `ArtifactMessage`, `ArtifactShape`, `ArtifactRole`, `ArtifactOrganogram`, `ArtifactPolicy`, `ArtifactDecision`, `ArtifactIntegration`, `ArtifactParameter`, `ArtifactSecret`, `ArtifactIcon`, `ArtifactManual`, `ArtifactPermission`, `ArtifactError`; fields `name`, `params`, `body`, `id?`, `roles?`, `artifacts?`, `permissions?` |
| `ArtifactPointer` | service index row: `artifact` (typed kind), `name`, `ref`, `path`, `line` |
| `ArtifactPermission` | auto-created permission: `name`, `namespace`, `action`, `ref`, `path`, `line`, `loc` |
| `PermissionPointer` | role permission: `name`, `ref`, `refKind: "permission"`, `line` |
| `Hierarchy` | `chains` (`hierarchy { A > B }`) |
| `HierarchyChain` | `roles` (ordered superior → subordinate) |
| `RoleRef` | `name`, `ref?`, `refKind: "role"`, `service?`, `path?`, `line?`, `loc` |
| `OrganogramRole` | derived `ArtifactOrganogram.roles`: `name`, `ref`, `refKind`, `service?`, `path`, `line`, `title?`, `permissions` (`PermissionPointer[]`), `inherits`, `manages` |
| `RolePointer` | `name`, `ref`, `refKind`, `service?`, `path`, `line` |
| `PropertyAssignment` | `name`, `value`, `typeRef?` (`std.Duration`, ...) |
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
`NullLiteral`, `List`, `Object` (`typeRef`, `members`), `ShapeValue`, `ShapeLiteral`,
`Call` (callee + args), `NamedArgument`, `MemberAccess`, `Binary`, `Unary`,
`Raw`.

## Artifact kinds

`kind` is the artifact type itself: `ArtifactDocument`, `ArtifactRole`,
`ArtifactOrganogram`, and so on. There is no separate `artifactKind` field.
Consumers switch on `kind`; the artifact name is always `name`.

## Hierarchy

`hierarchy { FinanceManager > CollectionAgent > Clerk }` becomes a `Hierarchy`
member with one `HierarchyChain` per line. The chain preserves the written
order; the builder derives the direct edges `A → B`, `B → C` from consecutive
roles. After the semantic pass (`--app`), each `RoleRef` carries the role's lock
`ref`, plus `service` and `path` so the builder reaches the role without a second
lookup.

```json
{
  "kind": "Hierarchy",
  "chains": [
    {
      "kind": "HierarchyChain",
      "roles": [
        { "kind": "RoleRef", "name": "FinanceManager", "ref": "d", "refKind": "role",
          "service": "example.com/services/billing", "path": "billing-organograms.archol", "line": 27,
          "loc": { "start": 73, "end": 87 } },
        { "kind": "RoleRef", "name": "CollectionAgent", "ref": "aj", "refKind": "role", "line": 18, "loc": { "start": 90, "end": 105 } },
        { "kind": "RoleRef", "name": "Clerk", "ref": "s", "refKind": "role", "line": 10, "loc": { "start": 108, "end": 113 } }
      ],
      "loc": { "start": 73, "end": 113 }
    }
  ],
  "loc": { "start": 57, "end": 117 }
}
```

## Organogram roles (derived)

With an application (`--app`), each `ArtifactOrganogram` gains a `roles` array
projected from the hierarchy and the `role` artifacts of the same service.
`manages` is the transitive closure of the hierarchy (direct and indirect
subordinates); `inherits` and `permissions` come from the role artifact;
`title` keeps the resolved `ShapeValue`. `ArtifactRole` artifacts stay at the
top level of `items`; the projection is not a second source of truth.

```json
{
  "kind": "ArtifactOrganogram",
  "name": "Finance",
  "roles": [
    {
      "kind": "OrganogramRole",
      "name": "FinanceManager",
      "ref": "d",
      "refKind": "role",
      "service": "example.com/services/billing",
      "path": "billing-organograms.archol",
      "line": 27,
      "title": { "kind": "ShapeValue", "typeRef": "std.bpcms.i18n", "value": { "pt-BR": "Gerente financeiro" } },
      "permissions": ["invoice.approve"],
      "inherits": [],
      "manages": [
        { "kind": "RolePointer", "name": "CollectionAgent", "ref": "aj", "refKind": "role", "path": "billing-organograms.archol", "line": 18 },
        { "kind": "RolePointer", "name": "Clerk", "ref": "s", "refKind": "role", "path": "billing-organograms.archol", "line": 10 }
      ]
    }
  ]
}
```

The hierarchy is validated in the check: `E_UNKNOWN_ROLE`, `E_HIERARCHY_CYCLE`,
`E_HIERARCHY_DUPLICATE`, plus parser diagnostics `E_HIERARCHY_ROLE`,
`E_HIERARCHY_CHAIN` and `E_HIERARCHY_TRAILING`.

## Service artifact index and permissions

After the semantic pass (`--app`), `ArtifactService` carries a typed index of
every artifact declared in its files (documents, processes, views, forms,
messages, shapes, roles, organograms, policies, decisions, integrations,
parameters, secrets, icons) plus the permissions auto-created from its roles.
The service never lists itself.

```json
{
  "kind": "ArtifactService",
  "name": "example.com/services/billing",
  "artifacts": [
    { "kind": "ArtifactPointer", "artifact": "ArtifactOrganogram", "name": "Finance", "ref": "x", "path": "billing-organograms.archol", "line": 1 },
    { "kind": "ArtifactPointer", "artifact": "ArtifactDocument", "name": "Invoice", "ref": "a", "path": "billing-invoice-document.archol", "line": 3 },
    { "kind": "ArtifactPointer", "artifact": "ArtifactPermission", "name": "invoice.view", "ref": "oy", "path": "billing-organograms.archol", "line": 10 }
  ],
  "permissions": [
    { "kind": "ArtifactPermission", "name": "invoice.view", "namespace": "invoice", "action": "view", "ref": "oy", "path": "billing-organograms.archol", "line": 10,
      "loc": { "start": 203, "end": 215 } }
  ]
}
```

A permission is **not** a source declaration: the compiler creates one
`ArtifactPermission` per unique `namespace.action` used by the service's roles,
assigns it a lock id, and rewrites every role `permissions` entry from
`MemberAccess`/`Identifier` to a `Reference` with `refKind: "permission"` and
`ref`. The permission name keeps the dotted source spelling; `namespace` is all
but the last segment and `action` is the last one. `OrganogramRole.permissions`
uses `PermissionPointer` with the same `ref` and the usage line.

## std.bpcms.i18n (I18N)

`title`, `goal`, `instructions`, `short` and `tooltip` become a dedicated
`ShapeValue` token with `typeRef: "std.bpcms.i18n"`. Its `value` is a plain
locale map: a string shorthand is expanded to the service's first locale, and
locales unsupported by the application are removed. The `PropertyAssignment`
does not carry the I18N type.

```json
{
  "kind": "PropertyAssignment",
  "name": "goal",
  "value": {
    "kind": "ShapeValue",
    "typeRef": "std.bpcms.i18n",
    "value": { "pt-BR": "Cobrar faturas vencidas" }
  }
}
```

`short`/`tooltip` are nested `I18N` entries and are not locales. The internal
syntax tree keeps values as written; locale resolution needs an application.

Other semantic types still use `PropertyAssignment.typeRef`: `sla` is
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

`SourceFile.pendings` is an array of typed pending nodes for comments beginning with
`TODO`, `FIXME`, `REVIEW`, `OPTIMIZE` or `HACK` (case-insensitive). A `Date`
value is the due date. Pendings are **metadata, not artifacts**: each keyword
has its own kind (`PendingTodo`, `PendingFixme`, `PendingReview`,
`PendingOptimize`, `PendingHack`) and each comment gets a `p:<base36>` id from
a counter separate from artifact ids. Those ids are written to the application
lock as `p:<base36>,pending:<file>:<start>` lines; the resolved application AST
lists them under `pendings`.

```json
{
  "kind": "PendingReview",
  "id": "p:a",
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
