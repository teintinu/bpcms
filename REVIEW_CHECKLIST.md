# BPCMS — Checklist de revisão por tipo de artefato

**Objetivo:** revisar a DSL inteira, um tipo por vez, antes de implementar. Cada rodada valida sintaxe, cláusulas, semântica, exemplos, decisões e consistência documental.

**Regra de ouro:** nesta fase, revisar **somente a DSL**. Nada de runtime, construtor, tema, resolução visual, providers ou implementação.

## Escopo desta fase — somente DSL

**Em escopo:**

- sintaxe e cláusulas de cada tipo de artefato;
- tipos, nomes, escopos e unicidade;
- exemplos e consistência entre `DSL_REFERENCE.md`, `DSL_EXAMPLES.md` e `examples/`;
- decisões de contrato da linguagem.

**Fora de escopo (deferido):**

- como o builder implementa;
- resolução de Theme, ícones, fontes e cores;
- runtime, providers, storage, protocolos;
- deploy, release e ambiente;
- qualquer detalhe de implementação.

Se uma cláusula só existe por causa de implementação, a pergunta é: **ela pertence à DSL?** A resposta fica para a revisão daquele artefato.

## Regra de mudança (obrigatória)

**Nenhuma remoção, alteração ou adição é aplicada durante a revisão sem explicação e confirmação explícita do usuário.**

Cada proposta deve ter:

1. **O que muda** — cláusula, exemplo, arquivo, comportamento.
2. **Por que** — problema, risco ou simplificação.
3. **Impacto** — arquivos, exemplos, compatibilidade, migração.
4. **Alternativas** — inclusive “não mudar”.
5. **Arquivos afetados**.
6. **Status** — `proposta`, `aguardando aprovação`, `aprovada`, `rejeitada`.

Enquanto o status não for **aprovada**, não editar. Reverter também é mudança e exige confirmação.

## Regra de namespaces (anotada; não aplicada)

- A raiz do serviço é a pasta que contém `<service>-service.bpcms`.
- **Toda subpasta é namespace para todos os tipos de artefato**, não só Icons.
- Arquivos e pastas: **kebab-case**.
- Referências: segmentos **camelCase** unidos por ponto.
- Local: `status.refunded`, `domain.order`, `sales.order.item`.
- Outro serviço: `AliasDoServico.status.refunded`.
- Raiz: artefatos sem namespace usam o nome simples.
- Mesmo nome em namespaces diferentes: permitido. Duplicado no mesmo tipo/namespace: erro.
- **Não mover/renomear artefatos existentes até esta regra ser detalhada e aprovada.**

## Como usar

1. Revisar **um artefato por vez**, na ordem abaixo.
2. Marcar as cláusulas conferidas em `DSL_REFERENCE.md`.
3. Conferir exemplos em `DSL_EXAMPLES.md` e os arquivos completos em `examples/`.
4. Registrar decisões e dúvidas no fim da seção do artefato.
5. Só passar para o próximo quando o status for **✅ aprovado**.
6. Se uma mudança afetar contrato, atualizar os três documentos vivos juntos.

## Legenda

- `⬜ pendente`
- `🟡 em revisão`
- `✅ aprovado`
- `⛔ bloqueado por decisão`
- `📝 proposta` — aguardando aprovação, sem alteração aplicada
- `⏸️ fora da fase` — DSL de builder/ambiente deferida

## Ordem de revisão (menos dependências primeiro)

| # | Tipo | Depende de | Exemplos | Status |
| :--- | :--- | :--- | :--- | :--- |
| 1 | Icon | — (DSL) | IC.1–IC.6; `examples/services/*/*-icons.bpcms` | ✅ |
| 2 | Shape | Icon (values) | H.1–H.9 | 🟡 |
| 3 | Role | permissões | R.1–R.6 | ⬜ |
| 4 | Organogram | Role | R.1–R.6 | ⬜ |
| 5 | Parameter | Shape/Secret | PR.1–PR.6 | ⬜ |
| 6 | Service | outros services, imports | S.1–S.8 | ⬜ |
| 7 | Document | Shape, Icon | D.1–D.6 | ⬜ |
| 8 | Event | Document, Shape | E.1–E.6 | ⬜ |
| 9 | View | Document, Role | V.1–V.6 | ⬜ |
| 10 | Decision | Document, Role | N.1–N.6 | ⬜ |
| 11 | Policy | Document, Role | P.1–P.6 | ⬜ |
| 12 | Job | Service/ação | J.1–J.6 | ⬜ |
| 13 | Integration | Shape, Service | I.1–I.6 | ⬜ |
| 14 | Report | Document, Shape, View | R.1–R.6 | ⬜ |
| 15 | Process | Event, Document, Role, Integration, View | P.1–P.7 | ⬜ |
| 16 | Evidence | Process, Document, Service | E.1–E.6 | ⬜ |
| 17 | Assumption | links ao modelo | U.1–U.6 | ⬜ |
| 18 | Conversation | Process, Role, Evidence, Assumption | C.1–C.6 | ⬜ |
| 19 | Widget | View, Shape, capability | W.1–W.6 | ⬜ |
| 20 | Theme | Builder | T.1–T.6 | ⏸️ |
| 21 | Locale | App/Builder | L.1–L.6 | ⏸️ |
| 22 | Builder | capabilities, Theme, Locale | B.1–B.6 | ⏸️ |
| 23 | Environment | App/Builder | N.1–N.6 | ⏸️ |
| 24 | Application | Services, Roles, Theme, Locale, Lock | A.1–A.8 | ⬜ |
| 25 | Release | Lock, Service, Application | V.1–V.6 | ⬜ |

