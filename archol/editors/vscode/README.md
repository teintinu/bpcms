# Archol para VS Code

Extensão oficial da linguagem **Archol** (`.archol`): realce de sintaxe, snippets,
cliente LSP escrito em JavaScript puro e um explorador de AST em webview.

A extensão **não tem dependências npm** e usa apenas a API do VS Code e módulos
nativos do Node.js (`child_process`, `crypto`, `events`, `path`). Funciona no
Windows, macOS e Linux com VS Code **1.80 ou superior**.

## Instalação

### Desenvolvimento (F5)

1. Abra a pasta `archol/editors/vscode` no VS Code.
2. Pressione `F5` para abrir o **Extension Development Host**.
3. Abra um arquivo `.archol` do workspace.

### Instalação em um comando (recomendada)

```bash
bash archol/scripts/install-vscode.sh
```

O script compila o servidor (`cargo build --release`) e instala a extensão em
`~/.vscode/extensions/archol.archol-vscode-0.1.0` por symlink (editar o plugin
passa a valer após *Reload Window*). Depois:

1. **Developer: Reload Window**;
2. abra a pasta do repositório (o binário é achado em
   `archol/target/release/archol`);
3. abra um `.archol` e confira a barra de status (`Archol: pronto`).

Para VS Code Insiders: `VSCODE_EXTENSIONS_DIR=~/.vscode-insiders/extensions bash archol/scripts/install-vscode.sh`.
Desinstalar: `rm ~/.vscode/extensions/archol.archol-vscode-0.1.0`.

### VSIX

A extensão não precisa de build. Se quiser empacotar:

```bash
npx @vscode/vsce package
```

Depois instale o `.vsix` em **Extensões → … → Install from VSIX…**.
Também é possível copiar/simbolizar esta pasta em `~/.vscode/extensions/`.

### Servidor LSP

O servidor é um executável separado, compilado a partir do código Rust do
repositório:

```bash
cd archol
cargo build --release
```

O binário fica em `archol/target/release/archol` (ou `archol.exe` no Windows).
A extensão o inicia como `archol lsp`; ajuste `archol.server.args` se necessário.

## Configuração

| Configuração | Padrão | Descrição |
| :--- | :--- | :--- |
| `archol.server.path` | `archol` | Caminho do executável do servidor LSP. |
| `archol.server.args` | `["lsp"]` | Argumentos repassados ao executável (o binário `archol` usa o subcomando `lsp`). Use `[]` para um servidor dedicado. |

Ordem de resolução:

1. o valor configurado (absoluto, relativo ao workspace ou nome no `PATH`);
2. `archol/target/release/archol` relativo à raiz do workspace;
3. `archol/target/debug/archol` relativo à raiz do workspace.

Se nenhum candidato existir, a extensão mostra uma mensagem com as tentativas e
atalhos para abrir a configuração e o canal **Archol LSP**. Depois de alterar o
caminho, execute **Archol: Reiniciar servidor LSP**.

## Recursos

- Realce de sintaxe TextMate: comentários `//`, strings com escapes, números com
  sufixos (`5d`, `15m`, `1h`, `10%`), palavras-chave, tipos PascalCase, chamadas
  `Nome.metodo(`, constantes (`SIMPLES`) e dobramento por chaves `{ }`.
- Autocomplete de pares, indentação e comentários de linha.
- Snippets: documento, processo, shape `base: classification` e view de tabela.
- Diagnósticos do servidor com severidade e código, fonte `archol`.
- Hover, completion, definição, símbolos de documento e referências via LSP.
  `Cmd+Click` (macOS) ou `Ctrl+Click` (Windows/Linux) e `F12` usam a mesma
  definição: campo de documento, variável, enum ou artefato.
- Hover com link **Abrir AST** (`command:archol.showAst?...`) que abre o
  explorador já destacando o nó da posição.
- Explorador de AST: árvore navegável, filtro por kind/nome, cores por categoria,
  clique para posicionar o cursor, **Ir para código** e **Copiar JSON**.
  O clique reaproveita o editor já aberto e **seleciona o trecho inteiro**
  (`span.start`..`span.end`), sem abrir uma segunda aba.
- Sincronização automática do explorador após edições (debounce de 400 ms).
- Canal **Archol LSP** com tráfego resumido (método, URI, posição, contagem de
  diagnósticos). A AST nunca é registrada no canal.
- Barra de status: `parado`, `iniciando`, `pronto` ou `erro`; clique reinicia.

## Comandos e atalhos

| Comando | Paleta | Atalho | Descrição |
| :--- | :--- | :--- | :--- |
| `archol.showAst` | Archol: Abrir explorador de AST | `Ctrl+Alt+A` / `Cmd+Alt+A` | Abre a AST e seleciona o nó da posição atual. |
| `archol.showLinks` | Archol: Mostrar links do servidor | `Ctrl+Alt+L` / `Cmd+Alt+L` | Pede `archol/links` e abre a resposta em um documento JSON. |
| `archol.restartServer` | Archol: Reiniciar servidor LSP | — | Encerra e inicia o servidor; reabre os arquivos abertos. |
| `archol.showOutput` | Archol: Mostrar saída do servidor | — | Exibe o canal **Archol LSP**. |

Os atalhos valem apenas para editores com `editorLangId == archol`; a paleta de
comandos funciona sempre. Também é possível clicar no item da barra de status
para reiniciar o servidor.

## Explorador de AST

O painel `archol.astExplorer` mantém o contexto ao ficar oculto. A extensão pede
`archol/ast` com `{ "uri": "file:///..." }` e desenha `ast` como árvore:

- **Filtro** compara o texto digitado com `kind` e com o resumo (`Document Invoice`).
- **Cores** por categoria (declaração, expressão, statement, bloco) são só CSS,
  usando `var(--vscode-*)` para acompanhar o tema claro/escuro.
