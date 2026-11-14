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

## 1.2 Metadados

| Cláusula | Onde | Sintaxe | Funcionamento |
| :--- | :--- | :--- | :--- |
| `version` | app, service, process, document, event, builder, release | `version: 1.4.0` ou `version: semver("1.4.0")` | Versão do artefato. `1.4.0` é açúcar para `semver("1.4.0")`. |
| `title` | app (e onde houver título) | `title: { pt-BR: "Loja do João" }` | Texto localizado por locale; chaves devem estar em `locales`. Ver 1.9. |
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
| `units` | `system()`, `convert(value, unit)`, `format(value, locale)`, `shapes()` | `regionalization` |

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

## 1.9 Texto localizado (`title`)

`title` é um mapa `locale -> texto`, não uma string global. As chaves precisam existir em `locales`; o primeiro locale é o default.

```bpcms
application "example.com/app/loja-do-joao" {
  locales: [pt-BR, en-US]
  title: {
    pt-BR: "Loja do João"
    en-US: "João's Store"
  }
}
```

Com `locales: [pt-BR, en-US]`, o título abaixo cobre `pt-BR`, mas deixa `en-US` sem tradução:

```text
title: { pt-BR: "Loja do João" }
```

O `check` emite `W_TRANSLATION`; um builder estrito para `en-US` falha com `E_TRANSLATION`; um builder de dev pode fazer fallback para `pt-BR`.

| Situação | Resultado |
| :--- | :--- |
| Builder gera `pt-BR` e a chave existe | ok |
| Builder gera `en-US` e a chave não existe | `W_TRANSLATION` no `check`; `E_TRANSLATION` se o builder for estrito |
| Chave fora de `locales` | `E_LOCALE_UNDECLARED` |
| Sem entrada para o locale default | `E_TITLE_MISSING` |

**Hífen:** `pt-BR`/`en-US` são chaves válidas; o parser aceita hífen tanto no nome da propriedade quanto na chave do mapa.

**Artefato reutilizável:** quando `title` aparece em um `Service`/`View` compartilhado, o conjunto válido é o `locales` do **app consumidor**; o artefato não fixa o conjunto.

**Política do construtor:** dev pode fazer fallback para o locale default e permitir editar o título em runtime, gravando de volta no fonte; produção é estrita por padrão. O construtor nunca inventa tradução.

---

# 2. Application

| Cláusula | Sintaxe | Funcionamento |
| :--- | :--- | :--- |
| declaração | `application "<uri>" { ... }` | Identidade + composição. |
| `title` | `title: { pt-BR: "..."; en-US: "..." }` | Nome localizado; chaves em `locales` (ver 1.9). |
| `services` | `services { "<uri>" as Alias ... }` | **Entry points**; alias local opcional. A closure vem dos imports de cada serviço. |
| `roles` | `roles { Customer; Clerk; Manager; Manager > Clerk }` | Papéis da aplicação e hierarquia (`>` = “superior a”). |
| `theme` | `theme: Nuvula` | Tema aplicado à experiência. |
| `locales` | `locales: [pt-BR, en-US]` | Idiomas disponíveis; o primeiro é o default. |
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
| `imports` | `imports { service "<uri>" as Alias; integration ERP }` | Dependências por URI, **sem versão**; alias local opcional; o lock escolhe a versão. |
| `override` | `override "<uri>#<Artifact>.<member>" { with: "<uri>#<Artifact>.<member>" }` | Substitui um método/ação; ativo quando o serviço substituto está na closure. |
| `requires` | `requires { oltp, attachments, identity }` | Capabilities de alto nível exigidas do construtor/ambiente (ver 1.7). |
| `provides` | `provides { catalog }` | Contrato de negócio oferecido a outros serviços/aplicações. |
| `roles` | `roles { CatalogManager, CatalogViewer }` | Papéis definidos pelo serviço. |

> **Contrato:** todo artefato de topo é público automaticamente; **não existe `exports`**. Privacidade só por membro (`private`).

**Funcionamento:** serviço é uma fatia vertical; todo artefato de topo é público e ele é dono dos seus documentos. O que é `private` dentro de um artefato não entra no contrato público. Importar um serviço não dá acesso de escrita aos dados dele. A URI registry resolve `/index.html` como documentação de API; repositório resolve README + código.

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

**Parâmetros de campo:**