> **Theme/Locale não são artefatos de serviço.** São assets do construtor. A revisão deles valida a sintaxe que o builder consome, não uma pasta de serviço.

## Checklist transversal (aplicar em toda rodada)

- [ ] `title: I18n` obrigatório onde a regra exige: service, shape, document, field, view, event, parameter, role, organogram, icon; application.
- [ ] Campos têm `title`; payloads de evento têm `title`; values de enum têm `title` + `icon`.
- [ ] `short`/`tooltip` são irmãos do mapa principal; fallback exato → idioma → default.
- [ ] `I18n` aceita `string | PluralForms` por locale; `other` obrigatório; fallback por categoria.
- [ ] Units usam `title` I18n com `one`/`other`; sem campo `plural` separado.
- [ ] Nomes de arquivos em kebab-case; referências e funções em camelCase; subpastas são namespaces de todos os artefatos.
- [ ] Artefatos em PascalCase, com exceções internacionais (`pH`).
- [ ] Nomes de artefatos são claros, sem prefixo de serviço; escopo é o serviço; sem duplicidade no mesmo tipo/serviço.
- [ ] Icons sem prefixo `Icon`; iguais em serviços diferentes são permitidos; duplicados no mesmo serviço são erro.
- [ ] Nenhum serviço criado só para biblioteca/agrupamento de Shapes/Icons/Themes/Locales.
- [ ] Nenhuma cláusula foi avaliada por comportamento de builder/runtime; apenas por contrato DSL.
- [ ] Shapes compartilhados ficam no serviço de negócio dono.
- [ ] Application não declara artefatos.
- [ ] `id` de document é implícito; `lookup` é tag de campo; FK é soft; extensão compartilha id.
- [ ] Nomenclatura de arquivos: `<service>-<type>.bpcms`; roles+organogram no mesmo `*-organograms.bpcms`.
- [ ] Exemplos em `DSL_EXAMPLES.md` e em `examples/` concordam com a referência.
- [ ] BRAINSTORM/DSL_REFERENCE/DSL_EXAMPLES atualizados juntos quando o contrato muda.
- [ ] Nenhuma decisão de implementação (font/color/layout/resolução visual) foi revisada nesta fase.
- [ ] `git diff --check`; sem `add/commit/push` sem pedido.

## 1. Icon ✅

**Depende de:** nada na DSL.

**Sintaxe/cláusulas:**

- [x] `icon <Name> { ... }`
- [x] `title: I18n` obrigatório
- [x] `Icon` definido dentro de um `Service`
- [x] `icon` sem prefixo `Icon` nem prefixo de serviço
- [x] Escopo do nome = serviço; mesmo nome em serviços diferentes permitido
- [x] `E_DUPLICATE_ICON` apenas no mesmo serviço
- [x] Sem imagem/binário no DSL
- [x] SVG file-based: qualquer subpasta sob a raiz do serviço
- [x] Nome do SVG: caminho relativo em camelCase com `.`; arquivos/pastas em kebab-case
- [x] Title do SVG: `aria-label` → `<title>` → `alt` → filename humanizado; sem I18n
- [x] Conflito SVG × Icon declarado = erro

**Exemplos:** IC.1–IC.6; 75 icons nos `*-icons.bpcms`; 7 SVGs em `billing/status`, `billing/finance/invoice`, `notifications/channels`, `inventory/stock`, `payments/status`.

**Foco da revisão (só DSL):**

- [x] Um Icon precisa de mais alguma cláusula além de nome + `title`?
- [x] `symbol` é cláusula de DSL ou detalhe de resolução visual do builder?
- [x] `set` é cláusula de DSL ou escolha do tema?
- [x] `title` de Icon precisa de `short`/`tooltip`? Provavelmente não; confirmar.
- [x] Nomes locais (`Draft`, `Paid`) e escopo por serviço são suficientes?

**Status:** ✅ revisado em DSL.

**Decisão:** Icon = nome + `title`; `symbol`/`set` ficam fora da DSL. SVG file-based faz parte da alternativa de Icon: qualquer subpasta, kebab-case, identificador camelCase com ponto, title por `aria-label` → `<title>` → `alt` → filename, sem I18n. `E_ICON_UNRESOLVED` é erro de resolução do builder, não de sintaxe DSL.

### P-ICON-SVG — SVG file-based Icons (aprovada)

