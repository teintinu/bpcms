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

O fonte é organizado por **contexto principal**, não por tipo de artefato. Regra completa em [DSL_REFERENCE.md](DSL_REFERENCE.md), seção 30.

- Serviço: `<service>-<contexto>-<tipo>.bpcms`, com `tipo` = `document` ou `process` e contexto em kebab-case. Ex.: `billing-invoice-document.bpcms`, `billing-invoice-issuance-process.bpcms`.
- Aplicação: `<app>-application.bpcms`; o contexto principal é a própria aplicação.
- Manifesto: `<service>-service.bpcms`; não use `service.bpcms` genérico.
- Compartilhado: `<service>-<tipo-no-plural>.bpcms` (`shapes`, `forms`, `views`, `messages`, `policies`, `decisions`, `integrations`, `parameters`, `organograms`, `icons`) só para artefato sem dono único ou de escopo do serviço.
- Um arquivo de contexto declara exatamente **um** artefato principal; o nome e o tipo do principal formam o nome do arquivo.
- Referência entre arquivos é permitida: `InvoiceNumber` pode ficar em `billing-invoice-document.bpcms` e ser usado por mensagens/processos de outros contextos.
- `*-documents.bpcms`, `*-processes.bpcms` e `*-reports.bpcms` não existem. Relatório é `view`; agendamento é `start { schedule: }` no arquivo do processo.
- Roles e organogram ficam no **mesmo** `<service>-organograms.bpcms`; parameters/secrets e icons são service-wide (`<service>-parameters.bpcms`, `<service>-icons.bpcms`).
- `bpcms.org/std` mantém `std-service.bpcms` + `std-shapes.bpcms`.
- Os arquivos `.bpcms` ficam na raiz do serviço; a regra de subpastas como namespace segue anotada e não se aplica ainda.
- **Nomes de artefatos não repetem o serviço.** O serviço já é o escopo: `Draft`, não `BillingDraft`. O mesmo nome pode existir em serviços diferentes; não duplique dentro do mesmo tipo/serviço.
- **PascalCase é o padrão para artefatos**, com exceções para formas internacionais consagradas. Exemplo: `pH`, não `Ph`.

## Documentos vivos

- `BRAINSTORM.md` é a baseline de decisões.
- `DSL_REFERENCE.md` documenta as cláusulas da DSL.
- `DSL_EXAMPLES.md` contém os exemplos usados como teste de aceitação da sintaxe.
- `REVIEW_CHECKLIST.md` controla a revisão por tipo de artefato, um por vez.

Ao alterar um desses arquivos, mantenha os outros dois consistentes quando a mudança afetar contrato, sintaxe ou exemplos.
