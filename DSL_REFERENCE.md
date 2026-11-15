# BPCMS — Referência de Cláusulas da DSL

*Sintaxe ilustrativa v0.1.* Não é gramática final nem código executável. Esta referência cobre **todas as cláusulas usadas em [DSL_EXAMPLES.md](DSL_EXAMPLES.md)** e serve de base para o SPECS.

## Como ler

| Notação | Significado |
| :--- | :--- |
| `<uri>` | URI web sem protocolo, literal entre aspas: `"example.com/app/x"` |
| `<name>` | identificador simples, sem URI: `Customer`, `OrderApproval` |
| `<expr>` | expressão segura, sem código arbitrário |
| `[A, B]` | lista |
| `T?` | opcional |
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
| 13–19 | Parameter, Environment, Builder, Theme, Locale, Widget, Release/Version |
| 20–24 | Integration, Report, Event, Job, Policy |
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
| `version` | app, service, process, document, event, builder, release | `version: 1.4.0` ou `version: semver("1.4.0")` | Versão do artefato. `1.4.0` é açúcar para `semver("1.4.0")`. |
| `title` | app, service, shape, document, field, view, event, parameter, role, organogram, icon | `title: { pt-BR: "Nome", en: "Name", short?: {...}, tooltip?: {...} }` | I18n obrigatório; ver 1.9. |
| `owner` | app, service, document, decision, assumption | `owner: "Financeiro"` | Dono humano/time; usado em revisão e promoção. |
| `description` | app | `description: "..."` | Texto livre para documentação e busca. |
| `source` | app/service, document, evidence, view/report, decision | `source: registry \| repository \| auto` (app/service); ver 1.3 | Origem, fonte de dados, dispositivo ou política — o significado depende do artefato. |
| `status` | evidence, conversation, assumption | `status: proposed \| confirmed \| ...` | Estado do artefato de descoberta. Valores variam por artefato. |
| `type` | task, view, release, report | `type: service`, `type: form`, `type: migration` | Discriminador; os valores válidos dependem do artefato. |
| `mode` | builder, theme, report | `mode: [interpret, generate]`, `mode: dark`, `mode: live` | Modo de operação/apresentação. |
| `links` | evidence, assumption | `links { process X.Y; document X.field; view V; decision D; evidence E; service "<uri>" }` | Liga o artefato de descoberta ao modelo; o `check` valida o alvo. |
| `audit` | document, policy | `audit: true` | Exige trilha de auditoria. |
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
| `Event` | start/end/on | `start on OrderApproved` | Referência a evento de negócio. |
| `Integration` | task/integration | `integration: Carrier` | Referência a integração. |
| `T?` | campos | `phone: Phone?` | Opcional; ausência ≠ vazio. |
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
| `overrides` | lista `{ target, by, id }` | Overrides ativos na app. |
| `target` | `target: "<uri>#<Artifact>.<member>"` | Alvo do override. |
| `by` | `by: "<uri>"` | Serviço que substitui. |

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
    "example.com/services/checkout#CheckoutSession.placeOrder": 21
  overrides:
    - target: "example.com/services/checkout#CheckoutSession.placeOrder"
      by: "example.com/services/fraud-checkout"
      id: 41
```

**Regras:** o lock é a única fonte de IDs por app; o serviço **não tem lock, IDs nem versões de imports**. Os IDs são usados para **minificação de código** e **compressão da serialização**; nomes humanos permanecem no fonte. Se dois serviços exigem versões incompatíveis do mesmo serviço, o `check` falha.

## 1.7 Capabilities

`requires` e `capabilities` usam conceitos de **alto nível**, não tecnologia. O construtor mapeia para HTTP, DB, UI kit, fila etc.

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

`provides` é o **contrato de negócio** oferecido (`catalog`, `checkout`, `identity`), não uma capability técnica. Não use `http`, `db`, `search`, `ui`, `files`, `auth`, `realtime`, `jobs`, `camera` ou `push` como capability.

**Extensões.** O conjunto acima é um núcleo aberto. Construtores podem declarar novas capabilities, genéricas (`llm`) ou específicas (`google-maps`, `acme.payments`). Use kebab-case e documente o contrato no construtor. Configuração/segredo da capability usa `Parameter`.

```bpcms
builder AiRuntime {
  capabilities: [llm, oltp, sync]
}