- **O que muda:** qualquer `.svg` sob `services/<service>/` (qualquer subpasta) vira Icon implícito.
- **Nome:** caminho relativo com segmentos camelCase e pontos; `status/refunded.svg` → `status.refunded`; `checkout/status/refunded.svg` → `checkout.status.refunded`.
- **Referência local:** `icon: status.refunded`.
- **Referência por alias:** `Checkout.status.refunded` quando `checkout` está importado como `Checkout`.
- **Title:** `aria-label` → `<title>` → `alt` → filename humanizado; sem I18n.
- **Conflito:** SVG × Icon declarado ou SVG × SVG com o mesmo identificador é erro (case-insensitive).
- **Arquivos/pastas:** kebab-case; identificador de referência camelCase.
- **Exemplos criados:** `billing/status/draft.svg`, `paid.svg`, `overdue.svg`, `billing/finance/invoice/issued.svg`, `notifications/channels/email.svg`, `inventory/stock/low-stock.svg`, `payments/status/refunded.svg`.
- **Status:** ✅ aprovado para file-based Icons. A aplicação da regra de namespace/camelCase a **todos** os artefatos continua pendente de validação própria.

### Regra de namespaces — perguntas abertas

- [ ] A declaração continua `document Order` e o namespace vem da pasta (`domain.order`), ou a declaração precisa incluir o namespace?
- [ ] Arquivos de tipo com múltiplos artefatos (ex.: `documents.bpcms`) podem coexistir com namespaces por subpasta?
- [ ] Dentro do mesmo namespace, a referência pode ser curta (`order`) ou sempre completa (`domain.order`)?
- [ ] O alias cross-service é `Checkout.domain.order` ou `checkout.domain.order`?
- [ ] A conversão kebab → camelCase vale para nomes multiword (`order-item` → `orderItem`, `money-brl` → `moneyBrl`)?
- [ ] O mesmo nome em namespaces diferentes é permitido sem restrição? (proposta: sim, duplicidade só no mesmo tipo/namespace)
- [ ] Arquivos `.bpcms` em subpastas continuam com prefixo `<service>-` ou usam só o nome do artefato? Hoje a regra de arquivos exige prefixo.

### P-ICON-01 — remover `symbol`/`set` (aprovada na revisão de Icon)

- **O que muda:** `Icon` perde `symbol` e `set`; fica apenas `title`; entra `E_ICON_UNRESOLVED`.
- **Por que:** `symbol`/`set` parecem detalhes de resolução visual, não contrato de negócio da DSL.
- **Impacto:** 75 icons, exemplos IC.1–IC.6, `DSL_REFERENCE` e `BRAINSTORM`.
- **Alternativas:** (a) restaurar `symbol`/`set`; (b) manter apenas nome + `title`; (c) `symbol` opcional.
- **Arquivos afetados:** `examples/services/*/*-icons.bpcms`, `DSL_REFERENCE.md`, `DSL_EXAMPLES.md`, `BRAINSTORM.md`.
- **Status:** ✅ aprovada retroativamente na revisão de Icon.

**Decisão:** manter `symbol`/`set` fora da DSL; resolução visual fica no builder/Theme.

## 2. Shape 🟡

**Depende de:** Icon (values enumerados), tipos embutidos.

**Sintaxe/cláusulas:**

- [ ] `shape <Name> { ... }`
- [ ] `title: I18n`
- [ ] `base: decimal | integer | string | classification | workflow | attachment | audio | image | video | richtext`
- [ ] `values { <value> { title; icon; ordinal; next } }` — `classification`/`workflow`; ordinal 10, 20, 30...
- [ ] `currency`
- [ ] `dimension`, `units { unidade: { factor, title } }`
- [ ] `units` titles têm plural I18n e `format` escolhe `one`/`other`
- [ ] `unit`, `min`, `max`, `scale` e `precision` são do field (exceto `unit` em Shape de unidade única)
- [ ] `unit` e `units` mutuamente exclusivos; `Percentage` usa `unit: "%"`
- [ ] `format` auto-seleciona unidade; `flags.unit` força
- [ ] `maxLength`; `min`/`max`/`scale` no field
- [ ] `initial` único em `workflow`; `next` obrigatório; `[]` terminal
- [ ] `canTransitionTo` automático; mudança inválida lança `E_INVALID_TRANSITION`
- [ ] `parse`, `format`, `compare`, `aggregate`
- [ ] `write(value, out)` com `out` inferido como `streamWriter`; `format(value, flags)` com `value` explícito; `read(in)` com `in` inferido como `streamReader`; `void` inferido
- [ ] `new Shape{raw, unit}` e shorthand de propriedades
- [ ] Erro de tipo/propriedade em `new` é `E_TYPE`/`E_CONSTRUCTOR_PROPERTY`
- [ ] `<name>(...)` sem keyword `method`; compilador detecta por `(`
- [ ] Argumentos obrigatórios; tipos/retorno explícitos ou inferidos
- [ ] Exemplos de métodos com tipos explícitos, inferidos e mistos
- [ ] `toNumber(value)` só em Shapes numéricos
- [ ] `toOrdinal(value)` é gerado automaticamente em classification/workflow; não aparece no código
- [ ] `Duration` em std; `Length / Duration -> Speed`
- [ ] `format`/`parse` de compostos exigem método próprio
- [ ] `convert` inteligente; `formula` com `to`/`from`
- [ ] `format` com `unitStyle: short | long`; `parse` exige unidade compatível
- [ ] `index`, `normalize`, `validate`, `sensitive`
- [ ] `fields { ... }` — shape composto
- [ ] `default`, `private`
- [ ] Enum: cada value tem `title` + `icon` + `ordinal`; workflow também `next`; icons são locais ao serviço
- [ ] `I18n` não é Shape; `shape I18n` é `E_RESERVED_TYPE`
- [ ] `<Document>Id` é Shape implícito de FK; não declarar

