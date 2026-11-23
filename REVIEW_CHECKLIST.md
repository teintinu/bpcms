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

- A raiz do serviço é a pasta que contém `<service>-service.archol`.
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
| 1 | Icon | — (DSL) | IC.1–IC.6; `examples/services/**/*-icons.archol` | ✅ |
| 2 | Shape | Icon (value) | H.1–H.9 | 🟡 |
| 3 | Role | permissões | R.1–R.6 | ⬜ |
| 4 | Organogram | Role | R.1–R.6 | ⬜ |
| 5 | Parameter | Shape/Secret | PR.1–PR.6 | ⬜ |
| 6 | Service | outros services, imports | S.1–S.8 | ⬜ |
| 7 | Document | Shape, Icon | D.1–D.8 | ✅ |
| 8 | Event | Document | E.1–E.6 | ✅ declarado no Document |
| 9 | View | Document, Role | V.1–V.8 | ✅ |
| 10 | Decision | Document, Role | N.1–N.6 | ⬜ |
| 11 | Policy | Document, Role | P.1–P.6 | ⬜ |
| 12 | Schedule | Process | S.1–S.4 | ✅ não é artefato (start) |
| 13 | Integration | Shape, Service | I.1–I.6 | ⬜ |
| 14 | Report | View | R.1–R.6 | ✅ view type: report |
| 15 | Process | Document (eventos), Role, Integration, View | P.1–P.24 | ✅ |
| 15.1 | Form | Document, View | F.1–F.8 | ✅ |
| 15.2 | Message | Document | M.1–M.4 | ✅ |
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

- [ ] `title: I18N` onde a regra exige (app/service/process/document/view/form/message/manual/shape/field/event/parameter/role/organogram/icon/end/transition/action); `goal` no process; `instructions` no manual.
- [ ] Campos têm `title`; payloads de evento têm `title`; cada `value` de enum tem `title` + `icon`.
- [ ] `short`/`tooltip` são irmãos do mapa principal; fallback exato → idioma → default.
- [ ] Todo `service` declara `locales` com **pelo menos um** idioma; o primeiro é o default do I18N em forma de string (`E_LOCALES_REQUIRED` se faltar/vazio).
- [ ] `I18N` aceita string (primeiro locale do serviço) ou mapa de locales; cada locale aceita `string | PluralForms`; `other` obrigatório. O compilador emite um `ShapeValue` com `typeRef: "std.bpcms.i18n"`, expande a string e filtra pelos locales da aplicação.
- [ ] Units usam `title` I18N com `one`/`other`; sem campo `plural` separado.
- [ ] Strings aceitam aspas simples/duplas; `format` prefere duplas e `lint` avisa `W_STRING_QUOTES`.
- [ ] Literal de `Duration` (`sla`) é string entre aspas (`sla: "30d"`); sem aspas é `E_SHAPE_LITERAL_QUOTED`.
- [ ] Nomes de arquivos em kebab-case; referências e funções em camelCase. `documents/` e `processes/` são organizacionais; `icons/` é contêiner de namespaces de ícone; as demais subpastas são namespaces.
- [ ] Artefatos em PascalCase, com exceções internacionais (`pH`).
- [ ] Nomes de artefatos são claros, sem prefixo de serviço; escopo é o serviço; sem duplicidade no mesmo tipo/serviço.
- [ ] Icons sem prefixo `Icon`; iguais em serviços diferentes são permitidos; duplicados no mesmo serviço são erro.
- [ ] Nenhum serviço criado só para biblioteca/agrupamento de Shapes/Icons/Themes/Locales.
- [ ] Nenhuma cláusula foi avaliada por comportamento de builder/runtime; apenas por contrato DSL.
- [ ] Shapes compartilhados ficam no serviço de negócio dono.
- [ ] Application não declara artefatos.
- [ ] `id` de document é implícito; `lookup` é tag de campo; FK é soft; extensão compartilha id.
- [ ] Nomenclatura e caminhos: app/service `<nome>-<tipo>.archol`; document/process `<service>-<nome>-<tipo>.archol`; `BPCMS_WORKSPACE = examples/`; um principal por arquivo, primeiro artefato, nome equivalente; compartilhado com um único tipo por arquivo (exceção: `-organograms.archol` com organogram antes das roles) e sem principal.
- [ ] Exemplos em `DSL_EXAMPLES.md` e em `examples/` concordam com a referência.
- [ ] BRAINSTORM/DSL_REFERENCE/DSL_EXAMPLES atualizados juntos quando o contrato muda.
- [ ] Nenhuma decisão de implementação (font/color/layout/resolução visual) foi revisada nesta fase.
- [ ] `git diff --check`; sem `add/commit/push` sem pedido.

## 1. Icon ✅

**Depende de:** nada na DSL.

**Sintaxe/cláusulas:**

- [x] `icon <Name> { ... }`
- [x] `title: I18N` obrigatório
- [x] `Icon` definido dentro de um `Service`
- [x] `icon` sem prefixo `Icon` nem prefixo de serviço
- [x] Escopo do nome = serviço; mesmo nome em serviços diferentes permitido
- [x] `E_DUPLICATE_ICON` apenas no mesmo serviço
- [x] Sem imagem/binário no DSL
- [x] SVG file-based: qualquer subpasta sob a raiz do serviço
- [x] Nome do SVG: caminho relativo em camelCase com `.`; arquivos/pastas em kebab-case
- [x] Title do SVG: `aria-label` → `<title>` → `alt` → filename humanizado; sem I18N
- [x] Conflito SVG × Icon declarado = erro

**Exemplos:** IC.1–IC.6; icons nos `*-icons.archol`; **8 SVGs sob `icons/`** (status, finance/invoice, channels, stock etc.).

**Foco da revisão (só DSL):**

- [x] Um Icon precisa de mais alguma cláusula além de nome + `title`?
- [x] `symbol` é cláusula de DSL ou detalhe de resolução visual do builder?
- [x] `set` é cláusula de DSL ou escolha do tema?
- [x] `title` de Icon precisa de `short`/`tooltip`? Provavelmente não; confirmar.
- [x] Nomes locais (`Draft`, `Paid`) e escopo por serviço são suficientes?

**Status:** ✅ revisado em DSL.

**Decisão:** Icon = nome + `title`; `symbol`/`set` ficam fora da DSL. SVG file-based faz parte da alternativa de Icon: qualquer subpasta, kebab-case, identificador camelCase com ponto, title por `aria-label` → `<title>` → `alt` → filename, sem I18N. `E_ICON_UNRESOLVED` é erro de resolução do builder, não de sintaxe DSL.

### P-ICON-SVG — SVG file-based Icons (aprovada)

- **O que muda:** qualquer `.svg` sob `services/<service>/` (qualquer subpasta) vira Icon implícito.
- **Nome:** caminho relativo com segmentos camelCase e pontos; `status/refunded.svg` → `status.refunded`; `checkout/status/refunded.svg` → `checkout.status.refunded`.
- **Referência local:** `icon: status.refunded`.
- **Referência por alias:** `Checkout.status.refunded` quando `checkout` está importado como `Checkout`.
- **Title:** `aria-label` → `<title>` → `alt` → filename humanizado; sem I18N.
- **Conflito:** SVG × Icon declarado ou SVG × SVG com o mesmo identificador é erro (case-insensitive).
- **Arquivos/pastas:** kebab-case; identificador de referência camelCase.
- **Exemplos criados:** `billing/icons/status/draft.svg`, `paid.svg`, `overdue.svg`, `billing/icons/finance/invoice/issued.svg`, `notifications/icons/channels/email.svg`, `inventory/icons/stock/low-stock.svg`, `payments/icons/status/refunded.svg`, `fraud-checkout/icons/status/fraud-checked.svg`.
- **Status:** ✅ aprovado para file-based Icons. A aplicação da regra de namespace/camelCase a **todos** os artefatos continua pendente de validação própria.

### Regra de namespaces — perguntas abertas

- [ ] A declaração continua `document Order` e o namespace vem da pasta (`domain.order`), ou a declaração precisa incluir o namespace?
- [ ] Arquivos de tipo com múltiplos artefatos (ex.: `documents.archol`) podem coexistir com namespaces por subpasta?
- [ ] Dentro do mesmo namespace, a referência pode ser curta (`order`) ou sempre completa (`domain.order`)?
- [ ] O alias cross-service é `Checkout.domain.order` ou `checkout.domain.order`?
- [ ] A conversão kebab → camelCase vale para nomes multiword (`order-item` → `orderItem`, `money-brl` → `moneyBrl`)?
- [ ] O mesmo nome em namespaces diferentes é permitido sem restrição? (proposta: sim, duplicidade só no mesmo tipo/namespace)
- [ ] Arquivos `.archol` em subpastas continuam com prefixo `<service>-` ou usam só o nome do artefato? Hoje a regra de arquivos exige prefixo.

### P-ICON-01 — remover `symbol`/`set` (aprovada na revisão de Icon)

- **O que muda:** `Icon` perde `symbol` e `set`; fica apenas `title`; entra `E_ICON_UNRESOLVED`.
- **Por que:** `symbol`/`set` parecem detalhes de resolução visual, não contrato de negócio da DSL.
- **Impacto:** 75 icons, exemplos IC.1–IC.6, `DSL_REFERENCE` e `BRAINSTORM`.
- **Alternativas:** (a) restaurar `symbol`/`set`; (b) manter apenas nome + `title`; (c) `symbol` opcional.
- **Arquivos afetados:** `examples/services/**/*-icons.archol`, `examples/services/**/icons/**`, `DSL_REFERENCE.md`, `DSL_EXAMPLES.md`, `BRAINSTORM.md`.
- **Status:** ✅ aprovada retroativamente na revisão de Icon.

**Decisão:** manter `symbol`/`set` fora da DSL; resolução visual fica no builder/Theme.

## 2. Shape ✅

**Depende de:** Icon (valores enumerados), tipos embutidos.

**Sintaxe/cláusulas:**

