# AGENTS.md — Regras de trabalho neste repositório

## Git: nunca alterar estado sem perguntar

**Não execute `git stash`, `git add`/stage, `git commit` ou `git push` sem pedir confirmação explícita ao usuário antes.**

- Isso vale para todos os agentes e sessões neste repositório.
- Se uma tarefa parecer exigir uma dessas operações, **pare e pergunte**, explicando o comando exato, o motivo e o que será alterado.
- Operações somente leitura (`git status`, `git diff`, `git log`, `git show`) são permitidas.
- Não crie tags, não reescreva histórico e não faça operações destrutivas (`git reset --hard`, `git clean`, `git checkout --`) sem a mesma confirmação.
- Se houver mudanças não commitadas, preserve-as; não tente “limpar” o working tree por conta própria.

## Modelo: fronteira Application/Service

**A `Application` não declara artefatos.** Todo artefato de negócio — `Parameter`, `Document`, `Shape`, `View`, `Form`, `Process`, `Message`, `Role`, `Policy`, `Decision`, `Integration`, `Icon`, etc. — vive dentro de um `Service`. `Theme`/`Locale` são assets do construtor, não artefatos de serviço.

A `Application` só pode:

- listar entry-point services;
- configurar papéis, ambientes e referenciar tema/idiomas fornecidos pelo construtor;
- apontar para o lock;
- no `main`, inicializar um `Parameter` já declarado em um serviço (`set <Alias>.<Parameter> = <valor>`) e, opcionalmente, fazer boot antecipado (`start`).

Não coloque `parameter`, `document`, `view` ou qualquer outro artefato diretamente na `Application` — nem em exemplos, nem em docs.

**Serviço é conceito de negócio.** Não crie serviço para agrupar artefatos compartilhados (Shapes, Icons, Themes, Locales) nem para ser biblioteca/package/módulo. Tipos compartilhados ficam no serviço de negócio dono (ex.: Money/Mass em Catalog, Email/Phone/Image em Citizen). `Theme`/`Locale`, fontes e cores são responsabilidade do construtor, não do serviço.

**`bpcms.org/std` é exceção reservada.** É um pseudo-serviço do construtor para medidas/constantes padrão; fica em `examples/services/bpcms.org/std/`, na pasta normal de serviços. Quem usa deve importar com alias (`imports { service "bpcms.org/std" as std }`) e referenciar `std.Mass` etc. Não criar endpoint nem lock para ele.

## Revisão por checklist: propor antes de mudar

Durante a revisão de artefatos, **não remova, altere nem adicione nada sem antes explicar e obter confirmação explícita do usuário**.

Para cada mudança proposta:

1. Diga **o que** muda.
2. Diga **por que**.
3. Mostre o **impacto** (arquivos, exemplos, compatibilidade).
4. Apresente **alternativas**.
5. Liste os **arquivos afetados**.
6. Aguarde um “aprovado” explícito.

Não aplique mudanças “óbvias” por conta própria. Reverter também é mudança e exige a mesma confirmação. Registre a decisão no `REVIEW_CHECKLIST.md`.

## Pastas e namespaces

`BPCMS_WORKSPACE` é o diretório raiz do workspace, passado ao compilador/construtor. Nos exemplos, `BPCMS_WORKSPACE = examples/`.

- A raiz de um serviço é a pasta que contém `<service>-service.archol`.
- `documents/`, `processes/`, `views/` e `messages/` são pastas **organizacionais**, não namespaces: `documents/billing-invoice-document.archol` continua `Invoice`, não `documents.Invoice`.
- `icons/` é um **contêiner de ícones**: as subpastas dentro dele são namespaces de ícone; `icons` não entra na referência. Ex.: `icons/status/draft.svg` -> `status.draft`.
- As demais subpastas seguem a regra de namespace: segmentos em camelCase, referências por ponto.
- Nomes de arquivos e pastas de artefato: **kebab-case**; o full path da URI preserva os segmentos da URI (ex.: `example.com`).
- Referências: camelCase unido por ponto. Referência por outro serviço: `AliasDoServico.status.refunded` (alias em `imports`).
- O mesmo nome pode existir em namespaces diferentes; duplicidade no mesmo tipo/namespace é erro.

Exemplo:

```text
examples/services/example.com/services/billing/
  billing-service.archol
  documents/billing-invoice-document.archol      -> Invoice (sem prefixo documents.)
  processes/billing-invoice-issuance-process.archol
  views/billing-invoice-table-view.archol        -> InvoiceTable (sem prefixo views.)
  messages/billing-invoice-issued-message.archol -> InvoiceIssued (sem prefixo messages.)
  billing-shapes.archol
  billing-organograms.archol
  billing-icons.archol
  icons/status/draft.svg                        -> status.draft
```

## Organização dos arquivos em `examples/`

`BPCMS_WORKSPACE = examples/` nesta fase. O fonte é organizado por **contexto principal**, não por tipo de artefato. São contextos principais: `application`, `service`, `document`, `process`, `view` e `message`. Regra completa em [DSL_REFERENCE.md](DSL_REFERENCE.md), seção 30.

