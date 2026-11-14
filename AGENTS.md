# AGENTS.md — Regras de trabalho neste repositório

## Git: nunca alterar estado sem perguntar

**Não execute `git stash`, `git add`/stage, `git commit` ou `git push` sem pedir confirmação explícita ao usuário antes.**

- Isso vale para todos os agentes e sessões neste repositório.
- Se uma tarefa parecer exigir uma dessas operações, **pare e pergunte**, explicando o comando exato, o motivo e o que será alterado.
- Operações somente leitura (`git status`, `git diff`, `git log`, `git show`) são permitidas.
- Não crie tags, não reescreva histórico e não faça operações destrutivas (`git reset --hard`, `git clean`, `git checkout --`) sem a mesma confirmação.
- Se houver mudanças não commitadas, preserve-as; não tente “limpar” o working tree por conta própria.

## Modelo: fronteira Application/Service

**A `Application` não declara artefatos.** Todo artefato — `Parameter`, `Document`, `Shape`, `View`, `Process`, `Role`, `Event`, `Report`, `Job`, `Policy`, etc. — vive dentro de um `Service`.

A `Application` só pode:

- listar entry-point services;
- configurar papéis, tema, idiomas e ambientes;
- apontar para o lock;
- no `main`, inicializar um `Parameter` já declarado em um serviço (`set <Alias>.<Parameter> = <valor>`) e, opcionalmente, fazer boot antecipado (`start`).

Não coloque `parameter`, `document`, `view` ou qualquer outro artefato diretamente na `Application` — nem em exemplos, nem em docs.

## Nomes de arquivos em `examples/`

Todo arquivo `.bpcms` começa com o **nome do serviço/app dono da pasta**, em kebab-case, seguido do tipo:

- `examples/services/identity/identity-documents.bpcms`
- `examples/services/billing/billing-service.bpcms`
- `examples/applications/loja-do-joao/loja-do-joao-application.bpcms`

Regras:

- O prefixo é o slug da pasta, não o tipo do artefato. `identity-documents.bpcms`, nunca `documents.bpcms`.
- O arquivo `*-service.bpcms` é o manifesto do serviço; não use `service.bpcms` genérico.
- O arquivo `*-application.bpcms` é a aplicação; não use `application.bpcms` genérico.
- O restante do nome é o tipo do artefato (`documents`, `views`, `events`, `parameters`, `policies`, `jobs`, `reports`, `integrations`, `processes`, `decisions`, `organograms`, `shapes`, `themes`, `locales`).
- Roles e organogram ficam no **mesmo** arquivo `<service>-organograms.bpcms`; não crie `<service>-roles.bpcms` separado.
- Use kebab-case e não repita o prefixo dentro da pasta.

## Documentos vivos

- `BRAINSTORM.md` é a baseline de decisões.
- `DSL_REFERENCE.md` documenta as cláusulas da DSL.
- `DSL_EXAMPLES.md` contém os exemplos usados como teste de aceitação da sintaxe.

Ao alterar um desses arquivos, mantenha os outros dois consistentes quando a mudança afetar contrato, sintaxe ou exemplos.
