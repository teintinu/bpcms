# BPCMS — Referência de Cláusulas da DSL

*Sintaxe ilustrativa v0.1.* Não é gramática final nem código executável. Esta referência cobre **todas as cláusulas usadas em [DSL_EXAMPLES.md](DSL_EXAMPLES.md)** e serve de base para o SPECS.

**A DSL é uma abstração de módulos.** O código-fonte descreve contratos, capacidades, dados e políticas; a implementação concreta é responsabilidade dos construtores. O compilador da DSL deve produzir um **AST simples, estável e poderoso**, validar dependências e impedir incoerências antes de qualquer geração de código.

## Como ler

| Notação | Significado |
| :--- | :--- |
| `<uri>` | URI web sem protocolo, literal entre aspas: `"example.com/app/x"` |
| `<name>` | identificador simples, sem URI: `Customer`, `OrderApproval` |
| `<expr>` | expressão segura, sem código arbitrário |
| `[A, B]` | lista |
| `T?` | parâmetro de ação opcional |
| `A \| B` | alternativa |
| `...` | repetição |
| `//` | comentário de linha |

Cada cláusula tem **onde aparece**, **sintaxe** e **funcionamento**. Cláusulas comuns ficam na seção 1 e são referenciadas nas seções seguintes.

## Mapa

| Seção | Artefatos |
| :--- | :--- |
| 1 | Léxico, metadados, origem, versão, lock/IDs, capabilities e serviços essenciais |
| 2–8 | Application, Service, Process, Document, Shape, View, Role/Organogram |
| 9–12 | Evidence, Conversation, Decision, Assumption |
| 13–19 | Parameter, Secret, Environment, Builder, Theme, Locale, Widget, Release/Version |
| 20–24 | Integration, Job, Policy |
| 25 | Expressões, operadores e valores |
| 26 | Validação e erros |
| 27 | Cobertura |
| 28 | Icon |
| 29 | bpcms.org/std |

---

# 1. Léxico e cláusulas comuns

## 1.1 Identificadores

| Cláusula | Onde | Sintaxe | Funcionamento |
| :--- | :--- | :--- | :--- |
| `application` | raiz | `application "<uri>" { ... }` | Declara uma aplicação. O identificador é URI sem protocolo; o nome curto é alias opcional. |
| `service` | raiz | `service "<uri>" { ... }` | Declara um serviço reutilizável. Também aceita URI sem protocolo. |
| `<name>` | demais artefatos | `document Customer`, `process OrderFulfillment` | Identificador simples, único no escopo do serviço/IR. |
| `<uri>` | app/service | `"example.com/app/loja-do-joao"`, `"github.com/acme/store"` | URI sem `http://`/`https://`. Resolve para registry BPCMS ou repositório Git. |
| `alias` | app/service | `"<uri>" as Nome` | Nome local para a URI em `services`/`imports`. Não é identidade e não vai para o lock. |

**Regras:** a URI é a identidade canônica; o alias não substitui a URI em imports/releases. O `check` rejeita duas identidades para o mesmo artefato.

## 1.1.1 Tipos básicos

Os tipos básicos são intrínsecos e não são Shapes:

| Tipo | Funcionamento |
| :--- | :--- |
| `string` | Texto UTF-8; substitui `Text`. |
| `number` | Número; inteiro ou decimal; substitui `Real`. |
| `boolean` | `true`/`false`; substitui `Boolean`. |
| `stream` | Cursor binário de leitura/escrita; substitui `Stream`. |

Tipos adicionais:

- `DateTime`, `Date`, `Unit`, `FormatFlags`, `void`;
- `streamReader` e `streamWriter` são as interfaces inferidas em `read` e `write`;
- `Int8`, `Int16`, `Int32`, `Int64`, `Float`, `Double` são usados apenas nos métodos de `stream`;
- `Shape` é o tipo de um valor de Classe; `new Classe{...}` cria um valor.

Chamadas de stream:

```ts
stream.readInt32(): Int32
stream.writeInt32(value: Int32): void
stream.readString(): string
stream.writeString(value: string): void
stream.readStream(): stream
stream.readStreamArray(): stream[]
```

## 1.2 Metadados

| Cláusula | Onde | Sintaxe | Funcionamento |
| :--- | :--- | :--- | :--- |
| `version` | app, service, process, document, builder, release | `version: 1.4.0` ou `version: semver("1.4.0")` | Versão do artefato. `1.4.0` é açúcar para `semver("1.4.0")`. |
| `title` | app, service, shape, document, field, view, event, parameter, role, organogram, icon | `title: { pt-BR: "Nome", en: "Name", short?: {...}, tooltip?: {...} }` | I18n obrigatório; ver 1.9. |
| `owner` | app, service, document, decision, assumption | `owner: "Financeiro"` | Dono humano/time; usado em revisão e promoção. |
| `description` | app | `description: "..."` | Texto livre para documentação e busca. |
| `source` | app, document, evidence, view/report, decision | `source: registry \| repository \| auto` (app); ver 1.3 | Origem, fonte de dados, dispositivo ou política — o significado depende do artefato. |
| `status` | evidence, conversation, assumption | `status: proposed \| confirmed \| ...` | Estado do artefato de descoberta. Valores variam por artefato. |
| `type` | task, view, release, report | `type: service`, `type: form`, `type: migration` | Discriminador; os valores válidos dependem do artefato. |
| `mode` | builder, theme, report | `mode: [interpret, generate]`, `mode: dark`, `mode: live` | Modo de operação/apresentação. |
| `links` | evidence, assumption | `links { process X.Y; document X.field; view V; decision D; evidence E; service "<uri>" }` | Liga o artefato de descoberta ao modelo; o `check` valida o alvo. |
| `audit` | policy | `audit: true` | Exige trilha da avaliação da policy. Document é auditado automaticamente por comando. |
| `retention` | document | `retention: 5y` | Prazo de retenção antes de anonimizar/expurgar. |
| `private` | membros de artefatos | `private <field/action>` | Membro interno; não faz parte do contrato público nem da serialização para outros serviços. Artefato de topo nunca é `private`. |

## 1.3 Origem e versão (app/service)

| Cláusula | Sintaxe | Funcionamento |
| :--- | :--- | :--- |
| `source` | `source: auto` | Detecta hosts de repositório conhecidos (GitHub, GitLab, Bitbucket e similares); o resto é registry BPCMS. |
| `source` | `source: registry` | Força documentação em `/<uri>/index.html` e distribuição pelo BPCMS. |
| `source` | `source: repository` | Força projeto Git: documentação é o **README**; código é baixado do projeto. |
| `version` | `version: 1.4.0` | Semver; açúcar para `semver("1.4.0")`. |
| `version` | `version: semver("1.4.0")` | Versão semântica publicada no registry. |
| `version` | `version: tag("v1.4.0")` | Tag Git imutável (recomendada em produção). |
| `version` | `version: commit("9f3a1c7")` | Commit Git imutável (produção/reprodutibilidade). |
| `@` | `"<uri>"@1.2.0`, `"<uri>"@tag("v1.2.0")`, `"<uri>"@commit("9f3a1c")` | Seletor de versão em releases e rollback. Imports de serviço não usam versão; o lock do app decide. |

**Regras:** em produção, use tag/commit (ou semver imutável); branch/HEAD só em dev/QA. Uma URI de repositório pode incluir subdiretório: `github.com/acme/monorepo/apps/store`.

## 1.4 Referências e tipos

| Cláusula | Onde | Sintaxe | Funcionamento |
| :--- | :--- | :--- | :--- |
| `Shape` | campos, binds | `peso: Mass` | Tipo de dado; dono de parse/validação/formato/unidade. |
| `Document` | reads/writes/source | `reads: [Order]` | Referência a documento; o `check` valida existência e dono. |
| `View` | form, actions | `form: ReviewForm` | Referência a uma view. |
| `Role` | roles/role | `role: Manager` | Referência a papel. |
| `Event` | start/end/on | `start on Order.Approved` | Referência a evento de Document. |
| `Integration` | task/integration | `integration: Carrier` | Referência a integração. |
| `T` | campos | `phone: Phone` | Sem `required` = opcional; `?` não é usado no field. |
| `T[]` | coleções | `items: InvoiceItem[]` | Lista ordenada de `T`. |
| `A \| B` | binds | `binds: Mass \| Length` | União de tipos aceitos pelo widget. |

## 1.5 Ambiente e segredo

| Cláusula | Sintaxe | Funcionamento |
| :--- | :--- | :--- |
| `env` | `env.ERP_URL` | Valor resolvido pelo ambiente na promoção; nunca hardcode. |
| `secret` | `secret.STRIPE_KEY` | Segredo resolvido no runtime; nunca aparece em logs/export. |
| `now()` | `now() - 24h` | Relógio do runtime (virtual em QA). |
| `currentUser` | `assignee = currentUser` | Usuário autenticado no contexto da view/ação. |

> Para configuração/segredo **tipado e contrato**, prefira `Parameter` (seção 13); `env.X`/`secret.X` são atalhos de resolução, não declarações.

## 1.6 Lock file e IDs

O app tem **um lock file** (`bpcms.lock`), gerado e versionado. Ele fixa **uma versão por serviço** e **um ID estável para cada identificador** do app.

| Cláusula | Sintaxe | Funcionamento |
| :--- | :--- | :--- |
| `lock` | `lock:` (raiz do YAML) | Bloco do lock. |
| `application` | `application: "<uri>"` | App dona do lock. |
| `version` | `version: 1` | Versão do formato do lock. |
| `nextId` | `nextId: 42` | Próximo ID; IDs não são reutilizados. |
| `services` | mapa `"<uri>" -> { id, version, source, integrity }` | **Uma versão por serviço** no app. |
| `id` | `id: 2` | ID estável do serviço/artefato. |
| `version` (serviço) | `version: 1.2.0` | Versão escolhida para o app. |
| `source` | `source: registry \| repository` | Origem da versão. |
| `integrity` | `integrity: "sha256:..."` | Integridade do artefato. |
| `artifacts` | mapa `"<fqn>" -> id` | Todos os identificadores: documentos, campos, ações, views, eventos, papéis, nós. |

```yaml
lock:
  application: "example.com/app/loja-com-antifraude"
  version: 1
  nextId: 48
  services:
    "example.com/services/checkout":
      id: 3
      version: 2.0.1
      source: registry
      integrity: "sha256:..."
  artifacts:
    "example.com/services/checkout#CheckoutSession": 20
    "example.com/services/checkout#Checkout": 21
```

**Regras:** o lock é a única fonte de IDs por app; o serviço **não tem lock, IDs nem versões de imports**. Os IDs são usados para **minificação de código** e **compressão da serialização**; nomes humanos permanecem no fonte. Se dois serviços exigem versões incompatíveis do mesmo serviço, o `check` falha.

## 1.7 Capabilities

`capabilities` usa conceitos de **alto nível**, não tecnologia. O construtor mapeia para HTTP, DB, UI kit, fila etc.

| Capability | Significado |
| :--- | :--- |
| `gui` | apresentação gráfica |
| `tui` | apresentação em terminal |
| `print` | impressão/PDF |
| `oltp` | persistência transacional |
| `sync` | sincronização em tempo real |
| `location` | localização/GPS |
| `timezone` | fuso horário, inclusive quando o device não oferece |
| `regionalization` | formatação de data/hora/número por locale |
| `units` | medidas, conversões e Shapes métrico/imperial |
| `identity` | autenticação/autorização |
| `scheduling` | agenda, timers e jobs |
| `attachments` | arquivos/anexos |
| `capture` | câmera/scanner |
| `notify` | notificação de saída |

`provides` não existe no Service: o contrato de negócio são os artefatos públicos. Não use `http`, `db`, `search`, `ui`, `files`, `auth`, `realtime`, `jobs`, `camera` ou `push` como capability.

**Extensões.** O conjunto acima é um núcleo aberto. Construtores podem declarar novas capabilities, genéricas (`llm`) ou específicas (`google-maps`, `acme.payments`). Use kebab-case e documente o contrato no construtor. Configuração/segredo da capability usa `Parameter`.

```bpcms
builder AiRuntime {
  capabilities: [llm, oltp, sync]
}

job RefreshAssistant {
  capabilities { scheduling, llm }
  schedule: "0 * * * *"
  action: Assistant.refresh()
}
```

## 1.8 Serviços essenciais

Alguns serviços são de plataforma e são acessados como capabilities, não importados como serviços de app.

| Capability | Operações | Depende de |
| :--- | :--- | :--- |
| `location` | `current()`, `distance(a, b)`, `timezoneOf(location)` | — |
| `timezone` | `current()`, `of(location)`, `convert(at, from, to)`, `offset(zone, at)` | `location` (opcional) |
| `regionalization` | `locale()`, `formatDate`, `formatTime`, `formatNumber`, `formatCurrency`, `parseDate`, `parseNumber` | `Locale` |
| `units` | `system()`, `format(value, locale)`, `shapes()` | `regionalization` |

```bpcms
service "example.com/services/field" {
  title: { pt-BR: "Serviço de campo", en: "Field service" }
}

view VisitForm {
  capabilities { location, timezone, regionalization, units }
  type: form
  computed {
    scheduledLocal: regionalization.formatDate(visit.scheduledAt, timezone.current())
    distance: units.format(location.distance(tech.position, visit.position), regionalization.system())
  }
}
```

**Shapes de unidades oferecidos por `units`:**

| Família | Métrico | Imperial | Conversão |
| :--- | :--- | :--- | :--- |
| Massa | kg, g, t | lb, oz | fator |
| Comprimento | m, cm, km | in, ft, yd, mi | fator |
| Volume | m3, L, cm3 | gal, fl oz | fator |
| Temperatura | C, K | F | afim |
| Área | m2, ha | ft2, acre | fator |

O Shape guarda **valor original + unidade** e projeção normalizada. A exibição usa `regionalization.system()`/`units.format`; o armazenamento não é convertido.

## 1.9 I18n (`title`)

`I18n` é um **tipo interno** de texto localizado. Não é um `Shape` declarável; `shape I18n` é `E_RESERVED_TYPE`.

`title` é obrigatório em `service`, `shape`, `document`, `field`, `view`, `event`, `parameter`, `role`, `organogram` e `icon`; a `Application` também usa o mesmo tipo.

Cada valor de locale pode ser uma string ou um mapa de formas plurais CLDR:

```ts
type PluralForms = {
  zero?: string;
  one?: string;
  two?: string;
  few?: string;
  many?: string;
  other: string;
};

type I18nValue = string | PluralForms;
type I18n = { [locale: string]: I18nValue };
```

```bpcms
title: { pt-BR: "Nome", en: "Name" }
```

```bpcms
title: {
  pt-BR: {
    one: "metro"
    other: "metros"
    many: "metros"
  }
  en: {
    one: "meter"
    other: "meters"
  }
}
```

- `other` é obrigatório quando o valor é um objeto plural.
- `one` deve corresponder ao título singular quando existir.
- Categorias `zero`, `two`, `few` e `many` são opcionais.
- O builder resolve a categoria pelo locale/número; não é o Shape que decide plural.
- `short` e `tooltip` aceitam o mesmo `I18nValue`.

O mapa aceita locales (`pt-BR`, `en-US`, `en`) e duas versões alternativas **opcionais**, irmãs do mapa principal:

```bpcms
name: string {
  title: {
    pt-BR: "Nome da pessoa"
    en: "Person name"
    short: { pt-BR: "Pessoa", en: "Person" }
    tooltip: { pt-BR: "Nome completo da pessoa", en: "Full person name" }
  }
  maxLength: 120
  required
}
```

- `short` é o label curto para tabelas/grids.
- `tooltip` é o texto do popover automático do title.
- `short` e `tooltip` são **irmãos**; não ficam aninhados um no outro.
- Se `short`/`tooltip` não existirem, o builder usa o title principal.
- A ordem dos campos com `lookup` define a ordem de exibição; os títulos vêm dos campos.

