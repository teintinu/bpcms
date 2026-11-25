# Application AST (`ArcholApplication`)

**Status:** draft contract, `schemaVersion: 1`. Emitted by
`archol application`; the contract may still evolve. The authoritative schema is
[`application-ast.schema.json`](application-ast.schema.json).

## 1. Purpose

The internal `SourceFile` tree answers "what is written in this file". The
application AST answers "what does this application mean": one closed tree with
the whole closure, every definition resolved and every reference linked by id.
It is the only emitted AST; its root is always `ArcholApplication`.

- A definition appears **once**, inside the `artifact` map.
- Every reference is a typed node with the target id and the use `loc`.
- `reintroduce` is already applied; the tree is the final state.
- Builders consume this tree; they do not resolve names, merge reintroductions
  or invert permissions.

## 2. Root

```jsonc
{
  "kind": "ArcholApplication",
  "schemaVersion": 1,

  "uri": "example.com/app/app",
  "path": "examples/applications/example.com/app/app/app-application.archol",
  "workspaceRoot": "/Users/acme/bpcms/examples",

  "title": {
    "kind": "ShapeValue",
    "typeRef": "std.bpcms.i18n",
    "value": { "pt-BR": "br title", "en-US": "en titl" }
  },
  "theme": { "kind": "ThemeRef", "id": "Nuvula" },
  "locales": [
    { "kind": "LocaleRef", "tag": "pt-BR", "default": true },
    { "kind": "LocaleRef", "tag": "en-US", "default": false }
  ],
  "environments": [
    { "kind": "EnvironmentEntry", "name": "dev" }
  ],

  "artifact": {
    "s1": { "kind": "ServiceDef", ... },
    "a2": { "kind": "RoleDef", ... },
    "a3": { "kind": "PermissionDef", ... },
    "a9": { "kind": "DocumentDef", ... }
  },

  "main": {
    "kind": "MainEntry",
    "parameters": [],
    "secrets": [],
    "start": [],
    "statements": []
  },
  "files": [],
  "pendings": [],
  "index": {
    "byKind": {
      "ServiceDef": ["s1"],
      "RoleDef": ["a2"],
      "PermissionDef": ["a3"],
      "DocumentDef": ["a9"]
    },
    "byService": { "s2": ["a2", "a3"] }
  }
}
```

`uri` is the application URL from the DSL. `path` is the application file,
relative to `workspaceRoot`. `workspaceRoot` is the only absolute path in the
document; `files[].path` is relative to it and `loc.file` is a file id from
`files`.

`title`, `theme`, `locales` and `environments` are application composition, not
artifacts. `theme` and `locales` reference builder assets; tokens and locale
formats do not enter the tree.

## 3. The `artifact` map

`artifact` is a JSON object keyed by the artifact id:

```jsonc
"artifact": {
  "s1": { "kind": "ServiceDef", "id": "s1", ... },
  "a2": { "kind": "RoleDef", "id": "a2", ... }
}
```

Rules:

- **`key == def.id`.** The compiler enforces it; the schema cannot compare a
  key with a value, so a consumer must validate this invariant.
- Every definition carries `kind`, `id`, `serviceId` (except `ServiceDef`),
  `name`, `loc` and (when it has members) `body`.
- `index.byKind` maps each `...Def` kind to its ids; `index.byService` maps each
  service id to the ids it owns. These are derived indexes for the builder; the
  map is the source of truth.
- Only `ServiceDef` carries a `url`. Other definitions derive their URL as
  `<service.url>/<name>`; references never carry URLs.
- There is **no** `provenance` field. The tree is the merged, final state. The
  origin of a reintroduced member is still visible through its `loc` and
  `files[]` (section 9).

## 4. Definitions

### 4.1 Common fields

```jsonc
{
  "kind": "RoleDef",
  "id": "a2",
  "serviceId": "s2",
  "name": "Customer",
  "loc": { "file": "f:a", "start": 610, "end": 780, "line": 24, "column": 1 },
  "body": [ ... ]
}
```