| Parâmetro | Sintaxe | Funcionamento |
| :--- | :--- | :--- |
| `required` | `required: true` | Campo obrigatório; ausência falha. |
| `maxLength` | `maxLength: 120` | Limite de texto; o `Shape` valida. |
| `unique` | `unique: true` | Unicidade lógica; índice realizando. |
| `unit` | `unit: kg` | Unidade de entrada/exibição; o `Shape` converte. |
| `min` / `max` | `min: 0.001, max: 50000` | Faixa; o `Shape` valida. |
| `default` | `default: BR` | Valor inicial; não é regra de negócio. |
| `private` | `private internalScore: Score` | Campo interno; não é exposto a outros serviços/API/serialização. |

**Índices:** `unique` (unicidade), `range` (ordenação/faixa), `search` (texto), `hash` (igualdade/partição). O campo referenciado precisa existir e ser indexável no `Shape`.

```bpcms
document Shipment {
  version: 1
  owner: "Dispatch"
  sync: live

  fields {
    id: ShipmentId
    pesoBruto: Mass { unit: kg, min: 0.001 }
    status: ShipmentStatus
    createdAt: DateTime
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
| `base` | `base: decimal \| integer \| text` | Representação base. |
| `values` | `values: [draft, published, archived]` | Enumera valores permitidos (com `base: text`). |
| `currency` | `currency: BRL` | Moeda; exige escala e formato. |
| `scale` | `scale: 2` | Casas decimais de exibição/agregação. |
| `dimension` | `dimension: mass` | Grandeza; impede comparar com outra dimensão. |
| `unit` | `unit: kg` | Unidade canônica do Shape. |
| `precision` | `precision: 6` | Precisão de armazenamento/cálculo. |
| `displayUnits` | `displayUnits: [cm, m, in, ft]` | Unidades permitidas na exibição. |
| `min` / `max` | `min: 0`, `max: 100` | Faixa semântica. |
| `parse` | `parse: quantity`, `locale-number`, `percent`, `digits`, `lower-trim`, `email` | Como texto vira valor. |
| `format` | `format: currency`, `locale-unit`, `percent`, `plain`, `postal`, `masked("...")` | Como valor vira texto por locale. |
| `compare` | `compare: exact` | Igualdade/ordenação; `exact` para dinheiro. |
| `aggregate` | `aggregate: sum` | Soma/média/min/max permitidos. |
| `convert` | `convert: mass`, `convert: units.mass` ou `convert: affine(C, F, K)` | Conversão de unidade; `units.mass` delega ao serviço de unidades. |
| `index` | `index: normalized` | Codificação para índice. |
| `normalize` | `normalize: digits`, `lower-trim`, `upper-trim` | Normalização antes de validar/comparar. |
| `validate` | `validate: cpf-check-digits`, `uuid`, `email`, `phone`, `year-month` | Validação semântica. |
| `sensitive` | `sensitive: true` | Exige mascaramento/permissão. |
| `maxLength` | `maxLength: 254` | Limite de texto. |
| `fields` | `fields { street: Text ... }` | Shape composto. |
| `default` | `default: BR` | Valor default de campo interno. |
| `private` | `private helper: Text` | Campo interno do Shape; fora do contrato público. |

**Valores usados:**

| Cláusula | Valores |
| :--- | :--- |
| `parse` | `locale-number`, `quantity`, `percent`, `digits`, `lower-trim`, `email`, `cpf-check-digits` |
| `format` | `currency`, `locale-unit`, `percent`, `plain`, `postal`, `masked("###.###.###-##")` |
| `convert` | `mass`, `length`, `affine(C, F, K)` |
| `aggregate` | `sum`, `weighted-average` |
| `index` | `normalized`, `hash`, `exact` |

**Funcionamento:** o Shape é o único lugar com parse/validação/formatação/conversão. O campo apenas parametriza (`unit`, `min`, `max`, `required`). Um Shape composto agrupa campos, mas continua sendo valor, não documento.

```bpcms
shape Mass {
  base: decimal
  dimension: mass
  unit: kg
  precision: 6
  min: 0
  parse: quantity
  convert: mass
  index: normalized
  aggregate: sum
}
```

# 7. View

| Cláusula | Sintaxe | Funcionamento |
| :--- | :--- | :--- |
| declaração | `view <Name> { ... }` | Projeção de dados/interação; não é dona do dado. |
| `type` | `type: form \| table \| dashboard \| wizard \| detail \| kanban` | Discriminador da view. |
| `document` | `document: Customer` | Documento principal da tela. |
| `source` | `source: Order` | Fonte de tabela/kanban. |
| `sources` | `sources: [Invoice, Order]` | Fontes de dashboard/relatório. |
| `roles` | `roles: [Clerk, Manager]` | Quem pode abrir/executar. |
| `fields` | `fields { name: Text { required: true } }` | Campos exibidos/editados; aceita `required`, `readonly` e parâmetros do Shape. |
| `readonly` | `readonly: true` | Campo apenas leitura na view. |
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

```bpcms
role Manager {
  inherits: Clerk
  permissions: [customer.manage, report.view, refund.approve]
}
organogram Store {
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
| `type` | `type: Text \| Secret \| Count \| Duration \| ...` | Shape do valor. |
| `source` | `source: config \| secret` | Origem do valor: configuração do ambiente ou secret store. |
| `required` | `required: true` | Falha no boot/uso se não houver valor. |
| `default` | `default: "..."` | Só para `config`; proibido para `secret`. |
| `description` | `description: "..."` | Documentação do parâmetro. |
| `min` / `max` | `min: 0, max: 10` | Constraints simples para numéricos. |
| `maxLength` | `maxLength: 120` | Limite para texto. |
| `Secret` | `type: Secret` | Shape embutido para segredo; nunca serializa, loga, exporta ou vai para a LLM. |

> Os parâmetros abaixo são trechos de um serviço. A `Application` só pode fazer `set` em um parâmetro já declarado no serviço.

```bpcms
parameter ErpUrl {
  type: Text
  source: config
  required: true
  default: "https://erp.example.com"
}

parameter PaymentKey {
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
| `theme` | `theme <Name> { ... }` | Tokens de apresentação. |
| `mode` | `mode: light \| dark \| print` | Modo visual. |
| `supports` | `supports: [WebApp, MobileShell]` | Builders que consomem o tema. |
| `tokens` | `tokens { color.primary: "#2563eb"; radius.md: 8 }` | Chave pontuada → valor. |
| `icons` | `icons: "lucide"` | Pacote de ícones. |
| `accessibility` | `accessibility: WCAG_AA` | Meta de acessibilidade. |
| `page` | `page: A4` | Formato de página (print/PDF). |
| `header` | `header: "logo"` | Cabeçalho de impressão. |
| `footer` | `footer: "page-number"` | Rodapé de impressão. |
| `density` | `density: compact` | Densidade de layout. |

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
| `locale` | `locale <tag> { ... }` | Idioma/região. |
| `fallback` | `fallback: en-US` | Idioma de fallback. |
| `currency` | `currency: BRL` | Moeda padrão. |
| `units` | `units: metric \| imperial` | Sistema de unidades padrão. |
| `date` | `date: "dd/MM/yyyy"` | Formato de data. |
| `time` | `time: "HH:mm"` | Formato de hora. |
| `direction` | `direction: ltr \| rtl` | Direção de layout. |
| `decimal` | `decimal: ","` | Separador decimal. |
| `thousands` | `thousands: "."` | Separador de milhar. |
| `messages` | `messages: "locales/pt-BR.json"` | Catálogo de mensagens. |

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
| `convert` | `convert: display` | Converte na exibição, sem alterar o valor. |
| `precision` | `precision: shape` | Usa a precisão do Shape. |
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
| `events` | `events { InvoiceIssued -> createInvoice }` | Evento de negócio → operação. |
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
  events { InvoiceIssued -> createInvoice }
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
| `version` | `version: 1` | Versão do contrato. |
| `payload` | `payload { orderId: OrderId; approvedAt: DateTime }` | Campos do evento. |
| `producer` | `producer: Order` | Documento/serviço que publica. |
| `consumers` | `consumers: [Fulfillment, Billing]` | Quem consome. |
| `key` | `key: orderId` | Chave de partição/ordenação. |
| `sensitivity` | `sensitivity: restricted` | Restrição de dados. |
| `severity` | `severity: warning` | Severidade operacional. |

```bpcms
event OrderApproved {
  version: 1
  payload {
    orderId: OrderId
    approvedBy: UserId
    approvedAt: DateTime
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
| Tipo | campo recebe Shape incompatível | `E_TYPE` |
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
| Shape | ✅ | 8 |
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

> Cobertura = todas as cláusulas de linguagem usadas nos exemplos estão documentadas. Campos de domínio (`name`, `email`, `id`) não entram na contagem. As cláusulas comuns (version/source/type/mode/status) aparecem na seção 1 e são referenciadas nas seções seguintes.

## O que ainda não é cláusula

* `extends`/herança de tipo — **fora da v1** (decisão B-34); composição + `override` cobrem o reuso.
* `migration` — **decidido**: é um `release` do tipo `migration`, não um artefato de topo (B-35). A gramática detalhada fica no SPECS.
* `alias`/`as` — **decidido e documentado** na seção 1.1.
* `private` — **decidido** para membros; artefato de topo nunca é privado.
* capabilities de UI/widget — **decidido**: vocabulário de alto nível (1.7), sem termos de tecnologia.
* `compensation` — **decidido**: task `type: compensation` com `for` (B-37).