**Fallback:** locale exato → idioma (`en-US` → `en`) → locale default do app. Para plural: categoria → `other` → string do locale/título. Com `locales: [pt-BR, en-US]`, `en` cobre `en-US`; o default é `pt-BR`.

**Campos:** o `title` do campo é o label; o `title` do `document`/`shape`/`view`/`event`/`parameter`/`secret`/`role`/`organogram`/`service` é o nome do artefato. Em `Parameter`, `title` substitui o antigo `description`.

| Situação | Resultado |
| :--- | :--- |
| Builder gera `pt-BR` e a chave existe | ok |
| Builder gera `en-US` e só existe `en` | ok por fallback de idioma |
| Builder gera `en-US` e não existe `en-US` nem `en` | `W_TRANSLATION` no `check`; `E_TRANSLATION` se o builder for estrito |
| Chave fora de `locales` | `E_LOCALE_UNDECLARED` |
| Sem entrada para o locale default | `E_TITLE_MISSING` |
| Plural sem `other`/categoria inválida | `E_PLURAL_OTHER` / `E_PLURAL_FORM` |
| `title` ausente no artefato/campo | `E_TITLE_MISSING` |
| `shape I18n` declarado | `E_RESERVED_TYPE` |

**Hífen:** `pt-BR`/`en-US` são chaves válidas; o parser aceita hífen no nome da propriedade e na chave do mapa.

**Artefato reutilizável:** quando `title` aparece em `Service`/`View` compartilhado, o conjunto válido é o `locales` do **app consumidor**; o artefato não fixa o conjunto.

**Política do construtor:** dev pode fazer fallback para o locale default e permitir editar o título em runtime, gravando de volta no fonte; produção é estrita por padrão. O construtor nunca inventa tradução.

---

# 2. Application

| Cláusula | Sintaxe | Funcionamento |
| :--- | :--- | :--- |
| declaração | `application "<uri>" { ... }` | Identidade + composição. |
| `title` | `title: { pt-BR: "..."; en-US: "..." }` | Nome localizado; chaves em `locales` (ver 1.9). |
| `services` | `services { "<uri>" as Alias ... }` | **Entry points**; alias local opcional. A closure vem dos imports de cada serviço. |
| `roles` | `roles { Customer; Clerk; Manager; Manager > Clerk }` | Papéis da aplicação e hierarquia (`>` = “superior a”). |
| `theme` | `theme: Nuvula` | ID de tema fornecido pelo **construtor**; não é artefato de serviço. |
| `locales` | `locales: [pt-BR, en-US]` | Idiomas da app/construtor; o primeiro é o default. |
| `environments` | `environments: [dev, qa, prod]` | Ambientes declarados para a aplicação. |
| `main` | `main { ... }` | Bloco opcional de inicialização/configuração; sem regra de negócio. |
| `set` | `set <Alias>.<Parameter\|Secret> = <valor\|secret.NOME>` | Inicializa/sobrescreve Parameter/Secret; segredo só como `secret.NOME`. |
| `start` | `start ["<uri>"]` | Boot antecipado opcional; não declara dependência. |
| `lock` | `lock: "bpcms.lock"` | Lock do app: uma versão por serviço + IDs. |

**Funcionamento:** a `Application` **não declara artefatos** e não contém regra nem `wire`; apenas compõe entry points, papéis, tema, idiomas, ambientes e o lock. Dependências vêm de `Service.imports`; o resolver calcula a closure e o lock fixa **uma versão por serviço**. A URI é o identificador; a página `http://<uri>/index.html` é download por SO (registry) ou README/releases (repositório).

```bpcms
application "example.com/app/loja-do-joao" {
  source: registry
  version: 1.4.0
  lock: "bpcms.lock"
  title: {
    pt-BR: "Loja do João"
    en-US: "João's Store"
  }

  services {
    "example.com/services/catalog" as Catalog
    "example.com/services/checkout" as Checkout
    "example.com/services/notifications" as Notifications
  }
  roles { Customer; Clerk; Manager; Manager > Clerk }
  theme: Nuvula
  locales: [pt-BR, en-US]
  environments: [dev, qa, prod]

  main {
    // DefaultCurrency é um Parameter declarado no serviço Catalog
    set Catalog.DefaultCurrency = "BRL"
  }
}
```

---

# 3. Service

| Cláusula | Sintaxe | Funcionamento |
| :--- | :--- | :--- |
| declaração | `service "<uri>" { ... }` | Identidade + contrato. |
| `title` | `title: { pt-BR: "...", en: "..." }` | I18n obrigatório (ver 1.9). |
| `imports` | `imports { service "<uri>" as Alias; integration ERP }` | Dependências por URI, **sem versão**; alias local opcional; o lock escolhe a versão. Vazio é válido. |
| `reintroduce` | `reintroduce <Alias>.<Artifact> { ... }` | Altera/adiciona definições de um artefato importado. |

> **Contrato:** todo artefato de topo é público automaticamente; **não existe `exports`**. Privacidade só por membro (`private`).

**Funcionamento:** serviço é um **conceito de negócio**, não biblioteca/package/módulo. Não crie serviço apenas para agrupar Shapes/Icons/Themes/Locales. Ele é uma fatia vertical; todo artefato de topo é público e ele é dono dos seus documentos. O que é `private` dentro de um artefato não entra no contrato público. Importar um serviço não dá acesso de escrita aos dados dele. A URI registry resolve `/index.html` como documentação de API; repositório resolve README + código.

```bpcms
service "example.com/services/catalog" {
  title: { pt-BR: "Catálogo", en: "Catalog" }
  version: 1.2.0
  owner: "Time de Catálogo"

  imports { service "example.com/services/identity" as Identity }
}
```

**Capabilities não ficam no Service.** Cada artefato declara as capacidades que usa:

- `Job` → `scheduling`;
- `Shape` de `audio`/`image`/`video` → `gui`;
- `View` → `gui` somente se a tela inteira exigir GUI por outro motivo;
- `Shape` de anexo → `attachments`;
- `Document` com device → `capture`/`units`;
- `Integration` de equipamento → `equipment`;
- `Role` de equipamento → `equipment`/protocolo.

O builder valida a closure. Se uma capability exigida por qualquer artefato não estiver disponível, o build falha. Capability indisponível falha no build, não no runtime.

**Sem `provides`.** O serviço não declara o que oferece ao construtor; `provides` é responsabilidade do construtor. O contrato de negócio continua sendo os artefatos públicos do serviço.

**Sem `roles` inline.** Roles ficam no arquivo `*-organograms.bpcms`; a lista inline duplicava a fonte de verdade.

**Sem `source`.** Publicação/origem pertence a `Release`/lock, não ao contrato do serviço.

## 3.1 `reintroduce`

`reintroduce` altera/adiciona definições de um artefato importado. Pode modificar qualquer parte do artefato: `value`, `fields`, `title`, `icon`, `ordinal`, `permissions`, etc.

```bpcms
reintroduce std.Currency {
  value BRL {
    title: { pt-BR: "Real", en: "Brazilian real" }
    prefix: {
      pt-BR: "R$"
      en: "BRL"
    }
    icon: Brl
    ordinal: 10
  }
}
```

- exige import do dono;
- é aditivo;
- não remove definição;
- conflito de metadados gera `E_REINTRODUCE_CONFLICT`;
- vale para a closure do app;
- não há cláusula de substituição total; `reintroduce` altera apenas a parte declarada;
- revisar depois de todas as keywords.

Para processos, cada passo é um nó reutilizável. O `fraud-checkout` adiciona um passo sem substituir o processo inteiro:

```bpcms
service "example.com/services/fraud-checkout" {
  imports {
    service "example.com/services/checkout" as Checkout
  }

  reintroduce Checkout.Checkout {
    task FraudCheck {
      type: service
      action: FraudCheck.evaluate
    }

    insert FraudCheck before AuthorizePayment
  }
}
```

- `FraudCheck` é adicionado ao processo existente;
- `insert ... before ...` religa o fluxo;
- os demais passos permanecem;
- não há substituição total do processo.


# 4. Process

| Cláusula | Sintaxe | Funcionamento |
| :--- | :--- | :--- |
| declaração | `process <Name> { ... }` | Processo executável/descritivo; BPMN é uma visão. |
| `version` | `version: 1` | Versão do processo; releases e diff usam. |
| `goal` | `goal: "..."` | Objetivo de negócio; usado na entrevista e no review. |
| `roles` | `roles: [Requester, Manager, Buyer]` | Atores que podem receber tarefas; referenciam `Role`. |
| `start on` | `start on PurchaseRequest.Requested` | Evento que inicia uma instância. |
| `task` | `task ReviewRequest { ... }` | Nó de trabalho humano (default) ou automático. |
| `type` | `type: service` ou `type: compensation` | Na task, distingue humana (default), automática e compensação. |
| `role` | `role: Manager` | Papel responsável pela tarefa humana. |
| `form` | `form: PurchaseReview` | View usada para executar a tarefa. |
| `reads` | `reads: [PurchaseRequest]` | Documentos/campos consultados; o `check` valida acesso. |
| `writes` | `writes: [PurchaseRequest]` | Documentos/campos alterados pela tarefa. |
| `sla` | `sla: 1d` | Prazo esperado; alimenta painel, escalonamento e simulação. |
| `action` | `action: PurchaseRequest.approve` | Ação de documento executada pela task de serviço. |
| `integration` | `integration: Carrier` | Integração chamada pela task de serviço. |
| `for` | `for: ChargePayment` | Na task de compensação, nó concluído que será compensado. |
| `retry` | `retry: exponential(3)` | Política de retry para task de serviço/erro. |
| `gateway` | `gateway Amount { ... }` | Decisão exclusiva; um caminho por vez. |
| `if` | `if <expr> -> <Node>` | Condição do gateway; primeira condição verdadeira vence. |
| `else` | `else -> <Node>` | Caminho default do gateway. |
| `->` | `A -> B` | Transição entre nós; também usado em casos/regras. |
| `insert` | `insert FraudCheck before AuthorizePayment` | Insere um nó no fluxo linear, religando predecessor → novo → alvo. |
| `parallel` | `parallel { task A { ... }; task B { ... } }` | Divide o fluxo; ambos os ramos executam. |
| `subprocess` | `subprocess Name { process: OtherProcess }` | Chama outro processo como subprocesso. |
| `process` | `process: ProvisionAccess` | Referência ao processo chamado. |
| `end on` | `end on Shipment.Dispatched` | Evento que encerra a instância. |
| `on timer` | `on timer 2d after ReviewRequest -> Escalate` | Boundary timer: dispara após duração, a partir do nó. |
| `after` | `after <Node>` | Nó de referência do timer. |
| `on error` | `on error CarrierUnavailable -> ManualQuote` | Boundary error: desvia quando a task/serviço falha. |

**Funcionamento:** a task humana cria uma inbox para o papel; a task de serviço chama `action`/`integration`. `reads`/`writes` limitam o que a tarefa pode tocar. Gateways não executam regra própria: usam `Decision`/`Rule` ou expressão segura. `parallel` espera todos os ramos; `subprocess` herda o caso. `on timer`/`on error` são boundaries do nó anterior. Uma instância mantém variáveis (`purchase.amount`, `shipment.pesoBruto`) e histórico.

```bpcms
process PurchaseApproval {
  version: 1
  goal: "Comprar com alçada e rastro"
  roles: [Requester, Manager, Buyer]
  start on PurchaseRequest.Requested

  task ReviewRequest {
    role: Manager
    form: PurchaseReview
    reads: [PurchaseRequest]
    writes: [PurchaseRequest]
    sla: 1d
  }

  gateway Amount {
    if purchase.amount <= 5000 -> AutoApprove
    else -> BuyerQuote
  }

  task AutoApprove {
    type: service
    action: PurchaseRequest.approve
  }

  end on PurchaseRequest.Approved
}
```

**Compensação:** uma task `type: compensation` declara `for: <Node>` (atividade concluída a compensar) e `action`. É disparada por `on error`/cancelamento, executa em ordem reversa e deve ser idempotente. `retry` não substitui compensação.

```bpcms
task ChargePayment {
  type: service
  integration: Payment
  action: Payment.charge
}
on error StockUnavailable -> RefundPayment
task RefundPayment {
  type: compensation
  for: ChargePayment
  action: Payment.refund
}
```

---

# 5. Document

| Cláusula | Sintaxe | Funcionamento |
| :--- | :--- | :--- |
| declaração | `document <Name> { ... }` | Unidade de dados + ações + regras + sync. |
| `title` | `title: { pt-BR: "...", en: "..." }` | I18n obrigatório (ver 1.9). |
| `id` | *(implícito)* | Todo document tem `id` único gerado; não declare o campo. |
| `lookup` | `nickname: string { lookup }` | Marca o campo para o lookup de exibição; a ordem dos campos marcados define a ordem. Extensões herdam os campos marcados. |
| `scopable` | `scopable document Store { ... }` | Documento também é um tipo de escopo. |
| referência | `store: Store { title: {...} }` | Campo de Document é implicitamente um `DocumentRef` com `id` + lookup do alvo. |
| escopo inferido | `store: Store` | Todo direto para `scopable document` cria um escopo. |
| `number` | `number: InvoiceNumber { format; sequence; unique }` | Chave alternativa de negócio; não é o `id` interno. |
| `extends` | `document Consumer extends User { ... }` | Especialização aditiva; herda campos, ações, regras e lookup; usa o mesmo id do pai. |
| `version` | `version: 1` | Versão do schema; migrações comparam. |
| `owner` | `owner: "Billing"` | Serviço/time dono; outros só referenciam. |
| `sync` | `sync: live` ou `sync: read` | `live` assina patches; `read` entrega leitura materializada. |

| `retention` | `retention: 5y` | Retenção antes de anonimizar/expurgar. |
| `capabilities` | `capabilities { capture, units }` | Capacidades exigidas pelo Document. |
| `fields` | `fields { name: Shape { ... } }` | Campos tipados; o campo só passa parâmetros simples. |
| `indexes` | `indexes { byStatus: [status, createdAt] range }` | Índices declarativos; o builder realiza. |
| `actions` | `actions { issue() { title; roles; transition: Issued; rules; body } }` | Comandos/botões; `title` e `roles` obrigatórios; sem retorno. |
| `events` | `events { Approved { title; key; payload } }` | Eventos sem workflow; `emit` na action. |
| `seed` | `seed { SP { code: "SP"; name: "São Paulo" } }` | Documentos pré-criados/idempotentes para bootstrap. |
| `rules` | `rules { total == subtotal + tax: { message } }` | Regras de documento, action e field; condição verdadeira = permitido. |
| `source: device` | `source: device "scale-01" for pesoBruto` | Campo é lido de dispositivo; atualização = releitura. |
| `computed from` | `computed from Invoice, Payment` | Documento inteiro é cálculo de outras fontes. |
| `computed <Name> from` | `computed frete from Shipment, CarrierTable { ... }` | Cálculo nomeado (campo/subdocumento) com frescor. |
| `freshness` | `freshness: 15m` | TTL/agenda do cálculo; expõe `calculatedAt`. |
| `exposes` | `exposes: [frete, calculatedAt]` | Campos resultantes do cálculo. |

**Campos calculados no Document.** A mesma cláusula vale para fields do Document:

```bpcms
document Invoice {
  fields {
    items: InvoiceItem[]
    subtotal: std.Money {
      title: { pt-BR: "Subtotal", en: "Subtotal" }
      currency: BRL
      computed: items.sum(total)
    }
    tax: std.Money {
      title: { pt-BR: "Imposto", en: "Tax" }
      currency: BRL
    }
    total: std.Money {
      title: { pt-BR: "Total", en: "Total" }
      currency: BRL
      computed: subtotal + tax
    }
  }
}
```

- `total` de cada item pode ser calculado;
- `subtotal` soma os itens;
- `total` soma subtotal + imposto;
- computed é somente leitura;
- não repetir a conta em `rules`.