- [x] `shape <Name> { ... }`
- [x] `title: I18N`
- [x] `base: decimal | integer | string | classification | workflow | attachment | audio | image | video | richtext`
- [x] Shapes de `audio`/`image`/`video` declaram `capabilities { gui }`
- [x] `value <name> { title; icon; ordinal }` — `classification`/`workflow`; ordinal 10, 20, 30...
- [x] `currency`
- [x] `dimension`, `units { unidade: { factor, title } }`
- [x] `units` titles têm plural I18N e `format` escolhe `one`/`other`
- [x] `unit`, `min`, `max`, `scale` e `precision` são do field (exceto `unit` em Shape de unidade única)
- [x] `unit` e `units` mutuamente exclusivos; `Percentage` usa `unit: "%"`
- [x] `format` auto-seleciona unidade; `flags.unit` força
- [x] `maxLength`; `min`/`max`/`scale` no field
- [x] `initial` único em `workflow`; `transition` define arestas; estado sem transição é terminal
- [x] `canTransitionTo` automático; mudança inválida lança `E_INVALID_TRANSITION`
- [x] `parse`, `format`, `compare`, `aggregate`
- [x] `write(value, out)` com `out` inferido como `streamWriter`; `format(value, flags)` com `value` explícito; `read(in)` com `in` inferido como `streamReader`; `void` inferido
- [x] `new Shape{raw, unit}` e shorthand de propriedades
- [x] Erro de tipo/propriedade em `new` é `E_TYPE`/`E_CONSTRUCTOR_PROPERTY`
- [x] `<name>(...)` sem keyword `method`; compilador detecta por `(`
- [x] Argumentos obrigatórios; tipos/retorno explícitos ou inferidos
- [x] Exemplos de métodos com tipos explícitos, inferidos e mistos
- [x] `toNumber(value)` só em Shapes numéricos
- [x] `toOrdinal(value)` é gerado automaticamente em classification/workflow; não aparece no código
- [x] `Duration` em std; `Length / Duration -> Speed`
- [x] `format`/`parse` de compostos exigem método próprio
- [x] `convert` inteligente; `formula` com `to`/`from`
- [x] `format` com `unitStyle: short | long`; `parse` exige unidade compatível
- [x] `index`, `normalize`, `validate`, `sensitive`
- [x] `fields { ... }` — shape composto
- [x] `default`, `private`
- [x] Enum: cada `value` tem `title` + `icon` + `ordinal` (exceto valores-base de `std.Currency`); workflow usa `transition`; icons são locais ao serviço
- [x] `I18N` não é Shape; `shape I18N` é `E_RESERVED_TYPE`
- [x] `<Document>Id` é Shape implícito de FK; não declarar

**Exemplos:** H.1–H.9; shapes de contexto e `<service>-shapes.archol` (compartilhados), como `citizen-shapes`, `notifications-shapes`, `billing-shapes`.

**Resultado da inspeção:**

- [x] 39 Shapes de negócio + 57 Shapes std; todos com `title` (exceto valores-base de `Currency`).
- [x] 22 enums / 77 `value`; todos com `title` + `icon` + `ordinal`; 16 workflows com `initial`/`transition`; 6 classifications sem transições.
- [x] `value` repetido por estado; sem bloco `values`.
- [x] Shape composto `TrackingEvent` com fields tipados/title.
- [x] `Email`, `Phone`, `Image`, `LocaleTag`, `YearMonth` continuam em serviços de negócio; `MoneyBRL` foi removido.
- [x] `Mass`, `Length`, `Volume`, `Percentage`, `Count` movidos para `bpcms.org/std`; `Money`/`Currency` vivem na std.
- [x] Catalog/Identity/Shipping/Inventory/Orders/Billing importam `bpcms.org/std` como `std`.
- [x] `Email`, `Phone`, `Image` em Citizen.
- [x] `LocaleTag` em Notifications.
- [x] `YearMonth` em Billing.

**Pontos para decidir (sem alterar):**
- [x] `formula: <expr>` e bloco `to/from` aceitos; shorthand só afim invertível.
- [x] `std.Currency` é exceção: `value` sem `title`/`icon`/`ordinal`; `reintroduce` completa.
- [x] `std.Currency` não tem `toOrdinal`; stream/format/parse usam o código string.
- [x] `Currency.format`/`parse` retornam o próprio código string.
- [x] `Address.format` formata BR/US conforme `std.regionalization`.
- [x] Mídia exemplificada: `ProductImage`, `ProductVideo`, `ProductAudio`, `ProductManual`, `ProductRichDescription`.

- [x] `Percentage` agora é `std.Percentage`; a assinatura de `Order.create` está correta?
- [x] `default`, `private` e `sensitive` exemplificados em `Ticket`; semântica de field.
- [x] Shape composto tem serialização JSON canônica por default; `format`/`parse` sobrescrevem.
- [x] Campo calculado em Shape e Document: `computed: expression`; somente leitura; sem rule redundante.
- [x] `std.Money` substitui `MoneyBRL`; `currency` é restrição de field; valor carrega moeda; multi-moeda exemplificado em `Ticket.budget`.
- [x] `std.Count` substitui o Count de Identity; a migração está correta?
- [x] `LocaleTag` em Notifications é tipo de negócio ou detalhe de regionalização?
- [x] `bpcms.org/std` definido e importado; 57 Shapes; migração concluída?
- [x] `format`/`parse`/`validate` aceitam valores que não estão nos exemplos; a lista de valores precisa ser fechada?
- [x] `Shape` tem `title`, mas fields de Shape composto precisam de `title` explícito em todos os casos?

**Status:** ✅ revisado e aprovado para a v0.1.

**Decisões:** Shape base + campos; moeda em `Money`/`Currency`; `formula` block ou shorthand afim; `Currency` sem `title`/`icon`/`ordinal` e sem `toOrdinal`, stream/format/parse como código string; composto com JSON canônico e `format`/`parse` sobrescrevíveis; `sensitive`/`maxLength` default no Shape com sobrescrita no field; `default`/`private` no field; 39 Shapes de negócio + 57 std; 178 moedas ISO.

**Stdlib:** definida em `examples/services/bpcms.org/std/std-service.archol` e `examples/services/bpcms.org/std/std-shapes.archol`; import `service "bpcms.org/std" as std`; uso `std.Mass`. Migração concluída; `Money` tem `toNumber` implícito; `Currency` é classification ISO com `scale` e `reintroduce`.

## 3. Role ✅

**Depende de:** permissões/capabilities.

**Sintaxe/cláusulas:**

- [x] `role <Name> { ... }`
- [x] `title: I18N`
- [x] `permissions: [...]`
- [x] `capabilities { ... }`
- [x] `inherits: Role | [Role...]`; permissões próprias + herdadas
- [x] `scopes`
- [x] `delegates to` por linha
- [x] `all` / `only` / `except`
- [x] Atribuição derivada da hierarquia; auto-atribuição proibida
- [x] Papel público; sem `private` de topo
- [x] Papel definido no serviço ou no organogram

**Exemplos:** R.1–R.8; `identity-organograms`, `support-organograms`, `laboratory-organograms` (organogram + roles).

**Foco da revisão:**

- [x] Papéis são escopados ao serviço ou globais?
- [x] `permissions` é lista de namespaces ou catálogo?
- [x] Herança de papel é suficiente?
- [x] Organogram + roles no mesmo arquivo, com o organogram primeiro, está funcionando?
- [x] O gerente atribui apenas papéis abaixo dele na hierarquia, nunca a si mesmo?
- [x] As permissões do papel atribuído são independentes das permissões do gerente?

**Status:** ✅ revisado e aprovado para a v0.1.

**Decisões:** role com `title`, `permissions` como namespaces de ação, `inherits` aditivo e `scopes`; não existe `manages`; a autoridade de atribuição deriva da hierarquia do organograma; auto-atribuição proibida; o papel atribuído define as permissões do alvo, não o papel do gerente; organogram e roles no mesmo `*-organograms.archol`, com o organogram antes das roles; Support tem `Manager` local; exemplos R.1–R.8 com títulos, hierarquia e herança múltipla.

## 4. Organogram ✅

**Depende de:** Role.

**Sintaxe/cláusulas:**

- [x] `organogram <Name> { ... }`
- [x] `scopes: [Tipo]`
- [x] `title: I18N`
- [x] `hierarchy { A > B > C }`; um caminho por linha
- [x] `fourEyes { ação requires Role }`
- [x] `segregation { ação != ação }`
- [x] `delegates to ... when absent for 7d`
- [x] Sem `manages`; atribuição deriva dos descendentes na hierarquia
- [x] `signature { Role }`
- [x] `requires` dentro de `fourEyes`
- [x] organogram antes das roles no mesmo arquivo (`*-organograms.archol`)

**Exemplos:** R.1–R.8; `identity-organograms`, `support-organograms`, `laboratory-organograms` (organogram + roles).

**Foco da revisão:**

- [x] `Manager > SupportAgent` em Support pode referenciar role de Identity? — Support usa `Manager` local.
- [x] `title` de organogram é obrigatório?
- [x] `fourEyes`/`segregation` são avaliados no runtime ou no build? — build valida estrutura; runtime decide.
- [x] `limits`/`dataMasking` pertencem ao Document, não ao Organogram.
- [x] `delegates to` aponta para uma role abaixo na hierarquia; sem `roleManagement`.

**Status:** ✅ revisado e aprovado para a v0.1.

**Decisões:** `scopes` plural; `delegates to` por linha e sem `->`; autoridade de atribuição derivada da hierarquia; `fourEyes` exige ator diferente com role; `segregation` e `signature` ficam no Organogram até revisão de Document; `limits` e `dataMasking` movidos para Document; build valida estrutura, runtime decide autorização.

## 5. Parameter / Secret ✅

**Depende de:** Shape.

**Sintaxe/cláusulas:**

- [x] `parameter <Name> { ... }`
- [x] `secret <Name> { ... }`
- [x] `title: I18N`
- [x] `type: Shape` obrigatório em ambos
- [x] `required` bare
- [x] `default` só em Parameter; mutuamente exclusivo com `required`
- [x] `min`/`max`/`maxLength` como no field
- [x] Parameter/Secret declarados apenas dentro de Service
- [x] App faz `set <Alias>.<Parameter|Secret>`
- [x] Secret usa `secret.NOME`, nunca literal
- [x] Secret nunca serializa/loga/exporta/vai à LLM
- [x] Parameter não referencia outro Parameter
- [x] `Secret` é artefato, não Shape
- [x] `source: config/secret` não existe

**Exemplos:** PR.1–PR.6; `*-parameters.archol` e `*-secrets.archol`.

**Foco da revisão:**

- [x] `title` em Parameter substituiu `description` por completo?
- [x] `required` + `default` são mutuamente exclusivos?
- [x] Secret sem default é suficiente?
- [x] `type` aponta para Shape em Parameter e Secret?
- [x] `min`/`max`/`maxLength` são constraints do Parameter/Secret, não do Shape?
- [x] `set` de Secret sempre usa `secret.NOME`?
- [x] Parameter/Secret não vão ao lock, só o ID?

**Status:** ✅ revisado e aprovado para a v0.1.

**Decisões:** Parameter e Secret são artefatos separados; `type` aponta para Shape; `source` removido; `required` e `default` mutuamente exclusivos; Secret sem default; `min`/`max`/`maxLength` no Parameter/Secret; sem referência entre Parameters; `set` de Secret usa `secret.NOME`.

## 6. Service ✅

**Depende de:** outros services, imports.

**Sintaxe/cláusulas:**