- **Clique** em um nó posiciona o cursor em `span.start` e revela o trecho,
  preservando o foco no painel.
- **Ir para código** faz o mesmo e leva o foco ao editor.
- **Copiar JSON** copia o nó selecionado; sem seleção, copia a AST inteira.
- Ao editar o documento aberto, um novo `archol/ast` é pedido após 400 ms.

Nós são identificados por caminhos de índices sobre os filhos estruturais
(propriedades que são nós e arrays de nós, na ordem do JSON). O mesmo critério é
usado pela extensão (para achar o nó da posição) e pelo webview (para desenhar a
árvore).

## Contrato LSP coberto

- **Hover**: o servidor devolve Markdown com
  `[Explorar AST](command:archol.showAst?<encodeURIComponent(JSON.stringify([uri, line, character]))>)`.
  O comando aceita os três argumentos (ou o array único que o VS Code entrega),
  abre o painel `archol.astExplorer` e destaca o nó do `span` correspondente.
- **`archol/ast`**: request `{ "uri": "file:///..." }` → `{ "ast": SourceFile }`.
  Os spans são convertidos de byte (AST) para posição UTF-16 (VS Code) e vice-versa.
- **`archol/links`**: request `{}` → `{ "links": ... }`, exibido como JSON em um
  documento sem título.
- **Diagnósticos**: `textDocument/publishDiagnostics` com severidade 1–4, `code`,
  `source: "archol"` e range mapeados para `vscode.Diagnostic`.
- **Ciclo do servidor**: `initialize` com as capabilities do contrato, `initialized`,
  `didOpen`/`didChange`/`didSave`/`didClose` e resposta a todas as requisições
  enviadas pelo servidor.

## Arquitetura

```text
archol/editors/vscode/
  package.json                  manifesto, comandos, configuração e atalhos
  language-configuration.json   comentários, pares e indentação
  syntaxes/archol.tmLanguage.json
  snippets/archol.code-snippets
  src/
    extension.js                ativação, status bar e listeners do VS Code
    session.js                  ciclo do servidor, sync de documentos e estado
    session-support.js          configuração, workspaceFolders e erro de spawn
    commands.js                 comandos da paleta e normalização dos argumentos
    diagnostics.js              mapeamento de severidade/range e showMessage
    lsp-client.js               JSON-RPC 2.0 sobre stdio, processo e requests
    server-path.js              resolução do executável e spawn multiplataforma
    lsp-log.js                  resumos de tráfego sem despejar a AST
    protocol.js                 framing Content-Length (encode/decode), puro
    lsp-features.js             providers de hover, completion, definição,
                                símbolos e referências
    vscode-utils.js             identificação de documentos Archol
    ast-view.js                 painel WebviewPanel e ponte com o editor
    ast-view-html.js            shell HTML/CSS inline do painel (sem CDN)
    ast-view-browser.js         script do webview (árvore, filtro e ações)
    ast-view-browser-utils.js   helpers puros do script do webview
    ast-tree.js                 utilidades puras da AST (offset, caminho, resumo)
  test/
    index.js                    carrega os testes para o comando por diretório
    protocol.test.js            framing, fragmentação, UTF-8 e erros
    ast-tree.test.js            árvore, spans e conversão de posição/bytes
    server-path.test.js         resolução do executável no PATH e fallbacks
```

O cliente fala o protocolo LSP canônico:

- cabeçalhos `Content-Length: N\r\n\r\n` e corpo JSON-RPC 2.0 em UTF-8;
- `initialize` declara exatamente as capabilities de hover, completion,
  `didSave`, documentSymbol hierárquico, definition links e workspaceFolders;
- requisições do servidor (`workspace/configuration`,
  `client/registerCapability`, `window/showMessage`, etc.) sempre são
  respondidas para o servidor não ficar bloqueado;
- notificações `textDocument/publishDiagnostics` e `window/showMessage` são
  tratadas; o restante é ignorado com segurança;
- `didOpen` envia o texto completo; `didChange` acompanha o `textDocumentSync`
  do servidor (incremental com `range`, ou texto completo), e `didSave` inclui
  o texto;
- o executável é iniciado como `archol lsp` por padrão; `archol.server.args`
  permite ajustar os argumentos (por exemplo, `[]` para um servidor dedicado);
- erros de spawn exibem instruções, e a queda do processo não reinicia
  automaticamente: a barra de status fica em `erro` e há ação **Reiniciar**.

## Desenvolvimento e testes

Do repositório:

```bash
for f in archol/editors/vscode/src/*.js; do node --check "$f"; done
node --test archol/editors/vscode/test
```

Dentro de `archol/editors/vscode`:

```bash
for f in src/*.js; do node --check "$f"; done
node --test test
```

Não é necessário `npm install`; os testes usam somente `node:test`,
`node:assert/strict` e módulos nativos.

## Limitações e contratos assumidos

- Sem formatador, semantic tokens, code actions, inlay hints ou renomeação; esses
  recursos não fazem parte do contrato atual do servidor.
- O servidor pode registrar capabilities dinamicamente; a extensão responde
  `client/registerCapability` com `null`, mas não implementa os providers
  dinâmicos (usa apenas os providers fixos acima).
- `span.start`/`span.end` são offsets em **bytes UTF-8**, como o AST do Archol
  documenta. A extensão converte linha/character LSP (UTF-16) → byte e byte →
  offset de string JavaScript, tratando `\r\n` e pares substitutos.
- A AST é tratada como objeto JSON acíclico; referências circulares não são
  suportadas.
- A fonte de verdade do servidor é externa a esta pasta: a extensão apenas
  consome o executável configurado.