**Seed.** `seed` declara documentos pré-criados necessários para o funcionamento:

```bpcms
scopable document Store {
  title: { pt-BR: "Loja", en: "Store" }

  fields {
    code: string { title: { pt-BR: "Código", en: "Code" }, unique, lookup }
    name: string { title: { pt-BR: "Nome", en: "Name" }, lookup }
  }

  seed {
    SP {
      code: "SP"
      name: "São Paulo"
    }
  }
}
```

Regras:

- o builder aplica o seed no bootstrap/migração;
- é idempotente;
- valida `required`, rules, unique e referências;
- usa um `id` determinístico por serviço/Document/chave;
- não executa actions;
- não emite eventos de domínio;
- `seed` não é `initial` de workflow.

**Identificador de seed.** A chave do seed vira uma referência válida:

```text
Store.SP
FiscalYear.Y2026
```

`Store.SP` é um `DocumentRef` imutável para a loja de São Paulo:

- carrega o `id` determinístico;
- carrega os campos `lookup` do Store;
- pode ser usado em fields, actions, rules, outros seeds e processos;
- é resolvido em tempo de build;
- não é string;
- não é ID solto.

Exemplo:

```bpcms
document Invoice {
  fields {
    store: Store { default: Store.SP }
    fiscalYear: FiscalYear { default: FiscalYear.Y2026 }
  }
}
```

```bpcms
seed {
  DefaultInvoiceConfig {
    store: Store.SP
    fiscalYear: FiscalYear.Y2026
  }
}
```

Identidade canônica:

```text
example.com/services/billing#Store/SP
```

Se o seed for renomeado, a referência quebra; renomear seed é mudança incompatível.

**Identidade e `DocumentRef`.** Todo `document` tem um `id` interno implícito; ele não é declarado. Um campo cujo tipo é um Document é implicitamente um `DocumentRef`: carrega `id` + todos os campos `lookup` do alvo. Não existe Shape `<Name>Id` nem tipo `XxxId` na DSL. **Exceção:** extensão (`extends`) compartilha o `id` do pai; `Consumer extends User` usa o mesmo `id` de `User`.

**Publicação/subscrição.** A referência/scope não define replicação. Publicação, subscrição, cópia e tier de armazenamento são conceitos separados, revisados com Document. O scope apenas determina a qual partição o dado pertence e quem pode vê-lo.

**Lookup/description.** Todo document base marca com `lookup` os campos que o representam para o usuário. Ex.: `nickname: string { lookup }` e `avatar: Image { lookup }` em `User`. A ordem dos campos marcados é a ordem do lookup. Todo para esse document recebe automaticamente esses campos; o builder decide se resolve na leitura ou materializa. Extensões **herdam** os campos marcados e não redeclaram um bloco.

**`scopable document`.** Um documento marcado como `scopable` é um tipo de escopo. Ele mantém o `id` interno implícito e pode ter chaves de negócio como `code`/`year`:

```bpcms
scopable document Store {
  fields {
    code: StoreCode { title: { pt-BR: "Código", en: "Code" }, unique }
    name: string { title: { pt-BR: "Nome", en: "Name" } }
  }
}

scopable document FiscalYear {
  fields {
    year: integer { title: { pt-BR: "Ano", en: "Year" }, unique }
    startsAt: DateTime { title: { pt-BR: "Início", en: "Start" } }
    endsAt: DateTime { title: { pt-BR: "Fim", en: "End" } }
  }
}
```

**Escopo inferido por referência.** Todo campo direto para um `scopable document` cria um escopo no documento consumidor. Não há `scopes { ... }`, `source`, `parentId` nem `path`.

```bpcms
document Invoice {
  fields {
    store: Store {
      title: { pt-BR: "Loja", en: "Store" }
    }
    fiscalYear: FiscalYear {
      title: { pt-BR: "Exercício", en: "Fiscal year" }
    }
    number: InvoiceNumber
    issuedAt: DateTime
  }
}
```

- `Invoice` tem os escopos `Store` e `FiscalYear`;
- o escopo usa `store.id` e `fiscalYear.id`;
- `code`/`year` são chaves de negócio e entram no lookup do alvo;
- links indiretos não inferem escopo;
- dois links para o mesmo tipo de escopo no mesmo documento são erro;
- opcional cria escopo opcional, com política explícita.

**Forma completa da referência.** Quando o campo tem `title`, `lookup`, `required` ou outros atributos, use o bloco:

```bpcms
assignedTo: User {
  title: { pt-BR: "Responsável", en: "Assignee" }
  lookup
}
```

- `title`, `lookup`, `required`, `sensitive`, `default` etc. são atributos do field;
- `id` é sempre implícito;
- a forma curta `User { name }` só vale quando não há atributos.

**`DocumentRef` não é ForeignKey.** É uma referência lógica:

```bpcms
store: Store {
  title: { pt-BR: "Loja", en: "Store" }
}
```

O alvo `Store` marca com `lookup` os campos `code` e `name`. O campo entrega, conceitualmente:

```json
{ "id": "st_123", "code": "0001", "name": "Loja 1" }
```

- `id` é sempre preservado;
- todos os campos `lookup` do alvo entram automaticamente na referência;
- o campo não escolhe nem declara campos;
- excluir o alvo não quebra o documento;
- não há cascade;
- o builder pode resolver o lookup na leitura ou materializar;
- archive persiste a projeção de lookup;
- scope usa o `id`, nunca `code`/`name`.

**Modos do builder:**

| Modo | Comportamento |
| :--- | :--- |
| `resolved` | Guarda só o `id`; resolve lookup na leitura |
| `materialized` | Guarda `id` + lookup do alvo |
| `archive` | Lookup materializado e imutável |

**`id` × número.** Todo documento tem `id` interno implícito. O número de negócio é uma chave alternativa:

```bpcms
number: InvoiceNumber {
  format: "{fiscalYear.year:yy}-{store.code:4}-{sequence:6}"
  sequence: per(Store, FiscalYear)
  unique
}
```

- `id` é técnico e usado em referências;
- `number` é exibido, pesquisável, etiquetável e rótulo;
- `number` não substitui o `id`;
- `sequence` é por escopo;
- overflow e lacunas têm política explícita;
- archive preserva o `number`.

**Migração.** Campos `customer: User` são `DocumentRef`; formas antigas `customerId: User` são legadas. O modelo novo usa `DocumentRef` implícito; a migração de cada documento é feita junto com a revisão de Document.

**Extensão.** `document Consumer extends User` modela dois conceitos: `User` é o usuário do sistema (login/papéis) e `Consumer` é o consumidor pagante. A extensão:

- é aditiva: herda campos, ações, regras e lookup; adiciona `citizen: Citizen { title }`;
- mantém o **mesmo `id`** do pai; `Consumer` não cria um segundo id;
- fica no mesmo serviço do pai (v1);
- pode ser referenciada como `User`; o `citizen` carrega o lookup do cidadão, e `User` sozinho entrega o lookup herdado.
- `citizen` **não é unique**: vários `Consumer`/`User` podem referenciar o mesmo `Citizen`; o `identity` mantém o vínculo 1:N; `CitizenMerged` reatribui os Users.

`citizen` é um para o `Citizen`, dono da identificação/PII; não é CPF/CNPJ nem taxa. É opcional: um `User` sem perfil `Consumer` tem `null`.

**Referência circular.** Em uma referência para o próprio documento, use um nome de campo diferente do id (ex.: `parent: Category { title }`). O id interno continua implícito.

**Parâmetros de campo:**

| Parâmetro | Sintaxe | Funcionamento |
| :--- | :--- | :--- |
| `title` | `title: { pt-BR: "...", en: "..." }` | I18n do campo (obrigatório); `short`/`tooltip` opcionais. |
| `required` | `required` | Campo obrigatório; ausência falha. `required: true` também é aceito. |
| `rules` | `rules { value != null: { message } }` | Regras condicionais do field. |
| `lookup` | `lookup` | Inclui o campo no lookup de exibição; a ordem é a ordem dos campos. |
| booleanos | `required`, `lookup`, `unique`, `readonly`, `sensitive` | Flag bare = `true`; `flag: true/false` também é aceito. |
| `maxLength` | `maxLength: 120` | Limite de texto; o `Shape` valida. |
| `unique` | `unique` | Unicidade lógica; índice realizando. `unique: true` também é aceito. |
| `unit` | `unit: kg` | Unidade do valor; default é a unidade base do Shape. |
| `currency` | `currency: BRL` | Restringe/default da moeda de `std.Money`. |
| `min` / `max` | `min: 0.001, max: 50000` | Faixa validada no field. |
| `scale` | `scale: 2` | Casas decimais do field. |
| `precision` | `precision: 3` | Precisão de armazenamento/exibição do field. |
| `unit` / `units` | — | `unit` só no Shape de unidade única; `units` só no Shape multiunidade. |
| `default` | `default: BR` | Valor inicial; não é regra de negócio. |
| `computed` | `computed: quantity * unitPrice` | Campo derivado; somente leitura; calculado a partir de outros campos. |
| `private` | `private internalScore: Score` | Campo interno; não é exposto a outros serviços/API/serialização. |
| `read` | `read: [Clerk, Manager]` | Roles que podem ler o campo. |
| `edit` | `edit: [Manager]` | Roles que podem alterar o campo. |

**Campo calculado em Shape.** Shape composto pode ter campo derivado:

```bpcms
shape InvoiceItem {
  title: { pt-BR: "Item da nota", en: "Invoice item" }

  fields {
    quantity: std.Count { title: { pt-BR: "Quantidade", en: "Quantity" } }
    unitPrice: std.Money { title: { pt-BR: "Preço unitário", en: "Unit price" }; currency: BRL }
    total: std.Money {
      title: { pt-BR: "Total", en: "Total" }
      currency: BRL
      computed: quantity * unitPrice
    }
  }
}
```

- o campo é somente leitura;
- não pode ser atribuído;
- é calculado a partir dos demais campos;
- é incluído na serialização;
- dependências circulares são erro;
- não precisa de rule de consistência.

**Ações são botões.** Toda ação declara `title`, `roles` e, opcionalmente, `transition` e `rules`.

```bpcms
actions {
  issue() {
    title: { pt-BR: "Emitir", en: "Issue" }
    roles: [Manager]
    transition: Issued

    rules {
      items.length > 0: {
        pt-BR: "Adicione itens antes de emitir a fatura"
        en: "Add items before issuing the invoice"
      }

      total > 0 BRL: {
        pt-BR: "Fatura sem valor"
        en: "Invoice has no value"
      }
    }

    {
      issuedAt = now()
    }
  }

  rename(name: string) {
    title: { pt-BR: "Renomear", en: "Rename" }
    roles: [Manager]

    {
      oldName = this.name
      this.name = name
      emit Renamed {
        oldName: old.name
        newName: new.name
      }
    }
  }
}
```

- `title`: label do botão;
- `roles: [Manager]`; `roles: public` quando não há restrição;
- `roles` é obrigatório;
- `roles: public` é explícito;
- `roles: []` é erro;
- `transition: Issued` liga a ação à transição;
- action de workflow não usa `status =>`;
- action de workflow não usa `emit`;
- `rules` são pré-condições;
- condição verdadeira = permitido;
- o compilador extrai os campos usados na expressão;
- body é o bloco de statements;
- body sem retorno;
- evento vem da transição ou de `emit` sem workflow.

**Ação pública.** Quando não há restrição de role:

```bpcms
archive() {
  title: { pt-BR: "Arquivar", en: "Archive" }
  roles: public
  transition: Archived
}
```

`roles: public` é explícito; `roles` ausente e `roles: []` são erros.

**Rules.** A mesma sintaxe vale no Document, na action e no field.

```bpcms
rules {
  total == subtotal + tax: {
    pt-BR: "Total da nota não confere"
    en: "Invoice total does not match"
  }
}
```

```bpcms
customer: User {
  title: { pt-BR: "Cliente", en: "Customer" }

  rules {
    value != null: {
      pt-BR: "Selecione o cliente da nota"
      en: "Select the invoice customer"
    }
  }
}
```

- o compilador extrai os campos usados na expressão;
- a UI usa os campos extraídos para navegar;
- `code` não existe;
- permissão e workflow têm motivo genérico do builder.

**`required` e rules de field.** Obrigatoriedade simples usa `required`; regra condicional usa `rules`:

```bpcms
customer: User {
  title: { pt-BR: "Cliente", en: "Customer" }
  required
}

tracking: TrackingCode {
  title: { pt-BR: "Rastreio", en: "Tracking" }

  rules {
    status != Dispatched || value != null: {
      pt-BR: "Informe o rastreio após o despacho"
      en: "Enter tracking after dispatch"
    }
  }
}
```

**Auditoria automática.** Não existe comando `audit()` na DSL.

Toda action é um comando e o compilador gera um nó de auditoria no AST para o builder registrar:

- comando executado;
- documento e `id`;
- ator;
- escopo;
- data/hora;
- correlation id;
- resultado: sucesso, negado ou erro;
- estado antes/depois ou diff;
- motivo de negação, quando houver;
- evento emitido, quando houver.

O registro é responsabilidade do builder. A DSL não declara `audit(...)` nem `audit: true` no Document.

O AST do compilador expõe:

```text
CommandAudit {
  command: "Billing.Invoice.issue"
  document: "Billing.Invoice"
  documentId: ...
  actorId: ...
  scope: ...
  occurredAt: ...
  correlationId: ...
  before: ...
  after: ...
  result: success | denied | error
  reasons: [...]
  event: "Billing.Invoice.Issued"?
}
```



**Permissões.** Não existe bloco `permissions` no Document. Permissões ficam no `field` (`read`/`edit`) e na `action` (`roles`). Busca é feita por um processo que declara `reads`; não existe permissão `search` no documento.

**Mascaramento.** Aplicado na leitura do field:

```bpcms
customer: User {
  title: { pt-BR: "Cliente", en: "Customer" }

  mask {
    roles: [Clerk]
    fields: [name, email]
  }
}
```

**Limites.** Aplicados em actions:

```bpcms
refund(amount: std.Money) {
  title: { pt-BR: "Reembolsar", en: "Refund" }
  roles: [Manager]

  limits {
    amount <= 50000 BRL: {
      roles: [Manager]
      pt-BR: "Acima do limite de reembolso"
      en: "Above the refund limit"
    }
  }
}
```

**Segregação e assinatura.** No Document:

```bpcms
segregation {
  issue != cancel
}

signature {
  release: Pathologist
}
```

- `segregation` impede o mesmo ator nas duas ações;
- `signature` exige role com autoridade de assinatura;
- ambos são validados no build e no runtime.

**Relatórios.** `type: report` é um Document calculado com cache:

```bpcms
document CashFlow {
  type: report
  title: { pt-BR: "Fluxo de caixa", en: "Cash flow" }
  computed from Invoice, Payment, Bill
  freshness: 1h
  exposes: [balance, calculatedAt]
}
```

**Índices:** `unique` (unicidade), `range` (ordenação/faixa), `search` (texto), `hash` (igualdade/partição). O campo referenciado precisa existir e ser indexável no `Shape`.

```bpcms
document Shipment {
  title: { pt-BR: "Remessa", en: "Shipment" }
  version: 1
  owner: "Dispatch"
  sync: live

  fields {
    order: Order {
      title: { pt-BR: "Pedido", en: "Order" }
      lookup
    }
    pesoBruto: std.Mass { title: { pt-BR: "Peso bruto", en: "Gross weight" }, unit: kg, min: 0.001, precision: 3 }
    tracking: TrackingCode { title: { pt-BR: "Rastreio", en: "Tracking" }, lookup }
    status: ShipmentStatus { title: { pt-BR: "Situação", en: "Status" }, lookup }
    createdAt: DateTime { title: { pt-BR: "Criado em", en: "Created at" } }
  }

  source: device "scale-01" for pesoBruto

  computed frete from Shipment, CarrierTable {
    freshness: 15m
    exposes: [frete, calculatedAt]
  }

  indexes {
    byStatus: [status, createdAt] range
  }

  actions {
    dispatch(carrier: CarrierRef)
  }

  rules {
    status == dispatched implies tracking != null
  }
}
```

