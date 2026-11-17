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

**`bpcms.org/std` é exceção reservada.** É um pseudo-serviço do construtor para medidas/constantes padrão; fica em `examples/services/std/`, na pasta normal de serviços. Quem usa deve importar com alias (`imports { service "bpcms.org/std" as std }`) e referenciar `std.Mass` etc. Não criar endpoint nem lock para ele.

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

## Pastas como namespace de artefatos

A raiz de um serviço é a pasta que contém `<service>-service.bpcms`. **Toda subpasta é um namespace de artefatos**, para todos os tipos.

- Nomes de arquivos e pastas: **kebab-case**.
- Segmentos de referência: **camelCase**, unidos por ponto.
- Referência local: `status.refunded`, `domain.order`, `sales.order.item`.
- Referência por outro serviço: `AliasDoServico.status.refunded` (o alias precisa estar em `imports`).
- Artefatos na raiz não têm namespace e usam o nome simples.
- O mesmo nome pode existir em namespaces diferentes; duplicidade dentro do mesmo tipo/namespace é erro.

Exemplo:

```text
checkout/
  checkout-service.bpcms
  status/refunded.svg       -> status.refunded
  domain/order.bpcms        -> domain.order
  sales/order/item.bpcms    -> sales.order.item
```

> Esta regra está anotada para as próximas validações. Os arquivos atuais ainda não foram migrados para subpastas; não mover nada sem aprovação.

## Organização dos arquivos em `examples/`

O fonte é organizado por **contexto principal**, não por tipo de artefato. São contextos principais: `application`, `service`, `document` e `process`. Regra completa em [DSL_REFERENCE.md](DSL_REFERENCE.md), seção 30.

- Todo contexto principal tem seu próprio arquivo, com **exatamente um** principal, e ele é o **primeiro artefato** do arquivo.
- Aplicação: `<app>-application.bpcms`; serviço: `<service>-service.bpcms`. O `<nome>` é o slug da pasta e o último segmento da URI.
- Documento: `<service>-<nome>-document.bpcms`; processo: `<service>-<nome>-process.bpcms`. O `<nome>` é o nome do artefato em kebab-case.
- O `<nome>` do arquivo é equivalente ao nome do artefato principal (em app/service, ao slug da URI).
- Compartilhado: `<service>-<tipo-no-plural>.bpcms`, com **um único tipo de artefato por arquivo** (`shapes`, `forms`, `views`, `messages`, `policies`, `decisions`, `integrations`, `parameters`, `secrets`, `organograms`, `icons`); a exceção é `-organograms.bpcms`, que contém exatamente um organogram e as roles, com o organogram primeiro. Arquivo compartilhado não declara principal. Ex.: `std-shapes.bpcms`.
- Referência entre arquivos é permitida: `InvoiceNumber` pode ficar em `billing-invoice-document.bpcms` e ser usado por mensagens/processos de outros contextos.
- `*-documents.bpcms`, `*-processes.bpcms` e `*-reports.bpcms` não existem. Relatório é `view`; agendamento é `start { schedule: }` no arquivo do processo.
- Roles ficam em `<service>-organograms.bpcms`, depois do organogram; não crie `<service>-roles.bpcms`. Parameters ficam em `<service>-parameters.bpcms`; secrets em `<service>-secrets.bpcms`.
- `bpcms.org/std` segue a regra de service: `std-service.bpcms` + `std-shapes.bpcms` (compartilhado).
- Os arquivos `.bpcms` ficam na raiz do serviço; a regra de subpastas como namespace segue anotada e não se aplica ainda.
- **Nomes de artefatos não repetem o serviço.** O serviço já é o escopo: `Draft`, não `BillingDraft`. O mesmo nome pode existir em serviços diferentes; não duplique dentro do mesmo tipo/serviço.
- **PascalCase é o padrão para artefatos**, com exceções para formas internacionais consagradas. Exemplo: `pH`, não `Ph`.

## Documentos vivos

- `BRAINSTORM.md` é a baseline de decisões.
- `DSL_REFERENCE.md` documenta as cláusulas da DSL.
- `DSL_EXAMPLES.md` contém os exemplos usados como teste de aceitação da sintaxe.
- `REVIEW_CHECKLIST.md` controla a revisão por tipo de artefato, um por vez.

Ao alterar um desses arquivos, mantenha os outros dois consistentes quando a mudança afetar contrato, sintaxe ou exemplos.