`body` is the merged, annotated per-file AST (`Member`/`Expr`) and keeps clauses
that do not have a typed projection. Its `Reference` nodes already carry lock
ids, but it is intentionally lenient and is **not** the typed reference
contract; builders should prefer the projections. `--no-loc` removes `loc` from
the whole tree.

### 4.2 `ServiceDef`

```jsonc
{
  "kind": "ServiceDef",
  "id": "s1",
  "name": "service-name",
  "url": "example.com/services/service-name",
  "path": "services/example.com/services/service-name/service-name-service.archol",
  "loc": { "file": "f:a", "start": 0, "end": 180, "line": 1, "column": 1 },
  "title": { "kind": "ShapeValue", "typeRef": "std.bpcms.i18n", "value": {} },
  "locales": ["pt-BR", "en-US"],
  "entryPoint": true,
  "aliases": [
    {
      "kind": "ServiceAlias",
      "name": "serviceAlias",
      "use": "entryPoint",
      "by": null,
      "path": "examples/applications/example.com/app/app/app-application.archol",
      "loc": { "file": "f:a", "start": 0, "end": 0 }
    }
  ],
  "imports": [
    { "kind": "ServiceRef", "id": "s2", "loc": { ... } }
  ],
  "body": []
}
```

- `entryPoint`: the application's `services` block lists this service.
- `aliases`: every use of the URL. `by: null` is the application; otherwise
  `by` is the importing service id. `use` is `entryPoint` or `import`.
- `imports`: services this service imports. The closure is `entryPoint` +
  transitive `imports`.
- `path`: the service root file; `loc.file` is its `f:<base36>` file id.

`bpcms.org/std` is a reserved builder pseudo-service. It is not a
`ServiceDef`, has no lock id and no endpoint. Built-in types use `StdRef`
(section 5.3).

### 4.3 `RoleDef` and `PermissionDef`

```jsonc
"a2": {
  "kind": "RoleDef",
  "id": "a2",
  "serviceId": "s2",
  "name": "Customer",
  "loc": { ... },
  "title": { ... },
  "organogram": { "kind": "OrganogramRef", "id": "o1", "loc": { ... } },
  "inherits": [
    { "kind": "RoleRef", "id": "a5", "loc": { ... } }
  ],
  "permissions": [
    { "kind": "PermissionRef", "id": "a3", "loc": { ... } }
  ],
  "scopes": ["Store", "FiscalYear"],
  "capabilities": ["identity"],
  "body": [ ... ]
}
```

- `permissions` is the **effective** set: own plus inherited. There is no
  separate `declaredPermissions`; the graph is in `inherits`.
- `PermissionDef.roles` is the inverse effective set. Both are generated
  consistently.
- A role selected by the application's `roles` block is listed in
  `application.roles`; there is no `appRole` boolean.

```jsonc
"a3": {
  "kind": "PermissionDef",
  "id": "a3",
  "serviceId": "s3",
  "name": "invoice.view",
  "namespace": "invoice",
  "action": "view",
  "roles": [
    { "kind": "RoleRef", "id": "a2", "loc": { ... } }
  ],
  "usedBy": [
    { "kind": "OrganogramRef", "id": "o1", "loc": { ... } }
  ],
  "loc": { ... }
}
```

`usedBy` lists non-role sites that require the permission (`fourEyes`,
`segregation`). Role grants are already in `roles`.

### 4.4 `OrganogramDef` and `Hierarchy`

```jsonc
"o1": {
  "kind": "OrganogramDef",
  "id": "o1",
  "serviceId": "s2",
  "name": "Platform",
  "loc": { ... },
  "title": { ... },
  "scopes": ["Store"],
  "hierarchy": {
    "kind": "Hierarchy",
    "chains": [
      [
        { "kind": "RoleRef", "id": "a2", "loc": { ... } },
        { "kind": "RoleRef", "id": "a5", "loc": { ... } }
      ]
    ]
  },
  "roles": [
    {
      "kind": "OrganogramRoleDef",
      "role": { "kind": "RoleRef", "id": "a2", "loc": { ... } },
      "manages": [],
      "inherits": [],
      "permissions": []
    }
  ],
  "fourEyes": [],
  "segregation": [],
  "delegations": [],
  "body": []
}
```