---

# 6. Shape

| Cláusula | Sintaxe | Funcionamento |
| :--- | :--- | :--- |
| declaração | `shape <Name> { ... }` | Tipo de dado; dono de semântica. |
| `title` | `title: { pt-BR: "...", en: "..." }` | I18n obrigatório (ver 1.9). |
| `base` | `base: decimal \| integer \| string \| classification \| workflow \| attachment \| audio \| image \| video \| richtext` | Representação base; define exibição/edição. |
| `capabilities` | `capabilities { attachments }` | Capacidades exigidas pelo Shape; falha de build se ausentes. |
| `mime` | `mime: [image/*, application/octet-stream]` | Restrição de MIME para `attachment` genérico. |
| `classification` | `classification: "<url>"` | Classificação externa; reservada à stdlib (`Currency`). |
| `value` | `value Draft { title; icon; ordinal; transition Issued { payload { ... } } }` | Enum; `classification`/`workflow`; a transição é nomeada pelo destino; `event:` opcional. Exceção: `value` base de `std.Currency` não exige metadados. |
| `initial` | `initial: draft` | Estado inicial do `workflow`; obrigatório e único. |
| `dimension` | `dimension: mass` | Grandeza; impede comparar com outra dimensão. |
| `units` | `units { m: { factor: 1 }, cm: { factor: 0.01 } }` | Primeira unidade é a referência; demais usam `factor` ou `formula`. |
| `unit` | `unit: %` | Unidade única; mutuamente exclusivo com `units`. |
| `parse` | `parse: quantity`, `locale-number`, `percent`, `digits`, `lower-trim`, `email` | Perfil default de texto → valor; composto aceita perfil; método sobrescreve. |
| `format` | `format: currency`, `locale-unit`, `percent`, `plain`, `postal`, `image`, `masked("...")` | Perfil default de valor → texto; composto aceita perfil; método sobrescreve. |
| `compare` | `compare: exact` | Igualdade/ordenação; `exact` para dinheiro. |
| `aggregate` | `aggregate: sum` | Soma/média/min/max permitidos. |
| `index` | `index: normalized` | Codificação para índice. |
| `normalize` | `normalize: digits`, `lower-trim`, `upper-trim` | Normalização antes de validar/comparar. |
| `validate` | `validate: cpf-check-digits`, `uuid`, `email`, `phone`, `url`, `year-month` | Validação semântica. |
| `sensitive` | `sensitive: true` | Default do Shape; o field pode sobrescrever. |
| `maxLength` | `maxLength: 254` | Default do Shape; o field pode sobrescrever. |
| `fields` | `fields { street: string ... }` | Shape composto. |
| `operator` | `operator +(left: A, right: B): C { ... }` | Operador próprio do Shape. |
| método | `<name>(...): <Type> { ... }` | Função própria do Shape; `value` é o receptor. O compilador detecta pelo identificador seguido de `(`. |

### 6.0 Bases de apresentação

- `string`: texto simples.
- `richtext`: texto rico; o builder valida, sanitiza, transporta e armazena.
- `attachment`: arquivo genérico; `mime` restringe.
- `audio`, `image`, `video`: subtipos de apresentação de `attachment`; já implicam MIME de áudio, imagem e vídeo.
- `mime` só é necessário no `attachment` genérico.

```bpcms
shape ProductImage {
  title: { pt-BR: "Imagem do produto", en: "Product image" }
  base: image
}

shape ProductManual {
  title: { pt-BR: "Manual do produto", en: "Product manual" }
  base: attachment
  mime: [application/pdf, application/octet-stream]
}
```

> `<Document>Id` é o Shape implícito de identidade do documento; não declare `shape Category` (ver 5).

**Métodos não usam keyword.** Um identificador seguido de `(` no corpo do Shape declara um método:

```bpcms
toNumber(value) {
  return value.raw;
}
```

`method toNumber(value)` é inválido.

## 6.1 Valores enumerados

Cada value é um bloco com `title` (I18n), `icon` e `ordinal` obrigatório.

```bpcms
shape InvoiceStatus {
  title: { pt-BR: "Situação da nota", en: "Invoice status" }
  base: workflow
  initial: Draft
  format: plain
  value Draft {
    title: { pt-BR: "Rascunho", en: "Draft" }
    icon: Draft
    ordinal: 10
    transition Issued {
      title: { pt-BR: "Emitida", en: "Issued" }
      payload {
        number: InvoiceNumber
        total: std.Money
      }
    }
    transition Cancelled {
      title: { pt-BR: "Cancelada", en: "Cancelled" }
      payload { reason: string }
    }
  }
  value Issued {
    title: { pt-BR: "Emitida", en: "Issued" }
    icon: Issued
    ordinal: 20
    transition Paid {
      title: { pt-BR: "Paga", en: "Paid" }
    }
    transition Overdue {
      title: { pt-BR: "Vencida", en: "Overdue" }
    }
    transition Cancelled {
      title: { pt-BR: "Cancelada", en: "Cancelled" }
    }
  }
  value Paid {
    title: { pt-BR: "Paga", en: "Paid" }
    icon: Paid
    ordinal: 30

  }
  value Overdue {
    title: { pt-BR: "Vencida", en: "Overdue" }
    icon: Overdue
    ordinal: 40
    transition Paid {
      title: { pt-BR: "Paga", en: "Paid" }
    }
    transition Cancelled {
      title: { pt-BR: "Cancelada", en: "Cancelled" }
    }
  }
  value Cancelled {
    title: { pt-BR: "Cancelada", en: "Cancelled" }
    icon: Cancelled
    ordinal: 50

  }
}
```

- O primeiro ordinal é **10**; os seguintes incrementam de **10 em 10**.
- `ordinal` é único, inteiro e estável; inserir um value no meio usa um ordinal livre.
- O ordinal é usado no stream binário; o title/icon são para UI.
- `value` sem `title`/`icon`/`ordinal` é erro, exceto `std.Currency`.

## 6.1.1 Units

`units` é um mapa de unidade → fator relativo à unidade base (fator `1`). Unidades não lineares usam `formula { to/from }`.

```bpcms
shape Length {
  title: { pt-BR: "Comprimento", en: "Length" }
  base: decimal
  dimension: length
  units {
    m: {
      factor: 1
      title: {
        pt-BR: { one: "metro", other: "metros" }
        en: { one: "meter", other: "meters" }
      }
    }
    cm: {
      factor: 0.01
      title: {
        pt-BR: { one: "centímetro", other: "centímetros" }
        en: { one: "centimeter", other: "centimeters" }
      }
    }
  }
}
```

- `unit`, `min` e `precision` **não são cláusulas de Shape**.
- O field define `unit`, `min` e `precision` quando necessário.
- `value.unit` é a unidade do valor; se o field não definir, usa a unidade de fator `1`.
- O `format` escolhe automaticamente a unidade mais legível, salvo `flags.unit`; o título usa `one`/`other` conforme o número.
- `title` aceita `string | PluralForms`; `plural` não é campo separado da unidade.
- A primeira unidade é a referência e tem `factor: 1`.
- `unit` e `units` são mutuamente exclusivos (`E_UNIT_UNITS_CONFLICT`).
- `Percentage` usa `unit: "%"`.

### 6.1.1.1 Exceção `std.Currency`

`std.Currency` é a única classificação que pode ter `value` sem `title`, `icon` e `ordinal`:

```bpcms
shape Currency {
  classification: "https://www.iso.org/iso-4217-currency-codes.html"
  value BRL { numeric: "986", scale: 2 }
  value USD { numeric: "840", scale: 2 }
}
```

- A exceção vale só para `std.Currency`.
- Para usar uma moeda, o serviço deve completar os metadados com `reintroduce`.
- Sem `reintroduce`: `E_CURRENCY_METADATA_MISSING`.
- `reintroduce` adiciona `title`, `icon`, `ordinal` e formatação.

### 6.1.1.2 Shape composto

Shape composto usa serialização JSON canônica por default:

```bpcms
shape TrackingEvent {
  fields {
    at: DateTime
    status: ShipmentStatus
    description: string
  }
}
```

- Ordem dos fields = ordem de declaração.
- `format`/`parse` explícitos sobrescrevem o default.
- Campos `sensitive` são omitidos/redigidos na serialização pública.
- Campos `private` não são serializados.

## 6.1.2 classification e workflow

`base: classification` enumera categorias; `base: workflow` modela estados e transições.

```bpcms
shape TaxRegimeCode {
  title: { pt-BR: "Regime tributário", en: "Tax regime" }
  base: classification
  value SIMPLES {
    title: { pt-BR: "Simples Nacional", en: "Simples Nacional" }
    icon: Simples
    ordinal: 10
  }
}
```

- `classification` não tem `initial` nem transições.
- `workflow` tem exatamente um `initial`.
- `transition` define as arestas; estado sem transição é terminal.
- a chave da transição referencia apenas states do mesmo workflow.
- Self-transition é inválida no v1.
- `canTransitionTo` usa o `next` gerado pelo compilador.
- `transitionTo` não existe; alterar o valor para um próximo estado inválido lança `E_INVALID_TRANSITION`.
- Ordinal continua explícito, 10, 20, 30...

### 6.1.1.3 formula e convert

Unidades não lineares usam `formula`:

```bpcms
units {
  mol_L: { factor: 1 }
  pH: {
    formula: {
      to(mol_L) { return -log10(mol_L); }
      from(pH) { return 10 ^ (-pH); }
    }
  }
}
```

Para fórmulas afins invertíveis, aceita-se o shorthand:

```bpcms
C: {
  formula: K - 273.15
}
```

- `formula: <expr>` mapeia referência → unidade.
- O compilador infere `from` por inversão.
- Só vale para expressão afim (`a * x + b`).
- Se não for invertível, usar o bloco `to/from`.
- As duas formas são válidas.

Arrow equivalente:

```bpcms
formula: {
  to: (mol_L) => -log10(mol_L)
  from: (pH) => 10 ^ (-pH)
}
```

- `to` = referência → unidade.
- `from` = unidade → referência.
- `factor` e `formula` são mutuamente exclusivos.
- O compilador gera `convert` para todas as direções.
- Fórmula inválida gera `E_UNIT_FORMULA`.

## 6.2 Valor e métodos

Cada Shape gera um tipo de valor na linguagem alvo. As operações reservadas são:

| Operação | Assinatura | Função |
| :--- | :--- | :--- |
| `write` | `write(value, out)` | Serializa; tipos/retorno inferidos |
| `read` | `read(in)` | Desserializa; `in` é inferido como `streamReader`; retorno inferido |
| `format` | `format(value, flags)` | Texto para UI; tipos/retorno inferidos |
| `parse` | `parse(text, flags)` | Texto → valor; tipos/retorno inferidos |
| `toNumber` | `toNumber(value)` | Valor numérico; tipos/retorno inferidos |
| `toOrdinal` | `toOrdinal(value)` | Gerado automaticamente em classification/workflow |

- `write(value, out)` recebe `value` explícito; `out` é inferido como `streamWriter`; o retorno `void` é inferido.
- `read(in)` infere `in` como `streamReader`.
- `write`/`read` existem para **todos** os Shapes, inclusive enums e compostos.
- `format`/`parse` têm default para primitivos, enums e medidas; **compostos exigem método próprio**.
- `toNumber(value)` só em Shapes numéricos; `Money.toNumber` é implícito e retorna `raw`.
- `toOrdinal(value)` é gerado pelo compilador em `classification`/`workflow`; não se escreve no Shape. Exceção: `std.Currency` não tem ordinal nem `toOrdinal`.
- `value` é explícito em `write(value, out)`, `format(value, flags)` e `toNumber(value)`. `read`/`parse` são estáticos.

### FormatFlags

```ts
type FormatFlags = {
  style?: normal | short | tooltip;
  unit?: Unit;
  unitStyle?: short | long;
};
```

```ts
format(date, { style: short })     // "14/nov/26"
format(date, { style: normal })    // "14 de novembro de 2026"
format(date, { style: tooltip })   // "Sábado, 14 de novembro de 2026"

format(mass, { style: normal, unit: kg })
format(mass, { style: short, unit: lb })
format(mass, { unit: kg, unitStyle: short })   // "1kg"
format(mass, { unit: kg, unitStyle: long })    // "1 quilograma"
```

`parse` recebe as mesmas flags:

```ts
parse("14/nov/26", { style: short })
parse("12,5 kg", { style: normal, unit: kg })
// texto sem unidade e sem flags.unit, ou unidade incompatível -> E_PARSE_UNIT
```

Regras do `parse`:

- Texto com unidade + sem `flags.unit`: usa a unidade do texto.
- Texto com unidade + `flags.unit` diferente: converte para `flags.unit`.
- Texto sem unidade + `flags.unit`: usa `flags.unit`.
- Texto sem unidade + sem `flags.unit`: `E_PARSE_UNIT`.
- Unidade incompatível/dimensionalmente inválida: `E_PARSE_UNIT`.
- Singular/plural vêm de `units.title`.
- Separadores e símbolos vêm da regionalization interna.
- Se a regionalization não for determinável: `E_REGIONALIZATION_UNKNOWN`.
- A stdlib usa `parseQuantity(text, flags)`.

## 6.3 stream canônico

`stream` é um tipo intrínseco de leitura/escrita binária. Não é ASN.1.

```ts
type streamReader = {
  readInt8(): Int8;
  readInt16(): Int16;
  readInt32(): Int32;
  readInt64(): Int64;
  readFloat(): Float;
  readDouble(): Double;
  readString(): string;
  readStream(): stream;
  readStreamArray(): stream[];
};

type streamWriter = {
  writeInt8(value: Int8): void;
  writeInt16(value: Int16): void;
  writeInt32(value: Int32): void;
  writeInt64(value: Int64): void;
  writeFloat(value: Float): void;
  writeDouble(value: Double): void;
  writeString(value: string): void;
  writeStream(value: stream): void;
  writeStreamArray(value: stream[]): void;
};

type stream = streamReader & streamWriter;
```

Layout canônico interlinguagens:

- little-endian;
- sem padding;
- campos na ordem declarada;
- `boolean` = `int8` 0/1;
- `Integer` = `int64`;
- `number` = `double`;
- `string` = `int32` comprimento + UTF-8;
- enum = `int32` ordinal; `Currency` = `string` ISO;
- composto = campos inline;
- array = `int32` quantidade + itens.

`write(value, out)` e `read(in)` usam esse layout. Medidas gravam a unidade como texto e o valor como `double`; o builder pode otimizar armazenamento/indexação, mas o stream lido/escrito é o mesmo entre linguagens.

Funções de apoio usadas pelos métodos da stdlib:

```ts
parseQuantity(text: string, flags: FormatFlags): { value: number, unit: Unit }
parseReal(text: string, unit: Unit): number
formatReal(raw: number, from: Unit, to: Unit, style: FormatStyle, unitStyle: UnitStyle): string
bestUnit(raw: number, from: Unit, units: Units): Unit
```

- `bestUnit` escolhe a unidade que evita muitos decimais/zeros à direita; `flags.unit` força uma unidade.
- `formatReal` resolve `title`/`PluralForms` da unidade; `format` usa `one`/`other` conforme o número.
- `value.raw` é o valor na unidade atual.
- `value.unit` é a unidade atual.
- `read`/`parse` são métodos estáticos do Shape; `write`/`format` usam `value`; `toNumber` recebe `value` explícito.
- `new Shape{...}` cria um valor; `new` é a forma geral de instanciar classes/Shapes.
- `Money.toNumber` é automático e retorna `raw`; `Currency` não tem `toNumber`.

## 6.4 Expressões e métodos próprios

