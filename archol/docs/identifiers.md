# Identifiers, references and the application lock

Archol gives every named declaration a **stable id**. References are typed
nodes that point to those ids instead of embedding a copy of the target. This
keeps the AST small, acyclic and usable by builders that generate other
languages.

## The lock file: `bpcms.lock`

One lock per application, stored next to the application file. It is generated
by `archol lock --app <application>` and versioned with the source.

- **JSON Lines**: one identifier per line, sorted by `path`, so merges are
  line-based.
- **Delete the file to reset the ids.** A new file assigns new ids.
- Entries for identifiers that no longer exist are removed by the next
  `archol lock` run.
- Duplicate ids caused by concurrent branches are a merge responsibility; the
  lock loader rejects duplicates and tells you to fix the merge.

```json
{"id":"Xq3m9_Kd2Aa","path":"example.com/services/billing#service","kind":"service","name":"billing"}
{"id":"Tz8n1Rf4Pq0","path":"example.com/services/billing#shape/InvoiceStatus/value/Overdue","kind":"value","name":"Overdue"}
{"id":"Lm5p2Wd7Yt1","path":"example.com/services/billing#document/Invoice/field/status","kind":"field","name":"status"}
```

### Id rules

Ids are base64url strings (no padding) with `-` mapped to `_`, so they are
valid identifiers in most target languages:

- first character is a letter or `_`, **never a digit**;
- remaining characters are ASCII letters, digits or `_`;
- at least 12 characters, random per lock;
- the builder may use an id directly as a name in the generated language.

### Safe paths

`path` locates the identifier without exposing a filesystem path:

| Identifier | Path |
| :--- | :--- |
| Application | `<app-uri>#application` |
| Service | `<service-uri>#service` |
| Artifact | `<service-uri>#<kind>/<Name>` (`document`, `shape`, `view`, `process`, ...) |
| Document field | `<service-uri>#document/<Document>/field/<field>` |
| Enum value | `<service-uri>#shape/<Shape>/value/<Value>` |
| Process parameter | `<service-uri>#process/<Process>/param/<param>` |

## AST annotations

Declarations carry `id`; references carry `ref` (`refKind` names the target
kind). Optional fields are omitted when absent.

```json
{
  "kind": "Binary",
  "op": "==",
  "left": {
    "kind": "MemberAccess",
    "name": "status",
    "fieldRef": "Lm5p2Wd7Yt1",
    "object": { "kind": "Identifier", "name": "invoice" }
  },
  "right": {
    "kind": "Reference",
    "refKind": "enumValue",
    "name": "Overdue",
    "ref": "Tz8n1Rf4Pq0"
  }
}
```

| Field | Meaning |
| :--- | :--- |
| `id` | declaration id from `bpcms.lock` |
| `ref` | target declaration id (references) |
| `refKind` | target kind (`enumValue`, `document`, `role`, ...) |
| `fieldRef` | id of the document field addressed by a member access |
| `typeRefId` | id of the type named by a type reference |

From that `if`, a builder can walk `Overdue` → enum value id → shape
`InvoiceStatus` → service `example.com/services/billing`, and back through
`invoice.status` → field `status`.

## Reference index

Usage lists live in the semantic index (`archol/usages` in the LSP), not inside
the AST. For every id:

- declaration location (`archol/declaration`);
- every usage: file, span and kind (`enumValue`, `fieldRef`, `artifactRef`,
  `roleRef`, `typeRef`, ...).

The AST explorer exposes **Go to Declaration** and **Find Usages** for these
ids.

## Application closure

Compilation starts from an application. `archol check`, `archol lock`,
`archol links` and the LSP require an application and only load its closure:
entry-point services plus services reachable through `imports`.

Service aliases and imports are references too: an application `services { "<uri>" as Alias }`
entry and a service `imports { service "<uri>" as Alias }` entry carry the `ref` of the
service declaration, so the explorer can jump from `Identity` to the service file.

Files outside the closure are inert. The LSP treats them as **commented out**:
no hover, completion, definition, references or symbols, a single information
diagnostic, and `archol/ast` returns `commentedOut: true`.

The VS Code setting `archol.application` selects the current application; the
status bar shows it and **Archol: Select Application** changes it.