`Hierarchy` is a projection, not an artifact: it has no id. `chains` preserve
the written order (superior -> subordinate). `OrganogramDef.roles` is the
builder projection: `manages` is the transitive closure; `permissions` is
effective.

### 4.5 `DocumentDef`

```jsonc
"a9": {
  "kind": "DocumentDef",
  "id": "a9",
  "serviceId": "s3",
  "name": "Invoice",
  "loc": { ... },
  "title": { ... },
  "fields": [
    {
      "kind": "FieldDef",
      "name": "total",
      "type": { "kind": "ShapeRef", "id": "a11", "loc": { ... } },
      "required": true,
      "private": false,
      "loc": { ... }
    }
  ],
  "lifecycle": {
    "kind": "LifecycleDef",
    "shape": { "kind": "ShapeRef", "id": "a12", "loc": { ... } },
    "initial": { "kind": "ShapeValueRef", "shapeId": "a12", "valuePath": "Draft", "loc": { ... } },
    "creationEvent": "Requested",
    "states": [
      { "kind": "ShapeValueRef", "shapeId": "a12", "valuePath": "Draft", "loc": { ... } }
    ]
  },
  "actions": [
    {
      "kind": "ActionDef",
      "name": "issue",
      "roles": [ { "kind": "RoleRef", "id": "a2", "loc": { ... } } ],
      "to": { "kind": "ShapeValueRef", "shapeId": "a12", "valuePath": "Issued", "loc": { ... } },
      "form": null,
      "limits": [],
      "body": [ ... ]
    }
  ],
  "indexes": [],
  "relations": [],
  "events": [],
  "body": [ ... ]
}
```

- A document has no `permissions` block. Authority is in `actions[].roles`,
  field `read`/`edit`/`mask`, `limits`, `segregation` and `signature`.
- `FieldDef` is a member definition: it has `name` but no `fieldPath` and no
  `documentId`; it lives in the document's `fields`.
- `FieldDef.type` is a `ShapeRef`, a `DocumentRef` (fields may reference a
  document), a `StdRef` (`std.Money`) or a `PrimitiveRef`
  (`string`/`number`/`boolean`/`stream`).
- `lifecycle` is the document's view of the state shape. Transitions live on
  `ShapeValueDef` (section 4.6).
- `creationEvent` is the event name of the `initial ... with transition`
  clause, when present.

### 4.6 `ShapeDef`

```jsonc
"a12": {
  "kind": "ShapeDef",
  "id": "a12",
  "serviceId": "s3",
  "name": "InvoiceStatus",
  "loc": { ... },
  "shapeKind": "state",
  "base": null,
  "unit": null,
  "values": [
    {
      "kind": "ShapeValueDef",
      "name": "Draft",
      "ordinal": 10,
      "title": { ... },
      "icon": null,
      "transitions": [
        {
          "kind": "TransitionDef",
          "name": "Issued",
          "to": { "kind": "ShapeValueRef", "shapeId": "a12", "valuePath": "Issued", "loc": { ... } },
          "event": "Invoice.Issued",
          "title": { ... },
          "payload": []
        }
      ],
      "loc": { ... }
    }
  ],
  "formula": null,
  "conversions": [],
  "methods": [],
  "body": [ ... ]
}
```

- `values` is ordered after `insert before/after` is applied.
- A one-character `icon` is emitted as `IconLiteral`; a named icon stays `IconRef`.
- `transitions` live on the source value; `name` is the event name; `to` is the
  destination value. There is no transition artifact and no `TransitionRef`.
- `ShapeValueDef` has `name`; references use `valuePath: "Draft"`.

### 4.7 `ProcessDef`

