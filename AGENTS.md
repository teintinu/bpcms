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

- A raiz de um serviço é a pasta que contém `<service>-service.bpcms`.
- `documents/` e `processes/` são pastas **organizacionais**, não namespaces: `documents/billing-invoice-document.bpcms` continua `Invoice`, não `documents.Invoice`.
- `icons/` é um **contêiner de ícones**: as subpastas dentro dele são namespaces de ícone; `icons` não entra na referência. Ex.: `icons/status/draft.svg` -> `status.draft`.
- As demais subpastas seguem a regra de namespace: segmentos em camelCase, referências por ponto.
- Nomes de arquivos e pastas de artefato: **kebab-case**; o full path da URI preserva os segmentos da URI (ex.: `example.com`).
- Referências: camelCase unido por ponto. Referência por outro serviço: `AliasDoServico.status.refunded` (alias em `imports`).
- O mesmo nome pode existir em namespaces diferentes; duplicidade no mesmo tipo/namespace é erro.

Exemplo:

```text
examples/services/example.com/services/billing/
  billing-service.bpcms
  documents/billing-invoice-document.bpcms      -> Invoice (sem prefixo documents.)
  processes/billing-invoice-issuance-process.bpcms
  billing-shapes.bpcms
  billing-organograms.bpcms
  billing-icons.bpcms
  icons/status/draft.svg                        -> status.draft
```

## Organização dos arquivos em `examples/`

`BPCMS_WORKSPACE = examples/` nesta fase. O fonte é organizado por **contexto principal**, não por tipo de artefato. São contextos principais: `application`, `service`, `document` e `process`. Regra completa em [DSL_REFERENCE.md](DSL_REFERENCE.md), seção 30.

- Todo contexto principal tem seu próprio arquivo, com **exatamente um** principal, e ele é o **primeiro artefato** do arquivo.
- Aplicação: `examples/applications/<fullpath app>/<app>-application.bpcms`.
- Serviço: `examples/services/<fullpath service>/<service>-service.bpcms`.
- Documento: `examples/services/<fullpath service>/documents/<service>-<nome>-document.bpcms`.
- Processo: `examples/services/<fullpath service>/processes/<service>-<nome>-process.bpcms`.
- Compartilhado: `examples/services/<fullpath service>/<service>-<tipo-no-plural>.bpcms`.
- SVG: `examples/services/<fullpath service>/icons/<namespace>/<icon>.svg`.
- `-icons.bpcms` (não-SVG): `examples/services/<fullpath service>/<service>-icons.bpcms`.
- `<fullpath>` é a URI sem protocolo; `<app>`/`<service>` é o último segmento da URI, igual ao slug da pasta.
- O `<nome>` do arquivo é equivalente ao nome do artefato principal (em app/service, ao slug da URI).
- Compartilhado: `<service>-<tipo-no-plural>.bpcms`, com **um único tipo de artefato por arquivo** (`shapes`, `forms`, `views`, `messages`, `policies`, `decisions`, `integrations`, `parameters`, `secrets`, `organograms`, `icons`); a exceção é `-organograms.bpcms`, que contém exatamente um organogram e as roles, com o organogram primeiro. Arquivo compartilhado não declara principal. Ex.: `std-shapes.bpcms`.
- Referência entre arquivos é permitida: `InvoiceNumber` pode ficar em `billing-invoice-document.bpcms` e ser usado por mensagens/processos de outros contextos.
- `*-documents.bpcms`, `*-processes.bpcms` e `*-reports.bpcms` não existem. Relatório é `view`; agendamento é `start { schedule: }` no arquivo do processo.
- Roles ficam em `<service>-organograms.bpcms`, depois do organogram; não crie `<service>-roles.bpcms`. Parameters ficam em `<service>-parameters.bpcms`; secrets em `<service>-secrets.bpcms`.
- `bpcms.org/std` segue a regra de service: `examples/services/bpcms.org/std/std-service.bpcms` + `std-shapes.bpcms` (compartilhado).
- Compilador: `E_FILE_NAME`, `E_FILE_PRINCIPAL`, `E_FILE_PRINCIPAL_FIRST`, `E_FILE_SHARED`, `E_FILE_SHARED_TYPE`, `E_FILE_ORGANOGRAM_FIRST`, `E_FILE_PATH` e `E_FILE_ICON_PATH`.
- **Nomes de artefatos não repetem o serviço.** O serviço já é o escopo: `Draft`, não `BillingDraft`. O mesmo nome pode existir em serviços diferentes; não duplique dentro do mesmo tipo/serviço.
- **PascalCase é o padrão para artefatos**, com exceções para formas internacionais consagradas. Exemplo: `pH`, não `Ph`.

## Documentos vivos

- `BRAINSTORM.md` é a baseline de decisões.
- `DSL_REFERENCE.md` documenta as cláusulas da DSL.
- `DSL_EXAMPLES.md` contém os exemplos usados como teste de aceitação da sintaxe.
- `REVIEW_CHECKLIST.md` controla a revisão por tipo de artefato, um por vez.

Ao alterar um desses arquivos, mantenha os outros dois consistentes quando a mudança afetar contrato, sintaxe ou exemplos.
