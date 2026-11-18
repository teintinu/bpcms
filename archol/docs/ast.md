# AST

O AST é serializável em JSON, no estilo do AST do TypeScript: todo nó tem
`kind` (nome do nó) e `span` (`start`/`end` em bytes). É a estrutura que o
explorador de AST, o LSP e o construtor consomem.

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

## Nós

| `kind` | Campos principais |
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
`IfStatement`, `TryStatement`, `ReturnStatement`, `RawStatement` (tolerância
para código não modelado).

Expressões: `Identifier`, `StringLiteral`, `NumberLiteral`, `BooleanLiteral`,
`NullLiteral`, `List`, `Object` (`typeRef`, `members`), `Call` (callee + args),
`NamedArgument`, `MemberAccess`, `Binary`, `Unary`, `Raw`.

## Compatibilidade com TypeScript

A semântica de nós segue o TS: `PropertyAssignment` corresponde a
`PropertyAssignment`, `Call` a `CallExpression`, `IfStatement` a
`IfStatement`, `Object` a `ObjectLiteralExpression` (com `typeRef` quando há
`Store { ... }`), `EnumValueDeclaration` a `EnumMember`. Campos em camelCase e
spans em bytes mantêm a exportação trivial para TypeScript, Python, Go ou Dart.

## Tolerância

O parser sempre devolve AST, mesmo com código que não modela (ex.: métodos de
`std`). Trechos não reconhecidos viram `RawStatement` com `span`; o check só
acusa o que entende. Isso mantém o explore e o LSP úteis em arquivos grandes
sem transformar o parser em um compilador de TypeScript.