**Exemplos:** H.1–H.9; `catalog-shapes`, `identity-shapes`, `citizen-shapes`, `notifications-shapes`, `billing-shapes`.

**Resultado da inspeção:**

- [x] 29 Shapes de negócio + 57 Shapes std; todos com `title`.
- [x] 22 enums / 77 values; todos com `title` + `icon` + `ordinal`; 16 workflows com `initial`/`next`; 6 classifications sem transições.
- [x] `values` em bloco; sem `values: [...]`.
- [x] Shape composto `TrackingEvent` com fields tipados/title.
- [x] `Email`, `Phone`, `Image`, `LocaleTag`, `YearMonth` continuam em serviços de negócio; `MoneyBRL` foi removido.
- [x] `Mass`, `Length`, `Volume`, `Percentage`, `Count` movidos para `bpcms.org/std`; `Money`/`Currency` vivem na std.
- [x] Catalog/Identity/Shipping/Inventory/Orders/Billing importam `bpcms.org/std` como `std`.
- [x] `Email`, `Phone`, `Image` em Citizen.
- [x] `LocaleTag` em Notifications.
- [x] `YearMonth` em Billing.

**Pontos para decidir (sem alterar):**

- [ ] `Percentage` agora é `std.Percentage`; a assinatura de `Order.create` está correta?
- [ ] `default`, `private` e `sensitive` de Shape não têm exemplo no tree; manter, remover ou exemplificar?
- [ ] `Shape.fields` composto só tem `TrackingEvent`; falta um exemplo com validação/privacidade?
- [ ] `std.Money` substitui `MoneyBRL`; `currency` é restrição de field; valor carrega moeda?
- [ ] `std.Count` substitui o Count de Identity; a migração está correta?
- [ ] `LocaleTag` em Notifications é tipo de negócio ou detalhe de regionalização?
- [ ] `bpcms.org/std` definido e importado; 57 Shapes; migração concluída?
- [ ] `format`/`parse`/`validate` aceitam valores que não estão nos exemplos; a lista de valores precisa ser fechada?
- [ ] `Shape` tem `title`, mas fields de Shape composto precisam de `title` explícito em todos os casos?

**Status:** 🟡 em revisão de DSL. Migração para std concluída; revisão de Shape pendente.

**Stdlib:** definida em `examples/services/std/std-service.bpcms` e `examples/services/std/std-shapes.bpcms`; import `service "bpcms.org/std" as std`; uso `std.Mass`. Migração concluída; `Money` tem `toNumber` implícito; `Currency` é classification ISO com `scale` e `reintroduce`.

## 3. Role ⬜

**Depende de:** permissões/capabilities.

**Sintaxe/cláusulas:**

- [ ] `role <Name> { ... }`
- [ ] `title: I18n`
- [ ] `permissions: [...]`
- [ ] `inherits: Role`
- [ ] `scope`
- [ ] Papel público; sem `private` de topo
- [ ] Papel definido no serviço ou no organogram

**Exemplos:** R.1–R.6; `identity-organograms`, `support-organograms`.

**Foco da revisão:**

- [ ] Papéis são escopados ao serviço ou globais?
- [ ] `permissions` é lista de namespaces ou catálogo?
- [ ] Herança de papel é suficiente?
- [ ] Role e organogram no mesmo arquivo está funcionando?

**Status:** ⬜ pendente.

## 4. Organogram ⬜

**Depende de:** Role.

**Sintaxe/cláusulas:**

- [ ] `organogram <Name> { ... }`
- [ ] `title: I18n`
- [ ] `hierarchy { A > B > C }`
- [ ] `fourEyes`
- [ ] `segregation`
- [ ] `delegation` / `when absent`
- [ ] `limits` / `unlimited`
- [ ] `dataMasking` / `masked`
- [ ] `signature`
- [ ] `requires`
- [ ] mesmo arquivo dos roles (`*-organograms.bpcms`)

**Exemplos:** R.1–R.6; `identity-organograms`, `support-organograms`.

**Foco da revisão:**