- [x] `service "<uri>" { ... }`
- [x] `title: I18N`
- [x] `locales` obrigatório (≥ 1, primeiro é default); sem `owner`/`version` em artefato
- [x] `imports { service "<uri>" as Alias; integration X }`; vazio permitido
- [x] `reintroduce <Alias>.<Artifact> { ... }`; altera qualquer parte de artefato importado
- [x] Sem `provides`; contrato são os artefatos públicos
- [x] Sem `roles` inline; organogram e roles no `*-organograms.archol`, organogram primeiro
- [x] Sem `source`/`owner`; origem inferida pela URI e versão no Release/lock
- [x] Sem `exports`; todo artefato de topo é público
- [x] Serviço é conceito de negócio, não biblioteca/package/módulo
- [x] Não criar serviço para Shapes/Icons/Themes/Locales compartilhados
- [x] Theme/Locale são do construtor

**Exemplos:** S.1–S.8; `examples/services/**/*-service.archol`.

**Foco da revisão:**

- [x] `imports` vazio é permitido? — sim.
- [x] `roles` inline vs arquivo de organogram? — apenas `*-organograms.archol`, com organogram antes das roles.
- [x] `capabilities` fica no artefato, não no Service; `provides` não existe.
- [x] Serviço de negócio pode ter `Shape`/`Icon` locais? — sim, se forem do domínio.
- [x] `reintroduce` pode alterar qualquer parte de qualquer artefato importado? — sim, aditivo; conflito é erro.
- [x] `capabilities` pode existir em qualquer artefato? — sim; role de equipamento exemplifica.

**Status:** ✅ revisado e aprovado para a v0.1.

**Decisões:** Service sem `provides`, `roles`, `source`, `requires`, `capabilities` ou cláusula de substituição; usa `imports` e `reintroduce`; capabilities ficam no artefato que as exige; `imports` vazio é válido; `capabilities` pode ser declarada em qualquer artefato e falha no build se o construtor não a oferecer; roles ficam em `*-organograms.archol`, não no service.

## 7. Document 🟡

**Depende de:** Shape, Icon, Role (ações), Service.

**Sintaxe/cláusulas:**

- [ ] `document <Name> { ... }` e `scopable document <Name> { ... }`
- [ ] `title: I18N`
- [ ] `id` interno implícito; não declarar
- [ ] `lookup` como tag de campo; ordem dos campos
- [ ] `extends` (pai único, mesmo serviço, aditivo, mesmo id)
- [ ] `sync`, `retention`; sem `owner`/`version` em artefato
- [ ] `fields { name: Shape { title; required; lookup; unique; rule; ... } }`; sem `?`
- [ ] Campo `Target` é `DocumentRef` implícito: `id` + lookup do alvo; sem `XxxId`; não é FK
- [ ] campo para `scopable document` infere escopo
- [ ] sem `source`/`scope` block/parentId/path
- [ ] ID interno × `number` de negócio
- [ ] `number` com `format`, `sequence`, `unique`
- [ ] `indexes` (unique/range/search/hash), inclusive `.id`
- [ ] `actions` (públicas/private) com `title`, `roles`/`roles: public`, `transition`, `rule`, body e sem retorno
- [ ] `events { ... }` apenas para eventos sem workflow
- [ ] `emit <Event>` apenas em action sem workflow
- [ ] `transition: <Event>` em ações que mudam workflow
- [ ] UI recebe botão habilitado/desabilitado, `fields` e motivo
- [ ] `report` como `document type: report`
- [ ] `seed` para documentos pré-criados idempotentes
- [ ] `Document.Seed` é `DocumentRef` válido e determinístico
- [ ] `Store.SP` pode ser usado como valor/default em fields, actions e processos
- [ ] renomear seed é breaking change
- [ ] `rule` no Document, action e field; condição verdadeira = permitido
- [ ] `required` para obrigatoriedade simples; `rule` para condicional
- [ ] `default` aplica antes de `required`; default inválido = erro
- [ ] `source: device ...`
- [ ] `computed from`, `computed <Name> from`, `freshness`, `exposes`
- [ ] referência circular usa nome diferente do id (`parent`)
- [ ] duplicidade de id e referência para si mesmo
- [ ] PII em `private` quando aplicável
- [ ] regras de leitura e edição por field
- [ ] sem bloco de permissões no documento
- [ ] permissões por ação: `roles` ou `roles: public`
- [ ] permissões por campo: `read`/`edit`
- [ ] search via processo que declara `reads`
- [ ] combinação documento × campo × role × scope
- [ ] `mask` em field
- [ ] `limits` em action
- [ ] `segregation` e `signature` no documento, se aplicável

**Exemplos:** D.1–D.6; `<service>-<nome>-document.archol`.

**Foco da revisão:**

- [ ] `lookup` é tag de campo; `short`/`tooltip` são do title.
- [ ] `extends` restrito a documento no mesmo serviço.
- [ ] `id` implícito e `<Doc>Id` implícito.
- [ ] `DocumentRef` não é FK; exclusão da origem não invalida vínculos antigos.
- [ ] `computed` + `lookup` + `title` interagem bem?
- [ ] `private` é suficiente para PII ou precisa `sensitive`?
- [ ] `Date`/`DateTime`/`YearMonth` são tipos internos?
- [ ] `dataMasking`, `limits`, `segregation` e `signature` pertencem ao Document ou ao Organogram?
- [ ] Permissões de field/ação usam `permissions` da role, `scopes` ou policy?
- [ ] UI recebe motivo genérico de permissão e de workflow?
- [ ] Ação que muda workflow declara `status =>`?

**Status:** 🟡 em revisão.

## 8. Event ✅ (merged into Document)

**Depende de:** Document.

**Decisão:** eventos são declarados dentro do Document; não há artefato `event` de topo.

- [x] `events { Approved { title; key; payload } }`
- [x] `emit Approved { ... }` na action
- [x] `producer` implícito
- [x] sem `consumers`
- [x] referência `Order.Approved`
- [x] payload só de negócio
- [x] envelope automático do builder
- [x] ação que muda estado emite evento
- [x] ação com `status =>` emite evento

**Status:** ✅ merged em Document.

## 9. View ⬜

**Depende de:** Document, Role, Shape.

**Sintaxe/cláusulas:**

- [ ] TODO precisamos de algo mais proximo do conceito de user task / people task do BPM
- [ ] `view <Name> { ... }`
- [ ] `title: I18N`
- [ ] `type: form | table | dashboard | wizard | detail | kanban`
- [ ] `document`, `source`, `sources`
- [ ] `roles`
- [ ] `fields` com `title`, `required`, `readonly`; sem `required` = opcional; sem `?`
- [ ] `columns`, `filters`, `filter`, `pagination`
- [ ] `actions`
- [ ] `metrics`, `freshness`, `units`, `export`
- [ ] `steps`, `data`, `integration`
- [ ] `sections`, `computed`, `device`
- [ ] `groupBy`, `card`
- [ ] `mode`
- [ ] `capabilities { gui }` na View apenas se a tela inteira exigir GUI por outro motivo
- [ ] Shapes de áudio/imagem/vídeo declaram `capabilities { gui }`

**Exemplos:** V.1–V.6; arquivos em `views/` (uma view por arquivo, incluindo `report`/`dashboard`).

**Foco da revisão:**

- [ ] Labels de colunas vêm dos fields/document titles?
- [ ] `fields` de view exigem `title` próprio?
- [ ] `actions` de view podem apontar para Job/Integration?
- [ ] `computed` de view vs Document computed?
- [ ] View pode ter `device` sem capability?
- [ ] `type: form` é suficiente para user task?
- [ ] View precisa de vínculo explícito com process task?
- [ ] `gui` aparece no Shape de mídia?
- [ ] `gui` aparece na View apenas quando a tela inteira não funciona em TUI?

**Decisões propostas:**

- View `type: form` é a UI de user task.
- A task referencia a View via `view:`.
- A View não executa a task.
- O processo mantém o andamento.
- A View reutiliza fields/actions/document.
- Form não é artefato; é `View type: form`.
- View `type: process` exibe andamento; não substitui o processo.
- View de user task é referenciada por `view:`.

**Status:** 🟡 design inicial.

## 10. Decision ⬜

**Depende de:** Document, Role.

**Sintaxe/cláusulas:**

- [ ] `decision <Name> { ... }`
- [ ] `title`? (não está na lista atual de obrigatórios — decidir)
- [ ] `appliesTo`
- [ ] `when`
- [ ] `cases { cond -> outcome }`
- [ ] `default`


**Foco da revisão:**

- [ ] Decision precisa de `title` I18N?
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

**Exemplos:** P.1–P.6; arquivos de contexto e `<service>-policies.archol` (compartilhadas).

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

**Exemplos:** J.1–J.6; `*-jobs.archol`.

**Foco da revisão:**

- [ ] `action` é método de Document ou serviço?
- [ ] `schedule` aceita cron + linguagem natural?
- [ ] `filter`/`condition` são expressões DSL?
- [ ] `idempotent` deve ser obrigatório em ação mutante?
- [ ] Job pode iniciar `process` em vez de `action`?
- [ ] Job deve continuar separado de Process?

**Decisões propostas:**

- Job é invocação agendada, não processo.
- Não tem lanes, user task ou progresso.
- Pode executar action.
- Pode iniciar processo no futuro.
- Auditoria é automática.
- Job não tem UI própria.

**Status:** 🟡 design inicial.

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

**Exemplos:** I.1–I.6; arquivos de contexto e `<service>-integrations.archol` (compartilhadas).

**Foco da revisão:**

- [ ] `auth` credenciais vêm de Parameter/Secret?
- [ ] Operation types são Shapes?
- [ ] Webhooks podem mapear Event/Document?
- [ ] `fallback`/`quality` são detalhes do builder?

**Status:** ⬜ pendente.

## 14. Report ✅ (merged into Document)

**Depende de:** Document.

**Decisão:** Report é um `document type: report` com caching de cálculo, não um artefato separado.

- [x] `document ReportName { type: report; ... }`
- [x] `computed from`
- [x] `freshness`
- [x] `exposes`
- [x] `report { ... }` opcional
- [x] sem `report` como artefato de topo
- [x] permissões por field/ação
- [x] sem CRUD automático

**Exemplos:** R.1–R.6 migram para Document.

**Status:** ✅ merged.

## 15. Process ⬜

**Depende de:** Document (eventos), Role, Integration, View.

**Sintaxe/cláusulas:**