```jsonc
"a17": {
  "kind": "ProcessDef",
  "id": "a17",
  "serviceId": "s3",
  "name": "Checkout",
  "loc": { ... },
  "title": { ... },
  "start": { "kind": "StartSpec", "trigger": null, "schedule": null },
  "roles": [],
  "variables": [
    { "kind": "VariableDef", "name": "value1", "type": { "kind": "PrimitiveRef", "name": "number" }, "loc": { ... } }
  ],
  "lanes": [
    {
      "kind": "LaneDef",
      "name": "Operator",
      "roles": [ { "kind": "RoleRef", "id": "a2", "loc": { ... } } ],
      "steps": [ ... ],
      "catches": [
        {
          "kind": "CatchDef",
          "variable": "error",
          "commands": [ { "kind": "AllowCommand", "action": "retry", "loc": { ... } } ],
          "steps": [ ... ],
          "loc": { ... }
        }
      ],
      "loc": { ... }
    }
  ],
  "steps": [
    {
      "kind": "StepDef",
      "name": "AuthorizePayment",
      "lane": "System",
      "action": { "kind": "ActionRef", "documentId": "a9", "actionPath": "pay", "loc": { ... } },
      "roles": [],
      "form": null,
      "event": null,
      "loc": { ... }
    }
  ],
  "flow": {
    "kind": "FlowDef",
    "nodes": ["AuthorizePayment", "FraudCheck"],
    "edges": [
      { "kind": "FlowEdge", "from": "FraudCheck", "to": "AuthorizePayment" }
    ]
  },
  "ends": [],
  "messages": [],
  "decisions": [],
  "sla": null,
  "body": [ ... ]
}
```

- `steps` is in effective order; `insert` is not exposed.
- `flow.nodes` and `flow.edges` use step names relative to `steps`, not
  `steps.AuthorizePayment`.
- `StepDef` has `name`; references use `stepPath: "AuthorizePayment"`.
- `variables` mirrors the process `vars` block and `lanes` mirrors `lane` blocks; `steps` is flattened in lane order.
- New step syntax adds `stepKind` (`form`, `compute`, `report`), `title`, `commands` (`EditCommand`, `ShowCommand`, `SleepCommand`, `AssignCommand`) and `guard` (a typed expression).
- A guard like `operation === Sum` becomes `Binary` with `VariableRef` on the left and `ShapeValueRef` on the right. `VariableRef` carries `name` and `processId`; variables are sub-artifacts and do not get lock ids yet.
- A lane `catch (error)` becomes `CatchDef` with `allow` commands and its own steps; the current step is the process status.
- `ActionRef` points to a document action (`documentId` + `actionPath`).

### 4.8 `ViewDef`, `FormDef`, `MessageDef`

```jsonc
"v1": {
  "kind": "ViewDef",
  "id": "v1",
  "serviceId": "s3",
  "name": "InvoiceTable",
  "loc": { ... },
  "viewKind": "table",
  "binds": [ { "kind": "DocumentRef", "id": "a9", "loc": { ... } } ],
  "commands": [
    { "kind": "ActionRef", "documentId": "a9", "actionPath": "issue", "loc": { ... } }
  ],
  "filters": [],
  "body": [ ... ]
}
```

```jsonc
"f1": {
  "kind": "FormDef",
  "id": "f1",
  "serviceId": "s3",
  "name": "OrderReview",
  "loc": { ... },
  "document": { "kind": "DocumentRef", "id": "a9", "loc": { ... } },
  "roles": [ { "kind": "RoleRef", "id": "a2", "loc": { ... } } ],
  "fields": [
    { "kind": "FieldDef", "name": "note", "type": { "kind": "PrimitiveRef", "name": "string" }, "loc": { ... } }
  ],
  "actions": [
    {
      "kind": "FormActionDef",
      "name": "approve",
      "calls": { "kind": "ActionRef", "documentId": "a9", "actionPath": "approve", "loc": { ... } },
      "stay": false
    }
  ],
  "body": [ ... ]
}
```