- [ ] `Manager > SupportAgent` em Support pode referenciar role de Identity?
- [ ] `title` de organogram é obrigatório?
- [ ] `fourEyes`/`segregation` são avaliados no runtime ou no build?
- [ ] Limites por papel usam Shape/Money?

**Status:** ⬜ pendente.

## 5. Parameter ⬜

**Depende de:** Shape, Secret.

**Sintaxe/cláusulas:**

- [ ] `parameter <Name> { ... }`
- [ ] `title: I18n` (substitui `description`)
- [ ] `type: Shape | Secret`
- [ ] `source: config | secret`
- [ ] `required` (bare)
- [ ] `default` (só config)
- [ ] `min`/`max`/`maxLength`
- [ ] Declarado apenas dentro de Service
- [ ] App faz `set <Alias>.<Parameter>`
- [ ] `Secret` nunca serializa/loga/exporta/vai à LLM

**Exemplos:** PR.1–PR.6; `*-parameters.bpcms`.

**Foco da revisão:**

- [ ] `title` em Parameter substituiu `description` por completo?
- [ ] `required` bare é suficiente?
- [ ] `default` de `secret` deve ser erro?
- [ ] Parameter pode referenciar outro Parameter?
- [ ] `min`/`max`/`maxLength` são constraints do Shape ou do Parameter?

**Status:** ⬜ pendente.

## 6. Service ⬜

**Depende de:** outros services, imports, capabilities.

**Sintaxe/cláusulas:**

- [ ] `service "<uri>" { ... }`
- [ ] `title: I18n`
- [ ] `version`, `owner`
- [ ] `imports { service "<uri>" as Alias; integration X }`
- [ ] `override "<uri>#<Artifact>.<member>" { with: ... }`
- [ ] `requires { capabilities }`
- [ ] `provides { contracts }`
- [ ] `roles { ... }` ou roles em arquivo
- [ ] Sem `exports`; todo artefato de topo é público
- [ ] Serviço é conceito de negócio, não biblioteca/package/módulo
- [ ] Não criar serviço para Shapes/Icons/Themes/Locales compartilhados
- [ ] Theme/Locale são do construtor

**Exemplos:** S.1–S.8; `examples/services/*/*-service.bpcms`.

**Foco da revisão:**

- [ ] `imports` vazio é permitido?
- [ ] `roles` inline vs arquivo de organogram?
- [ ] `requires` vs `provides`: vocabulário fechado?
- [ ] Override de view/action/doc: o que é permitido?
- [ ] Serviço de negócio pode ter `Shape`/`Icon` locais? Sim, se forem do domínio.

**Status:** ⬜ pendente.

## 7. Document ⬜

**Depende de:** Shape, Icon, Role (ações), Service.

**Sintaxe/cláusulas:**

- [ ] `document <Name> { ... }`
- [ ] `title: I18n`
- [ ] `id` implícito (`<Name>Id`); não declarar
- [ ] `lookup` como tag de campo; ordem dos campos
- [ ] `extends` (pai único, mesmo serviço, aditivo, mesmo id)
- [ ] `version`, `owner`, `sync`, `audit`, `retention`
- [ ] `fields { name: Shape { title; required; lookup; unique; ... } }`
- [ ] `indexes` (unique/range/search/hash)
- [ ] `actions` (públicas/private)
- [ ] `rules`
- [ ] `source: device ...`
- [ ] `computed from`, `computed <Name> from`, `freshness`, `exposes`
- [ ] FK: `CategoryId`; id inexistente bloqueia só novos vínculos
- [ ] Projeção: `UserId(citizenId)` só campos extras
- [ ] Circular: campo não pode ser `<document>Id`; `parentId` é permitido
- [ ] Duplicidade: `id`, `<Doc>Id`, `categoryId` no próprio Category
- [ ] PII em `private` quando aplicável

**Exemplos:** D.1–D.6; `*-documents.bpcms`.

**Foco da revisão:**

- [ ] `lookup` é tag de campo; `short`/`tooltip` são do title.
- [ ] `extends` restrito a documento no mesmo serviço.
- [ ] `id` implícito e `<Doc>Id` implícito.
- [ ] FK soft: apagar origem não invalida vínculos antigos.
- [ ] `computed` + `lookup` + `title` interagem bem?
- [ ] `private` é suficiente para PII ou precisa `sensitive`?
- [ ] `Date`/`DateTime`/`YearMonth` são tipos internos?

**Status:** ⬜ pendente.

## 8. Event ⬜

**Depende de:** Document, Shape.

**Sintaxe/cláusulas:**

- [ ] `event <Name> { ... }`
- [ ] `title: I18n`
- [ ] `version`
- [ ] `payload { field: Shape { title } }`
- [ ] `producer: Document`
- [ ] `consumers: [Service]`
- [ ] `key`
- [ ] `sensitivity`
- [ ] `severity`

**Exemplos:** E.1–E.6; `*-events.bpcms`.

**Foco da revisão:**

- [ ] Payload fields têm `title`?
- [ ] `consumers` são serviços ou eventos?
- [ ] `version` é semver ou inteiro?
- [ ] Evento carrega PII? Deve carregar só `CitizenId`/lookup?
- [ ] `key` é obrigatório?