- [ ] `process <Name> { ... }`
- [ ] `title`
- [ ] `goal` (`I18N`), `sla` (`std.Duration`), `roles`; sem `version` em artefato
- [ ] `start on Document.Event`, `end on Document.Event`
- [ ] `task` (type/lane/view/reads/writes/sla/progress)
- [ ] `pool` interno/externo/system
- [ ] `lane` com role/integration/system
- [ ] `variables` locais tipadas
- [ ] `view` como UI da user task
- [ ] People task / User task
- [ ] Form task / `form`
- [ ] Service task
- [ ] Business rule task / Decision
- [ ] Manual task
- [ ] Send task / Receive task
- [ ] Script task (se suportado)
- [ ] Subprocess / Call activity
- [ ] `type: compensation` + `for` + `action`
- [ ] Boundary events: timer, error, message, signal, escalation
- [ ] Start/end/intermediate events
- [ ] `gateway` exclusivo, inclusivo, paralelo, event-based, complexo
- [ ] `if/else`, default flow, conditional flow
- [ ] `insert`
- [ ] `assignment` (claim/delegate/reassign)
- [ ] `interventions` (pause/resume/cancel/retry/skip)
- [ ] `milestone` e `discretionary`
- [ ] `event subprocess`
- [ ] `send`/`receive` + `correlation`
- [ ] `loop`/multi-instance
- [ ] DMN `decision` + outputs
- [ ] coreografia/conversação via pools/mensagens
- [ ] Multi-instance / loop
- [ ] Pools / lanes / participantes
- [ ] Collaboration diagram
- [ ] Choreography diagram / choreography task
- [ ] Conversation diagram
- [ ] Decision diagram / DMN
- [ ] BPMN DI (formas e conexões)
- [ ] `integration`
- [ ] `type: external` (parceiro/device)
- [ ] `type: manual`
- [ ] `type: decision` / DMN
- [ ] `message` em send/receive

**Exemplos:** P.1–P.7; `<service>-<nome>-process.archol`.

**Foco da revisão:**

- [ ] Process precisa de `title`?
- [ ] `start`/`end` por Event é suficiente?
- [ ] Compensação sempre em ordem reversa?
- [ ] `reads`/`writes` são obrigatórios?
- [ ] `role` de task é Role do app?
- [ ] É possível mesclar workflow e process em um único artefato?
- [ ] `value` deve continuar sendo estado, não task?
- [ ] Processo pode disparar workflow via action?
- [ ] Subprocesso é o mecanismo de progresso/subpasso?
- [ ] View de user task é reutilizável?
- [ ] Pools/lanes cobrem externo e automação?
- [ ] Variáveis locais passam para actions?
- [ ] Document é armazenamento durável?
- [ ] Quais diagramas BPMN/DMN/CMMN suportar?

**Decisões propostas:**

- Workflow continua sendo máquina de estados do Document; não é process.
- Process é orquestração de tarefas.
- `value` não vira task.
- `pool`/`lane` definem executor interno/externo/sistema.
- User task usa `view`.
- Variables são locais da instância; Document é armazenamento durável.
- `reads`/`writes` são data objects do BPMN.
- Subprocesso é o mecanismo de progresso/subpasso.
- Tarefa automática pode ter `progress`.
- DMN via `decision`.
- Choreography/conversation/CMMN ficam fora da v1.

**Status:** 🟡 design inicial; exemplos/documentação em revisão.

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

**Exemplos:** W.1–W.6; `*-widgets.archol` se existir.

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
- [ ] `I18N` de titles depende de Locale?
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

**Exemplos:** B.1–B.6; `*-builders.archol` se existir.

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
- [ ] `title: I18N`
- [ ] `source`, `lock`; sem `version` no corpo
- [ ] `services { "<uri>" as Alias }`
- [ ] `roles` + hierarquia
- [ ] `theme` (builder)
- [ ] `locales` (app/construtor)
- [ ] `environments`
- [ ] `main { set Alias.Parameter; start }`
- [ ] Sem artefatos
- [ ] Sem `wire`; closure por imports
- [ ] Lock: uma versão/serviço + IDs

**Exemplos:** A.1–A.8; `applications/*/*-application.archol`.

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
- [ ] `@<selector>` (semver/tag/commit); sem `version:` no corpo
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

## 26. metodos
- [ ] precisam title, permissao, etc...

## 27. reintroduce

- [ ] Revisar uso de `reintroduce` para cada artefato.
- [ ] Revisar uso para cada atributo.
- [ ] Revisar uso para métodos/ações.
- [ ] Revisar uso em Process: `task`, `insert`, transições, gateways, eventos.
- [ ] Revisar `insert` em workflow: como resolver ordinal do novo estado?
- [ ] Reintrodução é aditiva e altera só a parte declarada.
- [ ] Conflito de metadados/nós = erro.
- [ ] Validar dependências, ciclo, alvo existente e assinatura.
- [ ] Definir efeito no AST, diff, lock e closure.
- [ ] Definir como o construtor materializa a alteração.
- [ ] Revisar depois de todas as keywords.

### Ordinal do `insert` em workflow

- [ ] O novo estado recebe ordinal gerado ou declarado?
- [ ] O ordinal é posicional antes/depois do alvo?
- [ ] Como evitar colisão de ordinal com estados existentes?
- [ ] O ordinal é estável entre builds/migrações?
- [ ] `insert`
- [ ] `assignment` (claim/delegate/reassign)
- [ ] `interventions` (pause/resume/cancel/retry/skip)
- [ ] `milestone` e `discretionary`
- [ ] `event subprocess`
- [ ] `send`/`receive` + `correlation`
- [ ] `loop`/multi-instance
- [ ] DMN `decision` + outputs
- [ ] coreografia/conversação via pools/mensagens recalcula ordinais dos vizinhos?
- [ ] O AST guarda a posição lógica ou apenas ordinal?
- [ ] O diff mostra inserção sem reordenar o restante?
- [ ] O lock preserva IDs/ordinais dos estados existentes?

## Reavaliação da ordem

**Próximo: Document.**

Motivo:

- Document é a base de `DocumentRef`, `scope`, `id`, `number`, permissões e leitura/edição;
- View, Decision, Policy, Job e Process dependem de Document;
- Process também depende de eventos do Document, Role, Integration e View;
- revisar Process antes de Document criaria decisões sobre nós/eventos sem o contrato de dados fechado;
- Process fica para depois de Document, View, Decision e Integration.

Ordem recomendada:

1. Document
2. View
3. Decision
4. Policy
5. Job
6. Integration
7. Process

O item 27 (`reintroduce`) fica como revisão transversal depois das keywords.

## Revisão de Icon — encerrada

**✅ Aprovado:** nome + `title`; sem `symbol`/`set`; SVG file-based com namespace local; title sem I18N; fallback de filename; conflito no mesmo serviço é erro.

Próximo: **Document** 🟡.

Service revisado; seguimos para **Document**.

## Pendências ao final do checklist

- [ ] Revisar `mask`, `limits`, `segregation` e `signature` com exemplos reais.


- [ ] Riscos da migração `next` → `transition`: payload perdido, processos escutando evento renomeado, workflows compartilhados, ações em estados sem transição.
- [ ] Validar FQN canônico versus alias local na aplicação.
- [ ] Revisar eventos não-workflow remanescentes.

## Pendências de revisão de keywords

- [ ] `reintroduce`: revisar depois de todas as keywords existentes.
- [ ] `classification: <url>`: revisar junto com `reintroduce`; por enquanto só na stdlib.
- [x] `prefix`/`suffix`/`infix` definidos: prefixo/sufixo e infix decimal (CVE).
- [ ] CPF nao pode ser um tipo, revizar Citizen

---

## Rodada Process + Document — 2026-11-17 (aprovada por solicitação direta do usuário)

**Pedido:** revisar tudo com a aplicação orientada a **Process** e **Document**, refazer os exemplos, criar pelo menos 20 processos, rever tipos de diagrama/tarefa e documentação BPM/BPMN/DMN/CMMN, e implementar a melhor solução mantendo consistência.

**Como foi feito:** quatro revisões independentes (BPM/BPMN/DMN/CMMN, gestão por processos/PMI, UX de workflow, gramática/compilador) + uma revisão de governança/KPIs. As decisões abaixo sintetizam o que foi aceito, o que foi recusado e o que ficou deferido. O usuário autorizou explicitamente a aplicação das mudanças nesta rodada; os artefatos anteriores permanecem no histórico do Git.

### Perguntas do usuário e resposta adotada

| Pergunta | Resposta | Consequência na DSL |
| :--- | :--- | :--- |
| "porque precisa `type: user` + `view: form`, está verboso?" | Porque a v1 dizia duas vezes a mesma coisa. O tipo é **inferido do contrato** e o form é o contrato, não a tela. | Removido `type:` de task; `task Review { lane: Sales; form: OrderReview }`. |
| "porque precisamos de Document e Report?" | Não precisamos dos dois: **documento é estado**, **relatório é projeção**. Relatório não tem identidade, lifecycle nem escrita. | Removido `document type: report` e o bloco `report { }`; relatório é `view type: report`/`dashboard`. Documento calculado durável continua (5.10). |
| "porque precisamos de Form e View?" | Form é **contrato de escrita/desfecho de uma tarefa humana**; View é **tela de leitura**. Só há duplicação quando o form redeclara tipo/título do documento — o que foi proibido. | `form` é artefato próprio; `view` perde `type: form` (e `wizard`); campos de form herdam do documento. |
| "Job seria só um evento para iniciar processo? Runtime?" | Sim. Agendamento é **gatilho de início**, não tipo de trabalho; o scheduler é runtime. | Removido o artefato `job`; `start { schedule: "cron" [select:] }`. Rotina em lote é fan-out por sujeito. |
| "podemos transformar workflow em processo, cada value como task?" | Não: estado não tem executor, fila, prazo nem form, e precisa valer fora de processo. **O workflow continua dono dos estados, das transições e dos eventos**; a action só executa o comando e aponta o destino. | `value` mantém `transition <Destino> { title; payload; event? }`; a action declara só `to:` (sem `from`, sem `emits: <Evento>`); Shape vira `base: state`. |
| "lane e pool: é importante saber quem executa, inclusive externo e automatizado" | Pool é participante; lane é executor. Externo é **natureza de pool**, não tipo de task. | `lane` obrigatória; `lane: System` para automação; pool externo com portal (form) ou caixa-preta (só mensagem). |
| "abrir o processo mostra andamento; se for user task, preenche; senão, progresso dentro do passo; subpassos = outro processo" | A unidade de progresso é a **instância de processo**. Subpasso com passos próprios é `runs:`. | Removido `progress: true`; `runs:` é call activity; tela de andamento derivada (4.13). |
| "variáveis locais alimentam actions; documento é armazenamento definitivo; ícones de leitura/escrita" | Variável é contexto transitório; documento é durável; leitura/escrita é **derivada** do contrato. | `reads`/`writes` removidos; AST deriva data objects e ícones; `with`/`gives` fazem a ligação. |

### Decisões estruturais