```bpcms
shape Distance {
  title: { pt-BR: "Distância", en: "Distance" }
  base: decimal
  dimension: length
  units {
    m: { factor: 1 }
    km: { factor: 1000 }
    mi: { factor: 1609.344 }
    ft: { factor: 0.3048 }
  }

  toNumber(value) {
    return value.raw;
  }

  toWholeKilometres() {
    const from = value.unit ?? m;
    return trunc(value.raw * units[from].factor / units[km].factor);
  }

  isLongerThan(limit) {
    const a = value.raw * units[value.unit ?? m].factor;
    const b = limit.raw * units[limit.unit ?? m].factor;
    return a > b;
  }
}
```

Gramática mínima:

- `const`;
- `if`/`else`;
- `for (const item of list)`;
- `while`;
- `return`;
- `throw Error("...")`;
- chamadas, membros (`value.field`), arrays, operadores e `??`;
- `round` e `trunc` intrínsecos;
- funções em **camelCase**: `parseReal`, `formatReal`, `bestUnit`, `toNumber`.
- `toNumber(value)` é declarado pelo Shape; as medidas de `bpcms.org/std` já o declaram.

Não há `let`, declaração de classes, funções aninhadas, async ou exceções customizadas no v1.

### 6.4.1 `new` e instanciação

Classes e Shapes são instanciados com `new <Type>{...}`:

```bpcms
const unit = in.readString();
const raw = in.readDouble();
return new Length{raw, unit};
```

- `new Length{raw, unit}` equivale a `new Length{raw: raw, unit: unit}`.
- A sintaxe de propriedades é `{ PROPNOME: PROPVALOR, ... }`.
- Quando a variável tem o mesmo nome da propriedade, o valor pode ser omitido (shorthand), como em TypeScript.
- O tipo de `in` em `read` é inferido como `streamReader`; o retorno declarado é `Length`.
- Em `write(value, out)`, `value` é o Shape e `out` é inferido como `streamWriter`; `void` também é inferido.
- Se o tipo de retorno, propriedade ou valor for inválido, o compilador gera erro (`E_TYPE`/`E_CONSTRUCTOR_PROPERTY`).
- Para `Money`, `new Money{raw, currency}`.

### 6.4.2 Tipos explícitos ou inferidos

Os argumentos **precisam existir**; o tipo pode ser explícito ou inferido.

Explícito:

```bpcms
toNumber(value: Length): number {
  return value.raw;
}

write(value: Length, out: streamWriter) {
  out.writeString(value.unit ?? m);
  out.writeDouble(value.raw);
}
```

Inferido:

```bpcms
toNumber(value) {
  return value.raw;
}

write(value, out) {
  out.writeString(value.unit ?? m);
  out.writeDouble(value.raw);
}
```

Misto:

```bpcms
convert(value: Length, unit) {
  return value.raw * units[value.unit ?? m].factor / units[unit].factor;
}
```

- O tipo de retorno também pode ser explícito ou inferido.
- O compilador valida os tipos inferidos e gera `E_TYPE` se forem inválidos.
- Não existe forma sem argumento: `write(out)` é inválido; o `value` é obrigatório.

## 6.4.3 Operadores

```bpcms
shape Length {
  operator +(left: Length, right: Length): Length {
    const rightInLeft = convert(right.raw, right.unit, left.unit);
    return new Length{raw: left.raw + rightInLeft, unit: left.unit};
  }

  operator /(left: Length, right: Duration): Speed {
    const l = convert(left.raw, left.unit, m);
    const r = convert(right.raw, right.unit, s);
    return new Speed{raw: l / r, unit: m_s};
  }
}
```

- `1 m + 20 cm` → `1.2 m`.
- `1 km / 3 h` → `Speed`.
- `1 m + 1 s` → `E_TYPE`/`E_OPERATOR`.
- Resultado de multiplicação/divisão usa a unidade de referência do Shape resultante.
- `Force * Length` é `Energy`; `Torque` exige operador explícito próprio.

## 6.5 `toNumber`

```bpcms
shape Rating {
  title: { pt-BR: "Avaliação", en: "Rating" }
  base: decimal

  toNumber(value) {
    return value.raw;
  }
}
```

- `value.raw` é o valor base do Shape.
- Enum pode usar `value.ordinal`.
- Composto pode usar `value.campo`.
- Em `bpcms.org/std`, medidas declaram `toNumber(value)` explicitamente.

## 6.6 `toOrdinal`

`toOrdinal(value)` é **gerado automaticamente** em todo `classification`/`workflow`.

- Não se declara `toOrdinal` no Shape.
- O compilador gera a função para retornar `value.ordinal`.
- Isso evita código repetido em 22 enums.
- `toNumber` continua restrito a Shapes numéricos; em `Money` é gerado automaticamente.

## 6.7 `convert`

`convert` é intrínseco e inteligente:

```ts
convert(value: number, from: Unit, to: Unit): number
```

- Usa a unidade de referência e o grafo `units`.
- Resolve `factor` e `formula` em todas as direções.
- `Length -> Duration` não é conversão; é operador.
- Unidades incompatíveis geram `E_UNIT`.
- Fórmula inválida gera `E_UNIT_FORMULA`.

# 7. View

| Cláusula | Sintaxe | Funcionamento |
| :--- | :--- | :--- |
| declaração | `view <Name> { ... }` | Projeção de dados/interação; não é dona do dado. |
| `title` | `title: { pt-BR: "...", en: "..." }` | I18n obrigatório (ver 1.9). |
| `type` | `type: form \| table \| dashboard \| wizard \| detail \| kanban` | Discriminador da view. |
| `document` | `document: Customer` | Documento principal da tela. |
| `source` | `source: Order` | Fonte de tabela/kanban. |
| `capabilities` | `capabilities { gui }` | Só quando a View inteira exige GUI por outro motivo; `audio`/`image`/`video` declaram `gui` no próprio Shape. |
| `sources` | `sources: [Invoice, Order]` | Fontes de dashboard/relatório. |
| `roles` | `roles: [Clerk, Manager]` | Quem pode abrir/executar. |
| `fields` | `fields { name: string { title: { pt-BR: "Nome", en: "Name" }, required } }` | Campos exibidos/editados; `title` é obrigatório; aceita `required`, `readonly` e parâmetros do Shape. |
| `readonly` | `readonly` | Campo apenas leitura na view; `readonly: true` também é aceito. |
| `columns` | `columns { number; customer.name; total }` | Colunas de tabela. |
| `filters` | `filters { status; period; customer }` | Filtros de tabela/relatório. |
| `filter` | `filter { assignee = currentUser }` | Filtro fixo/contextual (kanban). |
| `pagination` | `pagination: 50` | Tamanho de página. |
| `actions` | `actions { save -> Customer.create; cancel -> none }` | Ações disponíveis e alvo/comando; `none` significa sem efeito. |
| `metrics` | `metrics { revenue: sum(Invoice.total) by region }` | Métricas de dashboard. |
| `freshness` | `freshness: 15m` | Frescor dos dados calculados. |
| `units` | `units { revenue: BRL }` | Unidade de exibição por métrica. |
| `export` | `export: [pdf, csv]` | Formatos de exportação. |
| `steps` | `steps { Cart -> Address -> Payment -> Review }` | Etapas de wizard. |
| `data` | `data { Cart; Address; Payment }` | Dados compartilhados entre etapas. |
| `integration` | `integration: Payment` | Integração usada por uma etapa/ação. |
| `sections` | `sections { summary; items; timeline }` | Seções de detail. |
| `computed` | `computed { frete }` | Campos calculados exibidos. |
| `device` | `device { scale-01 }` | Dispositivo exibido/consultado. |
| `groupBy` | `groupBy: status` | Agrupamento de kanban. |
| `card` | `card { subject; priority; slaDue }` | Campos do cartão. |
| `mode` | `mode: live` | Dashboard em tempo real. |

**Tipos de view:**

| Tipo | Cláusulas típicas |
| :--- | :--- |
| `form` | `document`, `fields`, `actions` |
| `table` | `source`, `columns`, `filters`, `pagination`, `actions` |
| `dashboard` | `sources`, `metrics`, `filters`, `freshness`, `units`, `export` |
| `wizard` | `steps`, `data`, `actions`, `integration` |
| `detail` | `document`, `sections`, `actions`, `computed`, `device` |
| `kanban` | `source`, `groupBy`, `columns`, `card`, `filter`, `actions` |

```bpcms
view SalesDashboard {
  title: { pt-BR: "Painel de vendas", en: "Sales dashboard" }
  type: dashboard
  sources: [Invoice, Order]
  roles: [Manager]
  filters { period; region }
  metrics {
    revenue: sum(Invoice.total) by region
    orders: count(Order) by region
  }
  freshness: 15m
  units { revenue: BRL }
  export: [pdf, csv]
}
```

---

# 8. Role / Organogram

| Cláusula | Sintaxe | Funcionamento |
| :--- | :--- | :--- |
| `role` | `role <Name> { ... }` | Papel/permissões. |
| `title` | `title: { pt-BR: "...", en: "..." }` | I18n obrigatório (ver 1.9). |
| `permissions` | `permissions: [order.manage, report.view]` | Namespaces de capacidade/ação. |
| `capabilities` | `capabilities { equipment, laboratory-protocol }` | Capacidades exigidas pelo artefato; falha de build se ausentes. |
| `inherits` | `inherits: Clerk` ou `inherits: [Clerk, Auditor]` | Soma as permissões herdadas às permissões próprias. |
| `scopes` | `scopes: [Store, FiscalYear]` | Tipos de escopo da role/organograma. |
| `manages` | `LabManager manages [Collector, Analyst]` | Atribuição de roles a terceiros; nunca a si mesmo; não concede as permissões da role. |
| `all` | `Store: {all}` | Todos os escopos do tipo; bare, sem `true`. |
| `only` | `Store: {only: [storeForSP, storeForRJ]}` | Lista de **referências a documentos**; escopo novo não entra. |
| `except` | `Store: {except: [storeForSP]}` | Todos, exceto as referências listadas; escopo novo entra. |
| `organogram` | `organogram <Name> { ... }` | Hierarquia e regras de autorização. |
| `hierarchy` | `hierarchy { Manager > Clerk > Customer }` | `>` = superior a; um caminho por linha. |
| `fourEyes` | `fourEyes { payment.approve requires Approver }` | Exige um segundo ator diferente, com a role indicada. |
| `segregation` | `segregation { shipment.pick != shipment.check }` | Impede que o mesmo ator faça as duas ações. |
| `delegates to` | `Supervisor delegates to Checker when absent for 7d` | Delegação temporária e condicional. |
| `signature` | `signature { Pathologist }` | Papel com autoridade de assinatura. |
| `requires` | `fourEyes { X requires Role }` | Papel/condição exigida. |
| `when absent` | `... when absent for 7d` | Condição e vigência da delegação. |

> Roles e organogram do serviço ficam no mesmo arquivo `<service>-organograms.bpcms`.

```bpcms
role Clerk {
  title: { pt-BR: "Atendente", en: "Clerk" }
  permissions: [invoice.view]
}

role InvoiceManager {
  title: { pt-BR: "Gerente de notas", en: "Invoice manager" }
  scopes: [Store, FiscalYear]
  permissions: [invoice.view, invoice.issue, report.view]
}

role AnalyzerDevice {
  title: { pt-BR: "Equipamento analisador", en: "Analyzer device" }
  scopes: [Laboratory]
  capabilities { equipment, laboratory-protocol }
  permissions: [exam.perform.hemogram]
}

organogram Store {
  title: { pt-BR: "Organograma da loja", en: "Store organogram" }
  scopes: [Store]

  hierarchy {
    InvoiceManager > Clerk
  }

  InvoiceManager manages [Clerk]
  InvoiceManager delegates to Clerk when absent for 7d

  fourEyes {
    refund.approve requires InvoiceManager
  }
}
```

```bpcms
// fiscalYearFor2026: FiscalYear — referência ao documento
// storeForSP: Store — referência ao documento
assignRole(user, InvoiceManager, scopes: {
  Store: {all}
  FiscalYear: {only: [fiscalYearFor2026]}
})

assignRole(regionalUser, InvoiceManager, scopes: {
  Store: {except: [storeForSP]}
  FiscalYear: {only: [fiscalYearFor2026]}
})
```

**Herança de role.** `inherits` soma permissões; não substitui as próprias:

- `permissions` locais continuam valendo;
- `inherits` aceita uma role ou lista;
- herança é aditiva, sem substituição/remoção;
- ciclos são erro de build;
- a role herdeira mantém `title`, `scopes` e `manages` próprios;
- permissões herdadas não mudam a identidade da role.

```bpcms
role Collector {
  title: { pt-BR: "Coletador", en: "Collector" }
  permissions: [sample.collect]
}

role Auditor {
  title: { pt-BR: "Auditor", en: "Auditor" }
  permissions: [audit.view]
}

role Analyst {
  title: { pt-BR: "Analista", en: "Analyst" }
  inherits: [Collector, Auditor]
  permissions: [exam.result]
}
```

**Referência de escopo.** Em `only`/`except`, passe sempre uma **referência ao documento** de escopo, nunca um ID solto nem uma chave alternativa:

- `fiscalYearFor2026: FiscalYear` é uma referência válida;
- `2026` ou `"2026"` é chave alternativa e não localiza o documento;
- `storeForSP: Store` é uma referência válida;
- `"SP"` ou um ID textual solto não são referências;
- o builder extrai o ID interno da referência;
- a referência é resolvida no runtime, não no assignment;
- `all` não precisa de referência.

**Atribuição de papel.** `manages` define autoridade de atribuição, não permissões:

- `all` inclui escopos presentes e futuros;
- `only` é fail-closed: escopo novo não entra;
- `except` é fail-open: escopo novo entra, exceto os listados;
- `all`, `only` e `except` são mutuamente exclusivos por escopo;
- `except: []` equivale a todos, mas `all` é a forma explícita recomendada;
- todo `manages` é `others-only`; o gerente só atribui a terceiros;
- o gerente pode atribuir apenas papéis listados em `manages`;
- o alvo recebe as permissões do **papel atribuído**, não as do gerente;
- o gerente não precisa ter as permissões do papel que gerencia;
- auto-atribuição é proibida mesmo que o papel esteja em `manages`;
- `manages` não é `inherits` e não é `delegates to`;
- a atribuição é auditada;
- a operação é `assignRole(target, role)`, com `target != actor`.

---

# 9. Evidence

| Cláusula | Sintaxe | Funcionamento |
| :--- | :--- | :--- |
| `evidence` | `evidence <Name> { ... }` | Fonte de entendimento (arquivo/modelo). |
| `kind` | `kind: form \| ddl \| mer \| figma \| screenshot \| handdrawn` | Tipo da evidência; guia a extração. |
| `file` | `file: "evidence/form.pdf"` | Arquivo/URL de origem. |
| `hash` | `hash: "sha256:..."` | Integridade/proveniência. |
| `confidentiality` | `confidentiality: internal \| restricted` | Classificação; afeta redaction. |
| `source` | `source: "Compras > 2026-10"` | Origem humana/organizacional. |
| `extracted` | `extracted { document X { ... } }` | Candidatos extraídos; sempre propostos. |
| `document` | `document <Name> { ... }` | Documento candidato. |
| `fields` | `fields { costCenter; amount }` | Campos candidatos. |
| `relation` | `relation Appointment.patient -> Patient` | Relação candidata. |
| `links` | `links { process PurchaseApproval.ReviewRequest; service "<uri>" }` | Liga ao modelo existente. |
| `status` | `status: proposed \| confirmed \| rejected \| needsReview` | Estado da revisão humana. |

**Funcionamento:** a LLM propõe; o humano confirma. `extracted` nunca altera o modelo direto; `links` aponta para o elemento que a evidência sustenta. `hash`/`file` garantem proveniência.

```bpcms
evidence PurchaseFormPDF {
  kind: form
  file: "evidence/purchase-form.pdf"
  hash: "sha256:7f3a..."
  confidentiality: internal
  extracted {
    document PurchaseRequest {
      fields { costCenter; amount; justification }
    }
  }
  links { process PurchaseApproval.ReviewRequest }
  status: proposed
}
```