```jsonc
"m1": {
  "kind": "MessageDef",
  "id": "m1",
  "serviceId": "s3",
  "name": "PickupRequest",
  "loc": { ... },
  "about": { "kind": "DocumentRef", "id": "a9", "loc": { ... } },
  "payload": [
    { "kind": "FieldDef", "name": "requestedAt", "type": { "kind": "StdRef", "name": "std.DateTime" }, "loc": { ... } }
  ],
  "body": [ ... ]
}
```

Form-only fields are local `FieldDef`s; they are not `FieldRef`s, because a
`FieldRef` always points to a document field. Message payload fields are local
definitions too. If a future construct needs to reference a form/message field
by id, a new member ref kind is added.

### 4.9 `PolicyDef`, `DecisionDef`, `IntegrationDef`

These keep their full declaration in `body` and project the references the
builder needs:

- `PolicyDef`: `scope`, `rules`, `refs` (typed refs).
- `DecisionDef`: `inputs`, `outputs`, `rules`, `table`.
- `IntegrationDef`: `integrationKind`, `endpoint`, `secrets` (`SecretRef[]`),
  `parameters` (`ParameterRef[]`), `operations`.

### 4.10 `ParameterDef`, `SecretDef`, `IconDef`, `ManualDef`, `ErrorDef`

```jsonc
"p1": {
  "kind": "ParameterDef",
  "id": "p1",
  "serviceId": "s3",
  "name": "DefaultCurrency",
  "loc": { ... },
  "title": { ... },
  "type": { "kind": "ShapeRef", "id": "a20", "loc": { ... } },
  "required": false,
  "default": { "kind": "StringLiteral", "value": "BRL", "raw": "\"BRL\"" },
  "min": null,
  "max": null,
  "maxLength": null,
  "value": { "kind": "StringLiteral", "value": "BRL", "raw": "\"BRL\"" },
  "body": [ ... ]
}
```

```jsonc
"x1": {
  "kind": "SecretDef",
  "id": "x1",
  "serviceId": "s3",
  "name": "PaymentKey",
  "loc": { ... },
  "title": { ... },
  "type": { "kind": "PrimitiveRef", "name": "string" },
  "required": true,
  "body": [ ... ]
}
```

- `ParameterDef.value` is the effective value assigned in `main`; `default` is
  the declaration. `main.parameters` keeps the assignment site.
- `SecretDef` never has a value and never has `default`.
- `IconDef`: `namespace`, optional `path` (SVG file), `hash`, optional
  `content`. Icons declared in `.archol` have no `path`; file-based SVG icons
  are not loaded by the CLI yet.
- `ManualDef`: `title`, `steps`, `refs`.
- `ErrorDef`: `title`, `code`, `retryable`, `refs`.

## 5. References

### 4.9 `StepArtifactDef`

```jsonc
"k1": {
  "kind": "StepArtifactDef",
  "id": "k1",
  "serviceId": "s3",
  "name": "SumJob",
  "stepKind": "compute",
  "title": { ... },
  "guard": null,
  "commands": [
    { "kind": "AssignCommand", "target": "result", "value": { ... }, "loc": { ... } }
  ],
  "loc": { ... }
}
```

- Step artifacts are reusable by processes of the same service; a process lane step carries a `StepArtifactRef` to the definition.
- `stepKind` is `form`, `compute` or `report`; commands use process variables by name.
- The flow guard lives in the **lane step call** (`SumJob if operation === Sum`), not in the reusable Step artifact. The instance in `ProcessDef.steps` carries `guard` and a `StepArtifactRef`.

### 5.1 Typed artifact references

Every artifact reference is a node with its own kind and the **target id**:

```jsonc
{ "kind": "RoleRef", "id": "a2", "loc": { ... } }
{ "kind": "ServiceRef", "id": "s1", "loc": { ... } }
{ "kind": "DocumentRef", "id": "a9", "loc": { ... } }
```

`id` is the target's id, never a new id for the use. `loc` is the use site. A
reference has no `body`, no `name` and no URL; resolve it through `artifact[id]`.

