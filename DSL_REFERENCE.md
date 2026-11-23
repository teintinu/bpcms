# BPCMS — Referência da linguagem Archol

*Sintaxe ilustrativa (Process + Document).* A linguagem se chama **Archol** e seus fontes usam a extensão **`.archol`**. A sintaxe ainda evolui em conjunto com o compilador em [`archol/`](archol/README.md), que já lexa, parseia, checa e linka os exemplos. Esta referência cobre **todas as cláusulas usadas em [DSL_EXAMPLES.md](DSL_EXAMPLES.md)** e serve de base para o SPECS.

**O que mudou na v0.2 (resumo):** a aplicação é orientada a **Process** (trabalho) e **Document** (estado). A task não tem `type:` (o tipo é inferido do contrato), a lane é obrigatória, o fluxo é um grafo de arestas explícitas com `start`/`end` nomeados, o subpasso é sempre outro processo (`runs:`), `form` é artefato (contrato da tarefa humana) e `view` não tem `type: form`. `job` virou `start { schedule: }`; `report` virou `view type: report`; **o workflow define estados, transições e eventos** (`transition` no `value`, com `title`/`payload`/`event:`); a action só aponta o destino (`to:`) e executa o comando; `reads`/`writes`, `assignment`, `interventions`, `progress` e `type: user` deixaram de existir.

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
| 1 | Léxico, metadados, origem, release, lock/IDs, capabilities e serviços essenciais |
| 2–8 | Application, Service, Process (com Form e Message), Document, Shape, View, Role/Organogram |
| 9–12 | Evidence, Conversation, Decision, Assumption |
| 13–19 | Parameter, Secret, Environment, Builder, Theme, Locale, Widget, Release/Version |
| 20–24 | Integration, Report, Event, Schedule, Policy |
| 25 | Expressões, operadores e valores |
| 26 | Validação e erros |
| 27 | Cobertura |
| 28 | Icon |
| 29 | bpcms.org/std |
| 30 | Organização dos arquivos |

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

## 1.1.2 Literais

### Strings

Strings usam aspas simples ou duplas, como em TypeScript:

```archol
title: { pt-BR: "Nome", en: 'Name' }
```

O `format` prefere **aspas duplas** e converte `'texto'` para `"texto"` quando o conteúdo não tem aspas duplas nem escape que mudaria o valor. O `lint` avisa com `W_STRING_QUOTES` enquanto a string estiver em aspas simples.

### Duration e Shape literals

Um valor cujo tipo declarado é um **Shape** pode ser escrito como string. O compilador executa o `parse` declarativo do Shape em tempo de compilação e grava no AST um `ShapeLiteral`:

```archol
process Dunning(invoice: Invoice) {
  sla: "30d"
}
```

```json
{
  "kind": "ShapeLiteral",
  "shape": "std.Duration",
  "raw": "30d",
  "value": 2592000,
  "unit": "s",
  "loc": { "start": 0, "end": 0 }
}
```

- `shape` é a referência usada no fonte (`std.Duration`, `ProductVideo`, ...).
- `raw` é o texto sem aspas.
- `value` é o valor normalizado pelo `parse`; para quantidade é o número na unidade-base (30 dias = 2.592.000 s).
- `unit` é a unidade-base do Shape, quando houver.
- Shapes de mídia (`base: video|audio|image|attachment`) preservam o texto em `value`.

**Duration exige aspas.** `sla: 30d` é erro `E_SHAPE_LITERAL_QUOTED`; o correto é `sla: "30d"`. Um literal que não passa no `parse` do Shape é `E_SHAPE_LITERAL`; o valor não é adivinhado nem truncado.

A regra vale para qualquer valor com tipo Shape: `sla`, `let`/`const` tipados, campos de objeto tipado, defaults e `min`/`max`. O tipo do Shape vem de `bpcms.org/std` (`std.Duration`, `std.Mass`, `std.Money`, ...) ou de um Shape local do serviço.

`Parameter` usa o `type` para parsear `default`/`min`/`max`:

```archol
parameter DefaultSla {
  title: { pt-BR: "Prazo padrão" }
  type: std.Duration
  default: "2d"
}

parameter RefundLimit {
  title: { pt-BR: "Limite de reembolso" }
  type: std.Money
  default: "500 BRL"
  min: "0 BRL"
}
```

- `std.Money` produz `value: { raw: 500, currency: "BRL" }`.
- `std.Mass` normaliza para a unidade-base do Shape (`30 kg` → `30000 g`).
- Shapes com `normalize`/`validate` são normalizados/validados no compile-time (`lower-trim`, `upper-trim`, `email`, `phone`, `digits`, `year-month`).
- Shape de classificação sem `parse`/`normalize`/`validate` não vira `ShapeLiteral`; o valor continua sendo a referência/string do domínio.

## 1.2 Metadados

| Cláusula | Onde | Sintaxe | Funcionamento |
| :--- | :--- | :--- | :--- |
| `title` | app, service, process, document, view, form, message, manual, shape, field, event, parameter, role, organogram, icon, end, transition, action | `title: "Nome"` ou `title: { pt-BR: "Nome", en: "Name", short?: {...}, tooltip?: {...} }` | I18N obrigatório; string = primeiro locale do serviço; ver 1.9. |
| `locales` | app, service | `locales: [ "pt-BR", "en" ]` | Idiomas suportados; o primeiro é o default. No serviço é **obrigatório** (≥ 1) e define o locale do I18N em forma de string. |
| `description` | app | `description: "..."` | Texto livre para documentação e busca. |
| `status` | evidence, conversation, assumption | `status: proposed \| confirmed \| ...` | Estado do artefato de descoberta. Valores variam por artefato. |
| `type` | view, release | `type: table`, `type: report`, `type: migration` | Discriminador; **não existe em task** (o tipo é inferido do contrato) nem em document (relatório é view). |
| `mode` | builder, theme, report | `mode: [interpret, generate]`, `mode: dark`, `mode: live` | Modo de operação/apresentação. |
| `links` | evidence, assumption | `links { process X.Y; document X.field; view V; decision D; evidence E; service "<uri>" }` | Liga o artefato de descoberta ao modelo; o `check` valida o alvo. |
| `audit` | policy | `audit: true` | Exige trilha da avaliação da policy. Document é auditado automaticamente por comando. |
| `retention` | document | `retention: 5y` | Prazo de retenção antes de anonimizar/expurgar. |
| `private` | membros de artefatos | `private <field/action>` | Membro interno; não faz parte do contrato público nem da serialização para outros serviços. Artefato de topo nunca é `private`. |

## 1.3 Origem e versão

**Origem não é atributo.** `source`/`owner` não existem como metadados de artefato. A origem é inferida pela URI: hosts de repositório conhecidos (GitHub, GitLab, Bitbucket e similares) são projeto Git; o resto é registry BPCMS. Uma URI de repositório pode incluir subdiretório (`github.com/acme/monorepo/apps/store`).

| Cláusula | Sintaxe | Funcionamento |
| :--- | :--- | :--- |
| `@` | `"<uri>"@1.2.0`, `"<uri>"@tag("v1.2.0")`, `"<uri>"@commit("9f3a1c")` | Seletor de versão em releases e rollback; **não é atributo de artefato**. Imports de serviço não usam versão; o lock do app decide. |

**Versão não é atributo.** Nenhum artefato (`application`, `service`, `process`, `document`, `builder`, `release`, etc.) declara `version:` no corpo. A versão publicada é o seletor de `release`/`rollback` (`@...`) e, por serviço, o valor fixado no lock do app.

**Regras:** em produção, use tag/commit (ou semver imutável); branch/HEAD só em dev/QA.

## 1.4 Referências e tipos

| Cláusula | Onde | Sintaxe | Funcionamento |
| :--- | :--- | :--- | :--- |
| `Shape` | campos, binds | `peso: Mass` | Tipo de dado; dono de parse/validação/formato/unidade. |
| `Document` | variáveis, argumentos, actions | `shipment: Shipment` | Referência a documento; o `check` valida existência, dono e tipo. |
| `Form` | task humana | `Form.show({ order })` | Contrato da tarefa (4.14). |
| `View` | navegação, telas | `open -> OrderDetail` | Tela/projeção (7). |
| `Process` | subprocesso | `Provisioning.run({ employee })` | Subprocesso (call activity). |
| `Message` | send/receive | `send PickupRequest { ... }` | Contrato entre participantes (4.15). |
| `Role` | by, roles | `roles: [Clerk]` | Referência a papel. |
| `Event` | on, cancel on | `on [Order.Approved]` | Referência a evento de Document. |
| `Integration` | operação | `Carrier.quote({ shipment })` | Referência a operação de integração. |
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

O app tem **um lock file** na mesma pasta do arquivo da aplicação, com o mesmo nome e apenas a extensão trocada: `loja-do-joao-application.archol` → `loja-do-joao-application.lock`. Ele guarda **um ID estável para cada identificador usado** da closure.

**Formato:** CSV com duas colunas, `id,path` — um identificador por linha, sem cabeçalho e sem vírgula no path. Entradas fixadas vêm primeiro; as geradas seguem a ordem dos IDs.

```csv
checkoutStatus,example.com/services/checkout#shape/CheckoutStatus*
a,example.com/services/billing#service
b,example.com/services/billing#shape/InvoiceStatus/value/Overdue
```

| Campo | Funcionamento |
| :--- | :--- |
| `id` | ID estável; gerados são **sequenciais em base 36 minúscula começando em `a`**, com o **primeiro caractere sempre letra** (`a`…`z`, `aa`…`az`, `a0`…`a9`, `ba`…), sem duplicidade. Novos IDs vão primeiro para os identificadores **mais referenciados** (empate por path); IDs existentes não mudam. |
| `path` | Caminho seguro `<uri-do-serviço>#<tipo>/<Nome>[/...]`; nunca é caminho de arquivo. Um `*` no final marca o identificador como fixo. |

**Ciclo de compilação:**

1. durante a compilação, referências são resolvidas para **paths**, sem gerar número;
2. no final, IDs são atribuídos **apenas aos identificadores usados** — artefatos da closure são raízes, **exceto os do pseudo-serviço reservado `bpcms.org/std`**, que nunca entram no lock (referências como `std.Mass` mantêm o nome qualificado, sem ID); campos, valores de enum e parâmetros só recebem ID quando referenciados;
3. identificadores nunca referenciados ficam sem ID e sem linha no lock.

**Caminhos canônicos:** aplicação `<app-uri>#application`; serviço `<service-uri>#service`; artefato `<service-uri>#<tipo>/<Nome>`; campo `<service-uri>#document/<Documento>/field/<campo>`; valor de enum `<service-uri>#shape/<Shape>/value/<Valor>`; parâmetro `<service-uri>#process/<Processo>/param/<param>`.

**Regras:**

- Gerar/podar: `archol lock --app <uri-ou-arquivo>`; apagar o lock reinicia os IDs em `a`.
- **Fixar um identificador:** edite a linha e acrescente `*` ao final do path (o `*` fica no nome do identificador). O compilador apenas preserva o `*` nas reescritas e mantém essas linhas no topo; não renumera, não poda e não gera ID duplicado.
- IDs duplicados entre branches são responsabilidade de quem faz o merge; o carregador recusa duplicatas e pede para resolver.
- O AST usa esses IDs (`id`, `ref`, `fieldRef`, `typeRefId`); listas de usos ficam no índice semântico (LSP), não no JSON, para evitar ciclos e tamanho.
- O serviço não tem lock nem versões de imports. Versões de serviço/integridade por app ficam para a evolução do formato.

### 1.6.1 Pendências em comentários

Comentários que começam com **`TODO`**, **`FIXME`**, **`REVIEW`**, **`OPTIMIZE`** ou **`HACK`** viram nós `Pending` no AST (`SourceFile.pending`), utilizáveis pelo construtor.

**Sintaxe:** `// <MARCADOR> [YYYY-MM-DD] <comentário>` (data opcional entre colchetes).

```text
// todo implementar criptografia
// fixme precisamos corrigir uma possível invasão de código aqui
// REVIEW [2027-03-12] revisar isso depois do carnaval
// OPTIMIZE [2027-12-26] melhorar a performance depois do natal
```

- O marcador é **case-insensitive**; o AST e o `archol format` normalizam para maiúsculas.
- A data é o **due date**: sem data ou data no passado é **erro** (`E_PENDING_NO_DATE`/`E_PENDING_EXPIRED`); data inválida é `E_PENDING_DATE`; vencimento em até **15 dias** gera **warning** (`W_PENDING_SOON`).
- `archol format` (e o format-on-save do plugin) só ajusta o marcador para maiúsculas; `archol lint`/LSP publicam os diagnósticos.
- Durante o brainstorm da DSL, os comentários legados dos exemplos foram marcados com `REVIEW [2026-12-15]`.

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

```archol
builder AiRuntime {
  capabilities: [llm, oltp, sync]
}

process AssistantRefresh {
  title: { pt-BR: "Atualização do assistente", en: "Assistant refresh" }
  start Hourly { schedule: "0 * * * *" }
  task Refresh { lane: System; action: AssistantProfile.refresh }
  Hourly -> Refresh
  end Done { title: { pt-BR: "Atualizado", en: "Refreshed" } }
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

```archol
service "example.com/services/field" {
  title: { pt-BR: "Serviço de campo", en: "Field service" }
  locales: [ "pt-BR", "en" ]
}

form VisitReport {
  title: { pt-BR: "Relato da visita", en: "Visit report" }
  document: Visit
  fields {
    notes: string { title: { pt-BR: "Observações", en: "Notes" }, maxLength: 2000 }
    signature: Signature { title: { pt-BR: "Assinatura", en: "Signature" }, required }
  }
  actions {
    complete { calls: Visit.complete(notes, signature) }
    cancel { calls: Visit.cancel(notes) }
  }
}