**Status:** ⬜ pendente.

## 9. View ⬜

**Depende de:** Document, Role, Shape.

**Sintaxe/cláusulas:**

- [ ] `view <Name> { ... }`
- [ ] `title: I18n`
- [ ] `type: form | table | dashboard | wizard | detail | kanban`
- [ ] `document`, `source`, `sources`
- [ ] `roles`
- [ ] `fields` com `title`, `required`, `readonly`
- [ ] `columns`, `filters`, `filter`, `pagination`
- [ ] `actions`
- [ ] `metrics`, `freshness`, `units`, `export`
- [ ] `steps`, `data`, `integration`
- [ ] `sections`, `computed`, `device`
- [ ] `groupBy`, `card`
- [ ] `mode`

**Exemplos:** V.1–V.6; `*-views.bpcms`.

**Foco da revisão:**

- [ ] Labels de colunas vêm dos fields/document titles?
- [ ] `fields` de view exigem `title` próprio?
- [ ] `actions` de view podem apontar para Job/Integration?
- [ ] `computed` de view vs Document computed?
- [ ] View pode ter `device` sem capability?

**Status:** ⬜ pendente.

## 10. Decision ⬜

**Depende de:** Document, Role.

**Sintaxe/cláusulas:**

- [ ] `decision <Name> { ... }`
- [ ] `title`? (não está na lista atual de obrigatórios — decidir)
- [ ] `appliesTo`
- [ ] `when`
- [ ] `cases { cond -> outcome }`
- [ ] `default`
- [ ] `owner`

**Exemplos:** N.1–N.6; `*-decisions.bpcms`.

**Foco da revisão:**

- [ ] Decision precisa de `title` I18n?
- [ ] `when` lista campos de contexto, não expressão?
- [ ] `default` é obrigatório?
- [ ] Decision pode chamar ação ou só classificar?

**Status:** ⬜ pendente.

## 11. Policy ⬜

**Depende de:** Document, Role.

**Sintaxe/cláusulas:**

- [ ] `policy <Name> { ... }`
- [ ] `title`? (decidir)
- [ ] `appliesTo`
- [ ] `rule`
- [ ] `severity: block | require | enforce`
- [ ] `message`
- [ ] `audit`
- [ ] `exception`
- [ ] `notification`
- [ ] `mask`

**Exemplos:** P.1–P.6; `*-policies.bpcms`.

**Foco da revisão:**

- [ ] Policy precisa de `title`?
- [ ] `rule` pode referenciar parâmetro?
- [ ] `exception` é declarativa ou runtime?
- [ ] `notification` usa canais de Integration?

**Status:** ⬜ pendente.

## 12. Job ⬜

**Depende de:** Service/ação.

**Sintaxe/cláusulas:**

- [ ] `job <Name> { ... }`
- [ ] `title`? (decidir)
- [ ] `schedule`
- [ ] `timezone`
- [ ] `action`
- [ ] `timeout`
- [ ] `retry`
- [ ] `idempotent`
- [ ] `deadLetter`
- [ ] `input`
- [ ] `batch`
- [ ] `dryRun`
- [ ] `filter`
- [ ] `maxAttempts`
- [ ] `backoff`
- [ ] `cursor`
- [ ] `condition`

**Exemplos:** J.1–J.6; `*-jobs.bpcms`.

**Foco da revisão:**

- [ ] `action` é método de Document ou serviço?
- [ ] `schedule` aceita cron + linguagem natural?
- [ ] `filter`/`condition` são expressões DSL?
- [ ] `idempotent` deve ser obrigatório em ação mutante?

**Status:** ⬜ pendente.

## 13. Integration ⬜

**Depende de:** Shape, Service.

**Sintaxe/cláusulas:**

- [ ] `integration <Name> { ... }`
- [ ] `title`? (decidir)
- [ ] `protocol`
- [ ] `baseUrl`
- [ ] `auth`
- [ ] `timeout`
- [ ] `operations`
- [ ] `webhooks`
- [ ] `limits`
- [ ] `retry`
- [ ] `fallback`
- [ ] `cache`
- [ ] `quality`

**Exemplos:** I.1–I.6; `*-integrations.bpcms`.

**Foco da revisão:**

- [ ] `auth` credenciais vêm de Parameter/Secret?
- [ ] Operation types são Shapes?
- [ ] Webhooks podem mapear Event/Document?
- [ ] `fallback`/`quality` são detalhes do builder?

**Status:** ⬜ pendente.

## 14. Report ⬜

**Depende de:** Document, Shape, View.

**Sintaxe/cláusulas:**

- [ ] `report <Name> { ... }`
- [ ] `title`? (decidir)
- [ ] `type`
- [ ] `sources`
- [ ] `roles`
- [ ] `filters`
- [ ] `dimensions`
- [ ] `metrics`
- [ ] `sort`
- [ ] `freshness`
- [ ] `units`
- [ ] `export`
- [ ] `alert`