| # | Decisão | Razão | Impacto |
| :--- | :--- | :--- | :--- |
| 1 | Dois eixos: Process (trabalho) + Document (estado) | Evita dois modelos concorrentes de "onde estou" | referência §§4–5 |
| 2 | Tipo de task inferido de um contrato exclusivo | Removeu a redundância do `type:`; erro de build em conflito | §4.1, §4.7 |
| 3 | `lane` obrigatória em toda task | Trabalho sem dono não entra; alimenta KPI de handoff | §4.3 |
| 4 | Fluxo 100% por arestas explícitas | Ordem de declaração não é semântica; diff e merge não mudam comportamento | §4.10 |
| 5 | `start`/`end` são nós nomeados | Desfecho categorizado (`outcome`) alimenta KPI e UI; substitui `start on`/`end on` | §4.5, §4.6 |
| 6 | `runs:` como único subpasso visível | Uma árvore de instâncias, um histórico, um dono por passo | §4.7, §4.13 |
| 6a | Processo chamado declara `start Begin { called }` | Deixa explícita a entrada do filho (inclusive em fluxo com laço) e impede dupla instância (evento + chamada) | §4.5, §4.7 |
| 7 | `form` é artefato; View não tem form/wizard | Separa escrita (contrato) de leitura (tela); wizard é processo | §4.14, §7 |
| 8 | Transições **e eventos** no workflow (`transition` no `value`); action só aponta `to:` | Evento de workflow definido no workflow, como na v1; sem `from`/`emits:` na action, sem declaração duplicada | §5.3–5.5, §5.7, §22 |
| 9 | `job` removido; `schedule` é start | Um motor só; andamento/auditoria/SLA de graça | §4.5, §23 |
| 10 | `report` removido; relatório é View | Estado ≠ projeção; remove `metrics` duplicado | §21, §7.3 |
| 11 | `reads`/`writes` derivados | Sem drift; ícones BPMN saem do contrato; autorização fica no Document | §4.7, §5.6 |
| 12 | Erro aborta a tentativa e compensa | Evita deadlock de ramo paralelo; semântica de saga | §4.8 |
| 13 | Compensação = `compensates:` + `rollback` | Ordem reversa automática, handler sem aresta (BPMN-fiel) | §4.11 |
| 14 | `cancel on` substitui event sub-process | Cobre "cancelar a qualquer momento" com um nó de destino | §4.8 |
| 15 | `decision` DMN mínima (1 saída, primeiro-que-casa) | Puro, testável, exportável; sem FEEL/DRD na v1 | §4.17 |
| 16 | `assign: pull \| auto \| named` substitui `assignment` | Remove duplicidade com lane/role e com runtime | §4.7 |
| 17 | Inclusivo/complexo/event-based recusados ou deferidos | Ambíguos/caros de validar; "A ou B" fica para v2 | §4.18 |
| 18 | `milestone on:` evento (não `when`) | Marco carimbado no log vira KPI | §4.12 |
| 19 | `sla` do caso + `on breach` do processo | Lead time e escalonamento de caso | §4.2, §4.8 |
| 20 | `foreach` sem agregação declarada (v1) | Agregação é efeito no documento; simplicidade | §4.7 |
| 21 | **Correção:** evento de workflow volta para o `value` (`transition` com `title`/`payload`/`event:`) | A v0.2 tinha posto o evento em `events { }`, violando a regra "evento de workflow é definido no workflow"; a aresta pertence ao Shape e o evento é da aresta | §5.3, §5.7, §22; 213 transições migradas em `examples/` |
| 22 | **Evento de criação no `initial`** (`initial: V with transition X { ... }`), emitido pela action de criação | Fecha o último evento de workflow que ainda morava em `events { }`; o nascimento é a transição de entrada | §5.3, §5.7.2, §22.2; 18 documentos migrados em `examples/` |

### O que foi removido (e por quê)

`type:` em task; `view type: form`; `view type: wizard`; artefato `job`; `document type: report`; bloco `report { metrics }`; `from:` na action; `emits: <Evento> { ... }` (virou `emits { ... }` só de mapeamento); `emit` em action com `to:`; `reads`/`writes`; `assignment { candidates; claim; delegate; reassign }`; `interventions`; `progress: true`; `parallel { }`; `subprocess { }`; `event subprocess`; `end on` / `start on`; `roles:` no processo; `role:` na task; `type: compensation` + `for:`; `outputs { }`.

Motivo comum: duas fontes de verdade, sintaxe redundante, conceito de runtime vazando para a DSL, ou construção que o compilador não consegue validar.

### O que foi adicionado

`for <Document>` (sujeito); `start`/`end` como nós; `outcome` no end; `sla` do caso + `on breach`; `fork`/`join`; `wait { for \| until }`; `rollback`; `compensates:`; `cancel on`; `milestone on:`; `receives:` + `apply:` + `partner:`; artefato `message` com `about`; artefato `form`; `assign:`; `foreach { over; as; mode }`; `gives:`; `returns:` na action; `transition` no `value` (com `title`/`payload`/`event:`); `to:` na action; `emits { ... }` (só mapeamento); `view type: inbox`; `view type: report` (consolidado); `select:` em start agendado.

### Deferido (v2, com gatilho)

- `group` (subprocesso embutido) para escopo de boundary/compensação — gatilho: primeiro caso real de erro tratado dentro de `fork` que não possa ser um processo filho.
- Gateway inclusivo/event-based ("A ou B") — gatilho: dois catches concorrentes no mesmo passo.
- Variantes (`variant of`) e as-is/to-be como delta — gatilho: segundo cliente com variação regional.
- `compute` puro (script task determinística) — gatilho: transformação que não caiba em expressão/`with`.
- Agregação de multi-instância (`collect`/`until`) — gatilho: necessidade de resultado agregado sem gravar no documento.
- Coreografia/conversação como visão derivada — gatilho: primeiro contrato B2B com mais de dois participantes.
- DMN com múltiplas saídas/hit policy — gatilho: decisão que não caiba em um enum de retorno.

### Estado dos exemplos ao final da rodada

- `examples/services/`: **35 processos** em 19 serviços (14 existentes + 6 novos: procurement, contracts, credit, claims, field-service, hr).
- `examples/applications/`: 3 apps existentes + 2 novos (central-de-compras, claims-desk).
- 66 forms, 21 mensagens, 131 views, 55 documentos, **213 transições de estado** (com evento declarado na aresta).
- 165 eventos de transição saíram de `events { }` e voltaram para o `value` do estado de origem; `events { }` ficou só com eventos sem transição.
- `*-jobs.archol` removidos; agendamentos viraram `start { schedule: }` nos processos.
- Validação estrutural dos 35 processos: todo nó alcançável, toda aresta com destino existente, toda task com exatamente um contrato e lane, sem beco sem saída (exceto `end`, compensação, discricionária e task lateral de `keep`).

### Pendências desta rodada

- [ ] Validar com o usuário a remoção de `*-jobs.archol` (conteúdo migrado para processos agendados).
- [ ] Revisar `reintroduce` para as novas construções (nós, arestas, `runs`, `form`).
- [ ] Definir IDs estáveis por caminho de nomes no lock (renomear nó hoje quebra histórico).
- [ ] Decidir se `sla` é obrigatório (erro) ou recomendado (aviso) para task humana.
- [ ] Ícones dos 6 serviços novos (procurement, contracts, credit, claims, field-service, hr): as referências `icon:` existem nos shapes, mas faltam `*-icons.archol`/SVGs em `icons/`. É asset de builder, por isso ficou fora desta rodada.
- [ ] Diagramas: BPMN DI, DMN, CMMN — passo futuro; AST já separa modelo de layout.

## Organização por contexto principal — 2026-11-17 (aprovada por solicitação direta do usuário)

**Pedido:** revisar os nomes dos arquivos de exemplo e criar a regra da DSL: em vez de arquivos por tipo de artefato, arquivos pelo **contexto principal**; `application`, `service`, `document` e `process` são principais; cada principal tem seu próprio arquivo; ele é o primeiro artefato; o `<nome>` do arquivo é equivalente ao nome do artefato; `BPCMS_WORKSPACE = examples/`; pastas por full path da URI; `documents/` e `processes/` organizacionais; `icons/` para SVG; compartilhado com um único tipo por arquivo, exceto organogram + roles (organogram antes).

**Escopo aprovado:** migrar todos os 19 serviços de negócio e validar a estrutura no `check` (`E_FILE_*`); a atribuição semântica dos auxiliares fica como convenção. `application`/`service` mantêm `<nome>-<tipo>.archol` (decisão do usuário); `document`/`process` usam `SERVICE-<nome>-<tipo>.archol`.

**Decisão aplicada:**
- `BPCMS_WORKSPACE = examples/` nesta fase; é o parâmetro de raiz do compilador/construtor.
- Contexto principal: `application`, `service`, `document`, `process`.
- Todo contexto principal tem seu arquivo, com exatamente **um** principal; ele é o **primeiro artefato** do arquivo.
- O `<nome>` do arquivo é equivalente ao nome do principal: kebab-case do identificador em `document`/`process`; slug da pasta/URI em `application`/`service`.
- Caminhos:
  - application: `examples/applications/<fullpath app>/<app>-application.archol`
  - service: `examples/services/<fullpath service>/<service>-service.archol`
  - document: `examples/services/<fullpath service>/documents/<service>-<nome>-document.archol`
  - process: `examples/services/<fullpath service>/processes/<service>-<nome>-process.archol`
  - shared: `examples/services/<fullpath service>/<service>-<tipo-no-plural>.archol`
  - SVG: `examples/services/<fullpath service>/icons/<namespace>/<icon>.svg`
  - non-SVG icons: `examples/services/<fullpath service>/<service>-icons.archol`
- `documents/` e `processes/` são organizacionais, não namespaces; `icons/` é contêiner e não entra na referência.
- Arquivo compartilhado: `<service>-<tipo-no-plural>.archol`, com **um único tipo de artefato por arquivo** (`shapes`, `forms`, `views`, `messages`, `policies`, `decisions`, `integrations`, `parameters`, `secrets`, `organograms`, `icons`); a exceção é `-organograms.archol`, que contém o organogram e as roles, com o organogram **antes** das roles. Declara **zero** principais.
- `*-documents.archol`, `*-processes.archol` e `*-reports.archol` deixaram de existir; relatório é `view`; agendamento é `start { schedule: }` no arquivo do processo.
- `bpcms.org/std` segue a regra de service em `examples/services/bpcms.org/std/`: `std-service.archol` + `std-shapes.archol` (compartilhado).

**Regra do compilador:** o `check` percorre todos os `.archol` de serviços e aplicações; `application` e `service` também são principais.

