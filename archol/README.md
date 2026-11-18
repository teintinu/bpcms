# Archol

**Archol** é a linguagem de modelagem do BPCMS (antes chamada apenas de "DSL").
Todo fonte Archol usa a extensão **`.archol`**; o repositório mantém os exemplos
em [`../examples/`](../examples) e a especificação em
[`../DSL_REFERENCE.md`](../DSL_REFERENCE.md).

Este diretório contém o compilador, o servidor LSP e o plugin de editor.

## O que tem aqui

| Caminho | Conteúdo |
| :--- | :--- |
| `crates/archol-core` | lexer, tokens, AST JSON e parser |
| `crates/archol-analysis` | check semântico (enum, tipos, papéis) e link invertido |
| `crates/archol-lsp` | servidor Language Server Protocol sobre stdio |
| `crates/archol-cli` | binário `archol` (`check`, `ast`, `tokens`, `links`, `lsp`) |
| `editors/vscode` | extensão VSCode (cliente LSP + explorador de AST), sem dependências npm |
| `docs` | arquitetura, tokens, AST, LSP e testes |
| `scripts` | `smoke_lsp.py` (initialize, hover, completion, AST, links) |
| `examples` | fixtures pequenos usados nos testes e demonstrações |

Nada é publicado nem versionado: a linguagem e o compilador estão na **versão
inicial**, evoluindo em conjunto. Não há tags, releases ou compatibilidade a
preservar.

## Requisitos

- Rust estável (`rustup`, `cargo`) — o projeto não usa crates além de `serde`.
- Node.js (apenas para testar o plugin; a extensão não tem dependências).
- Windows, macOS ou Linux. Todos os caminhos são tratados com `/` internamente.

## Compilar e testar

```bash
cd archol
cargo build              # debug
cargo build --release    # binário final em target/release/archol
cargo test               # testes de lexer, parser, check, link e corpus
```

O `check` sobre os exemplos reais deve terminar sem erros:

```bash
cargo run -q --bin archol -- check ../examples
cargo run -q --bin archol -- links ../examples > links.json
cargo run -q --bin archol -- ast ../examples/services/example.com/services/billing/billing-shapes.archol
```

## Binário `archol`

```text
archol check [dir] [--json]    valida o workspace
archol ast <arquivo> [--json]  imprime o AST
archol tokens <arquivo>        imprime os tokens (depuração do lexer)
archol links [dir] [--json]    índice invertido para o construtor
archol lsp                     servidor LSP em stdio
```

`check` sai com código 1 quando há erro e imprime `arquivo:linha:coluna:
severidade código: mensagem`. `links` gera, entre outras coisas, quais
artefatos exigem cada `role`.

## Editor

Instalação em um comando:

```bash
bash archol/scripts/install-vscode.sh
```

O script compila o servidor e instala a extensão em
`~/.vscode/extensions/archol.archol-vscode-0.1.0`. Depois é só **Developer:
Reload Window** e abrir um `.archol` (com a pasta do repositório como workspace).
A extensão sobe o binário `archol lsp` automaticamente (ou o caminho configurado
em `archol.server.path`) e oferece diagnósticos, hover, completion, símbolos,
definição, referências, dobramento e o **explorador de AST**. Detalhes em
[`editors/vscode/README.md`](editors/vscode/README.md).

## Cobertura desta versão inicial

Já funciona: léxico completo do corpus, parser tolerante (0 erros nos 330
arquivos) com limite de profundidade, AST JSON, check de valores de enum em
`const`/`let` e comparações, enums resolvidos por serviço com `Alias.Tipo`,
tipos/roles/referências desconhecidos, duplicidade, `E_FILE_READ`, link
invertido, LSP com hover/completion/definição (inclusive valor de enum)/
referências/símbolos/folding e plugin VSCode com explorador de AST.

Ainda não implementado (documentado na especificação, fora desta rodada): regras
de caminho de arquivo `E_FILE_*` no check, `semanticTokens`, refatoração/rename,
code actions e validação profunda de todos os blocos de processo. O parser
preserva esses trechos como `RawStatement`, então nada quebra.

Robustez verificada por testes: string com `\` final, UTF-8 inválido,
aninhamento profundo, arquivo vazio, CRLF e Unicode não derrubam a CLI
(`archol-cli/tests/cli.rs`).

## Princípios de código

- Arquivos pequenos e responsabilidade única; tokens organizados em
  `crates/archol-core/src/token/` e testes por família.
- Sem macro pesada, sem async, sem runtime: um parser recursivo direto.
- O AST é JSON (estilo TypeScript): todo nó tem `kind` e `span`.
- O check valida **valores**, não só tipos: `const a: TaxRegimeCode = SIMPLES`
  e `regime == SIMPLES` são conferidos contra os `value` do shape.
- O link produz informação que o fonte não tem pronta: `role -> artefatos`,
  `enum -> valores`, `artefato -> referências`.
