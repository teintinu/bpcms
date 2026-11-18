# LSP

Servidor próprio, síncrono, sobre stdio, em `crates/archol-lsp`:

| Arquivo | Papel |
| :--- | :--- |
| `rpc.rs` | frames `Content-Length` + JSON-RPC |
| `server.rs` | estado dos documentos, análise, handlers |
| `features.rs` | consultas no AST (hover, símbolos, contexto) |
| `util.rs` | URI ↔ caminho e posição LSP (UTF-16) |

## Capabilities

- `textDocumentSync` full (didOpen/didChange/didSave/didClose)
- `hoverProvider` — tipo, artefato, valor de enum (com títulos) e link de AST
- `completionProvider` — valores de enum no contexto, papéis em `roles: [`, artefatos
- `definitionProvider` — vai para a declaração do artefato
- `referencesProvider` / `documentHighlightProvider` — ocorrências do identificador
- `documentSymbolProvider` — artefatos e membros (hierárquico)
- `workspaceSymbolProvider` — busca de artefatos por nome
- `foldingRangeProvider` — artefatos, blocos e statements

## Hover

`TaxRegimeCode` mostra a enumeração com os títulos de cada valor; `SIMPLES`
mostra que é valor de `TaxRegimeCode`; `Invoice` mostra o tipo de artefato e o
serviço. Todo hover termina com:

```md
[Explorar AST](command:archol.showAst?%5B%22file%3A...%22%2C12%2C4%5D)
```

O plugin abre o painel de AST e seleciona o nó da posição.

## Definition

Ordem de resolução (arquivo `crates/archol-lsp/src/resolve.rs`):

1. declaração do parâmetro/`const`/`let`;
2. `icon: X` → artefato `Icon` chamado `X`;
3. valor de enum em comparação tipada — `invoice.status == Overdue` resolve
   `status` como campo de `Invoice`, cujo tipo é `InvoiceStatus`, e leva ao
   `value Overdue` (não ao ícone `Overdue`);
4. `const a: Regime = SIMPLES` → `value SIMPLES`;
5. acesso a campo — `invoice.status` → campo `status` no documento `Invoice`;
6. variável/artefato/valor homônimo, respeitando o serviço.

## Completion

Em `const a: TaxRegimeCode = `, a extensão lista `SIMPLES`,
`LUCRO_PRESUMIDO`, `LUCRO_REAL` com o título de cada um. Em
`if (regime == ` a lista sai do tipo de `regime`. Em `roles: [` saem os papéis
declarados no workspace. O enum é resolvido no serviço do arquivo e aceita
`Alias.Tipo` vindo de `imports`; enums homônimos de serviços diferentes não se
misturam.

## Requests customizados

| Método | Params | Resultado |
| :--- | :--- | :--- |
| `archol/ast` | `{ "uri": "file:///..." }` | `{ "ast": <SourceFile> }` |
| `archol/links` | `{}` | `{ "links": <LinkIndex> }` |

## Diagnósticos

Publicados em `textDocument/publishDiagnostics` a cada open/change/save, com
`source: "archol"` e o mesmo código da CLI (`E_ENUM_VALUE`, `E_UNKNOWN_TYPE`,
...).

## Smoke test

```bash
python3 archol/scripts/smoke_lsp.py
```

Sobe o binário real, faz `initialize`, hover com link de AST, completion de
enum, `archol/ast`, `archol/links` e encerra o servidor.

## Limitações conhecidas

- Análise do workspace inteiro a cada mudança; suficiente para 330 arquivos,
  otimizável com cache por arquivo quando necessário.
- Colunas em UTF-16 (padrão LSP); offsets internos são bytes.
- Sem `semanticTokens` nesta versão inicial: a gramática TextMate cobre cores.
- Sync é Full (`change: 1`); mudanças incrementais não são aplicadas.