- `E_FILE_NAME`: nome fora do padrão (incluindo sufixo compartilhado fora da §30.2) ou `<nome>`/`<tipo>` divergente do principal.
- `E_FILE_PATH`: arquivo fora da pasta esperada, full path diferente da URI, documento fora de `documents/` ou processo fora de `processes/`.
- `E_FILE_PRINCIPAL`: arquivo de contexto principal (app/service/document/process) sem exatamente um principal.
- `E_FILE_PRINCIPAL_FIRST`: principal não é o primeiro artefato do arquivo.
- `E_FILE_SHARED`: arquivo compartilhado declara `application`, `service`, `document` ou `process`.
- `E_FILE_SHARED_TYPE`: arquivo compartilhado contém tipo diferente do sufixo (ex.: `shape` em `-views.archol`; `parameter` em `-secrets.archol`; `role` fora de `-organograms.archol`).
- `E_FILE_ORGANOGRAM_FIRST`: em `-organograms.archol`, o `organogram` não é o primeiro artefato (ou não existe).
- `E_FILE_ICON_PATH`: SVG fora de `icons/` ou namespace de ícone divergente.

**Critério de dono:** shape → documento/processo que o usa como tipo; form → processo da task (senão documento); view → `document`/`source`/`sources` de um documento ou `process:`; message → documento do `about`; policy → alvo do `appliesTo`; decision/integration → processo que usa; parameter/secret, role/organogram e icon → service-wide. Detalhe na §30 do `DSL_REFERENCE.md`.

**Resultado da migração:**
- 19 serviços de negócio (+ `std`) em full path (`examples/services/example.com/services/<service>/` e `examples/services/bpcms.org/std/`).
- 197 → 183 arquivos `.archol` de serviço; 5 aplicações em `examples/applications/example.com/app/<app>/`.
- 90 arquivos de contexto: 55 em `documents/` e 35 em `processes/`; 20 arquivos de service e 72 arquivos compartilhados de negócio + `std-shapes.archol` na raiz do serviço.
- 682 blocos de artefato preservados exatamente; 8 SVGs movidos para `<service>/icons/<namespace>/`.
- 19 arquivos `*-organograms` mantêm organogram + roles no mesmo arquivo, com o organogram primeiro; `notifications-parameters` separado em `-parameters` + `-secrets`; `payments-parameters` renomeado para `payments-secrets`.
- Billing: 12 → 17 arquivos, agora em `examples/services/example.com/services/billing/` com `documents/`, `processes/` e `icons/`.

**Arquivos afetados:** `DSL_REFERENCE.md` (§2, §3, §21, §23, §26, §28.1, §29, §30 e Mapa), `AGENTS.md`, `BRAINSTORM.md` (B-47/B-60/B-61/D-037/D-050/D-051), `DSL_EXAMPLES.md` (SVG), `REVIEW_CHECKLIST.md`, `examples/**`.

**Pendências:**
- [ ] Implementar `E_FILE_NAME`/`E_FILE_PATH`/`E_FILE_PRINCIPAL`/`E_FILE_PRINCIPAL_FIRST`/`E_FILE_SHARED`/`E_FILE_SHARED_TYPE`/`E_FILE_ORGANOGRAM_FIRST`/`E_FILE_ICON_PATH` no `check` real (a DSL ainda não tem parser executável).
- [ ] Revisar os casos de artefato sem dono único que ficaram em arquivo compartilhado (`Email`/`Phone` em Citizen, `TrackingEvent` em Shipping, `SupportLabGateway` em Laboratory, `ScaleDevice` em Shipping).

## Process — revisão e migração — 2026-11-17 (executada por solicitação direta)

**Pedido:** revisar o Process como código TS-like, assíncrono implícito, com variáveis duráveis da instância (referências a documentos), sem `for`, sem pool/lane e sem artefato `task`; forms/manuais/mensagens como artefatos separados, chamados pelo processo (ex.: `PaymentRetry.show({ session })`).

**Decisões aplicadas:**
- A seção 4 do `DSL_REFERENCE.md` foi reescrita; não há versão paralela. O workspace `examples/` é a referência executável.
- `for` removido; o processo é `process <Nome>(args)` com gatilho (`on`, `by`, `schedule`+`fanOut`, `message`, `called`) e `key` para correlação.
- Pool/lane removidos; `roles` no form/manual definem quem executa. Views de inbox usam `role`/`roles`.
- `task` removido; o processo chama operação (`action`, `call`, `compute`, `decision`) direto ou form/view/manual/job/process (`show`/`run`).
- Sintaxe TS-like: `let`/`const`, `try/catch`, `loop`/`forEach`, `if`, `return`, objeto de argumentos.
- **Variáveis somente no topo do processo**; nenhuma declaração dentro de `try`/`catch`/`if`/`loop`/`parallel`/`forEach`.
- **Ends ao final do processo**; fluxos terminam com `return <End>`.
- `owner` e `timezone` não existem em artefatos: origem é inferida pela URI; fuso é do tenant/scheduler.
- Assíncrono implícito; `parallel`, `wait`, `send`/`receive`, `run` e `cancel on` fazem parte da sintaxe.
- Variáveis duráveis por instância; documentos entram por referência.
- Decisão humana é lida no **estado do documento** após o form, não em `returns`.
- `timeout` é prazo duro; `sla` é alvo brando.
- Processo é a autoridade de escrita; roles do form controlam a execução humana.
- `milestone` e `discretionary` (CMMN) ficaram adiados.

**Escopo:** 35 processos de `examples/` migrados; `DSL_REFERENCE.md` §4 reescrita. `DSL_EXAMPLES.md` e `BRAINSTORM.md` ainda precisam de uma passada de consistência.

**Arquivos:** `DSL_REFERENCE.md`, `DSL_EXAMPLES.md` (aviso/Process), `examples/**`, `REVIEW_CHECKLIST.md`.

**Em aberto:** `key`/`search`; CMMN; runtime durável; semântica de `parallel`; `job` com semântica própria; `report` como `view`; form/view multi-documento; permissões; versionamento; `reintroduce` em processo-código; validações/erros.

**Consolidação pendente:** atualizar os exemplos de Process de `DSL_EXAMPLES.md` (P.1–P.24) e `BRAINSTORM.md`; ajustar §7.4/§30 para views/role e novos artefatos (`manual`, `send`/`receive`, `cancel on`); implementar o `check` (`E_PROCESS_START`, `E_CASE_KEY`, `E_BLOCK_DECLARATION`, `E_VAR_SCOPE`, `E_END_ORDER`, `E_CATCH_UNKNOWN` etc.).

## View e Message como contexto principal — 2026-11-18 (aprovada por solicitação direta do usuário)

**Pedido:** elevar `view` e `message` a contexto principal, para que cada uma tenha arquivo próprio e fique organizada em pastas, como `document` e `process`.

**Por que:** view e message têm identidade e contrato próprios — nome referenciável por outros artefatos e serviços, `roles`, fontes (`document`/`source`/`sources`/`process:`) e `payload`/`about`. Manter no arquivo do documento/processo misturava três contextos no mesmo arquivo e deixava o documento dominado por telas e mensagens.

**Decisão aplicada:**
- Contextos principais: `application`, `service`, `document`, `process`, `view`, `message`.
- Caminhos:
  - view: `examples/services/<fullpath service>/views/<service>-<nome>-view.archol`
  - message: `examples/services/<fullpath service>/messages/<service>-<nome>-message.archol`
- `views/` e `messages/` são pastas organizacionais, não namespaces: a view continua `InvoiceTable`, não `views.InvoiceTable`.
- Cada view e cada message é o primeiro e único principal do seu arquivo.
- `-views.archol` e `-messages.archol` não existem; view/message nunca vão para arquivo compartilhado.
- O vínculo fica no contrato (`document`/`source`/`sources`/`process:`, `about`), não no arquivo do dono.
- `report` é `view type: report` e fica em `views/`.

**Regra do compilador (§30.3):**
- `E_FILE_PRINCIPAL` inclui `view`/`message`; `E_FILE_PRINCIPAL_FIRST` continua valendo.
- `E_FILE_PATH`: view fora de `views/` ou message fora de `messages/` é erro.
- `E_FILE_SHARED`: arquivo compartilhado não declara `view`/`message`.
- `E_FILE_SHARED_TYPE`: exemplo atualizado (`view` em `-shapes.archol`, já que `-views.archol` não existe).

**Resultado da migração:**
- 134 views + 24 messages → 158 arquivos novos, um artefato por arquivo.
- 19 `-views.archol` removidos (17 com views e 2 que restaram só com comentário de arquivo).
- 62 arquivos de documento/processo perderam views/messages e mantiveram shapes/forms/policies/decisions/integrations.
- 741 blocos de artefato antes e depois, sem alteração de conteúdo; nenhuma view/message restante fora de `views/`/`messages/`.
- `DSL_EXAMPLES.md` não mudou: ele é syntax-only e não referencia a organização em arquivos.

**Arquivos afetados:** `DSL_REFERENCE.md` (§3, §21, §26, §30), `AGENTS.md`, `BRAINSTORM.md` (D-050, D-052, B-60, B-62), `REVIEW_CHECKLIST.md`, `examples/**`.

## Linguagem Archol, compilador Rust, LSP e plugin VSCode — 2026-11-18 (executada por solicitação direta)

**Pedido:** dar nome à DSL (**Archol**), trocar a extensão dos fontes para `.archol`, criar compilador com LSP em Rust, AST JSON no estilo TypeScript, check que valide valor de enum (não só tipo), link que responda perguntas invertidas (`role -> artefatos`), plugin VSCode com hover/completion/explorador de AST, tudo em `archol/`, código simples e testado. Versão inicial: sem releases, tags ou publicação.

**Decisões aplicadas:**
- Linguagem **Archol**, extensão **`.archol`**; o repositório mantém `examples/` como workspace (`BPCMS_WORKSPACE`).
- Workspace Rust em `archol/`: `archol-core` (tokens, lexer, AST JSON, parser), `archol-analysis` (check + link), `archol-lsp` (LSP stdio) e `archol-cli` (binário `archol`).
- Dependência única: `serde`/`serde_json`; LSP síncrono com JSON-RPC próprio (sem async/runtime) e plugin VSCode sem dependências npm.
- AST: todo nó tem `kind` + `loc` (bytes); `Artifact`, `PropertyAssignment`, `Block`, `EnumValueDeclaration`, `IfStatement`, `Call`, `Object` (`typeRef`) etc.
- Check: `const a: TaxRegimeCode = SIMPLES` e `regime == SIMPLES` validam contra os `value` do shape; também `E_UNKNOWN_TYPE`, `E_UNKNOWN_ROLE`, `E_UNKNOWN_REF` e `E_DUP_ARTIFACT`.
- Link: `roles[].usedBy` (role → artefatos), `enums[].values`, `artifacts[].refs` e `services[]`.
- LSP: hover (tipo, valor de enum com títulos, link para o AST), completion de valores de enum/papéis, definição, referências, símbolos, folding e requests `archol/ast` + `archol/links`.
- Plugin VSCode: cliente LSP mínimo, explorador de AST navegável (clique → código), status bar, comandos `archol.showAst`/`showLinks`/`restartServer`, gramática TextMate e snippets.
- Versão inicial: `publish = false` em todas as crates, sem tags/releases/versões publicadas.