---

# 10. Conversation

| Cláusula | Sintaxe | Funcionamento |
| :--- | :--- | :--- |
| `conversation` | `conversation <Name> { ... }` | Sessão de descoberta. |
| `process` | `process: PurchaseApproval` | Processo em investigação. |
| `goal` | `goal: "..."` | Objetivo da entrevista. |
| `participants` | `participants { Requester; Manager; Buyer }` | Pessoas/papéis envolvidos. |
| `evidence` | `evidence: [PurchaseFormPDF, ERP_DDL]` | Evidências anexadas. |
| `coverage` | `coverage { goal: confirmed; exceptions: unknown }` | Estado por dimensão. |
| `assumptions` | `assumptions: [A-001, A-002]` | Suposições abertas/confirmadas. |
| `status` | `status: in_progress \| ready_for_review \| approved` | Estado da sessão. |
| `next` | `next: "Confirmar alçada"` | Próxima pergunta/ação. |
| `sessions` | `sessions: 3` | Número de sessões realizadas. |
| `confidentiality` | `confidentiality: restricted` | Restrição da sessão. |

**Valores de `coverage`:** `confirmed`, `unknown`, `proposed` (e `em conflito`, quando houver contradição).

```bpcms
conversation PurchaseDiscovery {
  process: PurchaseApproval
  goal: "Entender compras acima de R$ 5 mil"
  participants { Requester; Manager; Buyer }
  evidence: [PurchaseFormPDF, ERP_DDL]
  coverage {
    goal: confirmed
    steps: confirmed
    exceptions: unknown
  }
  assumptions: [A-001]
  status: in_progress
  next: "Confirmar alçada do diretor"
}
```

# 11. Decision

| Cláusula | Sintaxe | Funcionamento |
| :--- | :--- | :--- |
| `decision` | `decision <Name> { ... }` | Tabela de decisão segura. |
| `appliesTo` | `appliesTo: PurchaseRequest.approve` | Ação/documento onde a decisão atua. |
| `when` | `when: amount` ou `when: [tier, margin]` | Entrada(s) avaliadas. |
| `cases` | `cases { <= 5000 BRL -> auto; else -> director }` | Condição → resultado; primeira verdadeira vence. |
| `default` | `default: GeneralQueue` | Resultado quando nenhum caso casa; opcional. |
| `source` | `source: "Política de Compras v3"` | Norma/política que fundamenta. |
| `owner` | `owner: Finance` | Dono da decisão. |
| `->` | `<condição> -> <resultado>` | Mapeia condição a ação/valor. |
| `else` | `else -> <resultado>` | Caso default inline. |

```bpcms
decision ApprovalLimit {
  appliesTo: PurchaseRequest.approve
  when: amount
  cases {
    <= 5000 BRL -> auto
    <= 50000 BRL -> manager
    else -> director
  }
  source: "Política de Compras v3"
  owner: Finance
}
```

---

# 12. Assumption

| Cláusula | Sintaxe | Funcionamento |
| :--- | :--- | :--- |
| `assumption` | `assumption <Id> { ... }` | Suposição explícita, com validação. |
| `statement` | `statement: "..."` | Afirmação assumida. |
| `impact` | `impact: high \| medium \| low` | Impacto se estiver errada. |
| `status` | `status: open \| confirmed \| rejected \| proposed` | Estado da validação. |
| `validate` | `validate: "Conferir com Logística"` | Como validar. |
| `owner` | `owner: "Ana"` | Responsável. |
| `due` | `due: 2026-11-20` | Prazo. |
| `links` | `links { process X.Y; decision Z }` | Elementos afetados. |
| `evidence` | `evidence: "Tabela de Frete 2026"` | Evidência que confirma/derruba. |

```bpcms
assumption A-001 {
  statement: "Todo frete é cotado por remessa"
  impact: high
  status: open
  validate: "Conferir com Logística"
  owner: "Ana"
  due: 2026-11-20
  links { process OrderFulfillment.QuoteFreight }
}
```

---

# 13. Parameter / Secret

`Parameter` é configuração pública; `Secret` é configuração sensível. Ambos são declarados **dentro de um `Service`**, o valor é resolvido pelo ambiente e **nunca** entra no lock. A `Application` não declara nenhum dos dois; no `main`, ela faz `set` em um parâmetro/segredo já declarado.

## 13.1 Parameter

| Cláusula | Sintaxe | Funcionamento |
| :--- | :--- | :--- |
| `parameter` | `parameter <Name> { ... }` | Configuração tipada; artefato público. |
| `title` | `title: { pt-BR: "...", en: "..." }` | I18n obrigatório; substitui `description`. |
| `type` | `type: Shape` | Shape do valor; obrigatório. |
| `required` | `required` | Falha no boot/uso se não houver valor; mutuamente exclusivo com `default`. |
| `default` | `default: "..."` | Valor fallback; mutuamente exclusivo com `required`. |
| `min` / `max` | `min: 0, max: 10` | Constraints do Parameter, como no field. |
| `maxLength` | `maxLength: 120` | Limite para texto, como no field. |

```bpcms
parameter ErpUrl {
  title: { pt-BR: "Endereço do sistema de gestão", en: "ERP URL" }
  type: string
  default: "https://erp.example.com"
}

parameter MaxRetries {
  title: { pt-BR: "Máximo de tentativas", en: "Max retries" }
  type: std.Count
  default: 3
  min: 0
  max: 10
}
```

- `type` é obrigatório;
- `required` e `default` são mutuamente exclusivos;
- `min`/`max`/`maxLength` são constraints do Parameter;
- o Shape valida o intrínseco;
- Parameter não referencia outro Parameter na v1;
- `source` não existe: Parameter é sempre config;
- `set` da Application valida tipo e constraints.

## 13.2 Secret

| Cláusula | Sintaxe | Funcionamento |
| :--- | :--- | :--- |
| `secret` | `secret <Name> { ... }` | Segredo tipado; artefato público, valor privado. |
| `title` | `title: { pt-BR: "...", en: "..." }` | I18n obrigatório. |
| `type` | `type: Shape` | Shape do valor; obrigatório, mas o valor nunca serializa. |
| `required` | `required` | Falha no boot/uso se não houver valor. |
| `min` / `max` | `min: 0, max: 10` | Constraints do Secret. |
| `maxLength` | `maxLength: 120` | Limite para texto. |

```bpcms
secret PaymentKey {
  title: { pt-BR: "Chave de pagamento", en: "Payment key" }
  type: string
  required
}
```

- `Secret` não é Shape; `type` aponta para um Shape;
- `default` não existe;
- `source` não existe;
- `required` é bare;
- o valor nunca serializa, loga, exporta ou vai à LLM;
- `secret.NOME` é a única forma de `set`;
- o lock guarda o ID do artefato, não o valor.

## 13.3 Application

```bpcms
set Billing.TaxRegime = "SIMPLES"
set Payments.GatewayKey = secret.GATEWAY_KEY
```

- `set` só no `main`;
- `Parameter` usa valor literal/tipado;
- `Secret` usa `secret.NOME`;
- literal para Secret é erro;
- Parameter/Secret precisam existir no serviço importado.

# 14. Environment

| Cláusula | Sintaxe | Funcionamento |
| :--- | :--- | :--- |
| `environment` | `environment <Name> { ... }` | Define um ambiente. |
| `runtime` | `runtime: interpreted \| generated` | Interpretado (dev/QA) ou gerado (prod). |
| `build` | `build: none \| verify \| reproducible` | Sem build, build de verificação ou reproduzível. |
| `database` | `database: sqlite` ou `postgres { ha: true }` | Banco e opções de HA. |
| `ha` | `postgres { ha: true }` | Alta disponibilidade. |
| `data` | `data: seed "..." \| fixtures \| anonymized \| live \| demo \| replica` | Estratégia de dados. |
| `integrations` | `integrations: mock \| stub \| sandbox \| live` | Comportamento das integrações. |
| `auth` | `auth: local \| oidc` | Autenticação. |
| `secrets` | `secrets: [ERP_URL, PAYMENT_KEY]` | Segredos necessários. |
| `promoteTo` | `promoteTo: qa` | Próximo ambiente da promoção. |
| `observability` | `observability: console \| test-report \| full \| otel \| minimal` | Telemetria. |
| `clock` | `clock: virtual` | Relógio virtual para testes. |
| `approvals` | `approvals: [QA, Owner]` | Aprovações para promover. |
| `backup` | `backup { rpo: 15m, rto: 1h }` | Objetivos de backup. |
| `rpo` / `rto` | `rpo: 15m`, `rto: 1h` | Perda máxima / tempo de restauração. |
| `reset` | `reset: nightly` | Reset periódico (sandbox). |
| `activation` | `activation: manual` | Ativação manual (DR). |
| `restoreFrom` | `restoreFrom: "backup/prod"` | Origem de restauração. |

```bpcms
environment dev {
  runtime: interpreted
  build: none
  database: sqlite
  data: seed "seeds/dev"
  integrations: mock
  auth: local
  secrets: [ERP_URL, PAYMENT_KEY]
  promoteTo: qa
  observability: console
}
```

---

# 15. Builder

| Cláusula | Sintaxe | Funcionamento |
| :--- | :--- | :--- |
| `builder` | `builder <Name> { ... }` | Plugin de realização. |
| `version` | `version: 1.0.0` | Versão do construtor. |
| `target` | `target: browser \| server \| database \| mobile \| terminal \| document` | Alvo de geração/interpretação. |
| `mode` | `mode: [interpret, generate]` | Modos suportados. |
| `capabilities` | `capabilities: [gui, sync, identity, attachments, timezone, regionalization, units]` | Capabilities de alto nível que oferece (ver 1.7). |
| `consumes` | `consumes: [Shape, Document, View, Process]` | Artefatos que sabe realizar. |
| `produces` | `produces: [html, css, js]` | Saídas geradas. |
| `limitations` | `limitations { widgets: [...]; process: readOnly }` | O que não suporta; evita promessa falsa. |
| `widgets` | `widgets: [SignaturePad, BarcodeScanner]` | Widgets suportados/limitados. |
| `process` | `process: readOnly` | Processo só leitura naquele alvo. |
| `images` | `images: unsupported` | Imagens indisponíveis. |

```bpcms
builder WebApp {
  version: 1.0.0
  target: browser
  mode: [interpret, generate]
  capabilities: [gui, sync, identity, attachments, timezone, regionalization, units]
  consumes: [Shape, Document, View, Process, Theme]
  produces: [html, css, js]
  limitations { process: readOnly }
}
```

---

# 16. Theme

| Cláusula | Sintaxe | Funcionamento |
| :--- | :--- | :--- |
| `theme` | `theme <Name> { ... }` | Asset do **construtor**; tokens de apresentação, não artefato de serviço. |
| `mode` | `mode: light \| dark \| print` | Modo visual. |
| `supports` | `supports: [WebApp, MobileShell]` | Builders que consomem o tema. |
| `tokens` | `tokens { color.primary: "#2563eb"; radius.md: 8 }` | Chave pontuada → valor. |
| `icons` | `icons: "lucide"` | Pacote de ícones. |
| `accessibility` | `accessibility: WCAG_AA` | Meta de acessibilidade. |
| `page` | `page: A4` | Formato de página (print/PDF). |
| `header` | `header: "logo"` | Cabeçalho de impressão. |
| `footer` | `footer: "page-number"` | Rodapé de impressão. |
| `density` | `density: compact` | Densidade de layout. |

> Theme é asset do **construtor**, não artefato de serviço. Fontes, cores e tokens são implementação gerada; a app só referencia o ID.

```bpcms
theme NuvulaLight {
  mode: light
  supports: [WebApp, MobileShell]
  tokens {
    color.primary: "#2563eb"
    color.surface: "#ffffff"
    radius.md: 8
  }
  icons: "lucide"
}
```

---

# 17. Locale

| Cláusula | Sintaxe | Funcionamento |
| :--- | :--- | :--- |
| `locale` | `locale <tag> { ... }` | Configuração do app/construtor; não é artefato de serviço. |
| `fallback` | `fallback: en-US` | Idioma de fallback. |
| `currency` | `currency: BRL` | Moeda padrão. |
| `units` | `units: metric \| imperial` | Sistema de unidades padrão. |
| `date` | `date: "dd/MM/yyyy"` | Formato de data. |
| `time` | `time: "HH:mm"` | Formato de hora. |
| `direction` | `direction: ltr \| rtl` | Direção de layout. |
| `decimal` | `decimal: ","` | Separador decimal. |
| `thousands` | `thousands: "."` | Separador de milhar. |
| `messages` | `messages: "locales/pt-BR.json"` | Catálogo de mensagens. |

> Locale é configuração da app/construtor; não é serviço. `I18n` de títulos é independente.

```bpcms
locale pt-BR {
  fallback: en-US
  currency: BRL
  units: metric
  date: "dd/MM/yyyy"
  time: "HH:mm"
  direction: ltr
  decimal: ","
  thousands: "."
  messages: "locales/pt-BR.json"
}
```

---

# 18. Widget custom

| Cláusula | Sintaxe | Funcionamento |
| :--- | :--- | :--- |
| `widget` | `widget <Name> { ... }` | Componente de interação de alto nível. |
| `binds` | `binds: Signature` ou `binds: Mass \| Length` | Shape/documento/coleção que o widget edita. |
| `capabilities` | `capabilities: [gui, attachments]` | Capabilities de alto nível exigidas. |
| `platforms` | `platforms { web: canvas; mobile: native; cli: unsupported }` | Implementação por plataforma. |
| `events` | `events { signed -> Signature.create }` | Evento do widget → ação. |
| `fallback` | `fallback: FileUpload` | Alternativa quando não suportado. |
| `options` | `options { maxSize: 20mb; mime: [...] }` | Configuração específica do widget. |

**Opções usadas nos exemplos:**

| Opção | Sintaxe | Funcionamento |
| :--- | :--- | :--- |
| `unit` | `unit: required` | Exige unidade na entrada. |
| `precision` | `precision: field` | Usa a precisão do field. |
| `order` | `order: asc` | Ordem da timeline. |
| `groupBy` | `groupBy: day` | Agrupamento da timeline. |
| `showActor` | `showActor: true` | Exibe o autor do evento. |
| `maxSize` | `maxSize: 20mb` | Tamanho máximo de arquivo. |
| `mime` | `mime: [application/pdf]` | Tipos aceitos. |
| `antivirus` | `antivirus: true` | Exige varredura. |

```bpcms
widget SignaturePad {
  binds: Signature
  capabilities: [gui, attachments]
  platforms {
    web: canvas
    mobile: native
    cli: unsupported
  }
  events { signed -> Signature.create }
  fallback: FileUpload
}
```

---

# 19. Release / Version

| Cláusula | Sintaxe | Funcionamento |
| :--- | :--- | :--- |
| `release` | `release "<uri>"@<selector> { ... }` | Artefato imutável de release. |
| `<selector>` | `1.4.0`, `semver("...")`, `tag("...")`, `commit("...")` | Versão publicada. |
| `application` | `application: "<uri>"` | Aplicação alvo. |
| `service` | `service: "<uri>"` | Serviço alvo (release de serviço). |
| `lock` | `lock: "bpcms.lock"` | Lock do app; fixa uma versão por serviço e os IDs. |
| `migrations` | `migrations: [M-2026-11-01]` | Migrações aplicadas. |
| `approvals` | `approvals: [QA, Owner]` | Aprovações exigidas. |
| `strategy` | `strategy: rolling \| immediate \| canary(10%, 30m)` | Estratégia de implantação. |
| `rollback` | `rollback: "<uri>"@<selector>` | Versão de retorno. |
| `changes` | `changes { document X { add ...; index ... } }` | Mudanças de schema/artefato. |
| `add` | `add <field>: <Shape>` | Adiciona campo em migração. |
| `index` | `index <field>` | Adiciona índice em migração. |
| `compatibility` | `compatibility: backward` | Compatibilidade declarada. |
| `type` | `type: hotfix \| migration \| rollback` | Tipo de release. |
| `reason` | `reason: "..."` | Justificativa (hotfix/rollback). |
| `backfill` | `backfill: chunked(1000)` | Estratégia de preenchimento. |
| `dryRun` | `dryRun: required` | Exige simulação antes de aplicar. |
| `to` | `to: "<uri>"@<selector>` | Alvo de rollback. |
| `preserveData` | `preserveData: true` | Mantém dados no rollback. |
| `guardrails` | `guardrails { errorRate: < 1%; p95Latency: < 500ms }` | Critérios de canary. |