The complete set: `ServiceRef`, `RoleRef`, `PermissionRef`, `OrganogramRef`,
`DocumentRef`, `ShapeRef`, `ViewRef`, `FormRef`, `ProcessRef`, `MessageRef`,
`PolicyRef`, `DecisionRef`, `IntegrationRef`, `ParameterRef`, `SecretRef`,
`IconRef`, `ManualRef`, `ErrorRef`. Pending comments are metadata, not
references: they live in `pendings` with their own `p:<base36>` ids.

### 5.2 Member references

A member reference names its owner and the member path **relative to the owning
collection**:

```jsonc
{ "kind": "FieldRef", "documentId": "a9", "fieldPath": "total", "loc": { ... } }
{ "kind": "ShapeValueRef", "shapeId": "a12", "valuePath": "Draft", "loc": { ... } }
{ "kind": "StepRef", "processId": "a17", "stepPath": "AuthorizePayment", "loc": { ... } }
{ "kind": "ActionRef", "documentId": "a9", "actionPath": "issue", "loc": { ... } }
```

| Ref | Owner | Path | Wrong |
| :--- | :--- | :--- | :--- |
| `FieldRef` | `documentId` | `total` | `fields.total` |
| `ShapeValueRef` | `shapeId` | `Draft` | `values.Draft` |
| `StepRef` | `processId` | `AuthorizePayment` | `steps.AuthorizePayment` |
| `ActionRef` | `documentId` | `issue` | `actions.issue` |

The prefix is implied by the array (`fields`, `values`, `steps`, `actions`).
Member definitions (`FieldDef`, `ShapeValueDef`, `StepDef`, `ActionDef`) carry
only `name`; they do not repeat the path.

`ActionRef` points to a document action. If a future process-owned action needs
its own ref, add `ProcessActionRef` rather than overloading `ActionRef`.

### 5.3 Built-in types

Local and std shapes are `ShapeRef`s. The four primitive types and the
reserved `std` namespace use dedicated nodes:

```jsonc
{ "kind": "PrimitiveRef", "name": "string" }
{ "kind": "StdRef", "name": "std.Money" }
```

`PrimitiveRef.name` is one of `string`, `number`, `boolean`, `stream`.
`StdRef.name` starts with `std.`. Neither has an id; both are built-in.

## 6. `main`

```jsonc
{
  "kind": "MainEntry",
  "parameters": [
    {
      "kind": "ParameterSet",
      "parameter": { "kind": "ParameterRef", "id": "p1", "loc": { ... } },
      "value": { "kind": "StringLiteral", "value": "BRL", "raw": "\"BRL\"" },
      "loc": { ... }
    }
  ],
  "secrets": [
    {
      "kind": "SecretSet",
      "secret": { "kind": "SecretRef", "id": "x1", "loc": { ... } },
      "name": "GATEWAY_KEY",
      "loc": { ... }
    }
  ],
  "start": [
    {
      "kind": "StartEntry",
      "service": { "kind": "ServiceRef", "id": "s2", "loc": { ... } },
      "loc": { ... }
    }
  ],
  "statements": []
}
```

`SecretSet` carries only the secret name, never a value. `start` does not
declare a dependency; it only boots early. `statements` keeps tolerant members
that are not modeled as `set`/`start`.

## 7. `files`, `pendings`, `index`

```jsonc
"files": [
  { "kind": "FileEntry", "id": "f:a",
    "path": "examples/applications/.../app-application.archol",
    "serviceId": null, "application": true },
  { "kind": "FileEntry", "id": "f:b",
    "path": "services/.../billing-organograms.archol",
    "serviceId": "s3", "application": false }
],
"pendings": [
  { "kind": "PendingReview", "id": "p:a", "dueDate": "2026-12-15",
    "message": "revisar", "loc": { ... } }
],
"index": {
  "byKind": { "RoleDef": ["a2"], "DocumentDef": ["a9"] },
  "byService": { "s3": ["a9", "a11", "a12"] }
}
```