service "example.com/services/assistant" {
  requires { oltp, llm }
  provides { assistant }
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
  requires { location, timezone, regionalization, units, oltp, sync }
}

view VisitForm {
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

**Campos:** o `title` do campo é o label; o `title` do `document`/`shape`/`view`/`event`/`parameter`/`role`/`organogram`/`service` é o nome do artefato. Em `Parameter`, `title` substitui o antigo `description`.

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
| `set` | `set <Alias>.<Parameter> = <valor>` | Inicializa/override de um `Parameter` de serviço; segredo só como `secret.NOME`. |
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
| `imports` | `imports { service "<uri>" as Alias; integration ERP }` | Dependências por URI, **sem versão**; alias local opcional; o lock escolhe a versão. |
| `override` | `override "<uri>#<Artifact>.<member>" { with: "<uri>#<Artifact>.<member>" }` | Substitui um método/ação; ativo quando o serviço substituto está na closure. |
| `requires` | `requires { oltp, attachments, identity }` | Capabilities de alto nível exigidas do construtor/ambiente (ver 1.7). |
| `provides` | `provides { catalog }` | Contrato de negócio oferecido a outros serviços/aplicações. |
| `roles` | `roles { CatalogManager, CatalogViewer }` | Papéis definidos pelo serviço. |

> **Contrato:** todo artefato de topo é público automaticamente; **não existe `exports`**. Privacidade só por membro (`private`).

**Funcionamento:** serviço é um **conceito de negócio**, não biblioteca/package/módulo. Não crie serviço apenas para agrupar Shapes/Icons/Themes/Locales. Ele é uma fatia vertical; todo artefato de topo é público e ele é dono dos seus documentos. O que é `private` dentro de um artefato não entra no contrato público. Importar um serviço não dá acesso de escrita aos dados dele. A URI registry resolve `/index.html` como documentação de API; repositório resolve README + código.

```bpcms
service "example.com/services/catalog" {
  version: 1.2.0
  // document Product, view ProductForm, event ProductUpdated são públicos automaticamente
  imports { service "example.com/services/auth" }
  requires { oltp, attachments, identity }
  provides { catalog }
  roles { CatalogManager, CatalogViewer }
}
```

### 3.1 Override

| Cláusula | Sintaxe | Funcionamento |
| :--- | :--- | :--- |
| `override` | `override "<uri>#<Artifact>.<member>" { ... }` | Declara substituição de método/ação do serviço alvo. |
| `with` | `with: "<uri>#<Artifact>.<member>"` | Implementação substituta; assinatura deve ser compatível (LSP). |
| ativação | — | Ativo se o serviço substituto estiver na closure do app; se só o original estiver presente, o original vale. |

**Regras:** o serviço que override deve importar o alvo; um override por alvo; dois overrides = `E_OVERRIDE_CONFLICT`; o lock registra `target`, `by` e `id`.

```bpcms
service "example.com/services/fraud-checkout" {
  imports { service "example.com/services/checkout" }
  override "example.com/services/checkout#CheckoutSession.placeOrder" {
    with: "example.com/services/fraud-checkout#FraudCheckout.placeOrder"
  }
}
```

# 4. Process

| Cláusula | Sintaxe | Funcionamento |
| :--- | :--- | :--- |
| declaração | `process <Name> { ... }` | Processo executável/descritivo; BPMN é uma visão. |
| `version` | `version: 1` | Versão do processo; releases e diff usam. |
| `goal` | `goal: "..."` | Objetivo de negócio; usado na entrevista e no review. |
| `roles` | `roles: [Requester, Manager, Buyer]` | Atores que podem receber tarefas; referenciam `Role`. |
| `start on` | `start on PurchaseRequested` | Evento que inicia uma instância. |
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
| `parallel` | `parallel { task A { ... }; task B { ... } }` | Divide o fluxo; ambos os ramos executam. |
| `subprocess` | `subprocess Name { process: OtherProcess }` | Chama outro processo como subprocesso. |
| `process` | `process: ProvisionAccess` | Referência ao processo chamado. |
| `end on` | `end on ShipmentDispatched` | Evento que encerra a instância. |
| `on timer` | `on timer 2d after ReviewRequest -> Escalate` | Boundary timer: dispara após duração, a partir do nó. |
| `after` | `after <Node>` | Nó de referência do timer. |
| `on error` | `on error CarrierUnavailable -> ManualQuote` | Boundary error: desvia quando a task/serviço falha. |

**Funcionamento:** a task humana cria uma inbox para o papel; a task de serviço chama `action`/`integration`. `reads`/`writes` limitam o que a tarefa pode tocar. Gateways não executam regra própria: usam `Decision`/`Rule` ou expressão segura. `parallel` espera todos os ramos; `subprocess` herda o caso. `on timer`/`on error` são boundaries do nó anterior. Uma instância mantém variáveis (`purchase.amount`, `shipment.pesoBruto`) e histórico.

```bpcms
process PurchaseApproval {
  version: 1
  goal: "Comprar com alçada e rastro"
  roles: [Requester, Manager, Buyer]
  start on PurchaseRequested

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

  end on PurchaseApproved
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
| `id` | *(implícito)* | Todo document tem `id: <Name>Id`, único e gerado; não declare o campo nem o Shape. |
| `lookup` | `nickname: string { lookup }` | Marca o campo para o lookup de exibição; a ordem dos campos marcados define a ordem. Extensões herdam os campos marcados. |
| referência | `categoryId: CategoryId` | ForeignKey para `Category`; novos vínculos exigem id existente; vínculos antigos continuam válidos. |
| projeção | `customerId: UserId(citizenId)` | Copia apenas campos **extras**; campos do `lookup` já são automáticos. |
| `extends` | `document Consumer extends User { ... }` | Especialização aditiva; herda campos, ações, regras e lookup; usa o mesmo id do pai. |
| `version` | `version: 1` | Versão do schema; migrações comparam. |
| `owner` | `owner: "Billing"` | Serviço/time dono; outros só referenciam. |
| `sync` | `sync: live` ou `sync: read` | `live` assina patches; `read` entrega snapshot. |
| `audit` | `audit: true` | Registra autor, diff e timestamp. |
| `retention` | `retention: 5y` | Retenção antes de anonimizar/expurgar. |
| `fields` | `fields { name: Shape { ... } }` | Campos tipados; o campo só passa parâmetros simples. |
| `indexes` | `indexes { byStatus: [status, createdAt] range }` | Índices declarativos; o builder realiza. |
| `actions` | `actions { create(...): Document; private recalcScore(): Score }` | Comandos tipados; `private` restringe a chamada ao serviço dono. |
| `rules` | `rules { total == items.sum(subtotal) }` | Invariantes/transições; falha = erro de validação. |
| `source: device` | `source: device "scale-01" for pesoBruto` | Campo é lido de dispositivo; atualização = releitura. |
| `computed from` | `computed from Invoice, Payment` | Documento inteiro é cálculo de outras fontes. |
| `computed <Name> from` | `computed frete from Shipment, CarrierTable { ... }` | Cálculo nomeado (campo/subdocumento) com frescor. |
| `freshness` | `freshness: 15m` | TTL/agenda do cálculo; expõe `calculatedAt`. |
| `exposes` | `exposes: [frete, calculatedAt]` | Campos resultantes do cálculo. |

**ID automático.** O `id` não é declarado: todo `document` base ganha `id: <Name>Id` (ex.: `Product` → `ProductId`). O Shape `<Name>Id` também é implícito e **não pode ser declarado**. Declarar `id` no documento ou `shape ProductId` é `E_DUPLICATE_ID_FIELD`/`E_DUPLICATE_ID_SHAPE`. **Exceção:** extensão (`extends`) compartilha o id do pai; `Consumer extends User` usa `UserId`. Se `ConsumerId` for usado, é **alias de tipo** do mesmo `UserId` — não é um segundo id nem um segundo Shape/valor. Declarar `shape ConsumerId` é `E_DUPLICATE_ID_SHAPE`.

**Lookup/description.** Todo document base marca com `lookup` os campos que o representam para o usuário. Ex.: `nickname: string { lookup }` e `avatar: Image? { lookup }` em `User`. A ordem dos campos marcados é a ordem do lookup. Toda referência a esse document grava automaticamente esse snapshot; o builder decide se armazena em objeto embutido, campos prefixados ou outro formato. Extensões **herdam** os campos marcados e não redeclaram um bloco.

**ForeignKey.** Usar `CategoryId` significa ForeignKey para `Category`. Se o id referenciado não existir, **novos vínculos** falham (`E_FK_MISSING`), mas vínculos já gravados continuam válidos e mantêm o lookup. Não há cascade destrutivo.

**Projeção.** A referência só declara os campos **extras**; o lookup já cobre o resto. Ex.: `customerId: UserId(citizenId)` grava o id + `nickname`/`avatar` (lookup) + `citizenId` (extra). É cópia de exibição, não substitui a identidade; campos projetados são somente leitura.

**Extensão.** `document Consumer extends User` modela dois conceitos: `User` é o usuário do sistema (login/papéis) e `Consumer` é o consumidor pagante. A extensão:

- é aditiva: herda campos, ações, regras e lookup; adiciona `citizenId: CitizenId?`;
- mantém o **mesmo id** do pai (`UserId`); `ConsumerId`, se usado, é alias de `UserId`, não um segundo id;
- fica no mesmo serviço do pai (v1);
- pode ser referenciada como `UserId`; `UserId(citizenId)` projeta o campo extra da extensão, e `UserId` sozinho entrega o lookup herdado.
- `citizenId` **não é unique**: vários `Consumer`/`User` podem referenciar o mesmo `Citizen`; o `identity` mantém o vínculo 1:N; `CitizenMerged` reatribui os Users.

`citizenId` é uma referência ao `Citizen`, dono da identificação/PII; não é CPF/CNPJ nem taxa. É opcional: um `User` sem perfil `Consumer` projeta `null`.

**Referência circular.** Em um documento que referencia a si mesmo, o campo não pode se chamar `<document>Id` (ex.: `categoryId` dentro de `Category`): use outro nome, como `parentId: CategoryId?` (`E_SELF_ID_FIELD`). `Product.categoryId: CategoryId` é válido porque o alvo é `Category`, não `Product`.

**Parâmetros de campo:**

| Parâmetro | Sintaxe | Funcionamento |
| :--- | :--- | :--- |
| `title` | `title: { pt-BR: "...", en: "..." }` | I18n do campo (obrigatório); `short`/`tooltip` opcionais. |
| `required` | `required` | Campo obrigatório; ausência falha. `required: true` também é aceito. |
| `lookup` | `lookup` | Inclui o campo no lookup de exibição; a ordem é a ordem dos campos. |
| booleanos | `required`, `lookup`, `unique`, `readonly`, `sensitive` | Flag bare = `true`; `flag: true/false` também é aceito. |
| `maxLength` | `maxLength: 120` | Limite de texto; o `Shape` valida. |
| `unique` | `unique` | Unicidade lógica; índice realizando. `unique: true` também é aceito. |
| `unit` | `unit: kg` | Unidade do valor; default é a unidade base do Shape. |
| `min` / `max` | `min: 0.001, max: 50000` | Faixa validada no field. |
| `scale` | `scale: 2` | Casas decimais do field. |
| `precision` | `precision: 3` | Precisão de armazenamento/exibição do field. |
| `unit` / `units` | — | `unit` só no Shape de unidade única; `units` só no Shape multiunidade. |
| `default` | `default: BR` | Valor inicial; não é regra de negócio. |
| `private` | `private internalScore: Score` | Campo interno; não é exposto a outros serviços/API/serialização. |

**Índices:** `unique` (unicidade), `range` (ordenação/faixa), `search` (texto), `hash` (igualdade/partição). O campo referenciado precisa existir e ser indexável no `Shape`.

```bpcms
document Shipment {
  title: { pt-BR: "Remessa", en: "Shipment" }
  version: 1
  owner: "Dispatch"
  sync: live

  fields {
    orderId: OrderId { title: { pt-BR: "Pedido", en: "Order" }, lookup }
    pesoBruto: std.Mass { title: { pt-BR: "Peso bruto", en: "Gross weight" }, unit: kg, min: 0.001, precision: 3 }
    tracking: TrackingCode? { title: { pt-BR: "Rastreio", en: "Tracking" }, lookup }
    status: ShipmentStatus { title: { pt-BR: "Status", en: "Status" }, lookup }
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
    dispatch(carrier: CarrierRef): Shipment
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
| `mime` | `mime: [image/*, application/octet-stream]` | Restrição de MIME para `attachment` genérico. |
| `classification` | `classification: "<url>"` | Classificação externa; reservada à stdlib (`Currency`). |
| `values` | `values { draft { title; icon; ordinal: 10; next: [...] } }` | Enum; `classification`/`workflow`; `workflow` exige `next`. |
| `initial` | `initial: draft` | Estado inicial do `workflow`; obrigatório e único. |
| `currency` | `currency: BRL` | Moeda; exige escala e formato. |
| `scale` | `scale: 2` | Casas decimais de exibição/agregação. |
| `dimension` | `dimension: mass` | Grandeza; impede comparar com outra dimensão. |
| `units` | `units { m: { factor: 1 }, cm: { factor: 0.01 } }` | Primeira unidade é a referência; demais usam `factor` ou `formula`. |
| `unit` | `unit: %` | Unidade única; mutuamente exclusivo com `units`. |
| `parse` | `parse: quantity`, `locale-number`, `percent`, `digits`, `lower-trim`, `email` | Perfil default de texto → valor; composto exige método. |
| `format` | `format: currency`, `locale-unit`, `percent`, `plain`, `postal`, `image`, `masked("...")` | Perfil default de valor → texto; composto exige método. |
| `compare` | `compare: exact` | Igualdade/ordenação; `exact` para dinheiro. |
| `aggregate` | `aggregate: sum` | Soma/média/min/max permitidos. |
| `index` | `index: normalized` | Codificação para índice. |
| `normalize` | `normalize: digits`, `lower-trim`, `upper-trim` | Normalização antes de validar/comparar. |
| `validate` | `validate: cpf-check-digits`, `uuid`, `email`, `phone`, `url`, `year-month` | Validação semântica. |
| `sensitive` | `sensitive: true` | Exige mascaramento/permissão. |
| `maxLength` | `maxLength: 254` | Limite de texto. |
| `fields` | `fields { street: string ... }` | Shape composto. |
| `default` | `default: BR` | Valor default de campo interno. |
| `private` | `private helper: string` | Campo interno do Shape; fora do contrato público. |
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

> `<Document>Id` é um Shape implícito de ForeignKey; não declare `shape CategoryId` (ver 5).

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
  title: { pt-BR: "Status da nota", en: "Invoice status" }
  base: workflow
  initial: draft
  format: plain
  values {
    draft {
      title: { pt-BR: "Rascunho", en: "Draft" }
      icon: Draft
      ordinal: 10
      next: [issued, cancelled]
    }
    issued {
      title: { pt-BR: "Emitida", en: "Issued" }
      icon: Issued
      ordinal: 20
      next: [paid, overdue, cancelled]
    }
    paid {
      title: { pt-BR: "Paga", en: "Paid" }
      icon: Paid
      ordinal: 30
      next: []
    }
    overdue {
      title: { pt-BR: "Vencida", en: "Overdue" }
      icon: Overdue
      ordinal: 40
      next: [paid, cancelled]
    }
    cancelled {
      title: { pt-BR: "Cancelada", en: "Cancelled" }
      icon: Cancelled
      ordinal: 50
      next: []
    }
  }
}
```

- O primeiro ordinal é **10**; os seguintes incrementam de **10 em 10**.
- `ordinal` é único, inteiro e estável; inserir um value no meio usa um ordinal livre.
- O ordinal é usado no stream binário; o title/icon são para UI.
- `values: [a, b]` sem title/icon/ordinal é erro.

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

## 6.1.2 classification e workflow

`base: classification` enumera categorias; `base: workflow` modela estados e transições.

```bpcms
shape TaxRegimeCode {
  title: { pt-BR: "Regime tributário", en: "Tax regime" }
  base: classification
  values {
    SIMPLES {
      title: { pt-BR: "Simples Nacional", en: "Simples Nacional" }
      icon: Simples
      ordinal: 10
    }
  }
}
```

- `classification` não tem `initial` nem `next`.
- `workflow` tem exatamente um `initial`.
- Todo state de `workflow` tem `next`; `next: []` é terminal.
- `next` referencia apenas states do mesmo workflow.
- Self-transition é inválida no v1.
- `canTransitionTo(next)` é um método automático do workflow.
- `transitionTo` não existe; alterar o valor para um próximo estado inválido lança `E_INVALID_TRANSITION`.
- Ordinal continua explícito, 10, 20, 30...

### 6.1.1.1 formula e convert

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
- `toOrdinal(value)` é gerado pelo compilador em `classification`/`workflow`; não se escreve no Shape.
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
// texto sem unidade ou unidade diferente de flags.unit -> E_PARSE_UNIT
```

Regras do `parse`:

- Shape com unidade exige unidade no texto.
- Se `flags.unit` existir, a unidade do texto precisa ser compatível; caso contrário, `E_PARSE_UNIT`.
- Se não existir `flags.unit`, a unidade do texto é usada.
- Singular/plural vêm de `units.title`.
- Separadores e símbolos vêm da regionalization interna.
- Se a regionalization não for determinável: `E_REGIONALIZATION_UNKNOWN`.
- A stdlib usa `parseQuantity(text, flags)`: unidade do texto vence/é convertida para `flags.unit`; sem unidade em nenhum lugar = erro.

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
- enum = `int32` ordinal;
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
| `permissions` | `permissions: [order.manage, report.view]` | Namespaces de capacidade. |
| `inherits` | `inherits: Clerk` | Herda permissões do papel. |
| `scope` | `scope: store` | Abrangência do papel/organograma. |
| `organogram` | `organogram <Name> { ... }` | Hierarquia e regras de autorização. |
| `hierarchy` | `hierarchy { Manager > Clerk > Customer }` | `>` = superior a; direção de autoridade. |
| `fourEyes` | `fourEyes { payment.approve requires Approver and Approver != Analyst }` | Exige segundo ator; pode comparar atores. |
| `segregation` | `segregation { shipment.pick != shipment.check }` | Impede que o mesmo ator faça as duas ações. |
| `delegation` | `delegation { Supervisor -> Checker when absent }` | Delegação condicional. |
| `limits` | `limits { Approver: 50000 BRL; Controller: unlimited }` | Alçada por papel; `unlimited` é explícito. |
| `dataMasking` | `dataMasking { Agent: customer.phone masked }` | Mascara campo para o papel. |
| `signature` | `signature { Pathologist }` | Papel com autoridade de assinatura. |
| `requires` | `fourEyes { X requires Role }` | Papel/condição exigida. |
| `when absent` | `delegation { A -> B when absent }` | Condição de delegação. |
| `unlimited` | `limits { Role: unlimited }` | Sem limite; decisão explícita. |
| `masked` | `dataMasking { Role: field masked }` | Campo mascarado na leitura. |

> Roles e organogram do serviço ficam no mesmo arquivo `<service>-organograms.bpcms`.

```bpcms
role Manager {
  title: { pt-BR: "Gerente", en: "Manager" }
  inherits: Clerk
  permissions: [customer.manage, report.view, refund.approve]
}
organogram Store {
  title: { pt-BR: "Organograma da loja", en: "Store organogram" }
  hierarchy { Manager > Clerk > Customer }
  fourEyes { refund.approve requires Manager and Manager != Clerk }
  delegation { Manager -> Clerk when absent }
}
```

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

# 13. Parameter

`Parameter` é o artefato de configuração e segredo, **sempre declarado dentro de um `Service`**. Ele declara o contrato; o valor é resolvido pelo ambiente e **nunca** entra no lock. A `Application` não declara `Parameter`; no `main`, ela faz `set` em um parâmetro de serviço.

| Cláusula | Sintaxe | Funcionamento |
| :--- | :--- | :--- |
| `parameter` | `parameter <Name> { ... }` | Declara configuração/segredo tipado dentro de um serviço; artefato público. |
| `title` | `title: { pt-BR: "...", en: "..." }` | I18n do parâmetro (obrigatório); substitui `description`; ver 1.9. |
| `type` | `type: string \| Secret \| Count \| std.Duration \| ...` | Shape do valor. |
| `source` | `source: config \| secret` | Origem do valor: configuração do ambiente ou secret store. |
| `required` | `required` | Falha no boot/uso se não houver valor; `required: true` também é aceito. |
| `default` | `default: "..."` | Só para `config`; proibido para `secret`. |
| `min` / `max` | `min: 0, max: 10` | Constraints simples para numéricos. |
| `maxLength` | `maxLength: 120` | Limite para texto. |
| `Secret` | `type: Secret` | Shape embutido para segredo; nunca serializa, loga, exporta ou vai para a LLM. |

> Os parâmetros abaixo são trechos de um serviço. A `Application` só pode fazer `set` em um parâmetro já declarado no serviço.

```bpcms
parameter ErpUrl {
  title: { pt-BR: "URL do ERP", en: "ERP URL" }
  type: string
  source: config
  required: true
  default: "https://erp.example.com"
}

parameter PaymentKey {
  title: { pt-BR: "Chave de pagamento", en: "Payment key" }
  type: Secret
  source: secret
  required: true
}
```

**Regras:** o lock guarda o **ID** do Parameter, não o valor. `Secret` não tem default e não aparece em logs, serialização, evidência ou prompt. Parameters são declarados **no serviço**; a app só inicializa com `set <Alias>.<Parameter> = <valor>`. Para segredo, use `secret.NOME`, nunca um literal.

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
    getSupplier(id: SupplierId): Supplier
    createInvoice(invoice: Invoice): ExternalId
  }
  events { Issued -> createInvoice }
}
```

---

# 21. Report / Dashboard

| Cláusula | Sintaxe | Funcionamento |
| :--- | :--- | :--- |
| `report` | `report <Name> { ... }` | Relatório/dashboard. |
| `type` | `type: report \| dashboard` | Forma de apresentação. |
| `sources` | `sources: [Invoice, Order]` | Documentos/fontes. |
| `roles` | `roles: [Manager]` | Quem pode ver. |
| `filters` | `filters { period; region }` | Filtros disponíveis. |
| `dimensions` | `dimensions { region; month }` | Agrupamentos. |
| `metrics` | `metrics { revenue: sum(Invoice.total) by region }` | Métricas e agregações. |
| `units` | `units { revenue: BRL }` | Unidade de exibição. |
| `freshness` | `freshness: 15m` ou `live` | Frescor do resultado. |
| `export` | `export: [pdf, csv]` | Formatos de exportação. |
| `alert` | `alert { compliance < 90% -> NotifyLead }` | Alerta condicional. |
| `sort` | `sort { value: desc }` | Ordenação. |
| `mode` | `mode: live` | Atualização em tempo real. |

```bpcms
report SalesByRegion {
  type: dashboard
  sources: [Invoice, Customer]
  roles: [Manager]
  filters { period; region }
  metrics {
    revenue: sum(Invoice.total) by region
    orders: count(Invoice) by region
  }
  freshness: 15m
  units { revenue: BRL }
  export: [pdf, csv]
}
```

---

# 22. Event

| Cláusula | Sintaxe | Funcionamento |
| :--- | :--- | :--- |
| `event` | `event <Name> { ... }` | Fato de negócio publicado. |
| `title` | `title: { pt-BR: "...", en: "..." }` | I18n obrigatório (ver 1.9). |
| `version` | `version: 1` | Versão do contrato. |
| `payload` | `payload { orderId: OrderId; approvedAt: DateTime }` | Campos do evento. |
| `producer` | `producer: Order` | Documento/serviço que publica. |
| `consumers` | `consumers: [Fulfillment, Billing]` | Quem consome. |
| `key` | `key: orderId` | Chave de partição/ordenação. |
| `sensitivity` | `sensitivity: restricted` | Restrição de dados. |
| `severity` | `severity: warning` | Severidade operacional. |

```bpcms
event OrderApproved {
  title: { pt-BR: "Pedido aprovado", en: "Order approved" }
  version: 1
  payload {
    orderId: OrderId { title: { pt-BR: "Pedido", en: "Order" } }
    approvedBy: UserId { title: { pt-BR: "Aprovado por", en: "Approved by" } }
    approvedAt: DateTime { title: { pt-BR: "Aprovado em", en: "Approved at" } }
  }
  producer: Order
  consumers: [Fulfillment, Billing]
  key: orderId
}
```

---

# 23. Job / Schedule

| Cláusula | Sintaxe | Funcionamento |
| :--- | :--- | :--- |
| `job` | `job <Name> { ... }` | Tarefa assíncrona/agendada. |
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
  audit: true
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
| ForeignKey | novo vínculo para id inexistente | `E_FK_MISSING` |
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
| Workflow | initial/next inválido, state inexistente ou self-transition | `E_WORKFLOW_INITIAL` / `E_WORKFLOW_NEXT` / `E_WORKFLOW_TRANSITION` |
| Classification | `initial`/`next` em classification | `E_CLASSIFICATION_TRANSITION` |
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
| Event | ✅ | 6 |
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
  values {
    draft {
      title: { pt-BR: "Rascunho", en: "Draft" }
      icon: status.draft
      ordinal: 10
    }
    issued {
      title: { pt-BR: "Emitida", en: "Issued" }
      icon: finance.invoice.issued
      ordinal: 20
    }
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

**`Currency`:** classificação da stdlib com URL externa ISO 4217 e values com `scale`, sem `title`/`icon`/`ordinal`. Para usar uma moeda, o serviço precisa reintroduzi-la.

```bpcms
shape Currency {
  title: { pt-BR: "Moeda", en: "Currency" }
  classification: "https://www.iso.org/iso-4217-currency-codes.html"
  values {
    BRL { scale: 2 }
    USD { scale: 2 }
    JPY { scale: 0 }
    KWD { scale: 3 }
    CVE { scale: 2 }
  }
}
```

- `CurrencyCode` não é Shape separado.
- `DefaultCurrency` usa `std.Currency`.
- A moeda usada sem `reintroduce` gera `E_CURRENCY_METADATA_MISSING`.

### 29.2.1.1 `reintroduce` e formatação de moeda

```bpcms
reintroduce std.Currency {
  values {
    BRL {
      title: { pt-BR: "Real", en: "Brazilian real" }
      prefix: {
        pt-BR: "R$"
        en: "BRL"
      }
      icon: Brl
      ordinal: 10
    }
    BTC {
      title: { pt-BR: "Bitcoin", en: "Bitcoin" }
      suffix: " BTC"
      icon: Bitcoin
      ordinal: 1000
    }
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




