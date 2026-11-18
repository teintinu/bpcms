# Tokens

O léxico fica em `crates/archol-core/src/token/`:

| Arquivo | Conteúdo |
| :--- | :--- |
| `mod.rs` | `TokenKind`, `Token` (kind + span) |
| `keyword.rs` | palavras reservadas, `is_artifact()`, `artifact_name()` |
| `../lexer.rs` | scanner manual, comentários, strings, números, operadores |
| `lexer.rs` (testes inline) | testes de cada família de token |

## Grupos

| Grupo | Tokens |
| :--- | :--- |
| identificador | `Ident` (ASCII e Unicode; aceita `µg`, `Å`) |
| literais | `String`, `Number` (com sufixo `5d`, `15m`, `2026y`), `Comment` |
| pontuação | `{ } ( ) [ ] , : ; . @ ?` |
| operadores | `= == === != !== < > <= >= + - * / % ! && \|\| & \| -> => ??` |
| palavras-chave | `application service document process view form message shape role organogram policy decision integration parameter secret icon manual error value transition end if else try catch return const let on by schedule fanOut called start imports as where in true false null` |
| especiais | `Eof`, `Error` (caractere inesperado, string não terminada) |

`Comment` é token de verdade, mas o parser o ignora; a CLI `tokens` o mostra.

## Regras ao adicionar um token

1. Declare o `TokenKind` em `token/mod.rs` (ou a palavra em `token/keyword.rs`).
2. Registre no lexer (tabela de operadores do mais longo para o mais curto).
3. Adicione um teste inline em `crates/archol-core/src/lexer.rs` cobrindo o texto
   e o `span`.
4. Se a palavra abrir uma construção nova, atualize o parser, o AST e
   `docs/ast.md`.
5. Rode `cargo test -p archol-core`.

## Por que um token por família e não um arquivo por token

A linguagem tem ~60 tokens; um diretório por token produziria arquivos de 30
linhas e navegação pior. O compromisso é: **uma família por arquivo** e **um
teste por comportamento**, com a tabela de palavras-chave como fonte única.