`files` maps every closure file to an `f:<base36>` id and its owner service.
The id is assigned by a counter separate from artifact ids and is what every
`loc.file` points to. The application file has `serviceId: null` and
`application: true`; files of the reserved `bpcms.org/std` pseudo-service also
have `serviceId: null`, because std has no `ServiceDef`. `pendings` covers
TODO/TOFIX/FIXME/REVIEW/OPTIMIZE/HACK comments with one kind per keyword
(`PendingTodo`, `PendingTofix`, `PendingFixme`, `PendingReview`, `PendingOptimize`,
`PendingHack`), a `p:<base36>` id from another separate counter, and a `loc`.
They are metadata, not artifacts and not references. `index` is derived; the
`artifact` map remains the source of truth.

## 8. `reintroduce`

`reintroduce` is applied before the tree is emitted. The AST contains the
merged definition only; there is no second version and no provenance field.

- The artifact identity belongs to the **owner** service: a value added by
  `fraud-checkout` to `Checkout.CheckoutStatus` keeps
  `serviceId: <checkout>` and is referenced by `shapeId` + `valuePath`.
- The added member keeps its own `loc`, which points to the contributing file
  (`fraud-checkout-service.archol`). `files[]` maps that file to the
  contributing service.
- `provenance` was intentionally omitted; if audit is needed, `loc.file` +
  `files[]` identifies the origin.
- `insert X before/after Y` is applied to the ordered collections (`fields`,
  `values`, `steps`, `actions`); the resulting order is final.
- A conflict (`E_REINTRODUCE_CONFLICT`) prevents emission, so the builder never
  sees two values for the same key.

## 9. Invariants

1. **Closed:** every reference id resolves in `artifact`; every member path
   exists in the owner.
2. **Unique:** `artifact` keys are unique and equal to `def.id`; an artifact
   appears once.
3. **Typed:** the ref kind matches the target def kind (`RoleRef` -> `RoleDef`,
   `DocumentRef` -> `DocumentDef`, ...).
4. **Owner-stable:** `serviceId` points to the owning service, even for members
   contributed by `reintroduce`.
5. **Relative paths:** `fieldPath`, `valuePath`, `stepPath`, `actionPath` never
   include the collection prefix.
6. **No secrets:** `SecretDef` has no value/default; `SecretSet` has only the
   name.
7. **No builder assets:** theme tokens, locale formats and environment policies
   are referenced, not embedded.
8. **Deterministic:** same input + same lock -> byte-identical output. Map keys
   are sorted by id; `byKind`/`byService` arrays are sorted; no timestamps.
9. **Opaque ids:** consumers never parse `id`.

## 10. Determinism and size

The full tree (with `body` and `loc`) is the builder contract. `--no-body`
removes every `body` and keeps the typed projections, which is the recommended
mode for large applications; `--no-loc` removes locations for diffs. The shape
and the ids stay the same in both modes. `SourceFile` is an internal parser
structure only.

## 11. Compatibility and migration

- Every CLI/LSP AST output requires an application and starts at
  `ArcholApplication`; there is no public `SourceFile` output.
- The application `roles` block must accept service-qualified references; bare
  names are accepted only when unambiguous.
- `reintroduce` is parsed, checked and merged for the corpus. Process-task
  inserts are not applied yet; the current `fraud-checkout` example only adds a
  shape value and a document action.
- The builder consumes only `ArcholApplication`; it must not re-resolve names,
  re-apply `reintroduce` or recompute permissions/hierarchies.
- Schema changes bump `schemaVersion`; consumers reject unknown major versions.

## 12. Test matrix

The tree is emitted by `archol application [dir] --app <uri|path>`
(`--json`/`--yaml`/`--no-loc`). The golden test lives in
`archol/crates/archol-cli/tests/application_ast.rs` and asserts:

- the tree validates against `application-ast.schema.json`;
- every reference id resolves and the ref kind matches the def kind;
- `artifact` key equals `def.id` for every entry;
- exactly one def per artifact identity;
- `RoleDef.permissions` equals the inherited closure; `PermissionDef.roles` is
  the inverse;
- every `application.roles` entry resolves to a `RoleDef`;
- `files` covers every `loc.file` and every closure file;
- i18n values contain only application locales;
- `SecretDef`/`SecretSet` never contain a value;
- two runs produce byte-identical JSON;