view VisitDetail {
  title: { pt-BR: "Visita técnica", en: "Field visit" }
  type: detail
  document: Visit
  capabilities { location, timezone, regionalization, units }
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

## 1.9 std.bpcms.i18n (`title`, `goal`, `instructions`)

`std.bpcms.i18n` (I18N) é o **tipo interno** de texto localizado. Não é um `Shape` declarável; declarar `shape std.bpcms.i18n` é `E_RESERVED_TYPE`.

`I18N` é o tipo de:

- `title` em qualquer contexto que o declare: `app`, `service`, `process`, `document`, `view`, `form`, `message`, `manual`, `shape`, `field`, `event`, `parameter`, `role`, `organogram`, `icon`, `end`, `transition` e `action`;
- `goal` em `process`;
- `instructions` em `manual`;
- `short` e `tooltip` dentro de um `title`.

O valor de um `I18N` pode assumir **duas formas**:

```ts
type PluralForms = {
  zero?: string;
  one?: string;
  two?: string;
  few?: string;
  many?: string;
  other: string;
};

type I18NValue = string | PluralForms;
type I18N =
  | string
  | { [locale: string]: I18NValue; short?: I18N; tooltip?: I18N };
```

### String: primeiro locale do serviço

Quando o valor é uma string, ela pertence **sempre ao primeiro locale** declarado em `locales` do serviço. O compilador expande a string para o mapa canônico, para que o AST diga explicitamente a que locale o texto pertence:

```archol
service "example.com/services/billing" {
  title: "Faturamento"
  locales: [ "pt-BR", "en" ]
}

process Dunning(invoice: Invoice) {
  title: "Cobrança"
  goal: "Cobrar faturas vencidas com lembrete, escalonamento e cancelamento"
}
```

O AST resultante (trecho) é:

```json
{
  "kind": "PropertyAssignment",
  "name": "goal",
  "value": {
    "kind": "ShapeValue",
    "typeRef": "std.bpcms.i18n",
    "value": {
      "pt-BR": "Cobrar faturas vencidas com lembrete, escalonamento e cancelamento"
    }
  }
}
```

Todo serviço **declara `locales` com pelo menos um idioma**; o primeiro é o default. Serviço sem `locales` ou com lista vazia é `E_LOCALES_REQUIRED`.

A regra do default depende do dono do artefato:

- **Artefato de serviço** (`document`, `process`, `view`, `form`, `message`, `shape`, `role`, etc.): o primeiro locale do `locales` **do serviço**.
- **`Application`**: o primeiro locale do `locales` **da própria aplicação**.

`std.bpcms.i18n` aceita string em qualquer um desses contextos; a string é expandida para `{ "<locale-default>": "..." }`.

### Objeto: traduções por locale

```archol
title: { pt-BR: "Nome", en: "Name" }
```

```archol
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
- `short` e `tooltip` aceitam o mesmo `I18N` e são **irmãos** do mapa principal; não ficam aninhados um no outro.
- Se `short`/`tooltip` não existirem, o builder usa o title principal.

```archol
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

### Filtragem pelo `locales` da aplicação

O conjunto de locales que chega ao AST é o da **aplicação** (`Application.locales`), não o do serviço. O compilador:

1. emite o valor como `ShapeValue` com `typeRef: "std.bpcms.i18n"`;
2. expande string no primeiro locale do serviço (na `Application`, no primeiro locale dela);
3. remove do mapa todo locale cuja língua não exista em `Application.locales`.

A comparação aceita o locale exato e o fallback de idioma (`en` cobre `en-US`, e vice-versa). Assim, `en` sobrevive quando a app declara `en-US`, mas `fr` é descartado. O builder **nunca recebe** traduções de idiomas que a app não usa; não é erro no compilador, é filtragem.

**Fallback do builder:** locale exato → idioma (`en-US` → `en`) → locale default da app. Para plural: categoria → `other` → string do locale/título. Com `locales: [pt-BR, en-US]`, o default é `pt-BR`.

**Campos:** o `title` do campo é o label; o `title` do `document`/`shape`/`view`/`event`/`parameter`/`secret`/`role`/`organogram`/`service` é o nome do artefato. Em `Parameter`, `title` substitui o antigo `description`.

| Situação | Resultado |
| :--- | :--- |
| Builder gera `pt-BR` e a chave existe | ok |
| Builder gera `en-US` e só existe `en` | ok por fallback de idioma |
| Builder gera `en-US` e não existe `en-US` nem `en` | `W_TRANSLATION` no `check`; `E_TRANSLATION` se o builder for estrito |
| Locale no fonte fora de `Application.locales` | ignorado pelo compilador; não entra no AST |
| Sem entrada para o locale default | `E_TITLE_MISSING` |
| Plural sem `other`/categoria inválida | `E_PLURAL_OTHER` / `E_PLURAL_FORM` |
| `title` ausente no artefato/campo | `E_TITLE_MISSING` |
| Service sem `locales` ou com lista vazia | `E_LOCALES_REQUIRED` |
| `shape std.bpcms.i18n` declarado | `E_RESERVED_TYPE` |

**Hífen:** `pt-BR`/`en-US` são chaves válidas; o parser aceita hífen no nome da propriedade e na chave do mapa.

**Artefato reutilizável:** quando `title` aparece em `Service`/`View` compartilhado, o conjunto válido é o `locales` do **app consumidor**; o artefato não fixa o conjunto. O `locales` do serviço só define o default da forma string.

**Política do construtor:** dev pode fazer fallback para o locale default e permitir editar o título em runtime, gravando de volta no fonte; produção é estrita por padrão. O construtor nunca inventa tradução.

---

# 2. Application

| Cláusula | Sintaxe | Funcionamento |
| :--- | :--- | :--- |
| declaração | `application "<uri>" { ... }` | Identidade + composição. |
| `title` | `title: "..."` ou `title: { pt-BR: "..."; en-US: "..." }` | I18N; string = primeiro locale de `locales` (ver 1.9). |
| `services` | `services { "<uri>" as Alias ... }` | **Entry points**; alias local opcional. A closure vem dos imports de cada serviço. |
| `roles` | `roles { Customer; Clerk; Manager; Manager > Clerk }` | Papéis da aplicação e hierarquia (`>` = “superior a”). |
| `theme` | `theme: Nuvula` | ID de tema fornecido pelo **construtor**; não é artefato de serviço. |
| `locales` | `locales: [pt-BR, en-US]` | Idiomas da app/construtor; o primeiro é o default. |
| `environments` | `environments: [dev, qa, prod]` | Ambientes declarados para a aplicação. |
| `main` | `main { ... }` | Bloco opcional de inicialização/configuração; sem regra de negócio. |
| `set` | `set <Alias>.<Parameter\|Secret> = <valor\|secret.NOME>` | Inicializa/sobrescreve Parameter/Secret; segredo só como `secret.NOME`. |
| `start` | `start ["<uri>"]` | Boot antecipado opcional; não declara dependência. |
| `lock` | `lock: "<app>.lock"` | Lock do app: IDs estáveis dos identificadores da closure (versões por serviço em evolução). |

**Funcionamento:** a `Application` **não declara artefatos** e não contém regra nem `wire`; apenas compõe entry points, papéis, tema, idiomas, ambientes e o lock. Dependências vêm de `Service.imports`; o resolver calcula a closure. **A compilação começa sempre pela aplicação**: `check`, `lock`, `links` e o LSP exigem uma aplicação e só carregam a closure; arquivos fora dela são inertes (o LSP os trata como comentado). O lock fixa os IDs dos identificadores da closure. A URI é o identificador; a página `http://<uri>/index.html` é download por SO (registry) ou README/releases (repositório). O arquivo da aplicação é `examples/applications/<fullpath>/<app>-application.archol`; a aplicação é o único contexto principal e deve ser o primeiro artefato do arquivo (30).

```archol
application "example.com/app/loja-do-joao" {
  lock: "<app>.lock"
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
| `title` | `title: "..."` ou `title: { pt-BR: "...", en: "..." }` | I18N obrigatório (ver 1.9). |
| `locales` | `locales: [ "pt-BR", "en" ]` | **Obrigatório** (≥ 1 idioma); o primeiro é o default do I18N em forma de string. |
| `imports` | `imports { service "<uri>" as Alias; integration ERP }` | Dependências por URI, **sem versão**; alias local opcional; o lock escolhe a versão. Vazio é válido. |
| `reintroduce` | `reintroduce <Alias>.<Artifact> { ... }` | Altera/adiciona definições de um artefato importado. |

> **Contrato:** todo artefato de topo é público automaticamente; **não existe `exports`**. Privacidade só por membro (`private`).

**Funcionamento:** serviço é um **conceito de negócio**, não biblioteca/package/módulo. Não crie serviço apenas para agrupar Shapes/Icons/Themes/Locales. Ele é uma fatia vertical; todo artefato de topo é público e ele é dono dos seus documentos. O que é `private` dentro de um artefato não entra no contrato público. Importar um serviço não dá acesso de escrita aos dados dele. A URI registry resolve `/index.html` como documentação de API; repositório resolve README + código.

```archol
service "example.com/services/catalog" {
  title: { pt-BR: "Catálogo", en: "Catalog" }
  locales: [ "pt-BR", "en" ]

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

**Sem `roles` inline.** Roles ficam no arquivo `*-organograms.archol`, depois do organogram; a lista inline duplicava a fonte de verdade.

**Arquivos por contexto principal.** O `service` é o contexto principal do arquivo `examples/services/<fullpath>/<service>-service.archol`, que declara apenas ele e o coloca como primeiro artefato. Os demais arquivos são contextos `document`/`process`/`view`/`message` ou arquivos compartilhados. A regra completa está na seção 30.

**Sem `source`/`owner`.** Origem é inferida pela URI e versão pertence a `Release`/lock, não ao contrato do serviço.

## 3.1 `reintroduce`

`reintroduce` altera/adiciona definições de um artefato importado. Pode modificar qualquer parte do artefato: `value`, `fields`, `title`, `icon`, `ordinal`, `permissions`, etc.

```archol
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

```archol
service "example.com/services/fraud-checkout" {
  locales: [ "pt-BR", "en" ]
  imports {
    service "example.com/services/checkout" as Checkout
  }

  reintroduce Checkout.CheckoutStatus {
    value ShippingQuoted {
      transition FraudChecked {
        title: { pt-BR: "Risco verificado", en: "Risk checked" }
      }
    }

    value FraudChecked {
      title: { pt-BR: "Risco verificado", en: "Risk checked" }
      icon: status.fraudChecked
      ordinal: 25
    }

    insert FraudChecked before PaymentAuthorized
  }

  reintroduce Checkout.CheckoutSession {
    action checkFraud() {
      title: { pt-BR: "Verificar risco", en: "Check risk" }
      roles: [RiskAnalyst]
      to: status.FraudChecked
    }
  }

  reintroduce Checkout.Checkout {
    task FraudCheck {
      lane: System
      action: CheckoutSession.checkFraud
    }

    insert FraudCheck before AuthorizePayment
  }
}
```

- `FraudChecked` é adicionado ao ciclo de vida (valor do Shape de estado);
- a transição é declarada na action reintroduzida (`from`/`to`), não no valor;
- `FraudCheck` é adicionado ao processo;
- `insert ... before ...` religa a aresta de entrada;
- os demais estados/passos permanecem;
- não há substituição total.


# 4. Process

**Processo é trabalho; Documento é estado.** O processo orquestra quem faz o quê, em que ordem, com que prazo e com que desvio. O documento guarda o dado durável, as invariantes e o ciclo de vida. Form, view, mensagem e decisão são contratos de trabalho ou projeções.

Um processo não é:

- uma máquina de estados do documento (isso é o **workflow** do Document, seção 5);
- um formulário (isso é o **Form**, seção 4.14);
- uma tela (isso é a **View**, seção 7);
- um agendador (agendamento é um **gatilho de start**; o scheduler é runtime);
- um relatório (isso é uma **View type: report**).

## 4.1 Processo é código

O processo é escrito como **código imperativo**, com sintaxe parecida com TypeScript:

- execução **sequencial e implícita**: cada passo só começa quando o anterior termina; não há `await` explícito;
- concorrência só com `parallel`;
- o processo recebe **documentos por argumento** e declara suas variáveis no topo;
- não existe `for <Documento>`: o caso é definido pelos argumentos e pela `key`;
- não existe `pool`/`lane`: quem executa um passo humano vem de `roles` no form/manual;
- não existe artefato `task`: o processo chama operações diretas ou artefatos (`Form.show`, `Manual.show`, `Job.run`, `Processo.run`).

As duas perguntas de um processo continuam explícitas:

| Pergunta | Onde vive |
| :--- | :--- |
| **Quem executa?** | `roles` do form/manual; operações rodam com autoridade do processo. |
| **O que o passo faz?** | operação (`Doc.action`, `Integracao.op`, `Decision.evaluate`, `Compute.fn`) ou task (`Form.show`, `Manual.show`, `Job.run`, `Processo.run`, `Message.send/receive`). |

## 4.2 Cabeçalho e gatilhos

```archol
process Checkout(session: CheckoutSession) {
  title: { pt-BR: "Finalização da compra", en: "Checkout" }
  goal: "Cotar o frete, autorizar o pagamento e confirmar a compra"
  sla: "1d"

  on [CheckoutSession.CartValidated]
  key: session

  // declarações de variáveis (somente aqui)
  let offer: FreightOffer | null = null
  let authorized = false
```

| Cláusula | Sintaxe | Funcionamento |
| :--- | :--- | :--- |
| `process` | `process <Name>(args)` | Unidade de orquestração; os argumentos são documentos/escalares do caso. |
| `title` | `title: std.bpcms.i18n` | Obrigatório; string ou mapa de locales (1.9). |
| `goal` | `goal: std.bpcms.i18n` | Objetivo de negócio; string ou mapa de locales. |
| `sla` | `sla: "1d"` (`std.Duration`) | Alvo brando do caso (exibição, KPI); não é timer. O literal quantidade+unidade é um valor de `std.Duration`. |
| `timeout` | `timeout: 30d` | Prazo duro do caso; dispara `Timeout` no processo. |
| `on` | `on [Doc.Event, ...]` | Lista de eventos que iniciam o processo. |
| `by` | `by: [Role, ...]` | Início manual por papéis. |
| `schedule` | `schedule: "0 6 * * *"` | Cron no fuso do tenant; o scheduler é runtime. |
| `fanOut` | `fanOut: Invoice where status == Overdue` | Com `schedule`: uma instância por documento; o argumento é o documento. |
| `message` | `message: Message` | Início pela mensagem; o `about` é o documento correlacionado. |
| `called` | `called` | Processo chamado por `Processo.run`; não é gatilho de topo. |
| `key` | `key: <expr>` | Identidade/correlação/dedup da instância. |

- Um processo tem **um** start. Para evento + manual, faça um wrapper fino.
- `key` é obrigatória para `on`, `message` e `fanOut`.
- O payload do evento/mensagem precisa ser atribuível aos argumentos (`E_START_PAYLOAD`).
- `owner` e `timezone` não existem no processo; fuso é do tenant/scheduler.

## 4.3 Roles — quem executa

Quem executa um passo humano é declarado no próprio form/manual:

```archol
form PaymentRetry {
  document: CheckoutSession
  roles: [Customer]
  timeout: 1d
  priority: 80
}
```

- `roles` substitui lane/pool; a role é a autoridade de execução da task.
- O processo executa operações de documento com autoridade de serviço; as `roles` das actions continuam valendo para chamadas diretas por view/API.
- `timeout` é prazo duro da task; `priority` e `assign` são da fila.
- Views de inbox usam `scope: roles`, `groupBy: role`, `filter { role in currentUser.roles }` e coluna `role`.

## 4.4 Variáveis

```archol
let offer: FreightOffer | null = null
let authorized = false
let authorization: string
let item: OrderItem
```

- `let`/`const` **somente no topo do processo**, antes do código.
- **Não declare variáveis dentro de `try`, `catch`, `if`, `loop`, `parallel` ou `forEach`.**
- Atribuição dentro de blocos é permitida.
- A instância é o escopo; as variáveis são **duráveis** e persistidas pelo builder.
- Documentos entram por **referência** (`DocumentRef`); valores de shape/resultado são valores.
- `loop` não declara contador:

```archol
loop PaymentAttempt max 3 {
  ...
  break
}
```

- `forEach`/`parallel each` usam variável declarada no topo:

```archol
forEach item in order.items {
  ...
}
```

- Toda variável precisa ser atribuída antes do uso em todos os caminhos (`E_DEFINITELY_ASSIGNED`).
- União/opcional é permitida (`FreightOffer | null`).

## 4.5 Start — como a instância nasce

```archol
on [Order.Placed]
key: order

by: [Clerk, Manager]

schedule: "0 2 1 * *"
fanOut: FiscalPeriod where status == Open
key: fiscalPeriod

message: PickupConfirmed
key: shipment

called
```

- `on` aceita lista de eventos; o documento do evento entra pelo argumento correspondente.
- `by` inicia manualmente; o usuário escolhe o documento/args.
- `schedule` sem `fanOut` cria uma instância por disparo; com `fanOut`, uma por documento.
- `message` correlaciona pelo `about` da mensagem.
- `called` é a entrada de processo chamado por `Processo.run`.
- Um start por processo (`E_PROCESS_START`); start duplicado para o mesmo evento é erro.
- `cancel on` não é start: é handler de cancelamento (§4.8).

## 4.6 End — como a instância termina

```archol
return Confirmed

end Confirmed {
  title: { pt-BR: "Compra confirmada", en: "Purchase confirmed" }
  outcome: success
}
```

- `end` é declaração do desfecho, **sempre ao final do processo** (`E_END_ORDER`).
- O fluxo termina com `return <End>`; o alvo precisa existir (`E_RETURN_UNKNOWN`).
- `title` é obrigatório; `outcome` é `success | cancelled | failure`.
- Todo caminho precisa terminar em `return` ou em erro tratado.
- Erro não tratado encerra a instância em falha, depois da compensação (§4.11).

## 4.7 Operações e tasks

**Operações** são chamadas diretas:

```archol
CheckoutSession.quoteShipping({ freightAmount: offer.freight, days: offer.days })
authorization = PaymentGateway.authorize({ total: session.total, method: session.paymentMethod })
queue = TicketRouting.evaluate({ category: ticket.category, priority: ticket.priority })
score = RiskScore.compute({ application })
```

| Operação | Contrato | Executor |
| :--- | :--- | :--- |
| `Doc.action({...})` | comando de documento | processo (serviço) |
| `Integracao.op({...})` | operação de integração | processo |
| `Decision.evaluate({...})` | tabela DMN pura | processo |
| `Compute.fn({...})` | função pura, empacotada pelo builder | processo |

**Tasks** são artefatos separados, chamados pelo processo:

| Task | Chamada | Contrato |
| :--- | :--- | :--- |
| `form` | `Form.show({...})` | task humana de escrita |
| `view` | `View.show({...})` | task humana de leitura/relatório |
| `manual` | `Manual.show({...})` | trabalho fora do sistema |
| `job` | `Job.run({...})` | trabalho de sistema com background/retry |
| `process` | `Processo.run({...})` | subprocesso (outra instância) |
| `message` | `Message.send/receive` | contrato entre participantes |

- O tipo é derivado do contrato; não há `type:`.
- Uma operação/task por passo: duas ações de documento são dois passos.
- `with`/`gives` não existem; argumentos são objeto e o retorno é atribuído a variável.
- Form/manual declaram `roles`, `timeout`, `priority`, `assign`; `returns` só quando devolvem dado sem gravar.
- A decisão humana é lida no **estado do documento** (`doc.status == X`) depois do `show`.
- `send`/`receive`:

```archol
send PickupRequest {
  key: shipment
  with { requestedAt: now() }
}

receive PickupConfirmed {
  key: shipment
  timeout: 2d
  apply: Shipment.confirmPickup
}
```

- `receive` sem `apply` devolve o payload; o parceiro/canal é configuração de runtime/integração (não há `partner`/pool).

## 4.8 Erros, prazos e cancelamento

```archol
error Declined {
  title: { pt-BR: "Pagamento recusado", en: "Payment declined" }
}

try {
  authorization = PaymentGateway.authorize({ total: session.total, method: session.paymentMethod })
} catch (Declined) {
  PaymentRetry.show({ session })
} catch (Timeout) {
  CheckoutSession.abandon({ reason: "Prazo esgotado" })
  return Expired
}
```

| Cláusula | Funcionamento |
| :--- | :--- |
| `error` | Erro de negócio com nome estável e `title` I18N. |
| `throw` | Lança erro de negócio; falha de operação chega pelo mesmo `catch`. |
| `try`/`catch` | Desvio explícito por erro; `catch` aceita lista. |
| `timeout` | Prazo duro de task/processo; dispara `Timeout` (reservado). |
| `finally` | Executa sempre; usado para limpeza/compensação. |
| `cancel on <Event> { ... }` | Cancela o trabalho ativo e executa o bloco. |

- `on error`, `on timer` e `on breach` não existem; use `try/catch` + `timeout`.
- `sla` é `std.Duration` (alvo brando); `timeout` é o prazo duro.
- `cancel on` é o handler de caso; pode compensar e `return` um end.
- Um erro não tratado encerra a instância em falha, depois da compensação.

## 4.9 Roteamento e controle

```archol
if (session.total > 10000 BRL) {
  ...
} else if (session.total > 0) {
  ...
} else {
  throw InvalidTotal
}

parallel {
  branch RevokeAccess { IT.revokeAccess({ employee }) }
  branch ReturnEquipment { IT.returnEquipment({ employee }) }
}

loop PaymentAttempt max 3 { ... }
forEach item in order.items { ... }

wait 2d
wait until invoice.dueAt
```

- `if`/`else if`/`else`: condição pura, primeiro verdadeiro vence; sem `then`.
- `parallel`: branches independentes, join implícito no fim; falha em uma branch cancela as irmãs (fail-fast).
- `loop` exige `max`; `forEach` é sequencial; `parallel each` é concorrente.
- `wait` por duração ou até uma expressão.
- Gateway inclusivo/complexo/event-based não existem: use `if`, `parallel` ou `receive` + `timeout`.

## 4.10 Fluxo

- A ordem do fonte é a ordem de execução; não há arestas explícitas.
- `if`/`parallel`/`loop`/`forEach` são blocos estruturados; não há `goto`.
- `break`/`continue` valem em `loop`/`forEach`.
- Um `return` encerra o processo; `parallel` só termina quando todas as branches terminam (ou falham).
- Não há `gateway`, `fork`, `join` ou arestas `A -> B`.

## 4.11 Compensação

```archol
try {
  PaymentGateway.capture({ payment })
} catch (CaptureFailed) {
  PaymentGateway.refund({ payment, amount: payment.amount })
  return Failed
}
```

- Compensação é explícita no `catch`/`finally`; não há `compensates:`/`rollback`.
- Execute handlers em ordem reversa à conclusão.
- O handler precisa ser idempotente; retry não substitui compensação.
- `runs` concluído é compensado pelo pai com uma chamada explícita; a instância filha não compensa sozinha.

## 4.12 Adiados

- `milestone` e `discretionary` (CMMN) ficam fora desta versão.
- Semântica completa de `parallel` (resultado/merge, settle) será fechada com o primeiro caso real.
- `job` com detach/agendamento próprio; hoje use `call`/`compute` com `timeout`/`retry`.
- Coreografia/conversação como visão derivada.

## 4.13 Progresso, subpassos e a tela do processo

A unidade de progresso visível é a **instância de processo**. Todo subpasso com passos próprios é um `Processo.run` — outra instância, com sua própria tela, histórico e dono.

| Seção | Conteúdo | Origem |
| :--- | :--- | :--- |
| identidade | caso, número da instância, início, prazo, desfecho | args, `title`, `key`, `end` |
| passo atual | 1 cartão por passo ativo: título, papel, dono, prazo | operação/task, `roles`, `assign`, `timeout` |
| formulário | form/manual embutido quando o passo é humano | `Form.show`/`Manual.show` |
| participantes | roles e filas | `roles` |
| histórico | passo, ator, hora, resultado, motivo de devolução/falha | blocos + auditoria |
| prazos e falhas | atraso, escalonamento previsto, tentativas, erro legível | `sla`, `timeout`, `catch` |
| subprocesso | cartão do filho: passo atual, marcos, drill-down, breadcrumb | `Processo.run` |
| documentos | lido/escrito por passo (ícones BPMN de data object) | derivado das operações/forms |
| próximos passos | sucessores e condição | `if`/`parallel`/sequência |
| marcos | conquistas do caso | adiado (CMMN) |

- Com `parallel`, N cartões de passo atual e "aguardando 2 de 3".
- Com `forEach`/`parallel each`, "3 de 5", dono e prazo por item.
- Task automatizada mostra tentativa N/M, próximo retry, heartbeat e erro legível.
- Todo nó mostra `title` (herdado do contrato quando não declarado).
- Documentos aparecem com ícone derivado do contrato; não se declara `reads`/`writes`.

## 4.14 Form — contrato da tarefa humana

Form não é tela: é o **contrato de admissão e desfecho** de uma task humana. A tela é do builder; a tela de andamento embute o form quando o passo atual é humano.

```archol
form OrderReview {
  title: { pt-BR: "Revisar pedido", en: "Review order" }
  document: Order
  roles: [Clerk]
  timeout: 4h
  assign: pull

  fields {
    customer
    items
    note: string {
      title: { pt-BR: "Observação", en: "Note" }
      maxLength: 500
    }
  }

  actions {
    approve { calls: Order.approve }
    reject { calls: Order.reject(note) }
    save { calls: Order.annotate(note); stay }
  }
}
```

| Cláusula | Sintaxe | Funcionamento |
| :--- | :--- | :--- |
| `form` | `form <Name> { ... }` | Contrato de task humana; chamado por `Form.show`. |
| `title` | `title: std.bpcms.i18n` | Obrigatório. |
| `document` | `document: Order` | Documento de contexto; define a linha da inbox e os campos herdados. |
| `roles` | `roles: [Clerk]` | Quem executa a task. |
| `timeout` | `timeout: 4h` | Prazo duro; `Timeout` no processo. |
| `priority`/`assign` | `priority: 50; assign: pull` | Fila e distribuição. |
| `fields` | `fields { customer; note: string { ... } }` | Campos exibidos/editados. |
| `actions` | `actions { approve { calls: Order.approve } }` | Botões. Cada botão executa um comando de documento. |
| `calls` | `calls: Order.reject(note)` | Comando e argumentos (campos do form). |
| `stay` | `save { calls: Order.update; stay }` | Botão que não avança o passo. |
| `returns` | `returns: Type` | Opcional: form que devolve dado sem gravar. |

- Campo não redeclara tipo nem título: herda do documento; só declare `nome: Shape { title }` para campo que não existe no documento.
- Obrigatoriedade é monotônica: o form pode endurecer, nunca relaxar.
- O form não escreve campo direto: toda escrita é `calls:` de action.
- Argumentos do `calls:` são campos do form ou literais; todo parâmetro obrigatório precisa de argumento (`E_FORM_PARAM`).
- Campo declarado e não usado por botão é `W_FORM_UNUSED_FIELD`; campos de contexto podem ser `readonly`.
- Papel/permissão vêm de `roles` do form e das roles da action.
- Rascunho não grava no documento; só o botão persiste.
- A conclusão é atômica: escrita + evento + auditoria + avanço do passo no mesmo commit.
- Botão irreversível exige confirmação derivada do contrato.
- Devolver é uma action (`calls: Order.reject(reason)`), contabilizada como retrabalho.
- **Form não é View.** Form é escrita/desfecho; View é leitura/navegação. Não existe `view type: form`.

## 4.15 Message — contrato entre participantes

```archol
message PickupRequest {
  title: { pt-BR: "Solicitação de coleta", en: "Pickup request" }
  about: Shipment
  payload {
    requestedAt: DateTime { title: { pt-BR: "Solicitado em" } }
    window: Duration { title: { pt-BR: "Janela" } }
  }
}
```

| Cláusula | Sintaxe | Funcionamento |
| :--- | :--- | :--- |
| `message` | `message <Name> { ... }` | Contrato de mensagem entre participantes. |
| `title` | `title: std.bpcms.i18n` | Obrigatório. |
| `about` | `about: Shipment` | Documento de correlação: a mensagem pertence a um caso. |
| `payload` | `payload { campo: Shape { title } }` | Contrato tipado do conteúdo. |

- A correlação é **por documento** (`about`): o runtime entrega a mensagem à instância que tem variável daquele documento.
- Envio: `send Message { key: doc; with { ... } }`.
- Recebimento: `receive Message { key: doc; timeout; apply: Doc.action }`; sem `apply`, devolve o payload.
- `key` é o documento de correlação; `with` mapeia campos do payload.
- Mensagem sem `about` é erro (`E_MESSAGE_ABOUT`).
- O parceiro/canal é configuração de runtime/integração; não há `partner`/pool.

## 4.16 Workflow × processo

**Workflow é do Documento; processo é do trabalho.** Um estado não tem executor, fila, prazo nem formulário; ele precisa valer fora de qualquer processo.

- As transições vivem no `value` de origem (`transition`), com `title`/`payload`/`event:`; a action só aponta `to:` e executa o comando.
- O processo **não duplica** o estado: chama a action, e o documento decide se a transição é legal.
- A tela de andamento mostra as duas coisas: passo do processo (trabalho) e situação do documento (dado).
- **Só action muda estado.** Não existe `status =` em expressão, form ou task; `status` é somente leitura fora da action que o transiciona.
- Decisão humana é lida no estado do documento, não em retorno do form.

## 4.17 Decision — regra de negócio (DMN)

```archol
decision ApprovalLimit {
  title: { pt-BR: "Alçada de aprovação", en: "Approval limit" }
  returns: ApproverTier
  inputs {
    amount: std.Money
    supplier: Supplier
  }
  cases {
    amount <= 5000 BRL -> Auto
    amount <= 50000 BRL and supplier.risk == Low -> Manager
    else -> Director
  }
}
```

- O processo executa com `queue = ApprovalLimit.evaluate({ amount, supplier })`.
- Decisão é **pura**: não altera documento, não chama integração, não emite evento.
- O resultado vira variável do topo e o `if` decide o caminho.
- Toda decisão tem `else` (`E_DECISION_NO_ELSE`) e todo `case` tem valor do tipo de retorno, na grafia declarada do enum (`E_DECISION_VALUE`).
- DMN completo (DRD executável, FEEL, BKM, múltiplas saídas) fica fora desta versão: hit policy única (primeiro que casa), uma saída.

## 4.18 Cobertura BPMN / DMN / CMMN

| Conceito | Suporte | Como |
| :--- | :--- | :--- |
| Start event (evento, mensagem, timer, agenda) | ✅ | `on` / `message` / `schedule` / `by` / `called` |
| End event nomeado | ✅ | `end` + `return` |
| User task / people task | ✅ | `Form.show` / `View.show` |
| Manual task | ✅ | `Manual.show` |
| Service task (documento) | ✅ | `Doc.action` |
| Service task (integração) | ✅ | `Integracao.op` |
| Send / receive task | ✅ | `send` / `receive` |
| Business rule task (DMN) | ✅ | `Decision.evaluate` |
| Call activity / subprocesso | ✅ | `Processo.run` |
| Multi-instance | ✅ | `forEach` / `parallel each` |
| Exclusive gateway | ✅ | `if` / `else if` / `else` |
| Parallel gateway | ✅ | `parallel` |
| Event-based gateway | ⏸️ | `receive` + `timeout`; "A ou B" fica para depois |
| Inclusive gateway | ❌ | ambíguo; use `if` ou `parallel` |
| Complex gateway | ❌ | nenhum motor de mercado implementa |
| Boundary timer / error | ✅ | `timeout` / `try/catch` |
| Boundary message / signal / escalation | ⏸️ | `receive` / `cancel on` |
| Event sub-process | 🟡 | `cancel on <Evento> { ... }` |
| Intermediate timer catch | ✅ | `wait` |
| Intermediate message catch | ✅ | `receive` |
| Compensation | ✅ | `catch`/`finally` explícitos |
| Terminate | 🟡 | erro não tratado / `cancel on` |
| Pools / lanes / message flows | ✅ | `roles` + `send`/`receive` |
| Collaboration diagram | ✅ | derivado de roles + mensagens |
| Choreography / conversation | ⏸️ | visão derivada (depois) |
| CMMN (case) | 🟡 | adiado (`milestone`/`discretionary`) |
| BPMN DI (layout) | ⏸️ | passo futuro |
| DMN 1.3 (tabela única) | ✅ | `decision` |

## 4.19 Validações do processo

| Regra | Erro |
| :--- | :--- |
| mais de um start / start ausente | `E_PROCESS_START` |
| `on`/`message`/`fanOut` sem `key` | `E_CASE_KEY` |
| payload incompatível com args | `E_START_PAYLOAD` |
| declaração fora do topo | `E_BLOCK_DECLARATION` |
| variável usada sem atribuição em algum caminho | `E_DEFINITELY_ASSIGNED` |
| operação/task/artefato inexistente | `E_UNKNOWN_TARGET` |
| atribuição incompatível | `E_ASSIGN_TYPE` |
| `catch` de erro não declarado | `E_CATCH_UNKNOWN` |
| `loop` sem `max` | `E_LOOP_BOUND` |
| `return` para end inexistente | `E_RETURN_UNKNOWN` |
| `end` antes do fim do processo | `E_END_ORDER` |
| form/manual sem `roles` | `E_ROLE` |
| `parallel` sem branches | `E_PARALLEL_EMPTY` |
| escrita conflitante em branches | `W_PARALLEL_WRITE` |
| ação de form sem `calls`/`returns` | `E_FORM_ACTION` |
| `send`/`receive` sem `key` | `E_MESSAGE_KEY` |
| `receive` com `apply` incompatível | `E_RECEIVE_APPLY` |
| `fanOut` sem documento/expressão | `E_FANOUT` |
| `timeout`/`sla` inválidos | `E_DURATION` |

---

# 5. Document

Um **documento** é a unidade de estado durável: campos tipados por Shape, identidade, invariantes, ações, eventos, ciclo de vida e sincronização. Ele **não é** tabela, DTO, tela, evento nem relatório: ele **projeta** tabelas, APIs, caches, estado de UI, índices e relatórios. O documento é o **armazenamento definitivo dos dados do processo**: a variável do processo é contexto transitório; o dado que precisa sobreviver, ser consultado e ser auditado mora no documento.

| Parte | O que declara | Exemplo |
| :--- | :--- | :--- |
| **Identidade** | id implícito, versão, etag, timestamps | `RemessaId` implícito |
| **Campos** | nome, Shape, parâmetros simples, lookup | `weight: std.Mass` |
| **Ciclo de vida** | campo de estado, valor inicial, transições | `status: ShipmentStatus` |
| **Ações** | comandos, parâmetros, papéis, transição, evento, retorno | `dispatch()` |
| **Eventos** | contrato do que o documento publica | `Dispatched` |
| **Regras** | invariantes | `weight <= limit` |
| **Índices** | único, range, composto, textual | `status + createdAt` |
| **Frescor** | TTL, agenda, estado | `FreightQuote` a cada 15 min |
| **Origem** | normal, calculado, dispositivo, snapshot | balança, nota fiscal |
| **Sync** | read/live, política de conflito | documento aberto por 3 pessoas |
| **Auditoria** | comando, ator, diff, correlação | automática por action |

## 5.1 Declaração e metadados

```archol
document Shipment {
  title: { pt-BR: "Remessa", en: "Shipment" }
  sync: live
  retention: 5y
}
```

| Cláusula | Sintaxe | Funcionamento |
| :--- | :--- | :--- |
| `document` | `document <Name> { ... }` | Declara o documento. |
| `scopable document` | `scopable document Store { ... }` | Documento que é tipo de escopo (5.9). |
| `extends` | `document Consumer extends User { ... }` | Especialização aditiva, mesmo id (5.9). |
| `title` | `title: std.bpcms.i18n` | I18N obrigatório (1.9). |
| `sync` | `sync: live \| read` | `live` assina patches; `read` entrega leitura materializada. |
| `retention` | `retention: 5y` | Retenção antes de anonimizar/expurgar. |
| `capabilities` | `capabilities { capture, units }` | Capacidades exigidas pelo documento. |
| `fields` | `fields { name: Shape { ... } }` | Campos tipados (5.2). |
| `actions` | `actions { issue() { ... } }` | Comandos (5.4). |
| `events` | `events { Issued { ... } }` | Eventos publicados (5.7). |
| `indexes` | `indexes { byStatus: [status, createdAt] range }` | Índices declarativos (5.8). |
| `rule` | `rule total == subtotal + tax { message }` | Invariante (5.11). |
| `seed` | `seed { SP { code: "SP" } }` | Documentos pré-criados idempotentes (5.9). |
| `computed from` | `computed from Invoice, Payment` | Documento derivado (5.10). |
| `freshness` | `freshness: 15m` | TTL do derivado; expõe `calculatedAt`. |
| `exposes` | `exposes: [balance, calculatedAt]` | Campos resultantes do derivado. |
| `source: device` | `source: device "scale-01" for weight` | Campo lido de dispositivo (5.10). |
| `segregation` | `segregation { issue != cancel }` | Impede o mesmo ator nas duas ações (5.6). |
| `signature` | `signature { release: Pathologist }` | Exige role com autoridade de assinatura (5.6). |

## 5.2 Campos, Shape e DocumentRef

**Campo = nome + Shape + parâmetros simples.** O campo não valida, não formata e não converte: isso é do Shape. O campo só parametriza.

```archol
fields {
  code: StoreCode { title: { pt-BR: "Código", en: "Code" }, unique, lookup }
  name: string { title: { pt-BR: "Nome", en: "Name" }, lookup }
  weight: std.Mass { title: { pt-BR: "Peso", en: "Weight" }, unit: kg, min: 0.001 }
  items: InvoiceItem[] { title: { pt-BR: "Itens", en: "Items" } }
}
```

| Parâmetro | Sintaxe | Funcionamento |
| :--- | :--- | :--- |
| `title` | `title: std.bpcms.i18n` | Obrigatório; `short`/`tooltip` opcionais. |
| `required` | `required` | Campo obrigatório; `required: true` também é aceito. |
| `lookup` | `lookup` | Inclui o campo no lookup de exibição; a ordem é a ordem de declaração. |
| `unique` | `unique` | Unicidade lógica; índice realizando. |
| `readonly` | `readonly` | Campo não editável por action; derivado de `computed` ou de dispositivo. |
| `private` | `private internalScore: Score` | Campo interno; fora do contrato/API/serialização. |
| `sensitive` | `sensitive` | PII; mascara na leitura e proíbe log/export. |
| `read` / `edit` | `read: [Clerk, Manager]` | Papéis que podem ler/editar o campo. |
| `mask` | `mask { roles: [Clerk]; fields: [name] }` | Mascaramento na leitura (5.6). |
| `rule` | `rule value != null { message }` | Regra condicional do campo. |
| `default` | `default: BR` | Valor inicial; aplicado antes de `required`; não é regra de negócio. |
| `computed` | `computed: quantity * unitPrice` | Campo derivado; somente leitura; sem dependência circular. |
| `min` / `max` | `min: 0.001, max: 50000` | Faixa validada pelo Shape. |
| `maxLength` | `maxLength: 120` | Limite de texto. |
| `scale` / `precision` | `scale: 2` | Casas decimais e precisão. |
| `unit` / `currency` | `unit: kg` | Unidade/moeda do valor; default é a base do Shape. |

**`id` implícito.** Todo documento tem `id` interno, gerado, que não se declara. Um campo cujo tipo é um Document é um **`DocumentRef`**: carrega `id` + todos os campos `lookup` do alvo. Não existe Shape `<Name>Id` nem tipo `XxxId`. Exceção: `extends` compartilha o `id` do pai.

```archol
store: Store { title: { pt-BR: "Loja", en: "Store" } }
```

O valor conceitual:

```json
{ "id": "st_123", "code": "0001", "name": "Loja 1" }
```

- o campo não escolhe campos: o alvo marca com `lookup`;
- excluir o alvo não quebra o documento (referência lógica, sem cascade);
- o builder resolve o lookup na leitura ou materializa;
- archive persiste a projeção de lookup;
- escopo usa o `id`, nunca `code`/`name`.

**Número de negócio.** `id` é técnico; `number` é a chave exibida:

```archol
number: InvoiceNumber {
  format: "{fiscalYear.year:yy}-{store.code:4}-{sequence:6}"
  sequence: per(Store, FiscalYear)
  unique
}
```

## 5.3 Ciclo de vida — estados e transições no Shape, comandos na action

O Shape de estado declara os **valores** e as **transições**, e cada transição carrega o **evento do workflow** (nome, título e payload). A action executa o comando e aponta o estado de destino; ela **não redefine a aresta nem o evento**.

O **nascimento** do documento também é uma transição: o `initial` declara a **transição de entrada**, com o evento de criação. Evento de criação não mora em `events { }`.

Isso restaura a regra: **evento de workflow é definido no workflow**. O bloco `events { }` do documento fica só para eventos **sem transição** (5.7).

```archol
shape InvoiceStatus {
  title: { pt-BR: "Situação da nota", en: "Invoice status" }
  base: state
  initial: Draft with transition Requested {
    title: { pt-BR: "Nota solicitada", en: "Invoice requested" }
    payload {
      order: Order { title: { pt-BR: "Pedido" } }
    }
  }

  format: plain

  value Draft {
    title: { pt-BR: "Rascunho", en: "Draft" }
    icon: Draft
    ordinal: 10

    transition Issued {
      title: { pt-BR: "Nota emitida", en: "Invoice issued" }
      payload {
        issuedAt: DateTime { title: { pt-BR: "Emitida em" } }
        number: InvoiceNumber { title: { pt-BR: "Número" } }
      }
    }
  }

  value Issued {
    title: { pt-BR: "Emitida", en: "Issued" }
    icon: Issued
    ordinal: 20

    transition Paid {
      title: { pt-BR: "Nota paga", en: "Invoice paid" }
      payload {
        paidAt: DateTime { title: { pt-BR: "Paga em" } }
        amount: std.Money { title: { pt-BR: "Valor" } }
      }
    }
    transition Overdue {
      title: { pt-BR: "Nota vencida", en: "Invoice overdue" }
    }
    transition Cancelled {
      title: { pt-BR: "Nota cancelada", en: "Invoice cancelled" }
      payload {
        reason: string { title: { pt-BR: "Motivo" } }
      }
    }
  }

  value Overdue {
    title: { pt-BR: "Vencida", en: "Overdue" }
    icon: Overdue
    ordinal: 40

    transition Paid {
      title: { pt-BR: "Nota paga", en: "Invoice paid" }
      payload {
        paidAt: DateTime { title: { pt-BR: "Paga em" } }
        amount: std.Money { title: { pt-BR: "Valor" } }
      }
    }
  }

  value Paid {
    title: { pt-BR: "Paga", en: "Paid" }
    icon: Paid
    ordinal: 30
  }

  value Cancelled {
    title: { pt-BR: "Cancelada", en: "Cancelled" }
    icon: Cancelled
    ordinal: 50
  }
}
```

Regras do Shape de estado:

- `base: state` e exatamente um `initial`; valores com `title` + `icon` + `ordinal`;
- `transition <Destino>` é declarada **no valor de origem**; o destino é valor do mesmo Shape;
- a transição pode ser **sem evento** (`transition Cancelled`, uma linha) ou **com evento** (bloco com `title` obrigatório, `payload` opcional e `event: <Nome>` opcional);
- **nome do evento**: o do estado de destino (`Issued` → `Invoice.Issued`); `event: RefundIssued` sobrescreve;
- **título do evento**: o `title` da transição (é o rótulo mostrado em histórico, integração e assinatura);
- **payload**: contrato tipado do evento; é o mesmo para todas as origens que chegam ao mesmo evento;
- duas transições para o **mesmo evento** precisam declarar o **mesmo payload** (`E_EVENT_SCHEMA_CONFLICT`);
- o mesmo destino pode ser alcançado por origens diferentes com **eventos diferentes** (`Contract.Active` vindo de `Signed` e de `Renewed`);
- não existe `self-transition`; `to` que não é valor do Shape = `E_TRANSITION_STATE`;
- `status` é **somente leitura** fora da action que o transiciona; não existe `status = ...` em expressão, form ou task;
- estado sem transição de saída é terminal (legítimo, sem aviso);
- estado sem caminho a partir do `initial` = `W_STATE_UNREACHABLE`;
- transição sem nenhuma action que a execute = `W_TRANSITION_UNUSED`.

Regras da transição de entrada (criação):

- `initial: <Valor> with transition <Evento> { title?; payload? }` declara o evento de criação;
- o identificador após `transition` é o **nome do evento** (não há estado de origem; o destino é o próprio `initial`);
- a action que **cria** o documento (`returns: <Document>`, sem `to:`) emite o evento de entrada no body; é o único `emit` de evento de workflow permitido;
- sem bloco (`initial: Draft`) o documento nasce sem evento;
- mais de uma action de criação no mesmo documento publica o mesmo evento de entrada;
- evento de criação declarado em `events { }` = `E_CREATION_EVENT`.

## 5.4 Actions — comandos que executam transições

```archol
actions {
  issue() {
    title: { pt-BR: "Emitir", en: "Issue" }
    roles: [Manager]
    to: status.Issued

    rule items.length > 0 {
      pt-BR: "Adicione itens antes de emitir a fatura"
      en: "Add items before issuing the invoice"
    }

    returns: Invoice
    {
      issuedAt = now()
    }
  }

  pay(paidAt: DateTime) {
    title: { pt-BR: "Pagar", en: "Pay" }
    roles: [Clerk]
    to: status.Paid
    emits { amount: total }
  }

  rename(name: string) {
    title: { pt-BR: "Renomear", en: "Rename" }
    roles: public
  }
}
```

| Cláusula | Sintaxe | Funcionamento |
| :--- | :--- | :--- |
| `title` | `title: std.bpcms.i18n` | Rótulo do comando/botão; obrigatório. |
| `roles` | `roles: [Manager]` ou `roles: public` | Quem pode executar; obrigatório. |
| `to` | `to: status.Issued` | Estado de destino; a aresta e o evento vêm do `transition` do Shape (5.3). |
| `emits` | `emits { amount: total }` | Mapeia campos do payload cujo nome difere da origem. Não declara evento. |
| `returns` | `returns: Invoice` | Valor de retorno (um único tipo); usado por `gives:` no processo. |
| `rule` | `rule <expr> { I18N }` | Pré-condição; verdadeira = permitido. |
| `limits` | `limits { amount <= 50000 BRL: { roles: [Manager] } }` | Limite por papel (5.6). |
| `body` | `{ issuedAt = now() }` | Statements: atribuição de campo, `emit`, chamada interna. |

Regras:

- ação com `to:` executa a transição declarada no Shape; ação **sem** `to` é comando comum (`rename`, `create`) e não muda estado;
- a ação é legal apenas nos estados que declaram `transition <destino>` (`E_TRANSITION_MISSING`);
- a ação **não** declara `from:`: a origem permitida é a que tem a aresta, e o runtime valida o estado atual;
- `emits { ... }` só mapeia valores do payload: campo sem mapeamento vem **pelo nome** do escopo (parâmetros + campos do documento); mapear campo que não é do payload = `E_EMITS_UNKNOWN`;
- campo do payload sem origem (nem por nome, nem em `emits`) = `E_EVENT_PAYLOAD`;
- ação com `to:` **não** usa `emit` no body: o evento é o da transição;
- `emit <Event>` no body vale para (a) evento declarado em `events { }` e (b) o **evento de entrada** do documento, na action de criação (sem `to:`);
- criação é comando (`create(...)`) e declara `returns:` com o documento criado;
- toda ação mutante é auditada automaticamente (5.12);
- `roles: public` é explícito; `roles: []` é erro;
- ação privada usa `private action` e não é chamável de fora do serviço;
- parâmetros são tipados por Shape/Document e viram campos de form automaticamente (4.14);
- o retorno é único; para múltiplos valores, retorne um Shape composto.

**Atomicidade.** Concluir uma action (e concluir a task que a chamou) é um único commit: campos + estado + evento + auditoria + avanço do passo. Não existe estado meio-gravado. Rascunho de formulário é do runtime, não do documento.


## 5.5 Grafo de estados (derivado)

O compilador deriva o grafo de **duas** fontes, sem duplicação:

- **forma**: os valores do Shape de estado;
- **arestas**: as `transition` declaradas em cada valor (origem = o valor que a declara; destino = o alvo), com o evento da aresta;
- **comandos**: as actions cujo `to:` é o destino (quais comandos executam a aresta);
- **rótulos**: `title` da transição (rótulo do evento) e `title` das actions (botões);
- **órfãs**: transição sem nenhuma action que a execute (`W_TRANSITION_UNUSED`);
- **inalcançáveis**: valor sem caminho a partir do `initial` (`W_STATE_UNREACHABLE`);
- **sumidouros**: valor sem `transition` de saída (terminal legítimo).

Esse grafo alimenta o diagrama de estados, o botão habilitado/desabilitado na UI, o histórico e o teste de transição.

```text
Invoice: Draft --issue--> Issued --pay--> Paid
                  \--cancel--> Cancelled
         Issued --markOverdue--> Overdue --pay--> Paid
```

| Origem | Evento | Destino | Comando |
| :--- | :--- | :--- | :--- |
| `Draft` | `Invoice.Issued` | `Issued` | `issue` |
| `Issued` | `Invoice.Paid` | `Paid` | `pay` |
| `Overdue` | `Invoice.Paid` | `Paid` | `pay` |
| `Issued` | `Invoice.Cancelled` | `Cancelled` | `cancel` |

## 5.6 Permissões, mascaramento e limites

Não existe bloco `permissions` no documento. A autoridade é composta por:

| Nível | Onde | Efeito |
| :--- | :--- | :--- |
| execução | lane da task (processo) | quem recebe o passo |
| comando | `roles` da action | quem pode executar |
| campo — leitura | `read` do field | quem vê |
| campo — escrita | `edit` do field | quem altera |
| mascaramento | `mask` do field | o que aparece mascarado |
| limite | `limits` da action | faixa por papel |
| segregação | `segregation` do documento | atores diferentes em ações incompatíveis |
| assinatura | `signature` do documento | papel com autoridade de assinatura |

```archol
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

segregation {
  issue != cancel
}

signature {
  release: Pathologist
}
```

- `read`/`edit` escondem o campo do contrato e da serialização para outros serviços? Não: `private` faz isso. `read`/`edit` são autorização de UI/API;
- `mask` mascara na leitura para os papéis indicados (dado continua íntegro);
- `limits` é faixa por papel, avaliada em ordem no build e no runtime; o primeiro limite excedido aplica a restrição e a mensagem;
- `segregation` impede o mesmo ator nas duas ações; a ação pode ser qualificada pelo documento quando houver ambiguidade (`PurchaseRequest.submit != PurchaseRequest.approve`);
- `signature` exige role com autoridade de assinatura e registra ator + hora + hash do conteúdo assinado.

**Documento não é fronteira de segurança de task.** `reads`/`writes` não existem: a task não "ganha" permissão por declarar acesso. Quem autoriza é o documento (campo/comando) e quem executa é a lane. O AST deriva o conjunto lido/escrito para exibir os ícones de data object — **para exibir, nunca para autorizar**.

## 5.7 Eventos

**Evento de workflow fica no workflow.** O evento é declarado na `transition` do valor de origem (5.3) e publicado quando a action executa a transição. O bloco `events { }` do documento existe **só** para eventos sem transição. Não existe artefato `event` de topo.

### 5.7.1 Evento de workflow

```archol
shape InvoiceStatus {
  title: { pt-BR: "Situação da nota", en: "Invoice status" }
  base: state
  initial: Draft

  value Draft {
    title: { pt-BR: "Rascunho", en: "Draft" }
    icon: Draft
    ordinal: 10

    transition Issued {
      title: { pt-BR: "Nota emitida", en: "Invoice issued" }
      payload {
        issuedAt: DateTime { title: { pt-BR: "Emitida em" } }
        number: InvoiceNumber { title: { pt-BR: "Número" } }
      }
    }
  }
}

document Invoice {
  title: { pt-BR: "Nota fiscal", en: "Invoice" }

  fields {
    status: InvoiceStatus { title: { pt-BR: "Situação", en: "Status" }, lookup }
    issuedAt: DateTime { title: { pt-BR: "Emitida em", en: "Issued at" } }
    number: InvoiceNumber { title: { pt-BR: "Número", en: "Number" } }
  }

  actions {
    issue() {
      title: { pt-BR: "Emitir", en: "Issue" }
      roles: [Manager]
      to: status.Issued
      { issuedAt = now() }
    }
  }
}
```

- a action não declara evento: `to: status.Issued` executa a transição `Draft → Issued`, que publica `Invoice.Issued`;
- o payload vem por nome do escopo (`issuedAt`, `number` são campos do documento);
- nome do evento = estado de destino; `event: <Nome>` na transição sobrescreve;
- o valor do evento é a `title` da transição;
- duas origens para o mesmo destino podem emitir **o mesmo** evento (`Issued` e `Overdue` → `Paid`): o payload precisa ser idêntico (`E_EVENT_SCHEMA_CONFLICT`);
- origens diferentes podem emitir **eventos diferentes** para o mesmo destino (`Signed` e `Renewed` → `Active`);
- transição sem bloco (`transition Cancelled`) não publica evento.

### 5.7.2 Evento de criação

O nascimento do documento é uma transição de entrada, declarada no `initial` (5.3). O evento é do workflow; a action de criação o emite.

```archol
shape InvoiceStatus {
  title: { pt-BR: "Situação da nota", en: "Invoice status" }
  base: state

  initial: Draft with transition Requested {
    title: { pt-BR: "Nota solicitada", en: "Invoice requested" }
    payload {
      order: Order { title: { pt-BR: "Pedido" } }
    }
  }

  value Draft {
    title: { pt-BR: "Rascunho", en: "Draft" }
    icon: Draft
    ordinal: 10
  }
}

document Invoice {
  title: { pt-BR: "Nota fiscal", en: "Invoice" }

  fields {
    order: Order { title: { pt-BR: "Pedido", en: "Order" } }
    status: InvoiceStatus { title: { pt-BR: "Situação", en: "Status" }, lookup }
  }

  actions {
    request(order: Order) {
      title: { pt-BR: "Solicitar nota", en: "Request invoice" }
      roles: [Clerk]
      returns: Invoice
      {
        customer = order.customer
        emit Requested { order: order }
      }
    }
  }
}
```

- `initial: Draft with transition Requested { ... }` declara o contrato do evento de criação;
- a action de criação tem `returns: <Document>` e **não** tem `to:`; ela emite o evento de entrada;
- o mesmo documento pode ter várias actions de criação (`registerPerson`, `registerOrganization`): todas emitem o mesmo evento;
- evento de criação em `events { }` = `E_CREATION_EVENT`; `emit` de entrada em ação com `to:` = `E_EMIT_WITH_TRANSITION`.

### 5.7.3 Evento sem workflow

```archol
document Store {
  title: { pt-BR: "Loja", en: "Store" }

  events {
    Renamed {
      title: { pt-BR: "Loja renomeada", en: "Store renamed" }
      payload {
        oldName: string { title: { pt-BR: "Nome anterior" } }
        newName: string { title: { pt-BR: "Nome novo" } }
      }
    }
  }

  actions {
    rename(name: string) {
      title: { pt-BR: "Renomear", en: "Rename" }
      roles: [Manager]
      {
        oldName = this.name
        this.name = name
        emit Renamed { oldName; newName: name }
      }
    }
  }
}
```

- `events { }` declara o contrato (título + payload) do evento que **não** é transição;
- `emit <Event> { campo: expr }` no body publica; campo não mapeado vem por nome;
- `emit` é proibido em action com `to:` (o evento de transição já é o da aresta);
- evento declarado e nunca emitido = `W_EVENT_UNUSED`; emitido e não declarado = `E_EVENT_UNKNOWN`.

### 5.7.4 Identidade e regras

- o envelope (documento, id, ator, hora, correlação) é do builder; o `payload` é só negócio;
- o evento pertence ao serviço dono do documento; a referência é `SERVICO.DOCUMENTO.EVENTO` (`Billing.Invoice.Paid`); canônica: `example.com/services/billing#Invoice.Paid`;
- não existe `key`: a chave é o documento (id) + correlação do envelope;
- dois eventos com o mesmo nome no documento = `E_EVENT_DUPLICATE`;
- processos assinam `Document.Event` no `start { event: }`, `cancel on` e `milestone on`;
- renomear estado (ou `event: <Nome>`) renomeia o evento: é breaking change para quem assina.

## 5.8 Índices e número

```archol
indexes {
  byStore: [store.id, createdAt] range
  byStatus: [status, createdAt] range
  byNumber: [number] unique
}
```

- `unique` (unicidade), `range` (ordenação/faixa), `search` (texto), `hash` (igualdade/partição);
- o campo precisa existir e ser indexável no Shape (`E_INDEX`);
- `.id` de referência é indexável;
- índice é declaração semântica; o builder escolhe a realização.

## 5.9 Escopos, extensão e seed

**`scopable document`.** Um documento marcado é um tipo de escopo. Todo campo direto para ele **infere um escopo** no documento consumidor; não há bloco `scopes`, `parentId` nem `path`.

```archol
scopable document Store {
  title: { pt-BR: "Loja", en: "Store" }
  fields {
    code: StoreCode { title: { pt-BR: "Código", en: "Code" }, unique }
    name: string { title: { pt-BR: "Nome", en: "Name" }, lookup }
  }
}

document Invoice {
  fields {
    store: Store { title: { pt-BR: "Loja", en: "Store" } }
    fiscalYear: FiscalYear { title: { pt-BR: "Exercício", en: "Fiscal year" } }
  }
}
```

- o escopo usa `store.id` e `fiscalYear.id`;
- `code`/`year` são chaves de negócio e entram no lookup;
- referências indiretas não inferem escopo;
- duas referências ao mesmo tipo de escopo no mesmo documento = erro;
- opcional cria escopo opcional, com política explícita.

**`extends`.** Especialização aditiva, no mesmo serviço:

```archol
document Consumer extends User {
  fields {
    citizen: Citizen { title: { pt-BR: "Cidadão", en: "Citizen" } }
  }
}
```

- herda campos, ações, regras e lookup;
- mantém o **mesmo `id`** do pai (não cria segundo id);
- não redeclara campo herdado; `citizen` não é unique.

**`seed`.** Documentos pré-criados e idempotentes:

```archol
scopable document Store {
  seed {
    SP { code: "SP"; name: "São Paulo" }
  }
}
```

- aplicado no bootstrap/migração; idempotente; valida `required`, `rule`, unique e referências;
- id determinístico por serviço/documento/chave; não executa actions; não emite eventos;
- a chave vira referência: `Store.SP` é um `DocumentRef` resolvido em build, usável em fields, actions e processos;
- renomear seed é breaking change.

## 5.10 Documentos derivados e dispositivos

**Derivado (cálculo).** Um documento pode ser o resultado de um cálculo sobre outros documentos. Ele é **estado durável** (não é relatório): serve quando o número precisa ser consultado, versionado ou auditado depois.

```archol
document CustomerScore {
  title: { pt-BR: "Score do cliente", en: "Customer score" }
  computed from Order, Payment, Ticket
  freshness: 15m
  exposes: [score, tier, calculatedAt]

  fields {
    score: std.Count { title: { pt-BR: "Score", en: "Score" } }
    tier: RiskTier { title: { pt-BR: "Faixa", en: "Tier" } }
  }
}
```

- o cálculo de cada campo é declarado no próprio campo com `computed:` (expressão sobre as fontes de `computed from`); `exposes` apenas lista o que é público;
- expõe `calculatedAt`, `state` (`fresh`, `stale`, `recalculating`, `error`) e `lastGood`;
- recálculo é um processo agendado (`start { schedule: }`) ou um comando `refresh()`;
- o consumidor exibe "calculado há X" e "recalculando";
- **relatório é View** (seção 7), não documento: se o número é só leitura agregada, não crie documento.

**Dispositivo.** `source: device "scale-01" for weight`: o campo é lido, não calculado; a atualização é releitura, com `readAt`, `staleAfter` e qualidade. A leitura pode gerar um documento derivado de negócio (`Pesagem` → `Shipment.weight`).

## 5.11 Regras

A mesma sintaxe vale no Document, na action e no field. Condição verdadeira = permitido.

```archol
rule total == subtotal + tax {
  pt-BR: "Total da nota não confere"
  en: "Invoice total does not match"
}

customer: User {
  title: { pt-BR: "Cliente", en: "Customer" }
  rule value != null {
    pt-BR: "Selecione o cliente da nota"
    en: "Select the invoice customer"
  }
}
```

- o compilador extrai os campos usados na expressão (para navegação/foco na UI);
- a API devolve `{ type, fields, message }`;
- `required` para obrigatoriedade simples; `rule` para condicional;
- `default` aplica antes de `required`; default incompatível com tipo = erro;
- permissão e workflow continuam com mensagem genérica do builder.

## 5.12 Auditoria

Não existe `audit()` na DSL. Toda action é um comando auditado automaticamente:

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

O registro é responsabilidade do builder. A DSL não declara `audit` no documento.

## 5.13 Sync, ciclo de vida e conflito

| Modo | O que o consumidor pede | O que recebe | Quando usar |
| :--- | :--- | :--- | :--- |
| `read` | consulta/snapshot | retrato com versão | relatório, consulta, integração |
| `live` | assinatura do documento | snapshot + patches | edição colaborativa, painel |

- o servidor é autoritativo; o cliente pode ser otimista e reconcilia;
- concorrência otimista por versão/etag; conflito devolve o estado atual e rejeita a ação;
- merge por campo e offline ficam para a v2;
- lifecycle de armazenamento: `draft`, `valid`, `stale`, `conflict`, `archived`, `deleted` (5.10).

## 5.14 Erros e idempotência

- action pode falhar com `throw <Error> { ... }`; o erro tem nome estável e mensagem i18n;
- `on error` no processo reage ao nome do erro;
- `Integration` declara idempotência; actions mutantes devem ser idempotentes por `(documentId, command, requestId)`;
- retry de action não pode duplicar efeito;
- `E_INVALID_TRANSITION` em runtime quando o estado mudou entre a leitura e o commit.

## 5.15 Validações do documento

| Regra | Erro |
| :--- | :--- |
| campo `id` declarado | `E_DUPLICATE_ID_FIELD` |
| campo de estado sem `base: state` ou sem `initial` | `E_STATE_SHAPE` |
| `transition` na value para estado inexistente | `E_TRANSITION_STATE` |
| self-transition | `E_TRANSITION_SELF` |
| ação com `to:` para destino sem transição declarada | `E_TRANSITION_MISSING` |
| ação sem `title` ou sem `roles` | `E_ACTION` |
| evento sem declaração (`emit`) / payload sem origem / mapeamento inválido | `E_EVENT_UNKNOWN` / `E_EVENT_PAYLOAD` / `E_EMITS_UNKNOWN` |
| evento do mesmo nome com payload divergente / `emit` em ação com `to:` | `E_EVENT_SCHEMA_CONFLICT` / `E_EMIT_WITH_TRANSITION` |
| evento de criação em `events { }` (deve ir no `initial`) | `E_CREATION_EVENT` |
| transição sem ação que a execute | `W_TRANSITION_UNUSED` |
| dois eventos com o mesmo nome | `E_EVENT_DUPLICATE` |
| valor de estado inalcançável | `W_STATE_UNREACHABLE` |
| `extends` fora do serviço / redeclaração | `E_EXTENDS` |
| documento sem campo `lookup` | `E_LOOKUP_MISSING` |
| índice em campo inexistente/não indexável | `E_INDEX` |
| campo `computed` com ciclo | `E_CYCLE` |
| `computed from` sem `freshness` | `W_DERIVED_NO_FRESHNESS` |

---
# 6. Shape

| Cláusula | Sintaxe | Funcionamento |
| :--- | :--- | :--- |
| declaração | `shape <Name> { ... }` | Tipo de dado; dono de semântica. |
| `title` | `title: { pt-BR: "...", en: "..." }` | I18N obrigatório (ver 1.9). |
| `base` | `base: decimal \| integer \| string \| classification \| state \| attachment \| audio \| image \| video \| richtext` | Representação base; define exibição/edição. `state` é o enum de ciclo de vida (com `initial`). |
| `capabilities` | `capabilities { attachments }` | Capacidades exigidas pelo Shape; falha de build se ausentes. |
| `mime` | `mime: [image/*, application/octet-stream]` | Restrição de MIME para `attachment` genérico. |
| `classification` | `classification: "<url>"` | Classificação externa; reservada à stdlib (`Currency`). |
| `value` | `value Draft { title; icon; ordinal }` | Enum; `classification` ou `state`. **Não declara transição**: a aresta vive na action do Document (`from:`/`to:`); o **evento do workflow** é declarado aqui (`event`), não em `events { }`. Exceção: `value` base de `std.Currency` não exige metadados. |
| `initial` | `initial: Draft` | Estado inicial do `state`; obrigatório e único. |
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

```archol
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

```archol
toNumber(value) {
  return value.raw;
}
```

`method toNumber(value)` é inválido.

## 6.1 Valores enumerados

Cada value é um bloco com `title` (I18N), `icon` e `ordinal` obrigatório. Um Shape `classification` enumera categorias; um Shape `state` declara o ciclo de vida (valores + `initial`) e **não declara transições** — elas ficam nas actions do Document (5.3–5.5).

```archol
shape InvoiceStatus {
  title: { pt-BR: "Situação da nota", en: "Invoice status" }
  base: state
  initial: Draft
  format: plain

  value Draft {
    title: { pt-BR: "Rascunho", en: "Draft" }
    icon: Draft
    ordinal: 10
  }
  value Issued {
    title: { pt-BR: "Emitida", en: "Issued" }
    icon: Issued
    ordinal: 20
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
- `state` tem exatamente um `initial`; a legalidade de cada transição é validada pela action do Document (`E_TRANSITION_STATE`).
- `classification` não tem `initial` nem participa de ciclo de vida.

## 6.1.1 Units

`units` é um mapa de unidade → fator relativo à unidade base (fator `1`). Unidades não lineares usam `formula { to/from }`.

```archol
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

```archol
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

```archol
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

## 6.1.2 classification e state

`base: classification` enumera categorias; `base: state` declara o **ciclo de vida** de um documento: valores possíveis e valor inicial. **As transições não ficam no Shape**: ficam nas actions do Document (5.3–5.5).

```archol
shape OrderStatus {
  title: { pt-BR: "Situação do pedido", en: "Order status" }
  base: state
  initial: Draft

  value Draft {
    title: { pt-BR: "Rascunho", en: "Draft" }
    icon: status.draft
    ordinal: 10
  }
  value Placed {
    title: { pt-BR: "Realizado", en: "Placed" }
    icon: status.placed
    ordinal: 20
  }
  value Cancelled {
    title: { pt-BR: "Cancelado", en: "Cancelled" }
    icon: status.cancelled
    ordinal: 30
  }
}

shape TaxRegimeCode {
  title: { pt-BR: "Regime tributário", en: "Tax regime" }
  base: classification
  value SIMPLES {
    title: { pt-BR: "Simples Nacional", en: "Simples Nacional" }
    icon: tax.simples
    ordinal: 10
  }
}
```

Regras:

- `state` tem exatamente um `initial`; `classification` não tem `initial`;
- `transition` é a aresta do workflow (5.3); a action só aponta o destino (`to:`), sem `from:`;
- a **transição e o evento do workflow** são declarados no próprio `value` de origem (`transition <Destino> { title; payload; event? }`); a action só aponta `to:` (5.3, 5.7);
- o `initial` pode declarar a **transição de entrada** (`with transition <Evento> { ... }`): é o evento de criação do documento (5.3, 5.7.2);
- o grafo de estados é derivado das actions; estado inalcançável gera `W_STATE_UNREACHABLE`;
- estado sem action de saída é terminal (legítimo, sem aviso);
- self-transition é inválida (`E_TRANSITION_SELF`);
- ordinal continua explícito (10, 20, 30…) e é usado na ordenação de UI/relatório;
- `canTransitionTo` é gerado pelo compilador a partir das actions;
- `toOrdinal` é gerado para `classification`/`state`;
- `base: state` só tem efeito em campo de Document; um Shape `state` fora de documento é um enum comum.

---

### 6.1.1.3 formula e convert

Unidades não lineares usam `formula`:

```archol
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

```archol
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

```archol
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

```archol
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

No nível de expressão não há `let`, declaração de classes, funções aninhadas, async ou exceções customizadas.

### 6.4.1 `new` e instanciação

Classes e Shapes são instanciados com `new <Type>{...}`:

```archol
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

```archol
toNumber(value: Length): number {
  return value.raw;
}

write(value: Length, out: streamWriter) {
  out.writeString(value.unit ?? m);
  out.writeDouble(value.raw);
}
```

Inferido:

```archol
toNumber(value) {
  return value.raw;
}

write(value, out) {
  out.writeString(value.unit ?? m);
  out.writeDouble(value.raw);
}
```

Misto:

```archol
convert(value: Length, unit) {
  return value.raw * units[value.unit ?? m].factor / units[unit].factor;
}
```

- O tipo de retorno também pode ser explícito ou inferido.
- O compilador valida os tipos inferidos e gera `E_TYPE` se forem inválidos.
- Não existe forma sem argumento: `write(out)` é inválido; o `value` é obrigatório.

## 6.4.3 Operadores

```archol
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

```archol
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

View é **apresentação**: ela lê documentos, processos ou tarefas e não é dona de nada. Não tem ciclo de vida, não tem permissão própria além da de leitura, não grava dado e não executa passo. Quando precisa mudar algo, chama uma action do documento (que valida a transição) ou navega.

**View não é form.** Form é o contrato de escrita e desfecho de uma tarefa humana (4.14); View é tela de leitura e navegação. Não existe `view type: form`.

| Cláusula | Sintaxe | Funcionamento |
| :--- | :--- | :--- |
| `view` | `view <Name> { ... }` | Tela/projeção. |
| `title` | `title: std.bpcms.i18n` | Obrigatório (1.9). |
| `type` | `type: table \| detail \| report \| dashboard \| kanban \| calendar \| process \| inbox` | Discriminador. |
| `document` | `document: Customer` | Documento principal (`detail`). |
| `process` | `process: OrderFulfillment` | Processo exibido (`process`). |
| `source` | `source: Order` | Fonte de `table`/`kanban`/`calendar`. |
| `sources` | `sources: [Invoice, Order]` | Fontes de `report`/`dashboard`. |
| `roles` | `roles: [Clerk, Manager]` | Quem pode abrir a tela. |
| `columns` | `columns { number; customer; total; status }` | Colunas; herdam título/tipo do campo do documento. |
| `filters` | `filters { status; period; customer }` | Filtros disponíveis. |
| `filter` | `filter { assignee = currentUser }` | Filtro fixo/contextual. |
| `pagination` | `pagination: 50` | Tamanho da página. |
| `actions` | `actions { open -> OrderDetail; cancel -> Order.cancel(reason) }` | Navegação ou comando (um por linha). |
| `date` | `date: scheduledAt` | Campo de data exibido no `calendar`. |
| `groupBy` | `groupBy: status` | Agrupamento (`kanban`, `inbox`). |
| `card` | `card { subject; priority; slaDue; assignee }` | Campos do cartão (`kanban`). |
| `sections` | `sections { summary; items; timeline }` | Seções (`detail`, `process`). |
| `computed` | `computed { frete }` | Campos calculados exibidos (`detail`). |
| `device` | `device { scale-01 }` | Dispositivo exibido/consultado (`detail`). |
| `metrics` | `metrics { revenue: sum(Invoice.total) by region }` | Métricas (`report`, `dashboard`). |
| `freshness` | `freshness: 15m` | Frescor do resultado; exibe "atualizado há X". |
| `units` | `units { revenue: BRL }` | Unidade de exibição por métrica. |
| `export` | `export: [pdf, csv]` | Formatos de exportação. |
| `mode` | `mode: live` | `dashboard` em tempo real. |
| `scope` | `scope: mine \| lanes \| all` | Escopo da `inbox`. |
| `capabilities` | `capabilities { gui }` | Só quando a tela inteira exige GUI por outro motivo. |

## 7.1 Tipos de View

| Tipo | Fonte | Cláusulas típicas | Para quê |
| :--- | :--- | :--- | :--- |
| `table` | documento | `source`, `columns`, `filters`, `pagination`, `actions` | lista operacional |
| `detail` | documento | `document`, `sections`, `computed`, `device`, `actions` | ficha/visão 360 |
| `report` | documentos | `sources`, `metrics`, `filters`, `units`, `export`, `freshness` | relatório tabular/agregado |
| `dashboard` | documentos | `sources`, `metrics`, `filters`, `mode`, `freshness` | painel |
| `kanban` | documento | `source`, `groupBy`, `columns`, `card`, `filter`, `actions` | quadro por estado |
| `calendar` | documento | `source`, `date`, `filters`, `actions` | agenda/ocupação |
| `process` | processo | `process`, `roles`, `sections` | andamento de instâncias |
| `inbox` | tarefas | `scope`, `groupBy`, `columns`, `filter` | minhas tarefas / fila |

## 7.2 Leitura, comando e navegação

```archol
view Orders {
  title: { pt-BR: "Pedidos", en: "Orders" }
  type: table
  source: Order
  roles: [Clerk, Manager]

  columns { number; customer; total; status; placedAt }
  filters { status; period; customer }
  pagination: 50

  actions {
    open -> OrderDetail
    cancel -> Order.cancel(reason)
  }
}
```

- `open -> <View>` navega;
- `cancel -> Document.action` executa comando; o builder pede os parâmetros ao usuário;
- a View não grava campo: só a action grava, e o documento valida a transição;
- ação de View que não é possível no estado atual aparece desabilitada com o motivo do documento;
- `roles` da View é autorização de **abertura**; a autorização do comando continua na action.

## 7.3 Relatório e dashboard

**Relatório é View.** Não existe `document type: report` nem bloco `report { }` dentro de documento. O documento é estado; a agregação é projeção.

```archol
view SalesByRegion {
  title: { pt-BR: "Vendas por região", en: "Sales by region" }
  type: report
  sources: [Invoice, Order]
  roles: [Manager]

  filters { period; region }
  metrics {
    revenue: sum(Invoice.total) by region
    orders: count(Order) by region
  }
  units { revenue: BRL }
  freshness: 15m
  export: [pdf, csv]
}
```

- métrica é agregação pura (`sum`, `count`, `avg`, `percentage`, `bucket`) com `where`/`by`;
- `units` define a unidade de exibição; a conversão acontece na leitura;
- `freshness` é do cache de leitura, não do dado;
- a tela mostra "atualizado há X" e permite recálculo;
- quando o número precisa ser **durável** (fechamento contábil, laudo, score), ele é um **documento derivado** (5.10) — aí sim há estado, versionamento e auditoria.

```archol
view OperationsControl {
  title: { pt-BR: "Controle operacional", en: "Operations control" }
  type: dashboard
  sources: [Shipment, Incident]
  roles: [Supervisor]
  mode: live
  filters { site; status }
  metrics {
    late: count(Shipment) where status != Delivered and dueAt < now()
    openIncidents: count(Incident) where status != Closed
  }
}
```

## 7.4 Andamento de processo

```archol
view OrderProgress {
  title: { pt-BR: "Andamento do pedido", en: "Order progress" }
  type: process
  process: OrderFulfillment
  roles: [Clerk, Manager, Customer]

  sections { current; participants; history; deadlines; subprocesses; documents; milestones; next }
}
```

- lista as instâncias do processo e abre o andamento de uma;
- `sections` usa o vocabulário fixo de 4.13 (o builder não inventa seções);
- quando o passo atual é humano e o usuário tem a lane, o form é embutido e a pessoa preenche ali;
- quando o passo é automático ou externo, mostra o andamento **dentro do passo** (tentativas, heartbeat, último erro, prazo);
- subprocesso aparece como cartão com drill-down para a instância filha;
- a View não cria transição, não cancela e não reatribui: intervenções são runtime, autorizadas por role;
- documentos lidos/escritos aparecem com ícone de data object derivado do contrato.

## 7.5 Inbox e filas

A inbox é projeção de **tarefas**, não de documentos. Ela é gerada pelo builder; a View só configura escopo e apresentação.

```archol
view MyWork {
  title: { pt-BR: "Minhas tarefas", en: "My work" }
  type: inbox
  roles: [Clerk, Manager, SupportAgent]
  scope: mine
  groupBy: process
  columns { title; subject; lane; dueAt; priority }
}

view TeamQueue {
  title: { pt-BR: "Fila do time", en: "Team queue" }
  type: inbox
  roles: [Manager]
  scope: lanes
  filter { lane in currentUser.lanes }
  groupBy: lane
}
```

- `scope: mine` — tasks atribuídas ou pegáveis pelo usuário;
- `scope: lanes` — tasks das lanes do usuário (gestor vê a fila, não executa);
- `scope: all` — auditoria/operação;
- o cartão mostra título do passo, sujeito do caso, lane, prazo e prioridade — todos derivados do processo;
- claim, delegate e reassign são ações de runtime; a inbox só as expõe conforme a política do papel.

## 7.6 Regras

- View não tem `fields`: campo de formulário é do Form (4.14);
- View não tem `wizard`: fluxo guiado com passos é **processo** (cada passo é uma task com form);
- colunas e cartões herdam `title`/`tipo` do campo; nomes qualificados (`customer.name`) são permitidos;
- `computed` de View é só exibição; cálculo durável é campo `computed` do documento;
- `metrics`/`filters`/`units`/`export` só em `report`/`dashboard`;
- `sections`/`computed`/`device` só em `detail`; `sections` de `process` usa o vocabulário fixo;
- `groupBy`/`card`/`filter` em `kanban`; `columns` lista os valores de `groupBy` exibidos (opcional, default = todos);
- `scope`/`groupBy` em `inbox`;
- `sections` de `detail` é lista livre de campos/seções do documento; em `process` usa o vocabulário fixo de 4.13;
- `computed { }` de View apenas nomeia campos computados do documento — a expressão mora no documento;
- `capabilities { gui }` só quando a tela inteira exige GUI por outro motivo; mídia declara `gui` no Shape;
- View não declara `integration`: integração é do processo ou da action.

## 7.7 Validações

| Regra | Erro |
| :--- | :--- |
| `type` inválido ou ausente | `E_VIEW_TYPE` |
| cláusula incompatível com o tipo (`metrics` em `table`) | `E_VIEW_CLAUSE` |
| `source`/`document`/`process` inexistente | `E_REFERENCE` |
| coluna/cartão com campo inexistente | `E_VIEW_FIELD` |
| `type: process` sem `process` | `E_VIEW_PROCESS` |
| `sections` fora do vocabulário | `E_VIEW_SECTION` |
| ação de View para alvo não chamável | `E_VIEW_ACTION` |
| `type: form` ou `type: wizard` | `E_VIEW_TYPE` |

---
# 8. Role / Organogram

| Cláusula | Sintaxe | Funcionamento |
| :--- | :--- | :--- |
| `role` | `role <Name> { ... }` | Papel/permissões. |
| `title` | `title: { pt-BR: "...", en: "..." }` | I18N obrigatório (ver 1.9). |
| `permissions` | `permissions: [order.manage, report.view]` | Namespaces de capacidade/ação. |
| `capabilities` | `capabilities { equipment, laboratory-protocol }` | Capacidades exigidas pelo artefato; falha de build se ausentes. |
| `inherits` | `inherits: Clerk` ou `inherits: [Clerk, Auditor]` | Soma as permissões herdadas às permissões próprias. |
| `scopes` | `scopes: [Store, FiscalYear]` | Tipos de escopo da role/organograma. |
| `all` | `Store: {all}` | Todos os escopos do tipo; bare, sem `true`. |
| `only` | `Store: {only: [storeForSP, storeForRJ]}` | Lista de **referências a documentos**; escopo novo não entra. |
| `except` | `Store: {except: [storeForSP]}` | Todos, exceto as referências listadas; escopo novo entra. |
| `organogram` | `organogram <Name> { ... }` | Hierarquia e regras de autorização. |
| `hierarchy` | `hierarchy { Manager > Clerk > Customer }` | `>` = superior a; um caminho por linha. No AST vira `Hierarchy` → `HierarchyChain` → `RoleRef` (com `ref`/`refKind`, `service` e `path` após a análise). |
| `fourEyes` | `fourEyes { payment.approve requires Approver }` | Exige um segundo ator diferente, com a role indicada. |
| `segregation` | `segregation { shipment.pick != shipment.check }` | Impede que o mesmo ator faça as duas ações. |
| `delegates to` | `Supervisor delegates to Checker when absent for 7d` | Delegação temporária e condicional para uma role abaixo na hierarquia. |
| `signature` | `signature { Pathologist }` | Papel com autoridade de assinatura. |
| `requires` | `fourEyes { X requires Role }` | Papel/condição exigida. |
| `when absent` | `... when absent for 7d` | Condição e vigência da delegação. |

> Roles e organogram do serviço ficam no mesmo arquivo `<service>-organograms.archol`, com o organogram antes das roles.

```archol
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

  InvoiceManager delegates to Clerk when absent for 7d

  fourEyes {
    refund.approve requires InvoiceManager
  }
}
```

```archol
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
- a role herdeira mantém `title` e `scopes` próprios;
- permissões herdadas não mudam a identidade da role.

```archol
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

**Atribuição de papel.** Não existe `manages`: a autoridade de atribuição é derivada da **hierarquia**:

- `hierarchy { A > B > C }` declara arestas diretas `A → B` e `B → C`;
- uma role pode atribuir qualquer role **abaixo dela** na hierarquia (descendentes transitivos), dentro dos `scopes` do organograma;
- auto-atribuição é proibida;
- o alvo recebe as permissões do **papel atribuído**, não as do gerente;
- o gerente não precisa ter as permissões do papel que atribui;
- a operação é `assignRole(target, role)`, com `target != actor`;
- a atribuição é auditada.

`all`/`only`/`except` continuam valendo para os escopos do assignment:

- `all` inclui escopos presentes e futuros;
- `only` é fail-closed: escopo novo não entra;
- `except` é fail-open: escopo novo entra, exceto os listados;
- `all`, `only` e `except` são mutuamente exclusivos por escopo;
- `except: []` equivale a todos, mas `all` é a forma explícita recomendada.

**Sem `manages`.** Quando for necessário permitir atribuição **fora** da linha hierárquica, a evolução prevista é uma cláusula explícita `assigns [Role]`; ela não existe nesta versão.

**Projeção no AST.** O `ArtifactOrganogram` anotado (`--app`) ganha um array
`roles` derivado, sem criar uma segunda fonte: cada `OrganogramRole` aponta para
o `ArtifactRole` de topo por `ref` (id do lock), `refKind`, `service` e `path`,
e traz `manages` (descendentes transitivos), `inherits`, `permissions`, `title` e
`line` da declaração. O construtor chega à role sem recompilar a hierarquia.
No `ArtifactService` anotado, `permissions` reúne as permissões auto-criadas do
serviço (`ArtifactPermission`, uma por `namespace.action`) e `artifacts` indexa
todos os artefatos do serviço (`ArtifactPointer`, com `artifact`, `name`, `ref`,
`path` e `line`).

---

# 9. Evidence

| Cláusula | Sintaxe | Funcionamento |
| :--- | :--- | :--- |
| `evidence` | `evidence <Name> { ... }` | Fonte de entendimento (arquivo/modelo). |
| `kind` | `kind: form \| ddl \| mer \| figma \| screenshot \| handdrawn` | Tipo da evidência; guia a extração. |
| `file` | `file: "evidence/form.pdf"` | Arquivo/URL de origem. |
| `hash` | `hash: "sha256:..."` | Integridade/proveniência. |
| `confidentiality` | `confidentiality: internal \| restricted` | Classificação; afeta redaction. |
| `extracted` | `extracted { document X { ... } }` | Candidatos extraídos; sempre propostos. |
| `document` | `document <Name> { ... }` | Documento candidato. |
| `fields` | `fields { costCenter; amount }` | Campos candidatos. |
| `relation` | `relation Appointment.patient -> Patient` | Relação candidata. |
| `links` | `links { process PurchaseApproval.ReviewRequest; service "<uri>" }` | Liga ao modelo existente. |
| `status` | `status: proposed \| confirmed \| rejected \| needsReview` | Estado da revisão humana. |

**Funcionamento:** a LLM propõe; o humano confirma. `extracted` nunca altera o modelo direto; `links` aponta para o elemento que a evidência sustenta. `hash`/`file` garantem proveniência.

```archol
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

```archol
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

A decisão é uma **tabela DMN pura** (4.17): entradas tipadas, casos com primeira-condição-verdadeira e um único retorno tipado. O processo a executa com `Decision.evaluate({...})`; o `if` roteia pelo valor.

| Cláusula | Sintaxe | Funcionamento |
| :--- | :--- | :--- |
| `decision` | `decision <Name> { ... }` | Tabela de decisão. |
| `title` | `title: std.bpcms.i18n` | Obrigatório. |
| `returns` | `returns: ApproverTier` | Um único retorno (normalmente `classification`). |
| `inputs` | `inputs { amount: std.Money; supplier: Supplier }` | Entradas tipadas. |
| `cases` | `cases { <cond> -> <value> }` | Primeira condição verdadeira vence. |
| `else` | `else -> <value>` | Obrigatório. |

```archol
decision ApprovalLimit {
  title: { pt-BR: "Alçada de aprovação", en: "Approval limit" }
  returns: ApproverTier
  inputs {
    amount: std.Money
    supplier: Supplier
  }
  cases {
    amount <= 5000 BRL -> Auto
    amount <= 50000 BRL and supplier.risk == Low -> Manager
    else -> Director
  }
}
```

```archol
task DecideApproval {
  lane: System
  decision: ApprovalLimit
  with { amount: purchaseRequest.amount; supplier: purchaseRequest.supplier }
  gives: approverTier
}

gateway Tier {
  if approverTier == Manager -> ManagerReview
  if approverTier == Director -> DirectorReview
  else -> AutoApprove
}
```

- decisão não altera documento nem chama integração;
- `appliesTo` não existe: quem usa a decisão é a task, e é ela que declara `with`;
- `default` não existe: o `else` é obrigatório;
- a tabela é exportável em DMN 1.3 pelo builder.

---

# 12. Assumption

| Cláusula | Sintaxe | Funcionamento |
| :--- | :--- | :--- |
| `assumption` | `assumption <Id> { ... }` | Suposição explícita, com validação. |
| `statement` | `statement: "..."` | Afirmação assumida. |
| `impact` | `impact: high \| medium \| low` | Impacto se estiver errada. |
| `status` | `status: open \| confirmed \| rejected \| proposed` | Estado da validação. |
| `validate` | `validate: "Conferir com Logística"` | Como validar. |
| `due` | `due: 2026-11-20` | Prazo. |
| `links` | `links { process X.Y; decision Z }` | Elementos afetados. |
| `evidence` | `evidence: "Tabela de Frete 2026"` | Evidência que confirma/derruba. |

```archol
assumption A-001 {
  statement: "Todo frete é cotado por remessa"
  impact: high
  status: open
  validate: "Conferir com Logística"
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
| `title` | `title: { pt-BR: "...", en: "..." }` | I18N obrigatório; substitui `description`. |
| `type` | `type: Shape` | Shape do valor; obrigatório. |
| `required` | `required` | Falha no boot/uso se não houver valor; mutuamente exclusivo com `default`. |
| `default` | `default: "..."` ou literal Shape | Valor fallback; quando `type` é Shape, segue 1.1.2 (`default: "2d"`, `default: "500 BRL"`). |
| `min` / `max` | `min: 0, max: 10` ou literal Shape | Constraints do Parameter; com Shape, `min: "0 BRL"`, `max: "500 kg"`. |
| `maxLength` | `maxLength: 120` | Limite para texto, como no field. |

```archol
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
| `title` | `title: { pt-BR: "...", en: "..." }` | I18N obrigatório. |
| `type` | `type: Shape` | Shape do valor; obrigatório, mas o valor nunca serializa. |
| `required` | `required` | Falha no boot/uso se não houver valor. |
| `min` / `max` | `min: 0, max: 10` | Constraints do Secret. |
| `maxLength` | `maxLength: 120` | Limite para texto. |

```archol
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

```archol
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

```archol
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
| `target` | `target: browser \| server \| database \| mobile \| terminal \| document` | Alvo de geração/interpretação. |
| `mode` | `mode: [interpret, generate]` | Modos suportados. |
| `capabilities` | `capabilities: [gui, sync, identity, attachments, timezone, regionalization, units]` | Capabilities de alto nível que oferece (ver 1.7). |
| `consumes` | `consumes: [Shape, Document, View, Process]` | Artefatos que sabe realizar. |
| `produces` | `produces: [html, css, js]` | Saídas geradas. |
| `limitations` | `limitations { widgets: [...]; process: readOnly }` | O que não suporta; evita promessa falsa. |
| `widgets` | `widgets: [SignaturePad, BarcodeScanner]` | Widgets suportados/limitados. |
| `process` | `process: readOnly` | Processo só leitura naquele alvo. |
| `images` | `images: unsupported` | Imagens indisponíveis. |

```archol
builder WebApp {
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

```archol
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

> Locale é configuração da app/construtor; não é serviço. `I18N` de títulos é independente.

```archol
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

```archol
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
| `lock` | `lock: "<app>.lock"` | Lock do app; fixa os IDs estáveis (versões por serviço em evolução). |
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

```archol
release "example.com/services/catalog"@1.2.0 {
  service: "example.com/services/catalog"
  changes { add Product.barcode }
  compatibility: backward
  migrations: [M-2026-10-12]
  approvals: [Owner]
  rollback: "example.com/services/catalog"@1.1.3
}
```

```archol
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

```archol
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

**Relatório é View; não é documento.** Não existe artefato `report`, `document type: report` nem bloco `report { metrics }` dentro de documento.

| Necessidade | Onde fica |
| :--- | :--- |
| número agregado para ler, filtrar e exportar | `view type: report` / `type: dashboard` |
| número que precisa durar, versionar e ser auditado | `document` derivado (`computed from`, `freshness`, `exposes`) |
| número que precisa de histórico por período | documento derivado com `for` de período (ex.: `FiscalPeriod`) |
| painel operacional em tempo real | `view type: dashboard` com `mode: live` |

```archol
view CashFlow {
  title: { pt-BR: "Fluxo de caixa", en: "Cash flow" }
  type: report
  sources: [Invoice, Payment, Bill]
  roles: [Manager]
  filters { period; account }
  metrics {
    inflow: sum(Payment.amount) where status == captured
    outflow: sum(Bill.amount) where status == paid
    balance: inflow - outflow
  }
  units { inflow: BRL; outflow: BRL; balance: BRL }
  freshness: 1h
  export: [pdf, csv]
}
```

```archol
document MonthlyRevenue {
  title: { pt-BR: "Receita mensal", en: "Monthly revenue" }
  computed from Invoice, Payment
  freshness: 1h
  exposes: [revenue, calculatedAt]

  fields {
    period: FiscalPeriod { title: { pt-BR: "Período", en: "Period" }, lookup }
    revenue: std.Money { title: { pt-BR: "Receita", en: "Revenue" }, currency: BRL }
  }
}
```

Regras:

- `metrics`/`filters`/`units`/`export` pertencem à View;
- `computed from`/`freshness`/`exposes`/`calculatedAt` pertencem ao documento derivado;
- a View mostra "atualizado há X" e oferece recálculo; o documento guarda o resultado e o frescor;
- relatório é `view`: fica em `views/` (`<service>-<nome>-view.archol`, 30); documento calculado durável fica no arquivo do seu contexto;
- não há CRUD automático de relatório: leitura é projeção.

---
# 22. Event

**Evento de workflow fica no workflow.** O evento é declarado na `transition` do valor de origem, dentro do Shape de estado, e publicado quando uma action executa a transição (5.3 e 5.7). O bloco `events { }` do documento é só para eventos **sem** transição. Não existe artefato `event` de topo.

## 22.1 Evento de workflow

```archol
shape InvoiceStatus {
  title: { pt-BR: "Situação da nota", en: "Invoice status" }
  base: state
  initial: Draft with transition Requested {
    title: { pt-BR: "Nota solicitada", en: "Invoice requested" }
    payload {
      order: Order { title: { pt-BR: "Pedido" } }
    }
  }

  value Draft {
    title: { pt-BR: "Rascunho", en: "Draft" }
    icon: Draft
    ordinal: 10

    transition Issued {
      title: { pt-BR: "Nota emitida", en: "Invoice issued" }
      payload {
        issuedAt: DateTime { title: { pt-BR: "Emitida em" } }
        number: InvoiceNumber { title: { pt-BR: "Número" } }
      }
    }
  }

  value Issued {
    title: { pt-BR: "Emitida", en: "Issued" }
    icon: Issued
    ordinal: 20

    transition Paid {
      title: { pt-BR: "Nota paga", en: "Invoice paid" }
      payload {
        paidAt: DateTime { title: { pt-BR: "Paga em" } }
        amount: std.Money { title: { pt-BR: "Valor" } }
      }
    }
  }

  value Paid {
    title: { pt-BR: "Paga", en: "Paid" }
    icon: Paid
    ordinal: 30
  }
}

document Invoice {
  title: { pt-BR: "Nota fiscal", en: "Invoice" }

  fields {
    status: InvoiceStatus { title: { pt-BR: "Situação", en: "Status" }, lookup }
    issuedAt: DateTime { title: { pt-BR: "Emitida em", en: "Issued at" } }
    number: InvoiceNumber { title: { pt-BR: "Número", en: "Number" } }
  }

  actions {
    issue() {
      title: { pt-BR: "Emitir", en: "Issue" }
      roles: [Manager]
      to: status.Issued
      { issuedAt = now() }
    }
  }
}
```

- o evento é **da aresta**: `Draft --Issued--> Issued` publica `Invoice.Issued`;
- nome do evento = estado de destino; `event: RefundIssued` sobrescreve quando o nome de negócio difere do estado;
- `title` da transição é o título do evento; `payload` é o contrato;
- a action só aponta `to: status.Issued`; não existe `emits: <Evento>` nem `from:`;
- `emits { campo: expr }` na action só mapeia valores do payload cujo nome difere do escopo;
- duas origens para o mesmo evento exigem o mesmo payload (`E_EVENT_SCHEMA_CONFLICT`); origens diferentes podem emitir eventos diferentes para o mesmo destino.

## 22.2 Evento de criação

O documento nasce por uma **transição de entrada**, declarada no `initial` do Shape. O evento é do workflow; a action de criação o emite.

```archol
shape InvoiceStatus {
  title: { pt-BR: "Situação da nota", en: "Invoice status" }
  base: state

  initial: Draft with transition Requested {
    title: { pt-BR: "Nota solicitada", en: "Invoice requested" }
    payload {
      order: Order { title: { pt-BR: "Pedido" } }
    }
  }

  value Draft {
    title: { pt-BR: "Rascunho", en: "Draft" }
    icon: Draft
    ordinal: 10
  }
}

document Invoice {
  title: { pt-BR: "Nota fiscal", en: "Invoice" }

  fields {
    order: Order { title: { pt-BR: "Pedido", en: "Order" } }
    status: InvoiceStatus { title: { pt-BR: "Situação", en: "Status" }, lookup }
  }

  actions {
    request(order: Order) {
      title: { pt-BR: "Solicitar nota", en: "Request invoice" }
      roles: [Clerk]
      returns: Invoice
      {
        customer = order.customer
        emit Requested { order: order }
      }
    }
  }
}
```

- `initial: <Valor> with transition <Evento> { ... }`: o identificador é o **nome do evento** (o destino é o próprio inicial);
- a action de criação (`returns: <Document>`, sem `to:`) emite o evento de entrada;
- mais de uma action de criação no documento publica o mesmo evento;
- `events { }` não declara evento de criação.

## 22.3 Evento sem workflow

```archol
document Store {
  title: { pt-BR: "Loja", en: "Store" }

  events {
    Renamed {
      title: { pt-BR: "Loja renomeada", en: "Store renamed" }
      payload {
        oldName: string { title: { pt-BR: "Nome anterior" } }
        newName: string { title: { pt-BR: "Nome novo" } }
      }
    }
  }

  actions {
    rename(name: string) {
      title: { pt-BR: "Renomear", en: "Rename" }
      roles: [Manager]
      {
        oldName = this.name
        this.name = name
        emit Renamed { oldName; newName: name }
      }
    }
  }
}
```

- `events { }` declara título e payload do evento sem transição;
- `emit` no body publica; `emit` em action com `to:` é erro.

## 22.4 Identidade

```text
SERVICO.DOCUMENTO.EVENTO
Billing.Invoice.Paid
example.com/services/billing#Invoice.Paid
```

- `SERVICO` é o dono do documento; alias é sintaxe local e não altera a identidade canônica;
- o envelope publica a forma canônica; `producer` é implícito; `consumers` não pertencem ao evento;
- o evento carrega o documento (id + lookup) e a correlação; o `payload` é só negócio;
- `key` não existe: a chave é o documento.

## 22.5 Regras

- evento de workflow: declarado na `transition` do Shape; publicado pela action que executa a transição;
- evento de criação: declarado no `initial` (`with transition`); publicado pela action de criação via `emit`;
- evento sem workflow: declarado em `events { }`; publicado por `emit` no body;
- `emit` é proibido em action com `to:`; `emits { }` só mapeia payload;
- processo/integração assinam `Document.Event` (`start { event: }`, `cancel on`, `milestone on`);
- evento não declarado = `E_EVENT_UNKNOWN`; declarado e nunca emitido = `W_EVENT_UNUSED`;
- dois eventos com o mesmo nome no documento = `E_EVENT_DUPLICATE`;
- renomear o estado (ou o `event: <Nome>`) renomeia o evento: breaking change para quem assina.

# 23. Schedule

**Agendamento é um gatilho de início de processo, não um artefato.** Não existe `job`.

```archol
process CloseMonth for FiscalPeriod {
  title: { pt-BR: "Fechamento mensal", en: "Month close" }
  sla: "6h"

  start Monthly {
    schedule: "0 2 1 * *"
    select: fiscalPeriod.status == Open
  }

  task PostEntries {
    lane: System
    action: Ledger.postPeriod
    with { fiscalPeriod }
  }

  task Review {
    lane: Controllership
    form: MonthCloseReview
    sla: "4h"
  }

  Monthly -> PostEntries
  PostEntries -> Review
  Review.approved -> Closed
  Review.rejected -> PostEntries

  end Closed { title: { pt-BR: "Fechado", en: "Closed" }, outcome: success }
  end Failed { title: { pt-BR: "Falhou", en: "Failed" }, outcome: failure }
}
```

| O que era `job` | Como fica |
| :--- | :--- |
| `schedule` | `start { schedule: "cron" }` |
| `action` | `task { lane: System; action: ... }` |
| `filter` de registros | `start { schedule; select }` com `for` (fan-out: 1 instância por sujeito) |
| `retry`/`timeout`/`deadLetter` | `on error`, `sla`, `on timer` da task; DLQ é runtime |
| `batch`/`cursor` | runtime/integração; cursor durável, se necessário, é um documento |
| `dryRun` | `Environment` |
| `condition` (audiência) | `gateway` no início |
| `input` | `variables`/`with` do processo |
| painel, histórico, auditoria | a tela de andamento do processo |

**Por que.** Agendamento não é um tipo de trabalho diferente: é um gatilho. Modelá-lo como processo dá andamento, auditoria, SLA, retry por task, cancelamento e histórico de graça — e evita um segundo motor com regras próprias.

`*-jobs.archol` não existe mais: os agendamentos viram `start { schedule: }` no arquivo `<service>-<nome>-process.archol` do processo (30).

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

```archol
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
| Relógio | `now()`, `now() - 24h`, `today()` | Relógio do runtime (virtual em QA); `today()` é a data local do processo. |
| Data + duração | `openedAt + slaWindow`, `dueAt - now()` | Aritmética entre `DateTime`/`Date` e `Duration`; resultado tipado (`DateTime` ou `Duration`). |
| Comparação de duração | `slaWindow <= 4h` | Durações comparáveis; unidade normalizada pelo Shape. |
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
| State | `initial` ausente/duplicado; `value` com `transition` | `E_STATE_INITIAL` / `E_STATE_TRANSITION` |
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
| Tipo reservado | `shape std.bpcms.i18n` declarado | `E_RESERVED_TYPE` |
| Locale | chave de `title` fora de `locales` | `E_LOCALE_UNDECLARED` |
| Título | sem entrada para o locale default | `E_TITLE_MISSING` |
| Task | sem contrato, com dois contratos, executor ausente ou inválido | `E_TASK_CONTRACT_MISSING` / `E_TASK_KIND_CONFLICT` / `E_TASK_EXECUTOR_MISSING` / `E_TASK_BLACKBOX_POOL` |
| Fluxo | nó órfão, saída ≠ 1, nó inalcançável, caminho sem `end` | `E_FLOW_ORPHAN` / `E_FLOW_ARITY` / `W_FLOW_UNREACHABLE` / `E_FLOW_DEAD_END` |
| Gateway | sem `else`, dois `else`, alvo inexistente | `E_GATEWAY_NO_ELSE` / `E_GATEWAY_DUP_ELSE` / `E_FLOW_UNKNOWN_NODE` |
| Fork/join | ramo que não alcança o join; join sem duas entradas | `E_JOIN_DEADLOCK` |
| Boundary | handler órfão, duplicado ou alvo fora do escopo | `E_BOUNDARY` |
| Compensação | handler com aresta, fora de escopo ou alvo não concluído | `E_COMPENSATION_SCOPE` |
| Form | ação sem aresta nem `stay`, argumento sem campo, campo não usado | `E_FORM_ACTION_NO_EDGE` / `E_FORM_PARAM` / `W_FORM_UNUSED_FIELD` |
| Dados | `with` desconhecido/ausente, `gives` incompatível, variável não atribuída | `E_WITH_UNKNOWN` / `E_WITH_MISSING` / `E_GIVES_TYPE` / `E_DEFINITELY_ASSIGNED` |
| Estado | `from`/`to` inválido, incompleto, self-transition ou estado inalcançável | `E_TRANSITION_STATE` / `E_TRANSITION_INCOMPLETE` / `E_TRANSITION_SELF` / `W_STATE_UNREACHABLE` |
| Evento | não declarado, duplicado, payload sem origem, schema divergente, criação fora do `initial`, nunca emitido | `E_EVENT_UNKNOWN` / `E_EVENT_DUPLICATE` / `E_EVENT_PAYLOAD` / `E_EMITS_UNKNOWN` / `E_EVENT_SCHEMA_CONFLICT` / `E_CREATION_EVENT` / `W_EVENT_UNUSED` |
| Mensagem | sem `about`, `about` incompatível, duplicada | `E_MESSAGE_ABOUT` / `E_MESSAGE_DUPLICATE` |
| Start | sem gatilho, dois gatilhos, duplicado, `select` inválido | `E_START_DUPLICATE` / `E_SELECT` / `E_PROCESS_NO_START` |
| Chamada | filho sem `start { called }`, com gatilho de topo, ou processo de topo com `called` | `E_PROCESS_TRIGGER_CONFLICT` |
| Subprocesso | ciclo entre processos | `E_SUBPROCESS_CYCLE` |
| View | tipo inválido, cláusula incompatível, seção desconhecida | `E_VIEW_TYPE` / `E_VIEW_CLAUSE` / `E_VIEW_SECTION` |
| Arquivo | nome fora do padrão (app/service `<nome>-<tipo>.archol`; document/process/view/message `<service>-<nome>-<tipo>.archol`) ou `<nome>`/`<tipo>` divergente do principal | `E_FILE_NAME` |
| Arquivo | arquivo fora da pasta esperada, full path diferente da URI, documento fora de `documents/`, processo fora de `processes/`, view fora de `views/` ou message fora de `messages/` | `E_FILE_PATH` |
| Arquivo | arquivo de contexto principal (app/service/document/process/view/message) sem exatamente um principal | `E_FILE_PRINCIPAL` |
| Arquivo | principal não é o primeiro artefato do arquivo | `E_FILE_PRINCIPAL_FIRST` |
| Arquivo | arquivo compartilhado declara `application`, `service`, `document`, `process`, `view` ou `message` | `E_FILE_SHARED` |
| Arquivo | arquivo compartilhado contém tipo diferente do sufixo (ex.: `view` em `-shapes.archol`; `parameter` em `-secrets.archol`; `role` fora de `-organograms.archol`) | `E_FILE_SHARED_TYPE` |
| Arquivo | em `-organograms.archol`, o `organogram` não é o primeiro artefato (ou não existe) | `E_FILE_ORGANOGRAM_FIRST` |
| Arquivo | SVG fora de `icons/` ou namespace de ícone divergente | `E_FILE_ICON_PATH` |

**Classes de runtime:** `validation_error` (400), `conflict` (409), `forbidden` (403), `not_found` (404), `dependency_unavailable` (503), `rate_limited` (429), `internal` (500). Erros de linguagem são bloqueantes no `check`; erros de runtime seguem o modelo do BRAINSTORM (2.6.11).

---

# 27. Cobertura

| Artefato | Cobertura | Exemplos |
| :--- | :--- | :--- |
| Application | ✅ | 8 |
| Service | ✅ | 8 |
| Process | ✅ | 24 |
| Form | ✅ | 8 |
| Document | ✅ | 8 |
| Shape | ✅ | 9 |
| View | ✅ | 8 |
| Role/Organogram | ✅ | 8 |
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
| Message | ✅ | 4 |
| Report/Dashboard | ✅ | 7 |
| Event | ✅ | 6 |
| Schedule (não é artefato) | ✅ | 4 |
| Policy | ✅ | 6 |
| Icon | ✅ | 6 |

> Cobertura = todas as cláusulas de linguagem usadas nos exemplos estão documentadas. Campos de domínio (`name`, `email`, `id`) não entram na contagem. `Report` e `Job` deixaram de ser artefatos: viraram `View` e `start { schedule: }`.

---

# 28. Icon

`Icon` é um artefato **semântico** definido dentro de um `Service`. O nome do Icon é o identificador; o construtor/Theme decide qual imagem usar. Não há `symbol` nem binário no DSL.

| Cláusula | Sintaxe | Funcionamento |
| :--- | :--- | :--- |
| `icon` | `icon <Name> { ... }` | Ícone público do serviço. |
| `title` | `title: { pt-BR: "...", en: "..." }` | I18N obrigatório (ver 1.9). |

```archol
icon Draft {
  title: { pt-BR: "Rascunho", en: "Draft" }
}
```

**Nomes:** `Icon` não usa prefixo `Icon` nem prefixo do serviço. O escopo do nome é o **serviço**: `Draft` em Billing e `Draft` em Catalog são icons diferentes; dentro do mesmo serviço, dois icons com o mesmo nome são erro (`E_DUPLICATE_ICON`). `icon Email` também coexiste com `shape Email` e `integration Email`; o namespace é por tipo dentro do serviço.

**Resolução:** `value.icon` → nome do `Icon` → mapeamento do builder/Theme → imagem. O Icon não carrega imagem nem binário; é um token semântico. O builder pode trocar o conjunto visual sem mudar o DSL.

### 28.1 Icons file-based (SVG)

Além do `icon` declarado, qualquer arquivo `.svg` sob `<service>/icons/` vira um **Icon implícito**. `icons/` é contêiner: as subpastas dentro dele são namespaces de ícone, mas `icons` não entra no identificador.

```text
examples/services/example.com/services/billing/
  billing-service.archol
  billing-icons.archol          <- icons declarados (não-SVG)
  icons/
    status/
      draft.svg                -> status.draft
      paid.svg                 -> status.paid
      overdue.svg              -> status.overdue
    finance/
      invoice/
        issued.svg             -> finance.invoice.issued
```

- Arquivos e pastas usam **kebab-case**.
- O identificador é o caminho relativo a `icons/`, sem `.svg`, com segmentos **camelCase** unidos por `.`.
- Referência local: `icon: status.draft`.
- Referência por outro serviço: `icon: AliasDoServico.status.draft` (alias em `imports`).
- O `alt`/`aria-label`/`<title>` é o title do Icon; **não usa I18N**.
- Precedência do title: `aria-label` → `<title>` → `alt` → filename humanizado.
- Sem `aria-label`, `<title>` ou `alt`, o filename é o fallback:
  - `low-stock.svg` → `"Low Stock"`.
- Um SVG e um `icon` declarado com o mesmo identificador no mesmo serviço são conflito (`E_DUPLICATE_ICON`).
- SVG fora de `<service>/icons/` é `E_FILE_ICON_PATH`.
- O builder sanitiza o SVG; script/recursos externos não entram no contrato.

```archol
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

**Erros:** `E_ICON_REFERENCE` para `icon:` que não resolve; `E_DUPLICATE_ICON` para conflito de identificador; `E_ICON_UNRESOLVED` para falha do builder/Theme; `E_FILE_ICON_PATH` para SVG fora de `icons/`.

**Erros:** `E_ICON_REFERENCE` (value aponta para Icon inexistente), `E_ICON_UNRESOLVED` (Theme/builder não resolve a imagem do Icon), `E_DUPLICATE_ICON` (dois icons com o mesmo nome no mesmo serviço), `E_ENUM_VALUE_TITLE`, `E_ENUM_VALUE_ICON` e `E_ENUM_VALUE_DUPLICATE`.

# 29. bpcms.org/std — biblioteca padrão

`bpcms.org/std` é um **pseudo-serviço reservado**, fornecido pelo construtor. Ele não é publicado, não tem endpoint, não entra no lock do app e não pode ser sobrescrito.

Quem usa precisa importar:

```archol
imports {
  service "bpcms.org/std" as std
}
```

O alias é obrigatório; o uso é sempre qualificado:

```archol
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

```archol
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

```archol
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
- `prefix`/`suffix`/`infix` aceitam `string | I18N`.
- `scale` default vem da moeda; o field pode sobrescrever.
- Sem `prefix/suffix/infix`, regionalization decide.
- `sformat` fica fora desta versão.

`Money` usa `prefix`/`suffix`/`infix` da moeda:

```archol
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

```archol
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

- Definição: `examples/services/bpcms.org/std/std-service.archol` e `examples/services/bpcms.org/std/std-shapes.archol`.
- Segue a regra de service: `std-service.archol` é o arquivo do principal e `std-shapes.archol` é compartilhado (30).
- Fica na pasta normal `examples/services/bpcms.org/std/`; o construtor reconhece a URI reservada e não a trata como serviço publicável.
- Não é versionado pelo lock; a std acompanha o construtor/DSL e não declara `version`.
- Os Shapes de medida declaram `toNumber(value)` explicitamente.
- `Mass`, `Length`, `Volume`, `Percentage` e `Duration` implementam `write`, `read`, `parse`, `format` e `toNumber`; `Money.toNumber` é automático; `Currency` não é numérico.

> **Migração feita:** `Mass`, `Length`, `Volume`, `Percentage` e `Count` foram removidos dos serviços de negócio e são referenciados como `std.*`. `MoneyBRL` foi removido; valores monetários usam `std.Money`. `CurrencyCode` não é Shape; `Currency` é a classificação. `Email`, `Phone`, `Image`, `LocaleTag` e `YearMonth` continuam nos serviços de negócio.

---

# 30. Organização dos arquivos

`BPCMS_WORKSPACE` é o diretório raiz do workspace, passado ao compilador/construtor. Nos exemplos, `BPCMS_WORKSPACE = examples/`.

A unidade de organização do fonte é o **contexto principal**, não o tipo de artefato. Contexto principal é o artefato que dá sentido ao arquivo: uma `application`, um `service`, um `document`, um `process`, uma `view` ou uma `message`.

`view` e `message` são contextos principais porque têm identidade e contrato próprios: nome referenciável por outros artefatos e serviços, e contrato de leitura (`document`/`source`/`sources`/`process:`, `roles`) ou de mensagem (`about`, `payload`). Por isso cada view e cada message tem **arquivo próprio**, em `views/` e `messages/`; não dividem o arquivo do documento, do processo ou do serviço que as usa.

## 30.0 Caminhos

| Arquivo | Caminho |
| :--- | :--- |
| Aplicação | `<workspace>/applications/<fullpath app>/<app>-application.archol` |
| Serviço | `<workspace>/services/<fullpath service>/<service>-service.archol` |
| Documento | `<workspace>/services/<fullpath service>/documents/<service>-<nome>-document.archol` |
| Processo | `<workspace>/services/<fullpath service>/processes/<service>-<nome>-process.archol` |
| View | `<workspace>/services/<fullpath service>/views/<service>-<nome>-view.archol` |
| Message | `<workspace>/services/<fullpath service>/messages/<service>-<nome>-message.archol` |
| Compartilhado | `<workspace>/services/<fullpath service>/<service>-<tipo-no-plural>.archol` |
| SVG | `<workspace>/services/<fullpath service>/icons/<namespace>/<icon>.svg` |
| `-icons.archol` (não-SVG) | `<workspace>/services/<fullpath service>/<service>-icons.archol` |

- `<fullpath>` é a URI sem protocolo, exatamente como declarada.
- O full path preserva os segmentos da URI; nomes de arquivos e pastas de artefato são kebab-case.
- `<app>`/`<service>` é o último segmento da URI, em kebab-case, e deve ser igual ao slug da pasta.
- `documents/`, `processes/`, `views/` e `messages/` são pastas **organizacionais**, não namespaces: o artefato continua `Invoice`/`CashFlow`, não `documents.Invoice` nem `views.CashFlow`.
- `icons/` é um contêiner: as subpastas dentro dele são namespaces de ícone, mas `icons` não entra no identificador. Ex.: `icons/status/draft.svg` → `status.draft`.
- As demais subpastas seguem a regra de namespace (camelCase unido por ponto).

Exemplos:

```text
examples/services/example.com/services/billing/billing-service.archol
examples/services/example.com/services/billing/documents/billing-invoice-document.archol
examples/services/example.com/services/billing/processes/billing-invoice-issuance-process.archol
examples/services/example.com/services/billing/views/billing-invoice-table-view.archol
examples/services/example.com/services/billing/messages/billing-invoice-issued-message.archol
examples/services/example.com/services/billing/billing-shapes.archol
examples/services/example.com/services/billing/icons/status/draft.svg
examples/applications/example.com/app/backoffice-loja/backoffice-loja-application.archol
```

**Regras.**

- Um arquivo de **contexto principal** declara **exatamente um** artefato principal (`application`, `service`, `document`, `process`, `view` ou `message`) (`E_FILE_PRINCIPAL`).
- O principal é o **primeiro artefato** do arquivo, antes de shapes, forms, policies, decisions e integrations (`E_FILE_PRINCIPAL_FIRST`).
- O `<nome>` do arquivo é equivalente ao nome do principal: kebab-case do identificador em `document`/`process`/`view`/`message`; slug da pasta/URI em `application`/`service` (`E_FILE_NAME`).
- O arquivo de contexto contém o principal e os auxiliares cujo contrato pertence a ele:
  - no documento: shapes de campo/estado, policies que se aplicam a ele;
  - no processo: forms de suas tasks, decisions/integrations que ele chama;
  - na view e na message: o principal e, se necessário, auxiliares sem outro dono.
- **View e message não ficam no arquivo do dono.** A view que lê `Invoice` e a message cujo `about` é `Invoice` continuam em `views/` e `messages/`; o vínculo é declarado no contrato (`document`, `source`, `sources`, `process:`, `about`), não pela posição do arquivo.
- **Referência entre arquivos é permitida.** O fato de outro contexto usar um artefato não o torna compartilhado; o dono é definido pelo contrato. Ex.: `InvoiceNumber` fica no arquivo do `Invoice` e pode aparecer no payload de uma mensagem de cobrança.
- Um artefato **sem dono único** (usado como tipo/campo por mais de um contexto) ou de **escopo do serviço** fica em `<service>-<tipo-no-plural>.archol`: `shapes`, `forms`, `policies`, `decisions`, `integrations`, `parameters`, `secrets`, `organograms`, `icons`. Ex.: `billing-shapes.archol`, `notifications-secrets.archol`.
- O arquivo compartilhado **não declara** `application`, `service`, `document`, `process`, `view` ou `message` (`E_FILE_SHARED`).
- `bpcms.org/std` é pseudo-serviço do construtor e segue a regra de service em `examples/services/bpcms.org/std/`: `std-service.archol` é o arquivo do principal; `std-shapes.archol` é compartilhado.
- `report` não é arquivo: relatório é `view type: report` e fica em `views/` como qualquer outra view.
- `view` e `message` **nunca** vão para arquivo compartilhado: cada uma é contexto principal e tem arquivo próprio.
- O caminho completo é validado: pasta fora do full path da URI, documento fora de `documents/`, processo fora de `processes/`, view fora de `views/` ou message fora de `messages/` é `E_FILE_PATH`; SVG fora de `icons/` é `E_FILE_ICON_PATH`.

## 30.1 Critério de dono

| Artefato | Arquivo |
| :--- | :--- |
| `shape` | documento/processo que o usa como tipo; dois ou mais → compartilhado |
| `form` | processo cuja task usa; dois ou mais → compartilhado; sem processo → documento de `document:` |
| `view` | arquivo próprio em `views/`, sempre; `document`/`source`/`sources`/`process:` do contrato aponta a fonte |
| `message` | arquivo próprio em `messages/`, sempre; `about` ou o processo que envia/recebe fica no contrato |
| `policy` | alvo do `appliesTo`; múltiplos ou nenhum → compartilhado |
| `decision` / `integration` | processo que usa; múltiplos ou nenhum → compartilhado |
| `parameter`, `secret`, `role`, `organogram`, `icon` | `parameter` em `-parameters`, `secret` em `-secrets`, `icon` em `-icons`; `role` e `organogram` no mesmo `-organograms`, com o organogram antes |

## 30.2 Arquivos compartilhados

Artefato compartilhado — sem dono único ou de escopo do serviço — fica em `<workspace>/services/<fullpath service>/<SERVICE>-<TIPO ARTEFATO NO PLURAL>.archol`:

| Sufixo | Tipos aceitos |
| :--- | :--- |
| `shapes` | `shape` |
| `forms` | `form` |
| `policies` | `policy` |
| `decisions` | `decision` |
| `integrations` | `integration` |
| `parameters` | `parameter` |
| `secrets` | `secret` |
| `organograms` | `organogram`, `role` |
| `icons` | `icon` (não-SVG) |

- O arquivo compartilhado tem **zero** artefatos principais.
- Cada arquivo compartilhado contém **um único tipo de artefato**, exceto `-organograms.archol`, que contém exatamente um `organogram` e as `role`s. O `organogram` vem **antes** das `role`s.
- Não misture `parameter` com `secret`, nem `shape` com `form`. Roles não têm arquivo próprio (`-roles.archol` não existe).
- `views` e `messages` **não são sufixos compartilhados**: view e message são contextos principais e ficam em `views/` e `messages/`.
- SVG não entra em `-icons.archol`: SVG fica em `icons/` e `-icons.archol` é só para `icon` declarado.
- Exemplos: `std-shapes.archol` só com shapes; `billing-organograms.archol` com o organogram primeiro e as roles depois; `notifications-parameters.archol` e `notifications-secrets.archol` separados.
- `reports` não é sufixo: relatório é `view type: report` e fica em `views/`.

## 30.3 Validação

O `check` percorre todos os `.archol` do workspace; `application` e `service` também contam como principal. Ele valida a estrutura, não a semântica da atribuição:

- `E_FILE_NAME`: nome fora do padrão ou `<nome>`/`<tipo>` divergente do principal;
- `E_FILE_PATH`: arquivo fora da pasta esperada, full path diferente da URI, documento fora de `documents/`, processo fora de `processes/`, view fora de `views/` ou message fora de `messages/`;
- `E_FILE_PRINCIPAL`: arquivo de contexto principal sem exatamente um principal;
- `E_FILE_PRINCIPAL_FIRST`: principal não é o primeiro artefato do arquivo;
- `E_FILE_SHARED`: arquivo compartilhado declara `application`, `service`, `document`, `process`, `view` ou `message`;
- `E_FILE_SHARED_TYPE`: arquivo compartilhado contém tipo diferente do sufixo (ex.: `view` em `-shapes.archol`, `parameter` em `-secrets.archol`, `role` fora de `-organograms.archol`);
- `E_FILE_ORGANOGRAM_FIRST`: em `-organograms.archol`, o `organogram` não é o primeiro artefato (ou não existe);
- `E_FILE_ICON_PATH`: SVG fora de `icons/` ou namespace de ícone divergente.

A escolha do dono de um auxiliar é convenção de modelagem; o check não tenta inferir dono semântico.

`application`/`service` usam o slug da pasta/URI como `<nome>`; `document`/`process`/`view`/`message` usam o identificador do artefato.

> **Estado da implementação:** o compilador em [`archol/`](archol/README.md) já executa léxico, parser, AST, check de valores de enum e link; as regras `E_FILE_*` acima continuam como especificação a implementar no `check`.

**Exemplo de árvore.**

```text
examples/
  applications/example.com/app/backoffice-loja/
    backoffice-loja-application.archol
  services/example.com/services/billing/
    billing-service.archol
    documents/
      billing-invoice-document.archol
      billing-bill-document.archol
    processes/
      billing-invoice-issuance-process.archol
    views/
      billing-cash-flow-view.archol
      billing-invoice-table-view.archol
    messages/
      billing-invoice-issued-message.archol
      billing-payment-reminder-message.archol
    billing-shapes.archol
    billing-parameters.archol
    billing-organograms.archol          // organogram primeiro, depois as roles
    billing-icons.archol
    icons/
      status/draft.svg                  // status.draft
```

---

## O que ainda não é cláusula

* Herança múltipla ou extensão fora do mesmo serviço — **fora desta versão**. `document Consumer extends User` é a extensão simples permitida (B-34/B-45).
* `migration` — **decidido**: é um `release` do tipo `migration`, não um artefato de topo (B-35). A gramática detalhada fica no SPECS.
* `alias`/`as` — **decidido e documentado** na seção 1.1.
* `private` — **decidido** para membros; artefato de topo nunca é privado.
* capabilities de UI/widget — **decidido**: vocabulário de alto nível (1.7), sem termos de tecnologia.
* `compensation` — **decidido**: task `type: compensation` com `for` (B-37).