**Exemplos:** R.1–R.6; `*-reports.bpcms`.

**Foco da revisão:**

- [ ] Report precisa de `title`?
- [ ] `metrics` usam `sum/count/avg` e `where`?
- [ ] `units` referencia Shape?
- [ ] `alert` é Policy/Job?

**Status:** ⬜ pendente.

## 15. Process ⬜

**Depende de:** Event, Document, Role, Integration, View.

**Sintaxe/cláusulas:**

- [ ] `process <Name> { ... }`
- [ ] `title`? (decidir)
- [ ] `version`, `goal`, `roles`
- [ ] `start on Event`
- [ ] `task` (role/form/reads/writes/sla)
- [ ] `type: service`
- [ ] `type: compensation` + `for` + `action`
- [ ] `gateway` + `if/else`
- [ ] `on error`
- [ ] `end on Event`
- [ ] `integration`

**Exemplos:** P.1–P.7; `*-processes.bpcms`.

**Foco da revisão:**

- [ ] Process precisa de `title`?
- [ ] `start`/`end` por Event é suficiente?
- [ ] Compensação sempre em ordem reversa?
- [ ] `reads`/`writes` são obrigatórios?
- [ ] `role` de task é Role do app?

**Status:** ⬜ pendente.

## 16. Evidence ⬜

**Depende de:** Process, Document, Service.

**Sintaxe/cláusulas:**

- [ ] `evidence <Name> { ... }`
- [ ] `title`? (decidir)
- [ ] `kind`
- [ ] `file`
- [ ] `hash`
- [ ] `confidentiality`
- [ ] `source`
- [ ] `extracted`
- [ ] `links`
- [ ] `status`

**Exemplos:** E.1–E.6; `evidence/`.

**Foco da revisão:**

- [ ] Evidence precisa de `title`?
- [ ] `extracted` é sempre proposto?
- [ ] `links` pode apontar para Service/Process/Document?
- [ ] PII em evidence é redigida pela segurança?

**Status:** ⬜ pendente.

## 17. Assumption ⬜

**Depende de:** links ao modelo.

**Sintaxe/cláusulas:**

- [ ] `assumption <Name> { ... }`
- [ ] `title`? (decidir)
- [ ] `statement`
- [ ] `impact`
- [ ] `status`
- [ ] `validate`
- [ ] `owner`
- [ ] `due`
- [ ] `links`

**Exemplos:** U.1–U.6; `assumptions/`.

**Foco da revisão:**

- [ ] Assumption precisa de `title`?
- [ ] `impact` é enum?
- [ ] `validate` é texto ou ação?
- [ ] `links` pode ser múltiplo?

**Status:** ⬜ pendente.

## 18. Conversation ⬜

**Depende de:** Process, Role, Evidence, Assumption.

**Sintaxe/cláusulas:**

- [ ] `conversation <Name> { ... }`
- [ ] `title`? (decidir)
- [ ] `process`
- [ ] `goal`
- [ ] `participants`
- [ ] `evidence`
- [ ] `coverage`
- [ ] `assumptions`
- [ ] `status`
- [ ] `next`
- [ ] `sessions`
- [ ] `confidentiality`

**Exemplos:** C.1–C.6; `conversations/`.

**Foco da revisão:**

- [ ] Conversation precisa de `title`?
- [ ] `coverage` por dimensão ou por artefato?
- [ ] `next` é texto livre?
- [ ] PII de participantes é tratada?

**Status:** ⬜ pendente.

## 19. Widget ⬜

**Depende de:** View, Shape, capability.

**Sintaxe/cláusulas:**

- [ ] `widget <Name> { ... }`
- [ ] `title`? (decidir)
- [ ] `capabilities`
- [ ] `renders`
- [ ] `props`
- [ ] `events`
- [ ] `platforms`
- [ ] `fallback`

**Exemplos:** W.1–W.6; `*-widgets.bpcms` se existir.

**Foco da revisão:**

- [ ] Widget precisa de `title`?
- [ ] `renders`/`props` são Shapes?
- [ ] `platforms` é builder ou capability?
- [ ] Widget é artefato de negócio ou de construtor?

**Status:** ⬜ pendente.

## 20. Theme ⏸️ (fora da fase)

**Depende de:** Builder. **Não é artefato de serviço.** Revisar somente se a sintaxe DSL justifica a cláusula; comportamento/implementação deferido.

**Sintaxe/cláusulas:**

- [ ] `theme <Name> { ... }`
- [ ] `title`? (decidir; hoje não está na lista)
- [ ] `mode`
- [ ] `supports`
- [ ] `tokens` (font/color/etc.)
- [ ] `icons`
- [ ] `accessibility`
- [ ] `page`
- [ ] `header`
- [ ] `footer`
- [ ] `density`

**Exemplos:** T.1–T.6; nenhum arquivo de serviço.

**Foco da revisão:**

