# Testes

```bash
cd archol
cargo test                 # tudo
cargo test -p archol-core  # lexer + parser
cargo test -p archol-analysis
node --test editors/vscode/test
```

## O que é coberto

| Teste | Garante |
| :--- | :--- |
| `archol-core/src/lexer.rs` (testes inline) | strings, comentários, números com sufixo, operadores de 3 caracteres, escape no EOF |
| `archol-core/tests/parser.rs` | artefatos, enum, const/if, listas, tolerância, erro de bloco não fechado, `E_PARSE_DEPTH` |
| `archol-analysis/tests/analysis.rs` | `E_ENUM_VALUE` em const e comparação (inclusive `Alias.Tipo`), enum homônimo por serviço, `E_UNKNOWN_ROLE`, `E_DUP_ARTIFACT`, link role→artefato |
| `archol-core/tests/corpus.rs` | os 330 exemplos reais parseiam sem erro |
| `archol-cli/tests/cli.rs` | exit codes, `E_FILE_READ` para UTF-8 inválido, `tokens` com `\` final, aninhamento profundo |
| `editors/vscode/test` | framing JSON-RPC e árvore do AST |

## Estado

Hoje: **30 testes Rust** (`cargo test`) e **27 testes JavaScript**
(`node --test editors/vscode/test`); `clippy` e `fmt --check` limpos.

## Corpus

`tests/corpus.rs` percorre `../../../examples` a partir da crate e falha se algum
`.archol` produzir diagnóstico de parse. É o teste de aceitação da sintaxe:
qualquer construção nova da linguagem entra primeiro em um exemplo real e depois
no parser, nunca o contrário.

## Fixtures

Fixtures pequenos e intencionalmente errados ficam em strings nos testes (não
poluem `examples/`). Para testar o LSP manualmente, rode:

```bash
cargo run -q --bin archol -- check ../examples
python3 scripts/smoke_lsp.py
```