> **Migração é release, não artefato de topo.** Use `release { type: migration, changes, backfill, dryRun, rollback }`. A gramática detalhada das operações (`add`, `index`, `rename`, `drop`, `change`) fica no SPECS.

```bpcms
release "example.com/services/catalog"@1.2.0 {
  service: "example.com/services/catalog"
  changes { add Product.barcode }
  compatibility: backward
  migrations: [M-2026-10-12]
  approvals: [Owner]
  rollback: "example.com/services/catalog"@1.1.3
}
```

```bpcms
release "example.com/app/loja-com-antifraude"@1.0.0 {
  application: "example.com/app/loja-com-antifraude"
  lock: "locks/loja-com-antifraude/1.0.0.lock"
  migrations: [M-2026-11-01]
  approvals: [QA, Owner]
  strategy: rolling
  rollback: "example.com/app/loja-com-antifraude"@0.9.0
}
```

# 20. Integration

| Cláusula | Sintaxe | Funcionamento |
| :--- | :--- | :--- |
| `integration` | `integration <Name> { ... }` | Conector com sistema/dispositivo. |
| `protocol` | `protocol: rest \| smtp \| webhook \| mqtt` | Protocolo de transporte. |
| `capabilities` | `capabilities { equipment }` | Capacidades exigidas pela integração. |
| `baseUrl` | `baseUrl: env.ERP_URL` | Endpoint base; resolvido por ambiente. |
| `host` | `host: env.SMTP_HOST` | Host de protocolo não-HTTP. |
| `topic` | `topic: "scales/+/reading"` | Tópico de mensageria. |
| `from` | `from: "no-reply@..."` | Remetente de e-mail. |
| `auth` | `auth: oauth2 \| api-key \| password \| token \| certificate` | Autenticação. |
| `timeout` | `timeout: 10s` | Tempo máximo por chamada. |
| `retry` | `retry: exponential(3)` | Política de retry. |
| `idempotency` | `idempotency: header "Idempotency-Key"` | Chave de idempotência. |
| `operations` | `operations { getSupplier(id): Supplier }` | Operações tipadas. |
| `events` | `events { Issued -> createInvoice }` | Evento de negócio → operação. |
| `webhooks` | `webhooks { payment.failed -> Payment.failed }` | Evento externo → evento interno. |
| `fallback` | `fallback { onUnavailable: ManualQuote; cache: 5m }` | Comportamento em falha. |
| `onUnavailable` | `onUnavailable: ManualQuote` | Ação/estado alternativo. |
| `cache` | `cache: 5m` | Cache de resposta. |
| `limits` | `limits { rate: 100/min; retry: exponential(5) }` | Limites de uso. |
| `rate` | `rate: 100/min` | Vazão máxima. |
| `quality` | `quality { staleAfter: 5m; calibrate: monthly }` | Frescor/calibração de dispositivo. |
| `staleAfter` | `staleAfter: 5m` | Tempo para considerar leitura vencida. |
| `calibrate` | `calibrate: monthly` | Periodicidade de calibração. |

```bpcms
integration ERP {
  protocol: rest
  baseUrl: env.ERP_URL
  auth: oauth2
  timeout: 10s
  retry: exponential(3)
  operations {
    getSupplier(supplier: Supplier): Supplier
    createInvoice(invoice: Invoice): ExternalId
  }
  events { Issued -> createInvoice }
}
```

---

# 21. Report / Dashboard

Report é um **Document com caching de cálculo**, não um artefato separado.

| Cláusula | Sintaxe | Funcionamento |
| :--- | :--- | :--- |
| `document` | `document CashFlow { ... }` | Documento calculado. |
| `type` | `type: report` | Marca o Document como relatório/dashboard. |
| `computed from` | `computed from Invoice, Payment` | Fontes do cálculo. |
| `freshness` | `freshness: 1h` | TTL do cache. |
| `exposes` | `exposes: [balance, calculatedAt]` | Campos calculados. |
| `report` | `report { sources; metrics; filters }` | Configuração de relatório, opcional. |

```bpcms
document CashFlow {
  type: report
  title: { pt-BR: "Fluxo de caixa", en: "Cash flow" }
  computed from Invoice, Payment, Bill
  freshness: 1h
  exposes: [balance, calculatedAt]

  report {
    filters { period; account }
    metrics {
      inflow: sum(Payment.amount) where status == captured
      outflow: sum(Bill.amount) where status == paid
      balance: inflow - outflow
    }
  }
}
```

Não há `report <Name> { ... }` como artefato de topo. Os arquivos `*-reports.bpcms` contêm `document` com `type: report`.


---

# 22. Event

Eventos de workflow ficam na `transition` do `value`. Eventos sem workflow ficam em `events` no Document.

## 22.1 Evento de workflow

```bpcms
value Issued {
  title: { pt-BR: "Emitida", en: "Issued" }

  transition Paid {
    title: { pt-BR: "Nota paga", en: "Invoice paid" }
    payload {
      paidAt: DateTime
    }
  }
}
```

- a chave da transição é o estado de destino;
- o evento padrão é o destino em PascalCase;
- `event:` sobrescreve o nome do evento;
- `payload` é o contrato do evento;
- a action usa `transition: Paid`;
- a action não usa `emit` para workflow.

## 22.2 Evento sem workflow

```bpcms
document Store {
  events {
    Renamed {
      title: { pt-BR: "Loja renomeada", en: "Store renamed" }
      key: this.id
      payload {
        oldName: string
        newName: string
      }
    }
  }

  actions {
    rename(name: string) {
      title: { pt-BR: "Renomear", en: "Rename" }
      roles: [Manager]
      emit Renamed {
        oldName: old.name
        newName: new.name
      }
    }
  }
}
```

## 22.3 FQN

```text
SERVICO.DOCUMENTO.EVENTO
```

Exemplo:

```text
Billing.Invoice.Paid
```

Canônico:

```text
example.com/services/billing#Invoice.Paid
```

- `SERVICO` é o dono do Document;
- alias é sintaxe local;
- em cada serviço, o código usa o alias daquele serviço;
- na aplicação, o código usa o alias da aplicação;
- o alias não altera a identidade canônica;
- o envelope publica a forma canônica;
- `producer` é implícito;
- `consumers` não pertencem ao evento.

Exemplo:

```text
App1: BillingOnApp1.Invoice.Paid
S1:   BillingOnS1.Invoice.Paid
S2:   BillingOnS2.Invoice.Paid
```

Todos resolvem para:

```text
example.com/services/billing#Invoice.Paid
```

## 22.3.1 Evento duplicado

O mesmo evento pode aparecer em transições de origens diferentes:

```bpcms
value Issued {
  transition Paid { ... }
}

value Overdue {
  transition Paid { ... }
}
```

O compilador:

- exige o mesmo `event`;
- exige o mesmo payload;
- exige o mesmo título;
- funde em `Invoice.Paid`;
- gera `E_EVENT_SCHEMA_CONFLICT` se divergir.

## 22.4 Regras

- evento de workflow não fica em `events`;
- evento sem workflow fica em `events` e usa `emit`;
- action com `transition:` não usa `emit`;
- payload de workflow fica na transição;
- payload sem workflow fica em `events`;
- `key` usa `this.id` ou `referencia.id`;
- processo/integração/job assinam `Document.Event`;
- evento inexistente = erro.

---

# 23. Job / Schedule

| Cláusula | Sintaxe | Funcionamento |
| :--- | :--- | :--- |
| `job` | `job <Name> { ... }` | Tarefa assíncrona/agendada. |
| `capabilities` | `capabilities { scheduling }` | Capacidades exigidas pelo Job; falha de build se ausentes. |
| `schedule` | `schedule: "0 2 1 * *"` | Cron; `*/15 * * * *` para intervalo. |
| `timezone` | `timezone: America/Fortaleza` | Fuso do cron. |
| `action` | `action: Billing.closeMonth()` | Operação executada. |
| `timeout` | `timeout: 30m` | Tempo máximo. |
| `retry` | `retry: 3` | Tentativas. |
| `idempotent` | `idempotent: true` | Pode repetir sem duplicar efeito. |
| `deadLetter` | `deadLetter: true` | Envia falha para DLQ. |
| `input` | `input { audience: [Manager] }` | Dados de entrada. |
| `batch` | `batch: 500` | Tamanho do lote. |
| `dryRun` | `dryRun: env == "qa"` | Simula sem aplicar. |
| `filter` | `filter { failedAt > now() - 24h }` | Seleção de itens. |
| `maxAttempts` | `maxAttempts: 5` | Limite de tentativas. |
| `backoff` | `backoff: exponential` | Estratégia de espera. |
| `cursor` | `cursor: "erp.supplier.cursor"` | Checkpoint incremental. |
| `condition` | `condition: SalesDashboard.viewers > 0` | Executa só se verdadeiro. |

```bpcms
job CloseMonth {
  capabilities { scheduling }
  schedule: "Todo último dia útil do mês"
  timezone: America/Fortaleza
  action: Billing.closeMonth()
  timeout: 30m
  retry: 3
  idempotent: true
  deadLetter: true
}
```

---

# 24. Policy / Rule

| Cláusula | Sintaxe | Funcionamento |
| :--- | :--- | :--- |
| `policy` | `policy <Name> { ... }` | Regra transversal de autorização/compliance. |
| `appliesTo` | `appliesTo: Payment.release` ou `[...]` | Alvos da política. |
| `rule` | `rule: approver != creator` | Condição que deve valer. |
| `severity` | `severity: block \| require \| enforce` | Bloqueia, exige aprovação ou aplica. |
| `message` | `message: "..."` | Mensagem ao usuário. |
| `audit` | `audit: true` | Registra avaliação. |
| `exception` | `exception: legalHold` | Exceção nomeada. |
| `notification` | `notification: [Email, WhatsApp]` | Canais de aviso. |
| `mask` | `mask: default` | Mascaramento padrão. |

```bpcms
policy FourEyesPayment {
  appliesTo: Payment.release
  rule: approver != creator
  severity: block
  message: "Pagamento exige segundo aprovador"
}
```

---

# 25. Expressões, operadores e valores

| Item | Sintaxe | Funcionamento |
| :--- | :--- | :--- |
| Igualdade | `a == b`, `a != b` | Comparação tipada; Shapes definem igualdade. |
| Ordem | `<`, `<=`, `>`, `>=` | Requer dimensão/tipo compatível. |
| Pertencimento | `status in [valid, archived]` | Lista/intervalo. |
| Lógica | `and`, `or`, `not` | Curto-circuito. |
| Implicação | `a implies b` | `not a or b`; usado em regras. |
| Transição | `A -> B` | Fluxo, caso ou ação. |
| Soma | `sum(Invoice.total)`, `sum(X) where cond` | Agregação; unidade do Shape; `where` filtra. |
| Contagem | `count(Order)`, `count(X) where cond` | Conta registros; `where` filtra. |
| Média | `avg(SupportTicket.resolutionTime)` | Média; unidade preservada. |
| Percentual | `percentage(SupportTicket.slaMet)` | Razão 0–100. |
| Bucket | `bucket(date, [30d, 60d, 90d])` | Faixas nomeadas. |
| Agrupamento | `... by region` | Dimensão de agregação. |
| Filtro | `... where cond` | Filtra registros antes de agregar. |
| Aritmética | `quantity * unitPrice`, `subtotal * 10%` | Multiplicação/divisão com unidade; resultado tipado. |
| Propriedade | `items.count` | Cardinalidade de coleção. |
| Relógio | `now()`, `now() - 24h` | Relógio do runtime (virtual em QA). |
| Usuário | `currentUser` | Ator autenticado. |
| Moeda | `5000 BRL`, `10,00 BRL` | Valor + unidade; não é string. |
| Percentual | `10%`, `< 1%` | Razão; cuidado com 0,1 x 10%. |
| Duração | `15m`, `2h`, `3d`, `5y` | Unidade temporal explícita. |
| Data | `2026-11-20` | Data ISO. |
| Cron | `"0 2 1 * *"` | Agenda; timezone separado. |
| Nulo | `null` | Ausência; `?` declara opcional. |
| Infinito | `unlimited` | Sem limite (apenas onde previsto). |
| Máscara | `masked` | Campo mascarado por política. |
| Retenção | `retainFor(5y) then anonymize` | Política de retenção. |

**Sem código arbitrário:** expressões são limitadas a operadores/funções declaradas. Código de extensão só existe em construtores/extensões de serviço, nunca em `document`, `process` ou `policy`.

---

# 26. Validação e erros

| Regra | Quando falha | Erro |
| :--- | :--- | :--- |
| Identidade | URI inválida, duplicada ou com protocolo | `E_IDENTITY` |
| Referência | `Shape`/`Document`/`View`/`Role`/`Event` inexistente | `E_REFERENCE` |
| ID duplicado | campo `id` declarado no document | `E_DUPLICATE_ID_FIELD` |
| ID duplicado | `shape <Document>Id` declarado | `E_DUPLICATE_ID_SHAPE` |
| ID circular | campo `<document>Id` dentro do próprio document | `E_SELF_ID_FIELD` |
| Link | alvo não encontrado na resolução | `E_LINK_TARGET_MISSING` |
| Lookup | document base sem campo `lookup` ou campo inválido | `E_LOOKUP_MISSING` / `E_LOOKUP_FIELD` |
| Enum | value sem `title`/`icon`/`ordinal`, ordinal inválido ou duplicado | `E_ENUM_VALUE_TITLE` / `E_ENUM_VALUE_ICON` / `E_ENUM_VALUE_ORDINAL` / `E_ENUM_VALUE_DUPLICATE` |
| Icon | value aponta para Icon inexistente | `E_ICON_REFERENCE` |
| Icon | Theme/builder não resolve a imagem do Icon | `E_ICON_UNRESOLVED` |
| Icon | dois icons com o mesmo nome no mesmo serviço | `E_DUPLICATE_ICON` |
| Method | assinatura/corpo/retorno inválido | `E_METHOD_SIGNATURE` / `E_METHOD_BODY` / `E_METHOD_RETURN` |
| stream | stream truncado, versão inválida ou bytes sobrando | `E_STREAM_INVALID` |
| Format/Parse | composto sem método próprio | `E_FORMAT_MISSING` / `E_PARSE_MISSING` |
| Lookup | bloco `lookup { ... }` usado | `E_LOOKUP_SYNTAX` |
| Extensão | pai inexistente, múltiplo, ciclo, fora do serviço ou tentativa de redeclarar herdado | `E_EXTENDS` |
| Tipo | campo recebe Shape incompatível | `E_TYPE` |
| Constructor | propriedade/valor inválido em `new` | `E_CONSTRUCTOR_PROPERTY` |
| Workflow | initial/transition inválido, state inexistente ou self-transition | `E_WORKFLOW_INITIAL` / `E_WORKFLOW_TRANSITION` |
| Classification | `initial`/`transition` em classification | `E_CLASSIFICATION_TRANSITION` |
| Runtime | mudança de estado não permitida | `E_INVALID_TRANSITION` |
| Moeda | metadado ausente, moeda incompatível, parse/scale inválido | `E_CURRENCY_METADATA_MISSING` / `E_CURRENCY_MISMATCH` / `E_MONEY_PARSE` / `E_CURRENCY_INFIX_SCALE` |
| Reintroduce | conflito ao completar definição externa | `E_REINTRODUCE_CONFLICT` |
| Unidade | `unit` + `units`, fórmula inválida, parse sem unidade | `E_UNIT_UNITS_CONFLICT` / `E_UNIT_FORMULA` / `E_PARSE_UNIT` |
| Operador | operandos incompatíveis ou operador inexistente | `E_OPERATOR` |
| Regionalization | locale/regionalization indeterminada | `E_REGIONALIZATION_UNKNOWN` |
| Dimensão | `Mass` onde se espera `Length`; soma incompatível | `E_DIMENSION` |
| Unidade | unidade não pertence à dimensão | `E_UNIT` |
| Índice | campo inexistente/não indexável; índice duplicado | `E_INDEX` |
| Ação | parâmetro/retorno inválido; ação sem permissão | `E_ACTION` |
| Processo | nó inalcançável; gateway sem default; evento desconhecido | `E_PROCESS` |
| Role | papel inexistente; quatro-olhos impossível | `E_ROLE` |
| Versão | seletor inválido; versão não resolvida | `E_VERSION` |
| Origem | `source` inválido; README ausente em repo | `E_SOURCE` |
| Sync | `sync` diferente de `live/read` | `E_SYNC` |
| Política | severidade inválida; exceção desconhecida | `E_POLICY` |
| Ciclo | import/computed circular | `E_CYCLE` |
| Tradução | locale sem `title` / builder estrito | `W_TRANSLATION` / `E_TRANSLATION` |
| Tipo reservado | `shape I18n` declarado | `E_RESERVED_TYPE` |
| Locale | chave de `title` fora de `locales` | `E_LOCALE_UNDECLARED` |
| Título | sem entrada para o locale default | `E_TITLE_MISSING` |