Schema validation is run separately with the JSON Schema (the Rust test focuses
on closure and typed references).

The `loja-do-joao`, `backoffice-loja`, `central-de-compras`, `rota-certa`,
`claims-desk` and `loja-com-antifraude` apps are the initial corpus. The last
one imports `fraud-checkout` and covers the merged `reintroduce` path.

## 13. Worked example

```jsonc
{
  "kind": "ArcholApplication",
  "schemaVersion": 1,
  "uri": "example.com/app/app",
  "path": "examples/applications/example.com/app/app/app-application.archol",
  "workspaceRoot": "examples",
  "title": { "kind": "ShapeValue", "typeRef": "std.bpcms.i18n",
             "value": { "pt-BR": "br title", "en-US": "en titl" } },
  "theme": { "kind": "ThemeRef", "id": "Nuvula" },
  "locales": [
    { "kind": "LocaleRef", "tag": "pt-BR", "default": true },
    { "kind": "LocaleRef", "tag": "en-US", "default": false }
  ],
  "environments": [{ "kind": "EnvironmentEntry", "name": "dev" }],

  "artifact": {
    "s1": {
      "kind": "ServiceDef",
      "id": "s1",
      "name": "service-name",
      "url": "example.com/services/service-name",
      "path": "services/example.com/services/service-name/service-name-service.archol",
      "loc": { "file": "f:a",
               "start": 0, "end": 200, "line": 1, "column": 1 },
      "title": { "kind": "ShapeValue", "typeRef": "std.bpcms.i18n", "value": {} },
      "locales": ["pt-BR", "en-US"],
      "entryPoint": true,
      "aliases": [
        { "kind": "ServiceAlias", "name": "serviceAlias", "use": "entryPoint",
          "by": null,
          "path": "examples/applications/example.com/app/app/app-application.archol",
          "loc": { "file": "f:a", "start": 120, "end": 180 } }
      ],
      "imports": []
    },
    "a2": {
      "kind": "RoleDef",
      "id": "a2",
      "serviceId": "s2",
      "name": "Customer",
      "loc": { "file": "f:a", "start": 0, "end": 0 },
      "title": { "kind": "ShapeValue", "typeRef": "std.bpcms.i18n",
                 "value": { "pt-BR": "Cliente", "en-US": "Customer" } },
      "organogram": null,
      "inherits": [],
      "permissions": [],
      "scopes": [],
      "capabilities": [],
      "body": []
    },
    "a3": {
      "kind": "PermissionDef",
      "id": "a3",
      "serviceId": "s2",
      "name": "order.create",
      "namespace": "order",
      "action": "create",
      "roles": [],
      "usedBy": [],
      "loc": { "file": "f:a", "start": 0, "end": 0 }
    },
    "a9": {
      "kind": "DocumentDef",
      "id": "a9",
      "serviceId": "s2",
      "name": "Order",
      "loc": { "file": "f:a", "start": 0, "end": 0 },
      "title": { "kind": "ShapeValue", "typeRef": "std.bpcms.i18n", "value": {} },
      "fields": [
        { "kind": "FieldDef", "name": "total",
          "type": { "kind": "StdRef", "name": "std.Money" },
          "loc": { "file": "f:a", "start": 0, "end": 0 } }
      ],
      "lifecycle": null,
      "actions": [],
      "indexes": [],
      "relations": [],
      "events": [],
      "body": []
    }
  },

  "main": { "kind": "MainEntry", "parameters": [], "secrets": [], "start": [], "statements": [] },
  "files": [
    { "kind": "FileEntry", "id": "f:a",
      "path": "examples/applications/example.com/app/app/app-application.archol",
      "serviceId": null, "application": true }
  ],
  "pendings": [],
  "index": {
    "byKind": {
      "ServiceDef": ["s1"],
      "RoleDef": ["a2"],
      "PermissionDef": ["a3"],
      "DocumentDef": ["a9"]
    },
    "byService": { "s2": ["a2", "a3", "a9"] }
  }
}
```