**Migração e correções:**
- 330 arquivos de `examples/` renomeados de `.bpcms` para `.archol`; referências em `DSL_REFERENCE.md`, `DSL_EXAMPLES.md`, `AGENTS.md`, `BRAINSTORM.md` e `REVIEW_CHECKLIST.md` atualizadas.
- `examples/services/bpcms.org/std/std-shapes.archol`: shape `Currency` estava com a chave de fechamento fora de ordem (fechava antes de `write`/`read`/`parse`/`format`); rebalanceado para o corpus parsear sem erro.
- Fixture `archol/examples/demo.archol` para o explorador.

**Resultados:**
- 330 arquivos reais parseados com **0 erros** e 805 artefatos; `check` no workspace: **0 erros**.
- Índices gerados: 72 enums, 54 roles, 20 serviços.
- 35 testes Rust verdes (`cargo test`) e 29 testes do plugin verdes (`node --test`).


**Auditoria adversarial (2026-11-18):** um agente revisor testou entradas hostis e apontou seis bloqueadores, todos corrigidos:

- string terminando em `\` não estoura mais o span no lexer (`tokens` sai 1, não 101);
- aninhamento profundo gera `E_PARSE_DEPTH` (limite 200) em vez de stack overflow;
- `Alias.Tipo` resolve enum via `imports` (falso negativo corrigido);
- enum homônimo por serviço não colide mais no índice (link mostra os 3 `PaymentMethod`);
- comparação com literal à esquerda aponta para o literal, não para a variável;
- `check` reporta `E_FILE_READ` para arquivo ilegível/UTF-8 inválido em vez de ignorar;
- `check <arquivo>` e ordem de `walk` corrigidos; LSP: hover de parâmetro/valor, `definition` de valor de enum, highlight só no documento, folding sem duplicata e sync documentado como Full.

Testes passaram de 21 para **30 Rust** (+9: profundidade, escape, alias, homônimo, span do literal, CLI) e **27 JavaScript**; `clippy` e `fmt` limpos; corpus continua com 0 erros.


**Correções de navegação (2026-11-18, 2ª rodada):**
- Explorador de AST: clique reutiliza o editor aberto (mesma `viewColumn`) e seleciona `loc.start`..`loc.end`; o webview passou a enviar `start`/`end` e a extensão converte bytes UTF-8 → offsets UTF-16 antes de selecionar.
- Definition: resolução semântica própria (`resolve.rs`) — parâmetro/`const`, campo de documento (`invoice.status` → `status: InvoiceStatus` em `Invoice`), valor de enum em comparação tipada (`invoice.status == Overdue` → `value Overdue`, não o `icon Overdue`), sujeito implícito de `fanOut`, e contexto de `icon:` para ícones.
- `Cmd+Click`/`Ctrl+Click`/`F12` usam o mesmo provider; documentado que no macOS é `Cmd`.
- Testes: 35 Rust (5 novos de resolução) e 29 JavaScript (2 de conversão de span); smoke do LSP cobre as definições reais do processo de cobrança.

**Idioma do compilador e do plugin (2026-11-18):** por pedido direto, todo o `archol/` passou a ser escrito em **inglês** — comentários, diagnósticos, saída da CLI, hover/LSP, UI do plugin, testes, scripts e docs (`archol/README.md`, `archol/docs/*`). Ficaram de fora apenas dados de exemplo Archol com `pt-BR` e fixtures Unicode (`João`, `ação`, `ç`) usados para testar UTF-8. A regra está registrada no `AGENTS.md`.

**Arquivos afetados:** `archol/**` (novo), `examples/**` (extensão e `std-shapes`), `DSL_REFERENCE.md`, `DSL_EXAMPLES.md`, `AGENTS.md`, `BRAINSTORM.md` (D-050…D-053, B-60…B-63), `REVIEW_CHECKLIST.md`.

## Identificadores, referências e compilação por aplicação — 2026-11-18 (executada por solicitação direta)

**Pedido:** o AST não deve tratar valores de enum como `Identifier`; precisa haver ligação tipada com a origem (`Overdue` → `value Overdue` → `InvoiceStatus` → serviço), referências em várias direções, IDs estáveis no lock da aplicação e compilação somente a partir de uma Application; o plugin deve saber a aplicação atual e o LSP tratar como comentário os arquivos fora da closure.

**Decisões aplicadas:**
- Lock na mesma pasta/nome da aplicação, só trocando a extensão (`X-application.archol` → `X-application.lock`), **CSV `id,path`**, IDs atribuídos no fim **somente a identificadores usados**, priorizando os **mais referenciados** (empate por path), sequenciais `a`, `b`, …, `z`, `aa`, … sem duplicidade, `path` seguro (`<uri>#<kind>/<Nome>[...]`), poda de não usados e reset em `a` ao apagar o arquivo. Linhas com `*` no fim do path são fixas: ficam no topo, mantêm o ID e nunca são podadas/renumeradas. Duplicatas de merge são responsabilidade de quem faz o merge; o loader recusa e pede correção.
- AST anotado: declarações com `id`; referências com `ref`/`refKind` (`enumValue`, `document`, `role`, ...); `fieldRef` em acesso a campo; `typeRefId` em tipo; `arg` de `fanOut` e comparações resolvidos. Listas de usos ficam no índice semântico (sem ciclos nem JSON gigante).
- Compilação por **Application**: closure de serviços via `services` + `imports` transitivos; `check`, `lock`, `links` e LSP exigem aplicação (`--app`/`archol.application`); arquivos fora da closure são inertes.
- Plugin: setting `archol.application`, comandos **Select Application** e **Update Identifier Lock**, status bar com a app, `initializationOptions.application`, banner “commented out” e navegação **Go to Declaration** / **Find Usages**.
- LSP: `archol/applications`, `archol/setApplication`, `archol/status`, `archol/declaration`, `archol/usages`, `archol/lock`; `archol/ast` com `inApplication`/`commentedOut`; diagnósticos só da closure.

**Resultado:**
- `archol check examples --app example.com/app/backoffice-loja` → **172 arquivos, 0 erros** (closure de 11 serviços).
- `archol lock` testado em workspace temporário: cria 5 IDs, mantém IDs ao rodar de novo e poda `value/Cash` após remoção.
- Navegação LSP: `Overdue` → declaração em `billing-invoice-document.archol`; `archol/usages` com 3 ocorrências; arquivo de outra app retorna `commentedOut: true`.
- **72 testes Rust** e **45 JavaScript** verdes; `clippy`/`fmt` limpos.
- Aliases de aplicação/import (`as Identity`) recebem `ref` do serviço; hover/definition levam ao `identity-service.archol`. Documentos fora da closure recebem `W_OUTSIDE_APPLICATION` e o editor os exibe em itálico/apagado.
- Menu de contexto: **Archol: Use as Current Application** no editor (só aparece em arquivos com declaração `application`, via context key `archol.isApplication`) e no Explorer para `.archol` (valida a declaração e recusa não-aplicações).
- Lock atualizado automaticamente ao selecionar a app (servidor) e, após mudanças em `.archol`, com debounce de 10s (`archol.lockDebounceMs`); o plugin só atualiza se o arquivo alterado estiver na closure (`archol/inApplication`).
- Popover de hover mostra o ID estável do identificador sob o cursor (declaração ou referência), permitindo correlacionar com o AST/lock.
- Right-click em arquivo de aplicação (editor/Explorer) define a app atual; IDs manuais no lock ficam no topo e sobrevivem à poda.
- `archol format`/`archol lint` e formatação LSP; pendências `TODO`/`FIXME`/`REVIEW`/`OPTIMIZE`/`HACK` no AST (`SourceFile.pending`) com due date, erro para sem data/passada e warning em até 15 dias. Pendências são metadados: aparecem em `SourceFile.pending`, mas não recebem ID nem linha no lock; o explorador as intercala na posição do código. Comentários dos 330 arquivos de exemplo marcados com `REVIEW [2026-12-15]` (13.064 comentários).

**Arquivos afetados:** `archol/crates/archol-core` (AST/parser), `archol/crates/archol-analysis` (`ids`, `application`, `annotate`, lib), `archol/crates/archol-cli`, `archol/crates/archol-lsp`, `archol/editors/vscode`, `archol/scripts/smoke_lsp.py`, `archol/docs/*`, `archol/README.md`, `DSL_REFERENCE.md` (§1.6/§2), `AGENTS.md`, `REVIEW_CHECKLIST.md`.

## Rodada I18N + remoção de `version` — 2026-11-18 (executada por solicitação direta)

**Pedido:** remover o atributo `version` dos artefatos; `title` e `goal` do processo passam a ser `I18N`; revisar todos os `I18N`. O valor `I18N` pode ser string (primeiro locale de `locales` do serviço) ou objeto de traduções; o compilador ignora locales ausentes nos `locales` da aplicação e envia ao AST só os existentes; o tipo precisa ficar explícito para o construtor.

**Decisões aplicadas:**

- `version` foi removido de **todos** os artefatos (`application`, `service`, `process`, `document`, `builder`). A versão publicada vive no seletor de `release`/`rollback` (`@...`) e, por serviço, no lock do app.
- `service` ganhou `locales` **obrigatório** (ex.: `locales: [ "pt-BR", "en" ]`), com **pelo menos um** idioma; o primeiro é o default do `I18N` em forma de string. Serviço sem `locales` ou com lista vazia é `E_LOCALES_REQUIRED`. Exemplo real no serviço Billing.
- `I18N` cobre `title` (todos os contextos), `goal` (process), `instructions` (manual) e `short`/`tooltip`.
- Forma string: o compilador expande para o mapa canônico no primeiro locale do serviço (ou default da app, se o serviço não declarar `locales`).
- Forma objeto: o compilador mantém apenas locales cuja língua exista em `Application.locales` (comparação exata ou por idioma; `en` sobrevive a `en-US`; `fr` cai).
- AST: o valor é um `ShapeValue` com `typeRef = "std.bpcms.i18n"` e `value` como mapa de locales. O builder recebe o locale explícito de cada texto.
- Nomenclatura da documentação normalizada para `I18N`.

**Implementação:** `archol-analysis/src/i18n.rs` (novo), chamada em `analyze_application` antes do check; `ApplicationInfo.locales`; campo `type` no `PropertyAssignment` do AST; `locales` documentado em `DSL_REFERENCE.md` §1.2/§1.9/§2/§3/§4.2/§5.1; exemplos e `DSL_EXAMPLES.md` atualizados.

**Testes:** `cargo test` verde, incluindo o teste `i18n_properties_are_typed_and_filtered_by_application_locales`; `cargo run -q --bin archol -- ast ... --app example.com/app/backoffice-loja` mostra `"kind": "ShapeValue"` e `"typeRef": "std.bpcms.i18n"`.

**Em aberto:** validação de `locales` incompletos (`E_TITLE_MISSING`), diagnóstico para string sem locale conhecido e política estrita do builder continuam responsabilidade do construtor/check futuro.

## Rodada literais de Shape — 2026-11-18 (executada por solicitação direta)

**Pedido:** regras para literais; strings como TypeScript com preferência por aspas duplas no format/lint; Duration entre aspas; toda string atribuída a tipo Shape deve usar o `parse` em tempo de compilação; AST com `ShapeLiteral` (`shape`, `raw`, `value`, `unit`).

**Decisões aplicadas:**
- Strings aceitam aspas simples e duplas; `archol format` converte para duplas quando não há aspas duplas/escape no conteúdo; `lint` emite `W_STRING_QUOTES` enquanto estiver em aspas simples.
- `sla` é `std.Duration` e exige literal entre aspas: `sla: "30d"`. Sem aspas: `E_SHAPE_LITERAL_QUOTED`.
- Strings atribuídas a tipos Shape (propriedades tipadas, `let`/`const`, campos de objeto tipado, `Parameter.default`/`min`/`max` e defaults de field) são parseadas em tempo de compilação; falha vira `E_SHAPE_LITERAL`.
- `std.Money` vira `{ raw, currency }`; `std.Mass`/`std.Duration` normalizam para a unidade-base; shapes com `normalize`/`validate` (ex.: `Email`) são normalizados/validados.
- AST: `ShapeLiteral` com `shape`, `raw`, `value` normalizado e `unit` quando quantidade. Ex.: `"30d"` → `value: 2592000`, `unit: "s"`.
- `std.Duration` já existia em `std-shapes.archol`; não foi criado tipo novo.

**Implementação:** lexer com aspas simples; `W_STRING_QUOTES`; `format` com `quote_edits`; `shape_literal.rs` no analysis; `Expr::ShapeLiteral`/`ShapeValue` no AST.

**Testes:** parser (aspas simples), pending (`format`), analysis (Duration, video, inválido, sem aspas), corpus e as 5 aplicações verdes.

## Rodada metadados `source`/`owner` — 2026-11-19 (executada por solicitação direta)

**Pedido:** remover os atributos `source` e `owner` dos artefatos.

**Decisões aplicadas:**
- Removidos `owner:` de `service`, `document` e `decision` (78 linhas nos exemplos).
- Removidos `source:` de metadados de `application`, `decision` e `evidence` (10 linhas).
- Origem é inferida pela URI: hosts Git conhecidos viram repositório; o resto é registry BPCMS. Não há atributo para forçar `source`.
- **Mantidos** os `source` estruturais, que não são metadado: `view.source` (documento da tela), `source: device ...` (campo lido de dispositivo), `source: config|secret` em Parameter e o campo `source` de dados.
- `owner` não existe mais como metadado; governança fica em release/lock.

**Impacto:** exemplos, `DSL_REFERENCE.md` (§1.2/§1.3/§3/§5.1/§4.17/§9/§10), `DSL_EXAMPLES.md`, `BRAINSTORM.md` (B-27 e resumos) e `REVIEW_CHECKLIST.md`. Os snapshots `.expected-ast.yml` das 5 aplicações foram regenerados.

## Rodada `span` → `loc` — 2026-11-19 (executada por solicitação direta)

**Pedido:** renomear a propriedade `span` do AST para `loc`; prefixos viram `*Loc` (`keywordSpan` → `keywordLoc`); ajustar export e visualizador, que deve colocar propriedades de loc ao final.

**Decisões aplicadas:**
- AST serializa `loc` em todos os nós e `keywordLoc` em `Pending`.
- `archol ast --no-loc` substitui `--no-span` e remove `loc`/`keywordLoc`.
- O visualizador JSON lê `loc` e ordena propriedades terminadas em `loc`/`Loc` por último.
- Teste dedicado `ast_locs.rs`; snapshots usam `--no-loc`.
- Campos Rust internos continuam `span`; a mudança é no contrato JSON/YAML.

## Rodada tipo I18N — 2026-11-19 (executada por solicitação direta)

- O valor de título/goal/instructions/short/tooltip passou a ser um token `ShapeValue` com `typeRef: "std.bpcms.i18n"`.
- Parser e passe I18N atualizados; testes e snapshots das 5 aplicações regenerados.
- O nome curto I18N permanece apenas como conceito/documentação; o contrato do AST usa `std.bpcms.i18n`.

## Rodada organograma — 2026-11-19 (executada por solicitação direta)

**Pedido:** repensar a duplicação entre `hierarchy` e `manages`; implementar a alternativa A.

**Decisão aplicada:**
- `hierarchy` é a única fonte da árvore de reporte.
- `manages` foi removido de exemplos, roles e organogramas.
- A autoridade de `assignRole` é derivada dos **descendentes transitivos** da hierarquia, dentro dos `scopes` do organograma.
- Auto-atribuição continua proibida.
- `delegates to` continua separado e aponta para uma role abaixo na hierarquia.
- `assigns` para autoridade fora da linha hierárquica fica como evolução futura, não implementada nesta versão.
- `fourEyes`, `segregation` e `signature` não mudaram.

**Impacto:** 17 organogramas de exemplo (linhas `manages` + comentários), `DSL_REFERENCE.md`, `DSL_EXAMPLES.md`, `REVIEW_CHECKLIST.md` e o keyword do plugin (`archol.tmLanguage.json`).

## Rodada AST tipado + hierarchy — 2026-11-19 (executada por solicitação direta)

**Pedido:** cada artefato no AST deve ter seu próprio `kind` (sem `artifactKind`) e
`hierarchy { A > B > C }` deve virar um nó estruturado; toda referência a role
deve carregar o necessário para o construtor chegar rápido à role.

**Decisão aplicada:**
- `kind` do artefato passa a ser o tipo: `ArtifactApplication`, `ArtifactService`,
  `ArtifactDocument`, `ArtifactProcess`, `ArtifactView`, `ArtifactForm`,
  `ArtifactMessage`, `ArtifactShape`, `ArtifactRole`, `ArtifactOrganogram`,
  `ArtifactPolicy`, `ArtifactDecision`, `ArtifactIntegration`,
  `ArtifactParameter`, `ArtifactSecret`, `ArtifactIcon`, `ArtifactManual`,
  `ArtifactError`. `artifactKind` deixou de existir no AST.
- `hierarchy` deixa de ser `Block`/`Binary`: vira `Hierarchy` → `HierarchyChain`
  → `RoleRef`, com `ref` (id do lock), `refKind: "role"`, `service`, `path` e
  `line` da declaração após a análise.
- `ArtifactOrganogram` anotado ganha `roles` derivado (`OrganogramRole`):
  `manages` transitivo, `inherits`, `permissions`, `title` e ponteiros
  `RolePointer` com `ref`/`refKind`/`service`/`path`/`line`.
- `ArtifactRole` continua sendo artefato de topo; a projeção não é fonte.
- Check novo: `E_UNKNOWN_ROLE`, `E_HIERARCHY_CYCLE`, `E_HIERARCHY_DUPLICATE`;
  parser: `E_HIERARCHY_ROLE`, `E_HIERARCHY_CHAIN`, `E_HIERARCHY_TRAILING`.

**Impacto:** `archol-core` (AST/parser), `archol-analysis` (annotate/check/link/
organograma), LSP, CLI/AST explorer, docs (`ast.md`, `compiler.md`, `README.md`),
`DSL_REFERENCE.md` e snapshots das 5 aplicações regenerados.


## Rodada permissões + índice do service — 2026-11-21 (executada por solicitação direta)

**Pedido:** permissões como artefato auto-criado dentro do serviço; AST do serviço
com a relação de todos os seus artefatos; identificadores e referências corretos.

**Decisão aplicada:**
- `ArtifactService.permissions` contém um `ArtifactPermission` por
  `namespace.action` único usado pelas roles do serviço; `namespace` é tudo
  menos o último segmento e `action` é o último.
- Cada permissão ganha id no lock (`<service>#permission/<nome>`) e as entradas
  `permissions` das roles deixam de ser `MemberAccess`/`Identifier` e passam a
  `Reference` com `refKind: "permission"` e `ref`.
- `ArtifactService.artifacts` indexa todos os artefatos do serviço com
  `ArtifactPointer` (`artifact` tipado, `name`, `ref`, `path`, `line`),
  incluindo as permissões auto-criadas; o próprio service não entra na lista.
- `OrganogramRole.permissions` usa `PermissionPointer` (`name`, `ref`,
  `refKind`, `line`).
- Sem permissões duplicadas por serviço; a primeira ocorrência define o `loc` e
  a linha do artefato auto-criado.

**Impacto:** `archol-core` (AST), `archol-analysis` (annotate/service/organogram),
snapshots, docs (`ast.md`, `compiler.md`, `README.md`), `DSL_REFERENCE.md` e
testes de análise.

## Rodada std fora do lock + alias de tipos — 2026-11-23 (executada por solicitação direta)

**Pedido:** o lock deve conter apenas identificadores realmente usados; o
`bpcms.org/std` não pode poluir o lock com shapes que a aplicação não usa, e
referências qualificadas (`std.Money`, `Citizen.Email`) precisam resolver de
verdade.

**Decisão aplicada:**
- `bpcms.org/std` continua na closure para resolução/validação, mas não é
  declarado como raiz nem entra no `scope` de IDs: nenhum `std#service` ou
  `std#shape/*` é gravado no lock (§29/B-53). `typeRef` qualificado permanece no
  AST sem `typeRefId`.
- `Scope::type_artifact` resolve tipos qualificados pelo mapa de `imports` do
  serviço (`Alias.Tipo`), inclusive tipos de outros serviços de negócio; antes
  `base_type` não removia o alias e nenhum `typeRefId` era gerado nesses casos.
- Locks das 5 aplicações regenerados. O backoffice-loja perdeu 58 linhas de
  std sem renumerar os demais IDs; central-de-compras, claims-desk,
  loja-do-joao e rota-certa também receberam as permissões que estavam
  pendentes desde a rodada anterior.
- Snapshot `backoffice-loja.expected-ast.yml` atualizado para os IDs vigentes
  (já estava defasado em relação ao lock do HEAD).

**Impacto:** `archol-analysis` (`annotate.rs`, `lib.rs`), testes de análise,
`examples/applications/**/*.lock`, `backoffice-loja.expected-ast.yml`,
`archol/docs/identifiers.md` e `DSL_REFERENCE.md`.