**Classes de runtime:** `validation_error` (400), `conflict` (409), `forbidden` (403), `not_found` (404), `dependency_unavailable` (503), `rate_limited` (429), `internal` (500). Erros de linguagem são bloqueantes no `check`; erros de runtime seguem o modelo do BRAINSTORM (2.6.11).

---

# 27. Cobertura

| Artefato | Cobertura | Exemplos |
| :--- | :--- | :--- |
| Application | ✅ | 8 |
| Service | ✅ | 8 |
| Process | ✅ | 7 |
| Document | ✅ | 6 |
| Shape | ✅ | 9 |
| View | ✅ | 6 |
| Role/Organogram | ✅ | 6 |
| Evidence | ✅ | 6 |
| Conversation | ✅ | 6 |
| Decision | ✅ | 6 |
| Assumption | ✅ | 6 |
| Parameter | ✅ | 6 |
| Environment | ✅ | 6 |
| Builder | ✅ | 6 |
| Theme | ✅ | 6 |
| Locale | ✅ | 6 |
| Widget | ✅ | 6 |
| Release | ✅ | 6 |
| Integration | ✅ | 6 |
| Report | ✅ | 6 |
| Job | ✅ | 6 |
| Policy | ✅ | 6 |
| Icon | ✅ | 6 |

> Cobertura = todas as cláusulas de linguagem usadas nos exemplos estão documentadas. Campos de domínio (`name`, `email`, `id`) não entram na contagem. As cláusulas comuns (version/source/type/mode/status) aparecem na seção 1 e são referenciadas nas seções seguintes.

# 28. Icon

`Icon` é um artefato **semântico** definido dentro de um `Service`. O nome do Icon é o identificador; o construtor/Theme decide qual imagem usar. Não há `symbol` nem binário no DSL.

| Cláusula | Sintaxe | Funcionamento |
| :--- | :--- | :--- |
| `icon` | `icon <Name> { ... }` | Ícone público do serviço. |
| `title` | `title: { pt-BR: "...", en: "..." }` | I18n obrigatório (ver 1.9). |

```bpcms
icon Draft {
  title: { pt-BR: "Rascunho", en: "Draft" }
}
```

**Nomes:** `Icon` não usa prefixo `Icon` nem prefixo do serviço. O escopo do nome é o **serviço**: `Draft` em Billing e `Draft` em Catalog são icons diferentes; dentro do mesmo serviço, dois icons com o mesmo nome são erro (`E_DUPLICATE_ICON`). `icon Email` também coexiste com `shape Email` e `integration Email`; o namespace é por tipo dentro do serviço.

**Resolução:** `value.icon` → nome do `Icon` → mapeamento do builder/Theme → imagem. O Icon não carrega imagem nem binário; é um token semântico. O builder pode trocar o conjunto visual sem mudar o DSL.

### 28.1 Icons file-based (SVG)

Além do `icon` declarado, qualquer arquivo `.svg` sob a raiz do serviço — em qualquer subpasta — vira um **Icon implícito**.

```text
services/billing/
  billing-service.bpcms       <- raiz do serviço
  status/
    draft.svg                 -> status.draft
    paid.svg                  -> status.paid
    overdue.svg               -> status.overdue
  finance/
    invoice/
      issued.svg              -> finance.invoice.issued
```

- Arquivos e pastas usam **kebab-case**.
- O identificador é o caminho relativo, sem `.svg`, com segmentos **camelCase** unidos por `.`.
- Referência local: `icon: status.draft`.
- Referência por outro serviço: `icon: AliasDoServico.status.draft` (alias em `imports`).
- O `alt`/`aria-label`/`<title>` é o title do Icon; **não usa I18n**.
- Precedência do title: `aria-label` → `<title>` → `alt` → filename humanizado.
- Sem `aria-label`, `<title>` ou `alt`, o filename é o fallback:
  - `low-stock.svg` → `"Low Stock"`.
- Um SVG e um `icon` declarado com o mesmo identificador no mesmo serviço são conflito (`E_DUPLICATE_ICON`).
- O builder sanitiza o SVG; script/recursos externos não entram no contrato.

```bpcms
shape InvoiceStatus {
  value Draft {
    title: { pt-BR: "Rascunho", en: "Draft" }
    icon: status.draft
    ordinal: 10
  }
  value Issued {
    title: { pt-BR: "Emitida", en: "Issued" }
    icon: finance.invoice.issued
    ordinal: 20
  }
}
```

**Erros:** `E_ICON_REFERENCE` para `icon:` que não resolve; `E_DUPLICATE_ICON` para conflito de identificador; `E_ICON_UNRESOLVED` para falha do builder/Theme.

**Erros:** `E_ICON_REFERENCE` (value aponta para Icon inexistente), `E_ICON_UNRESOLVED` (Theme/builder não resolve a imagem do Icon), `E_DUPLICATE_ICON` (dois icons com o mesmo nome no mesmo serviço), `E_ENUM_VALUE_TITLE`, `E_ENUM_VALUE_ICON` e `E_ENUM_VALUE_DUPLICATE`.

# 29. bpcms.org/std — biblioteca padrão

`bpcms.org/std` é um **pseudo-serviço reservado**, fornecido pelo construtor. Ele não é publicado, não tem endpoint, não entra no lock do app e não pode ser sobrescrito.

Quem usa precisa importar:

```bpcms
imports {
  service "bpcms.org/std" as std
}
```

O alias é obrigatório; o uso é sempre qualificado:

```bpcms
fields {
  weight: std.Mass
  length: std.Length
  discount: std.Percentage
  currency: std.Currency
}
```

Não existe `Mass` implícito vindo da std. Sem import, `std.Mass` não resolve.

## 29.1 Modos e unidades

A biblioteca cobre, no mesmo Shape, unidades de vários sistemas. `units` mapeia unidade → fator relativo à base; o field escolhe a unidade do valor.

| Modo | Exemplos de unidades |
| :--- | :--- |
| SI | m, kg, s, A, K, mol, cd; N, Pa, J, W, C, V, F, Ω, Wb, T, Hz |
| Imperial/US | in, ft, yd, mi, lb, oz, gal, qt, pt, °F, psi, BTU, hp |
| CGS | cm, g, dyn, erg, P, St, G, Mx |
| Astronômico | AU, ly, pc, Msun |
| Atômico/nuclear | eV, keV, MeV, Da, Ci, rad, rem |
| Química | mol/L, mol/kg, pH, ppm, ppb, kat |
| Dados | bit, B, KB/KiB, MB/MiB, GB/GiB, bps |
| Estatística | count, %, probability, ratio, rate, z-score |

**Convenção de tokens:** `_` representa divisão ou produto; `2`/`3` são potências; `u` é micro; `ohm` é Ω; `deg` é grau. Exemplos: `m_s`, `m_s2`, `kg_m3`, `W_mK`, `cd_m2`, `ft_lbf`.

## 29.2 Shapes da biblioteca

| Grupo | Shapes |
| :--- | :--- |
| Geometria/medidas | `Length`, `Mass`, `Volume`, `Area`, `Angle`, `Temperature`, `Duration` |
| Mecânica | `Speed`, `Acceleration`, `Force`, `Pressure`, `Energy`, `Power`, `Frequency`, `Density`, `Momentum`, `Torque` |
| Fluidos/calor | `HeatCapacity`, `Entropy`, `ThermalConductivity`, `DynamicViscosity`, `KinematicViscosity`, `SurfaceTension` |
| Eletromagnetismo | `ElectricCurrent`, `ElectricCharge`, `Voltage`, `Resistance`, `Capacitance`, `Inductance`, `MagneticFlux`, `MagneticFluxDensity` |
| Luz/som | `LuminousIntensity`, `LuminousFlux`, `Illuminance`, `Luminance`, `SoundPressure`, `SoundLevel`, `AngularVelocity` |
| Química | `AmountOfSubstance`, `MolarMass`, `MolarConcentration`, `Molality`, `pH`, `CatalyticActivity` |
| Radiação | `Radioactivity`, `AbsorbedDose`, `EquivalentDose` |
| Dados/estatística | `DataSize`, `DataRate`, `Count`, `Percentage`, `Probability`, `Ratio`, `Rate`, `ZScore` |
| Dinheiro | `Money`, `Currency` |

**`Money`:** tipo de valor monetário com qualquer `Currency`; `base: decimal`. O valor carrega `raw` + `currency`; `toNumber` é automático e retorna `raw`. Sem conversão cambial.

**`Currency`:** classificação da stdlib com URL externa ISO 4217 e `value` com `scale`, sem `title`/`icon`/`ordinal`. Para usar uma moeda, o serviço precisa reintroduzi-la.

```bpcms
shape Currency {
  title: { pt-BR: "Moeda", en: "Currency" }
  classification: "https://www.iso.org/iso-4217-currency-codes.html"
  value BRL { numeric: "986", scale: 2 }
  value USD { numeric: "840", scale: 2 }
  value JPY { numeric: "392", scale: 0 }
  value KWD { numeric: "414", scale: 3 }
  value CVE { numeric: "132", scale: 2 }
}
```

- A stdlib contém os **178 códigos ativos da ISO 4217**.
- Cada value tem `numeric` e `scale`.
- `subunitRatio: 5` em MGA e MRU; default é `10^scale`.
- `CurrencyCode` não é Shape separado.
- `DefaultCurrency` usa `std.Currency`.
- A moeda usada sem `reintroduce` gera `E_CURRENCY_METADATA_MISSING`.
- `Currency` não tem `toOrdinal`.
- `Currency.write`/`read` usam o código como `string`.
- `Currency.format(value)` retorna o próprio código; `Currency.parse(text)` retorna o próprio texto.

### 29.2.1.1 `reintroduce` e formatação de moeda

```bpcms
reintroduce std.Currency {
  value BRL {
    title: { pt-BR: "Real", en: "Brazilian real" }
    prefix: {
      pt-BR: "R$"
      en: "BRL"
    }
    icon: Brl
    ordinal: 10
  }
  value BTC {
    title: { pt-BR: "Bitcoin", en: "Bitcoin" }
    suffix: " BTC"
    icon: Bitcoin
    ordinal: 1000
  }
  value CVE {
    title: { pt-BR: "Escudo cabo-verdiano", en: "Cape Verdean escudo" }
    infix: "$"
    icon: CVE
    ordinal: 30
  }
}
```

- `prefix`, `suffix` e `infix` são mutuamente exclusivos.
- `infix` fica na posição decimal: `CVE 100$00`.
- `infix` exige `scale > 0`.
- `prefix`/`suffix`/`infix` aceitam `string | I18n`.
- `scale` default vem da moeda; o field pode sobrescrever.
- Sem `prefix/suffix/infix`, regionalization decide.
- `sformat` fica fora da v1.

`Money` usa `prefix`/`suffix`/`infix` da moeda:

```bpcms
price: std.Money {
  title: { pt-BR: "Preço", en: "Price" }
  currency: BRL
  scale: 2
}
```

- `Money.toNumber` é automático e retorna `raw`.
- `Money` sempre carrega `currency`.
- `Money` com moedas diferentes em `+`/`-`/comparação gera `E_CURRENCY_MISMATCH`.
- `parse` de Money usa prefix/suffix/infix; texto sem moeda usa `currency` do field; ausência total = `E_MONEY_PARSE`.
- `scale` do field > `scale` da moeda.
- Não arredondar no meio dos cálculos.
- Sem conversão cambial.

Operadores:

```bpcms
R$ 10 + R$ 5      // R$ 15
R$ 10 - R$ 5      // R$ 5
R$ 10 * 2         // R$ 20
R$ 10 / R$ 5      // 2 (número)
R$ 10 + USD 5     // E_CURRENCY_MISMATCH
```

- `Money + Money` e `Money - Money` exigem a mesma moeda.
- `Money * number`, `number * Money` e `Money / number` são permitidos.
- `Money / Money` retorna número.
- Comparações exigem a mesma moeda.

## 29.2.1 Regionalization

`Regionalization` é uma interface/capability do construtor, não um Shape:

```ts
Regionalization {
  current(): Locale
  parseNumber(text: string, locale?: Locale): number
  formatNumber(value: number, style: NumberStyle, locale?: Locale): string
  parseUnit(text: string, units: Units, locale?: Locale): UnitValue
  formatUnit(value: number, unit: Unit, style: UnitStyle, locale?: Locale): string
}
```

- `parse`, `format`, `convert` e `bestUnit` acessam internamente.
- Se não for possível determinar a regionalization: `E_REGIONALIZATION_UNKNOWN`.
- O serviço não precisa importar.


## 29.3 Onde fica

- Definição: `examples/services/std/std-service.bpcms` e `examples/services/std/std-shapes.bpcms`.
- Fica na pasta normal `examples/services/std/`; o construtor reconhece a URI reservada e não a trata como serviço publicável.
- Não é versionado pelo lock; a versão da std é a do construtor/DSL.
- Os Shapes de medida declaram `toNumber(value)` explicitamente.
- `Mass`, `Length`, `Volume`, `Percentage` e `Duration` implementam `write`, `read`, `parse`, `format` e `toNumber`; `Money.toNumber` é automático; `Currency` não é numérico.

> **Migração feita:** `Mass`, `Length`, `Volume`, `Percentage` e `Count` foram removidos dos serviços de negócio e são referenciados como `std.*`. `MoneyBRL` foi removido; valores monetários usam `std.Money`. `CurrencyCode` não é Shape; `Currency` é a classificação. `Email`, `Phone`, `Image`, `LocaleTag` e `YearMonth` continuam nos serviços de negócio.

## O que ainda não é cláusula

* Herança múltipla ou extensão fora do mesmo serviço — **fora da v1**. `document Consumer extends User` é a extensão simples permitida (B-34/B-45).
* `migration` — **decidido**: é um `release` do tipo `migration`, não um artefato de topo (B-35). A gramática detalhada fica no SPECS.
* `alias`/`as` — **decidido e documentado** na seção 1.1.
* `private` — **decidido** para membros; artefato de topo nunca é privado.
* capabilities de UI/widget — **decidido**: vocabulário de alto nível (1.7), sem termos de tecnologia.
* `compensation` — **decidido**: task `type: compensation` com `for` (B-37).