- [ ] Theme precisa de `title`?
- [ ] `tokens` pode ter qualquer chave?
- [ ] `icons` e resolução de Icon são builder?
- [ ] Theme pode ser definido por app ou só builder?

**Status:** ⬜ pendente.

## 21. Locale ⏸️ (fora da fase)

**Depende de:** App/Builder. **Não é artefato de serviço.** Revisar somente sintaxe DSL; comportamento/implementação deferido.

**Sintaxe/cláusulas:**

- [ ] `locale <tag> { ... }`
- [ ] `title`? (decidir)
- [ ] `fallback`
- [ ] `currency`
- [ ] `units`
- [ ] `date`
- [ ] `time`
- [ ] `direction`
- [ ] `decimal`
- [ ] `thousands`
- [ ] `messages`

**Exemplos:** L.1–L.6; nenhum arquivo de serviço.

**Foco da revisão:**

- [ ] Locale precisa de `title`?
- [ ] Locale é app config ou builder asset?
- [ ] `I18n` de titles depende de Locale?
- [ ] `currency`/`units` são defaults de regionalization?

**Status:** ⬜ pendente.

## 22. Builder ⏸️ (fora da fase)

**Depende de:** capabilities, Theme, Locale. Fora da fase atual; não avaliar implementação.

**Sintaxe/cláusulas:**

- [ ] `builder <Name> { ... }`
- [ ] `title`? (decidir)
- [ ] `target`
- [ ] `mode`
- [ ] `capabilities`
- [ ] `consumes`
- [ ] `produces`
- [ ] `limitations`

**Exemplos:** B.1–B.6; `*-builders.bpcms` se existir.

**Foco da revisão:**

- [ ] Builder precisa de `title`?
- [ ] `consumes`/`produces` listam artefatos e formatos?
- [ ] `limitations` é por plataforma?
- [ ] Builder implementa Theme/Locale?

**Status:** ⬜ pendente.

## 23. Environment ⏸️ (fora da fase)

**Depende de:** App/Builder. Fora da fase atual; revisar só sintaxe se necessário.

**Sintaxe/cláusulas:**

- [ ] `environment <Name> { ... }`
- [ ] `title`? (decidir)
- [ ] `type`
- [ ] `data`
- [ ] `reset`
- [ ] `promoteTo`
- [ ] `seed`
- [ ] `config`

**Exemplos:** N.1–N.6; `environments/`.

**Foco da revisão:**

- [ ] Environment precisa de `title`?
- [ ] `config`/`seed` entram no lock?
- [ ] `promoteTo` é release ou environment?
- [ ] Environment pertence à Application?

**Status:** ⬜ pendente.

## 24. Application ⬜

**Depende de:** Services, Roles, Theme, Locale, Lock.

**Sintaxe/cláusulas:**

- [ ] `application "<uri>" { ... }`
- [ ] `title: I18n`
- [ ] `source`, `version`, `lock`
- [ ] `services { "<uri>" as Alias }`
- [ ] `roles` + hierarquia
- [ ] `theme` (builder)
- [ ] `locales` (app/construtor)
- [ ] `environments`
- [ ] `main { set Alias.Parameter; start }`
- [ ] Sem artefatos
- [ ] Sem `wire`; closure por imports
- [ ] Lock: uma versão/serviço + IDs

**Exemplos:** A.1–A.8; `applications/*/*-application.bpcms`.

**Foco da revisão:**

- [ ] `theme`/`locales` realmente não são artefatos de serviço?
- [ ] `start` só pode referenciar entry-point service?
- [ ] `set` só Parameter de serviço?
- [ ] Roles da app vêm de qual serviço?
- [ ] Lock é gerado, não escrito à mão?

**Status:** ⬜ pendente.

## 25. Release ⬜

**Depende de:** Lock, Service, Application.

**Sintaxe/cláusulas:**

- [ ] `release <Name> { ... }`
- [ ] `title`? (decidir)
- [ ] `version`/`semver`
- [ ] `lock`
- [ ] `migration`
- [ ] `channel`
- [ ] `publish`
- [ ] `notes`

**Exemplos:** V.1–V.6; `releases/`.

**Foco da revisão:**

- [ ] Release precisa de `title`?
- [ ] Release aponta para o lock do app?
- [ ] `migration` é release ou artefato?
- [ ] `channel`/`publish` são builder?

**Status:** ⬜ pendente.

## Revisão de Icon — encerrada

**✅ Aprovado:** nome + `title`; sem `symbol`/`set`; SVG file-based com namespace local; title sem I18n; fallback de filename; conflito no mesmo serviço é erro.

Próximo: **Shape** 🟡.

Depois de decidir Icon, seguimos para **Shape**.

## Pendências de revisão de keywords

- [ ] `reintroduce`: revisar depois de todas as keywords existentes.
- [ ] `classification: <url>`: revisar junto com `reintroduce`; por enquanto só na stdlib.
- [x] `prefix`/`suffix`/`infix` definidos: prefixo/sufixo e infix decimal (CVE).
- [ ] CPF nao pode ser um tipo, revizar Citizen
