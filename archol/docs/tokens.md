# Tokens

The lexer lives in `crates/archol-core/src/token/`:

| File | Contents |
| :--- | :--- |
| `mod.rs` | `TokenKind`, `Token` (kind + span) |
| `keyword.rs` | reserved words, `is_artifact()`, `artifact_name()` |
| `../lexer.rs` | manual scanner, comments, strings, numbers, operators |
| `lexer.rs` (inline tests) | tests for each token family |

## Groups

| Group | Tokens |
| :--- | :--- |
| identifier | `Ident` (ASCII and Unicode; accepts `µg`, `Å`) |
| literals | `String`, `Number` (with suffix `5d`, `15m`, `2026y`), `Comment` |
| punctuation | `{ } ( ) [ ] , : ; . @ ?` |
| operators | `= == === != !== < > <= >= + - * / % ! && \|\| & \| -> => ??` |
| keywords | `application service document process view form message shape role organogram policy decision integration parameter secret icon manual error value transition end if else try catch return const let on by schedule fanOut called start imports as where in true false null` |
| special | `Eof`, `Error` (unexpected character, unterminated string) |

`Comment` is a real token, but the parser ignores it; the `tokens` CLI shows it.

## Rules for adding a token

1. Declare the `TokenKind` in `token/mod.rs` (or the word in `token/keyword.rs`).
2. Register it in the lexer (operator table from longest to shortest).
3. Add an inline test in `crates/archol-core/src/lexer.rs` covering the text
   and the `span`.
4. If the word opens a new construct, update the parser, the AST, and
   `docs/ast.md`.
5. Run `cargo test -p archol-core`.

## Why one token family per file and not one file per token

The language has ~60 tokens; a directory per token would produce 30-line files
and worse navigation. The tradeoff is: **one family per file** and **one test
per behavior**, with the keyword table as the single source of truth.