- Todo contexto principal tem seu próprio arquivo, com **exatamente um** principal, e ele é o **primeiro artefato** do arquivo.
- Aplicação: `examples/applications/<fullpath app>/<app>-application.archol`.
- Serviço: `examples/services/<fullpath service>/<service>-service.archol`.
- Documento: `examples/services/<fullpath service>/documents/<service>-<nome>-document.archol`.
- Processo: `examples/services/<fullpath service>/processes/<service>-<nome>-process.archol`.
- View: `examples/services/<fullpath service>/views/<service>-<nome>-view.archol`.
- Message: `examples/services/<fullpath service>/messages/<service>-<nome>-message.archol`.
- Compartilhado: `examples/services/<fullpath service>/<service>-<tipo-no-plural>.archol`.
- SVG: `examples/services/<fullpath service>/icons/<namespace>/<icon>.svg`.
- `-icons.archol` (não-SVG): `examples/services/<fullpath service>/<service>-icons.archol`.
- `<fullpath>` é a URI sem protocolo; `<app>`/`<service>` é o último segmento da URI, igual ao slug da pasta.
- O `<nome>` do arquivo é equivalente ao nome do artefato principal (em app/service, ao slug da URI).
- Compartilhado: `<service>-<tipo-no-plural>.archol`, com **um único tipo de artefato por arquivo** (`shapes`, `forms`, `policies`, `decisions`, `integrations`, `parameters`, `secrets`, `organograms`, `icons`); a exceção é `-organograms.archol`, que contém exatamente um organogram e as roles, com o organogram primeiro. Arquivo compartilhado não declara principal. Ex.: `std-shapes.archol`.
- `view` e `message` **não** vão para arquivo compartilhado: são contextos principais e têm arquivo próprio em `views/` e `messages/`.
- Referência entre arquivos é permitida: `InvoiceNumber` pode ficar em `billing-invoice-document.archol` e ser usado por mensagens/processos de outros contextos.
- `*-documents.archol`, `*-processes.archol`, `*-views.archol`, `*-messages.archol` e `*-reports.archol` não existem. Relatório é `view`; agendamento é `start { schedule: }` no arquivo do processo.
- Roles ficam em `<service>-organograms.archol`, depois do organogram; não crie `<service>-roles.archol`. Parameters ficam em `<service>-parameters.archol`; secrets em `<service>-secrets.archol`.
- `bpcms.org/std` segue a regra de service: `examples/services/bpcms.org/std/std-service.archol` + `std-shapes.archol` (compartilhado).
- Compilador: `E_FILE_NAME`, `E_FILE_PRINCIPAL`, `E_FILE_PRINCIPAL_FIRST`, `E_FILE_SHARED`, `E_FILE_SHARED_TYPE`, `E_FILE_ORGANOGRAM_FIRST`, `E_FILE_PATH` e `E_FILE_ICON_PATH`.
- **Nomes de artefatos não repetem o serviço.** O serviço já é o escopo: `Draft`, não `BillingDraft`. O mesmo nome pode existir em serviços diferentes; não duplique dentro do mesmo tipo/serviço.
- **PascalCase é o padrão para artefatos**, com exceções para formas internacionais consagradas. Exemplo: `pH`, não `Ph`.

## Linguagem Archol e compilador

A linguagem deste repositório se chama **Archol**; todo fonte usa a extensão **`.archol`**. O compilador, o LSP e o plugin VSCode ficam em [`archol/`](archol/README.md):

- `archol/crates/archol-core` — tokens, lexer, AST JSON (estilo TypeScript) e parser.
- `archol/crates/archol-analysis` — check (valores de enum, tipos, papéis, referências) e link invertido (`role -> artefatos`).
- `archol/crates/archol-lsp` + `archol/editors/vscode` — LSP e extensão com explorador de AST.
- `cd archol && cargo test` e `cargo run -q --bin archol -- check ../examples`.

Regras:

- A linguagem e o compilador estão na **versão inicial**: não crie versões, tags, releases, changelog de compatibilidade nem publique nada até uma decisão explícita.
- O corpus `examples/` é o teste de aceitação: `cargo test -p archol-core --test corpus` precisa ficar verde; construção nova entra primeiro em um exemplo real, depois no parser.
- Arquivos de código pequenos e legíveis, um arquivo por responsabilidade; tokens organizados por família em `archol/crates/archol-core/src/token/`.
- Ao mudar sintaxe, mantenha `archol/docs/tokens.md`, `archol/docs/ast.md` e os testes consistentes.
- **Compilação só a partir de uma Application.** `check`, `lock`, `links` e o LSP exigem `--app`/aplicação atual; o workspace é a closure (entry points + `imports` transitivos). Arquivo fora da closure é inerte e o LSP o trata como comentado.
- **Identidade estável em `bpcms.lock`.** Um lock por aplicação, ao lado do `.archol` da app, em JSON Lines (um identificador por linha, ordenado por `path`). IDs base64url seguros para outra linguagem (nunca começam por dígito); `path` seguro `<uri>#<kind>/<Nome>[...]`. Gerar/podar com `archol lock --app <app>`; não editar à mão; apagar o lock reinicia os IDs.
- **AST não carrega listas de usos.** Declarações têm `id`; referências têm `ref`/`refKind`, `fieldRef`, `typeRefId`. Usos e navegação vêm do índice semântico (`archol/declaration`, `archol/usages`), evitando ciclos e JSON grande.
- **Idioma do compilador e do plugin: inglês.** Todo o `archol/crates`, `archol/scripts`, `archol/docs`, `archol/README.md` e `archol/editors/vscode` usam **inglês** em identificadores, comentários, mensagens de diagnóstico/CLI/LSP, UI do plugin, testes e documentação. O restante do repositório (specs de produto em português) permanece em português; dados de exemplo Archol com `pt-BR` não são traduzidos.

## Documentos vivos

- `BRAINSTORM.md` é a baseline de decisões.
- `DSL_REFERENCE.md` documenta as cláusulas da DSL.
- `DSL_EXAMPLES.md` contém os exemplos usados como teste de aceitação da sintaxe.
- `REVIEW_CHECKLIST.md` controla a revisão por tipo de artefato, um por vez.

Ao alterar um desses arquivos, mantenha os outros dois consistentes quando a mudança afetar contrato, sintaxe ou exemplos.
