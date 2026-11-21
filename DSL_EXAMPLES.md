# BPCMS — Exemplos de DSL

*Sintaxe ilustrativa do Archol (Process + Document).* Não é gramática final nem código executável. O objetivo é **validar conceitos, ergonomia e limites** antes de fechar PRD/SPECS/PLAN. A sintaxe e o funcionamento de cada cláusula estão em [DSL_REFERENCE.md](DSL_REFERENCE.md).

A aplicação é orientada a **Process** (trabalho) e **Document** (estado). O processo orquestra quem faz o quê; o documento guarda o dado durável; o **Form** é o contrato da tarefa humana; a **View** é tela de leitura; a **Message** é contrato entre participantes. `job` e `report` não são artefatos (agendamento é `schedule`; relatório é `view type: report`), e o ciclo de vida do documento vive no `transition` do `value` e no `to:` da action.

Este documento cobre o catálogo de artefatos. Os **35 processos completos** — o teste de aceitação do modelo — estão em [`examples/`](examples/); os recortes de Process abaixo mostram a sintaxe. Alguns exemplos compartilham o mesmo domínio de propósito (composição e reuso); outros mudam de domínio para testar generalidade.

## Como ler

* Cada bloco `bpcms` é um exemplo independente.
* `//` é comentário.
* A linha *Valida:* diz o que aquele exemplo quer provocar.
* Nomes de artefatos são em inglês (`application`, `document`, `shape`); nomes de negócio podem ser em português (`Pedido`, `Remessa`).
* Nada aqui substitui a baseline do [BRAINSTORM.md](BRAINSTORM.md); os exemplos são um **teste de sintaxe e modelagem**.
* Fragmentos que usam `std.*` assumem `imports { service "bpcms.org/std" as std }` no serviço; o import não é repetido em todo bloco.
* Os exemplos são **independentes entre si e das pastas `examples/`**: um bloco pode citar artefatos que não existem no repositório de exemplos (é teste de sintaxe, não um projeto compilável).
* **Application e Service usam URI web sem protocolo** como identificador: `"example.com/app/..."` e `"example.com/services/..."`. A URL `http://<uri>/index.html` é a página de download (app) ou de documentação (serviço); em produção, `https://`.

## Convenções usadas

| Construção | Forma | Observação |
| :--- | :--- | :--- |
| Artefato | `application "example.com/app/x" { ... }` | Application/Service usam URI; os demais usam nome simples |
| Campo | `nome: Shape` | `Shape` é o tipo de dado, não uma forma visual |
| Campo com parâmetros | `weight: std.Mass { unit: kg, min: 0 }` | o campo só passa parâmetros simples |
| Lista | `[A, B, C]` | ordem preservada quando importa |
| Opcional | `T` | ausência é diferente de vazio |
| Referência | `Document`, `Form`, `View`, `Role` | por nome; o `check` valida existência e ciclo |
| Ação | `issue()` | comando tipado; pode declarar `returns:` |
| Transição | `transition Issued { ... }` no `value` + `to: status.Issued` na action | o workflow define a aresta e o evento; a action só aponta o destino |
| Índice | `byStatus: [status, createdAt] range` | `unique`, `range`, `search`, `hash` |
| Regra | `total == subtotal + tax` | expressão segura, sem código arbitrário |
| Configuração | `env.ERP_URL`, `secret.STRIPE_KEY` | ambiente e segredo são resolvidos na promoção |
| Processo | `process Nome(args)` + gatilho + `key` | código estruturado; sem arestas |
| Task | `Form.show({…})`, `Manual.show({…})`, `Processo.run({…})` | o tipo é inferido do contrato; não existe `type:` |
| Executor | `roles: [Role]` no form/manual | sem lane/pool; roles controlam a task humana |
| Subpasso | `Processo.run({…})` | sempre outra instância de processo |
| Agendamento | `schedule` + `fanOut` | uma instância por documento |
| Relatório | `view type: report` | agregação é projeção; documento derivado é estado |
| Origem | URI (`github.com/...` = repositório; resto = registry) | inferida; não há atributo `source` |
| Versão | `"<uri>"@1.2.3` / `@tag("v1.2.3")` / `@commit("9f3a1c")` | Seletor de release/rollback; nenhum artefato declara `version:` no corpo |
| String | `"texto"` ou `'texto'` | Aspas simples/duplas; `format` prefere duplas (`W_STRING_QUOTES` no lint) |
| Shape literal | `sla: "30d"` | Parse do Shape em tempo de compilação; AST `ShapeLiteral` com `raw`, `value` e `unit` |
| I18N | `title: "..."` ou `title: { pt-BR: "...", en: "..." }` | Tipo `I18N`; string = primeiro locale do serviço; o compilador filtra pelos locales da app |
| Idiomas | `locales: [ "pt-BR", "en" ]` | App/service; no serviço é obrigatório (≥ 1); o primeiro é o default |
| Página canônica | `http://<uri>/index.html` | app = download por SO; service = docs/integração |
| Evento | `transition Paid { title; payload }` no `value` | evento do workflow definido no workflow; `initial ... with transition X` é o evento de criação; `events { }` só para evento sem transição |
| Permissão | `roles: [Manager]` na action; `roles` no form | autoridade do documento + execução da task |

## Cobertura

O catálogo cobre Application, Service, Form, Message, Document, Shape, View, Role/Organogram, Evidência/Conversação/Decisão, plataforma e operação. Os 35 processos completos estão em [`examples/`](examples/).

---

# Núcleo

> **Páginas canônicas.** `http://example.com/app/loja-do-joao/index.html` é a página de **download**, com opções por SO. `http://example.com/services/catalog/index.html` é a página de **documentação para integração por API**, com opções de nuvem, protocolos (sockets/REST/RPC) e iframe para microfrontend, conforme o builder. Em produção, use `https://`.

> **Duas origens.** `example.com/...` é registry BPCMS (páginas `/index.html`). `github.com/...` ou `gitlab.com/...` é repositório Git: a documentação é o **README** e o código é baixado do projeto (clone/zip/release); a versão pode ser `commit(...)` ou `tag(...)`. `source` deixa isso explícito; a versão publicada vem do seletor de release.

## Application

### A.1 Loja virtual completa

```archol
application "example.com/app/loja-do-joao" {
  title: {
    pt-BR: "Loja do João"
    en-US: "João's Store"
  }
  description: "Loja virtual de ponta a ponta"

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
    // DefaultCurrency é um Parameter do serviço Catalog
    set Catalog.DefaultCurrency = "BRL"
  }
}
```

*Valida: só entry points; `cart`, `payment`, `shipping` e `auth` entram implicitamente. `main` faz `set` em um Parameter **do serviço Catalog**; a app não declara artefatos.*

### A.2 Clínica com prontuário e faturamento

```archol
application "example.com/app/clinica-vida" {
  title: { pt-BR: "Clínica Vida" }

  services {
    "example.com/services/encounters"
    "example.com/services/billing"
    "example.com/services/notifications"
  }
  roles {
    Reception
    Nurse
    Doctor
    Manager
    Manager > Doctor > Nurse > Reception
  }
  theme: Clinica
  locales: [pt-BR]
  environments: [dev, qa, prod]
}
```

*Valida: `patients`, `scheduling` e `auth` são implícitos; billing e notifications são entry points.*

### A.3 Logística com despacho e rastreio

```archol
application "example.com/app/rota-certa" {
  title: {
    pt-BR: "Rota Certa"
    en-US: "Right Route"
  }

  services {
    "example.com/services/dispatch"
    "example.com/services/tracking"
    "example.com/services/billing"
    "example.com/services/notifications"
  }
  roles { Dispatcher; Driver; Supervisor; Supervisor > Dispatcher > Driver }
  theme: Ops
  locales: [pt-BR, en-US]
  environments: [dev, qa, prod]
}
```

*Valida: `orders`, `fleet` e `auth` implícitos; tracking reage ao dispatch sem wire.*

### A.4 Backoffice que reusa o mesmo catálogo

```archol
application "example.com/app/backoffice-loja" {
  title: { pt-BR: "Backoffice da Loja" }

  services {
    "example.com/services/inventory"
    "example.com/services/orders"
    "example.com/services/reports"
  }
  roles { Clerk; Manager; Admin; Admin > Manager > Clerk }
  theme: Backoffice
  locales: [pt-BR]
  environments: [dev, qa, prod]
}
```

*Valida: o mesmo `catalog` usado pela loja entra implicitamente via inventory; billing via reports.*

### A.5 Serviço de campo

```archol
application "example.com/app/campo-agil" {
  title: { pt-BR: "Campo Ágil" }

  services {
    "example.com/services/work-orders"
    "example.com/services/billing"
    "example.com/services/notifications"
  }
  roles { Technician; Dispatcher; Manager; Manager > Dispatcher > Technician }
  theme: Field
  locales: [pt-BR]
  environments: [dev, qa, prod]
}
```

*Valida: scheduling, inventory e auth entram por work-orders; billing é entry point.*

### A.6 Marketplace com disputas

```archol
application "example.com/app/mercado-aberto" {
  title: {
    pt-BR: "Mercado Aberto"
    en-US: "Open Market"
  }

  services {
    "example.com/services/sellers"
    "example.com/services/orders"
    "example.com/services/disputes"
    "example.com/services/notifications"
  }
  roles { Buyer; Seller; Support; Admin; Admin > Support > Seller > Buyer }
  theme: Market
  locales: [pt-BR, en-US]
  environments: [dev, qa, prod]
}
```

*Valida: listings, payments e auth implícitos; disputes é entry point e depende de orders/payments.*

### A.7 Aplicação hospedada em repositório Git

```archol
application "github.com/acme/field-app" {
  title: { pt-BR: "Field App" }

  services {
    "github.com/acme/work-orders"
    "example.com/services/auth"
    "example.com/services/notifications"
  }
  roles { Technician; Dispatcher; Manager; Manager > Dispatcher > Technician }
  theme: Field
  locales: [pt-BR]
  environments: [dev, qa, prod]
}
```

*Valida: app em repositório, versão por commit e mistura de serviço do repo com registry.*

### A.8 Aplicação com serviço que reintroduz

```archol
application "example.com/app/loja-com-antifraude" {
  title: { pt-BR: "Loja com Antifraude" }

  services {
    "example.com/services/checkout"
    "example.com/services/fraud-checkout"
    "example.com/services/notifications"
  }
  roles { Customer; Clerk; Manager; Manager > Clerk }
  theme: Nuvula
  locales: [pt-BR]
  environments: [dev, qa, prod]
}
```

*Valida: `fraud-checkout` importa `checkout` e reintroduz apenas o passo `FraudCheck`; a app não precisa configurar substituição.*

## Service

> Todo artefato declarado no serviço é público automaticamente; não há `exports`. Estes exemplos focam no contrato (imports, reintroduce).

### S.1 Catálogo com contrato e dependências

```archol
service "example.com/services/catalog" {
  title: { pt-BR: "Catálogo", en: "Catalog" }
  locales: [ "pt-BR", "en" ]
  // document Product, std.Money, view ProductForm, event Product.Updated: público

  parameter DefaultCurrency {
    title: { pt-BR: "Moeda padrão", en: "Default currency" }
    type: std.Currency
    default: "BRL"
  }

  imports {
    service "example.com/services/auth"
    service "example.com/services/media"
  }
}
```

*Valida: imports sem versão; o lock do app escolhe uma versão de cada serviço.*

### S.2 Checkout como orquestrador

```archol
service "example.com/services/checkout" {
  title: { pt-BR: "Finalização da compra", en: "Checkout" }
  locales: [ "pt-BR", "en" ]
  // document CheckoutSession, event OrderPlaced: públicos

  imports {
    service "example.com/services/cart" as Cart
    service "example.com/services/catalog" as Catalog
    service "example.com/services/payment" as Payment
    service "example.com/services/shipping" as Shipping
    service "example.com/services/auth" as Auth
  }
}
```

*Valida: serviço compõe outros serviços sem declarar versões; a app não repete essas dependências.*

### S.3 Estoque com reservas e alerta

```archol
service "example.com/services/inventory" {
  title: { pt-BR: "Estoque", en: "Inventory" }
  locales: [ "pt-BR", "en" ]
  // document StockItem, document Reservation, view StockTable, event StockItem.Low: público

  imports {
    service "example.com/services/catalog"
    service "example.com/services/auth"
  }
}
```

*Valida: dois documentos, evento de alerta e imports sem versão.*

### S.4 Faturamento com job e relatório

```archol
service "example.com/services/billing" {
  title: { pt-BR: "Faturamento", en: "Billing" }
  locales: [ "pt-BR", "en" ]
  // document Invoice, document Payment, report CashFlow, job CloseMonth, events: públicos

  imports {
    service "example.com/services/orders"
    service "example.com/services/auth"
    integration ERP
  }
}
```

*Valida: job, relatório e integração no mesmo serviço; versões ficam no lock.*

### S.5 Identidade e sessão

```archol
service "example.com/services/auth" {
  title: { pt-BR: "Identidade", en: "Identity" }
  locales: [ "pt-BR", "en" ]
  // document User, document Session, roles Customer/Clerk, view LoginForm, event Session.LoggedIn: público

  imports { integration Email }
}
```

*Valida: identidade exporta papéis e view; sem lock/IDs dentro do serviço.*

### S.6 Notificações multicanal

```archol
service "example.com/services/notifications" {
  title: { pt-BR: "Notificações", en: "Notifications" }
  locales: [ "pt-BR", "en" ]
  // document NotificationTemplate, event Notification.Sent: público

  imports {
    service "example.com/services/auth"
    integration Email
    integration WhatsApp
  }
}
```

*Valida: canais são integrações e o consumidor é entry point da app, sem wire.*

### S.7 Serviço hospedado em repositório Git

```archol
service "gitlab.com/acme/lab-quality" {
  title: { pt-BR: "Qualidade laboratorial", en: "Laboratory quality" }
  locales: [ "pt-BR", "en" ]
  // document LabQualityReport, event QualityAlert: públicos

  imports {
    service "example.com/services/auth"
    integration Email
  }
}
```

*Valida: serviço em repositório, versão por tag e README como documentação.*

### S.8 Serviço que reintroduz um passo de processo

```archol
service "example.com/services/fraud-checkout" {
  title: { pt-BR: "Finalização da compra antifraude", en: "Fraud checkout" }
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

  // A composição do passo FraudCheck no processo Checkout fica adiada:
  // o processo é código e não há insert/task (DSL_REFERENCE.md §4.12).

}
```

*Valida: o ciclo de vida e a action são editados via `reintroduce`; a composição de passo em processo-código fica adiada.*

## Lock file do app

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
    "example.com/services/fraud-checkout":
      id: 8
      version: 1.0.0
      source: registry
    "example.com/services/notifications":
      id: 6
      version: 1.1.0
      source: registry
  artifacts:
    "example.com/services/checkout#CheckoutSession": 20
    "example.com/services/checkout#Checkout": 21
    "example.com/services/fraud-checkout#FraudCheck": 40
```

> O lock fixa **uma versão por serviço** para o app inteiro e um **ID estável por identificador** (usado para minificação e compressão da serialização). O serviço não tem lock, IDs nem versões de imports.

## Process

O processo é **código TS-like**: execução sequencial e implícita, concorrência com `parallel`, variáveis declaradas no topo e tarefas chamadas como funções. Os **35 processos completos** estão em [`examples/`](examples/); os recortes abaixo mostram a sintaxe.

Convenções:

- `process Nome(args)` + um gatilho (`on`, `by`, `schedule` + `fanOut`, `message`, `called`) + `key`;
- sem `pool`/`lane`/`task`; `roles` no form/manual definem quem executa;
- operações diretas (`Doc.action`, `Integracao.op`, `Decision.evaluate`) e tasks (`Form.show`, `Manual.show`, `Processo.run`);
- sem arestas: `if`/`else if`, `parallel`, `loop`, `wait`, `try/catch` e `return`;
- `sla` é `std.Duration`: `sla: "1d"` guarda quantidade + unidade e o AST recebe `typeRef: "std.Duration"` e um `ShapeLiteral`;
- `end` no fim do processo, com `outcome: success | cancelled | failure`;
- o documento só muda por `action`; decisão humana é lida no estado do documento.

### P.1 Checkout com retentativa do cliente (checkout)

```archol
process Checkout(session: CheckoutSession) {
  title: { pt-BR: "Finalização da compra", en: "Checkout" }
  // I18N em forma de string: primeiro locale do serviço.
  goal: "Cotar o frete, autorizar o pagamento e confirmar a compra"
  sla: "1d"

  on [CheckoutSession.CartValidated]
  key: session

  let offer: FreightOffer | null = null
  let authorized = false
  let authorization: string

  try {
    try {
      offer = FreightGateway.quote({ session })
    } catch (Unavailable) {
      FreightReview.show({ session })
    }

    if (offer != null) {
      CheckoutSession.quoteShipping({
        freightAmount: offer.freight,
        days: offer.days
      })
    }

    loop PaymentAttempt max 3 {
      try {
        authorization = PaymentGateway.authorize({
          total: session.total,
          method: session.paymentMethod
        })

        CheckoutSession.authorizePayment({ authorization })
        authorized = true
        break
      } catch (Declined) {
        PaymentRetry.show({ session })

        if (session.status == Abandoned) {
          return Abandoned
        }
      }
    }

    if (!authorized) {
      CheckoutSession.abandon({ reason: "Tentativas de pagamento esgotadas" })
      return Expired
    }

    CheckoutSession.confirmOrder()
    return Confirmed
  } catch (Timeout) {
    CheckoutSession.abandon({ reason: "Prazo de finalização esgotado sem conclusão" })
    return Expired
  }

  end Confirmed {
    title: { pt-BR: "Compra confirmada", en: "Purchase confirmed" }
    outcome: success
  }
  end Abandoned {
    title: { pt-BR: "Compra abandonada", en: "Purchase abandoned" }
    outcome: cancelled
  }
  end Expired {
    title: { pt-BR: "Expirada", en: "Expired" }
    outcome: failure
  }
}
```

```archol
form PaymentRetry {
  title: { pt-BR: "Repetir pagamento", en: "Retry payment" }
  document: CheckoutSession
  roles: [Customer]
  timeout: 1d
  priority: 80

  fields {
    customer { readonly }
    total { readonly }
    method: PaymentMethod { required }
    reason: string { maxLength: 500 }
  }

  actions {
    retry { calls: CheckoutSession.retryPayment(method) }
    cancel { calls: CheckoutSession.abandon(reason) }
  }
}
```

*Valida: variáveis duráveis no topo, operações diretas, task humana com roles/timeout, retry com `loop max`, desvio por erro e decisão pelo estado do documento.*

### P.2 Aprovação de compra com alçada (procurement)

```archol
process PurchaseApproval(purchaseRequest: PurchaseRequest) {
  title: { pt-BR: "Aprovação de compra", en: "Purchase approval" }
  sla: "5d"

  on [PurchaseRequest.Submitted]
  key: purchaseRequest

  let tier: ApproverTier

  tier = ApprovalLimit.evaluate({
    amount: purchaseRequest.amount,
    supplier: purchaseRequest.supplier
  })

  if (tier == Auto) {
    purchaseRequest.approve()
    return Approved
  }

  ApprovalDecision.show({ purchaseRequest })

  if (purchaseRequest.status == Approved) {
    return Approved
  }

  return Rejected

  end Approved {
    title: { pt-BR: "Compra aprovada", en: "Purchase approved" }
    outcome: success
  }
  end Rejected {
    title: { pt-BR: "Compra rejeitada", en: "Purchase rejected" }
    outcome: failure
  }
}
```

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

*Valida: decisão DMN pura chamada como operação, `if/else` sobre o resultado, form com múltiplas ações e desfecho lido no documento.*

### P.3 Despacho com paralelismo e coleta (shipping)

```archol
process DispatchOrder(shipment: Shipment) {
  title: { pt-BR: "Despacho do pedido", en: "Order dispatch" }
  sla: "1d"

  on [Shipment.Created]
  key: shipment

  let tracking: TrackingCode

  parallel {
    branch Pick { Pick.show({ shipment }) }
    branch Pack { Packing.show({ shipment }) }
  }

  shipment.applyFreight({
    carrier: "CarrierX",
    amount: shipment.freightAmount
  })

  tracking = Carrier.createLabel({ shipment, carrier: "CarrierX" })
  Shipment.dispatch({ trackingCode: tracking })

  send PickupRequest {
    key: shipment
    with { requestedAt: now() }
  }

  try {
    receive PickupConfirmed {
      key: shipment
      timeout: 2d
      apply: Shipment.confirmPickup
    }
  } catch (Timeout) {
    ManualPickup.show({ shipment })
  }

  DeliveryTracking.run({ shipment })
  return Dispatched

  end Dispatched {
    title: { pt-BR: "Despachado", en: "Dispatched" }
    outcome: success
  }
}
```

*Valida: `parallel`, `send`/`receive` com `key` e `apply`, timeout com desvio manual, subprocesso com `run` e ends nomeados.*

### P.4 Cancelamento com compensação (orders)

```archol
process OrderCancellation(order: Order) {
  title: { pt-BR: "Cancelamento de pedido", en: "Order cancellation" }
  sla: "1d"

  on [Order.CancellationRequested]
  key: order

  CancellationReview.show({ order })

  if (order.status != CancellationApproved) {
    return Rejected
  }

  try {
    Inventory.Reservation.release()
    Order.cancel({ reason: "Cancelamento aprovado" })
  } catch (ReleaseFailed) {
    Order.reopen()
    return Rejected
  }

  return Cancelled

  end Cancelled {
    title: { pt-BR: "Cancelado", en: "Cancelled" }
    outcome: success
  }
  end Rejected {
    title: { pt-BR: "Cancelamento recusado", en: "Cancellation rejected" }
    outcome: failure
  }
}
```

*Valida: ação em outro serviço (`Inventory.Reservation`), compensação explícita no `catch` e `return` para ends nomeados.*

## Form

O form é o **contrato da tarefa humana**: o que a pessoa vê, o que ela pode fazer e qual ação de documento cada botão executa. A tela é do builder; o form declara `roles`, `timeout`, `priority` e `assign`, não escreve campo direto e não tem ciclo de vida.

### F.1 Aprovação com três desfechos e rascunho

```archol
form PurchaseReview {
  title: { pt-BR: "Revisar compra", en: "Review purchase" }
  document: PurchaseRequest
  roles: [Buyer]
  timeout: 1d
  assign: pull

  fields {
    requester
    supplier
    amount
    justification { readonly }
    note: string {
      title: { pt-BR: "Observação ao solicitante", en: "Note to requester" }
      maxLength: 500
    }
  }

  actions {
    approve { calls: PurchaseRequest.approve }
    reject { calls: PurchaseRequest.reject(note) }
    sendBack { calls: PurchaseRequest.returnToRequester(note) }
    save { calls: PurchaseRequest.saveDraft(note); stay }
  }
}
```

*Valida: três desfechos + botão que não avança; campos herdados do documento declarados só pelo nome.*

### F.2 Formulário de criação com campos novos

```archol
form NewOrder {
  title: { pt-BR: "Novo pedido", en: "New order" }
  document: Order

  fields {
    customer { required }
    channel: Channel {
      title: { pt-BR: "Canal de venda", en: "Sales channel" }
      required
    }
    promisedAt: Date {
      title: { pt-BR: "Data prometida", en: "Promised date" }
    }
    discount: std.Percentage {
      title: { pt-BR: "Desconto", en: "Discount" }
      min: 0
      max: 30
    }
  }

  actions {
    create { calls: Order.create(customer, channel, promisedAt, discount) }
  }
}
```

*Valida: campo novo declara Shape + title; campo herdado aceita `required` (endurece, nunca relaxa); botão sem comando.*

### F.3 Form preenchido no portal do fornecedor

```archol
form SupplierRegistration {
  title: { pt-BR: "Cadastro de fornecedor", en: "Supplier registration" }
  document: Supplier

  fields {
    legalName { required }
    tradeName
    email { required }
    phone
    bankAccount: BankAccount {
      title: { pt-BR: "Conta bancária", en: "Bank account" }
      required
      sensitive
    }
    documents: File[] {
      title: { pt-BR: "Documentos", en: "Documents" }
      required
    }
  }

  actions {
    submit { calls: Supplier.submitRegistration(legalName, tradeName, email, phone, bankAccount, documents) }
    save { calls: Supplier.saveDraft; stay }
  }
}
```

*Valida: o mesmo form serve a um papel externo (portal); campo sensível declarado no form e no documento; anexos.*

### F.4 Form de campo com assinatura e prova

```archol
form VisitReport {
  title: { pt-BR: "Relato da visita", en: "Visit report" }
  document: Visit

  fields {
    customer
    site
    notes: string {
      title: { pt-BR: "Observações", en: "Notes" }
      maxLength: 2000
      required
    }
    photos: Image[] { title: { pt-BR: "Fotos", en: "Photos" } }
    reading: DeviceReading<std.Mass> {
      title: { pt-BR: "Leitura do equipamento", en: "Device reading" }
      readonly
    }
    signature: Signature {
      title: { pt-BR: "Assinatura do cliente", en: "Customer signature" }
      required
    }
  }

  actions {
    complete { calls: Visit.complete(notes, photos, signature) }
    reschedule { calls: Visit.requestReschedule(notes) }
  }
}
```

*Valida: assinatura, foto, leitura de dispositivo somente leitura e dois desfechos com prova.*

### F.5 Confirmação de ação irreversível

```archol
form ShipmentCancellation {
  title: { pt-BR: "Cancelar remessa", en: "Cancel shipment" }
  document: Shipment

  fields {
    tracking { readonly }
    reason: CancellationReason {
      title: { pt-BR: "Motivo", en: "Reason" }
      required
    }
  }

  actions {
    confirmCancel { calls: Shipment.cancel(reason) }
  }
}
```

*Valida: um único desfecho; a confirmação com resumo do efeito é derivada de a ação ser terminal.*

### F.6 Rejeição com motivo obrigatório

```archol
form RefundRejection {
  title: { pt-BR: "Recusar reembolso", en: "Reject refund" }
  document: ReturnRequest

  fields {
    amount { readonly }
    reason: RejectionReason {
      title: { pt-BR: "Motivo da recusa", en: "Rejection reason" }
      required
    }
    detail: string {
      title: { pt-BR: "Detalhe ao cliente", en: "Detail to customer" }
      maxLength: 400
      required
    }
  }

  actions {
    reject { calls: ReturnRequest.reject(reason, detail) }
  }
}
```

*Valida: rejeição como ação de primeira classe, com motivo tipado e texto ao cliente; conta como retrabalho no KPI.*

### F.7 Form com contexto de documentos relacionados

```archol
form ClaimRegistration {
  title: { pt-BR: "Registro de sinistro", en: "Claim registration" }
  document: Claim

  fields {
    policy
    insured
    occurredAt { required }
    description: string {
      title: { pt-BR: "Descrição", en: "Description" }
      maxLength: 4000
      required
    }
    estimatedAmount: std.Money {
      title: { pt-BR: "Valor estimado", en: "Estimated amount" }
      currency: BRL
    }
    attachments: File[] { title: { pt-BR: "Anexos", en: "Attachments" } }
  }

  actions {
    register { calls: Claim.register(occurredAt, description, estimatedAmount, attachments) }
    save { calls: Claim.saveDraft(description); stay }
  }
}
```

*Valida: campos readonly de contexto (apólice, segurado) e entrada de dados em um único comando.*

### F.8 O mesmo form em duas tasks

```archol
form ApprovalDecision {
  title: { pt-BR: "Decisão de aprovação", en: "Approval decision" }
  document: PurchaseRequest

  fields {
    amount { readonly }
    supplier { readonly }
    justification { readonly }
    note: string {
      title: { pt-BR: "Justificativa", en: "Justification" }
      maxLength: 500
    }
  }

  actions {
    approved { calls: PurchaseRequest.approve(note) }
    rejected { calls: PurchaseRequest.reject(note) }
  }
}
```

```archol
process PurchaseApproval(purchaseRequest: PurchaseRequest) {
  // ...
  ApprovalDecision.show({ purchaseRequest })
  // ...
}
```

*Valida: o form declara `roles` e o processo o chama com `Form.show`; não há task/lane.*

## Message

Mensagem é contrato entre participantes: tem `about` (o documento que correlaciona), `payload` tipado e título. O envio é `send`; o recebimento é `receive`; o canal/parceiro é configuração de runtime/integração (não há `partner`).

### M.1 Envio de solicitação

```archol
message PickupRequest {
  title: { pt-BR: "Solicitação de coleta", en: "Pickup request" }
  about: Shipment
  payload {
    requestedAt: DateTime { title: { pt-BR: "Solicitado em" } }
    window: Duration { title: { pt-BR: "Janela de coleta" } }
  }
}

send PickupRequest {
  key: shipment
  with { requestedAt: now(); window: shipment.pickupWindow }
}
```

*Valida: mensagem com correlação por documento e envio por `send`.*

### M.2 Recebimento com aplicação no documento

```archol
message PickupConfirmed {
  title: { pt-BR: "Coleta confirmada", en: "Pickup confirmed" }
  about: Shipment
  payload {
    pickupAt: DateTime { title: { pt-BR: "Coletado em" } }
    tracking: TrackingCode { title: { pt-BR: "Rastreio" } }
  }
}

try {
  receive PickupConfirmed {
    key: shipment
    timeout: 1d
    apply: Shipment.applyPickup
  }
} catch (Timeout) {
  ManualPickup.show({ shipment })
}
```

*Valida: recebimento correlacionado aplica o payload numa action; timeout com desvio.*

### M.3 Atualização contínua de rastreio

```archol
message TrackingUpdated {
  title: { pt-BR: "Rastreio atualizado", en: "Tracking updated" }
  about: Shipment
  payload {
    status: ShipmentStatus { title: { pt-BR: "Situação" } }
    at: DateTime { title: { pt-BR: "Quando" } }
    note: string { title: { pt-BR: "Observação" } }
  }
}

loop Tracking max 5 {
  receive TrackingUpdated {
    key: shipment
    timeout: 5d
    apply: Shipment.applyTracking
  }
}
```

*Valida: recebimento recorrente dentro de `loop`; aplicação idempotente no documento.*

### M.4 Mensagem que inicia processo

```archol
message PaymentConfirmed {
  title: { pt-BR: "Pagamento confirmado", en: "Payment confirmed" }
  about: Order
  payload {
    paidAt: DateTime { title: { pt-BR: "Pago em" } }
    amount: std.Money { title: { pt-BR: "Valor" } }
  }
}

process OrderPacking(order: Order) {
  title: { pt-BR: "Separação do pedido", en: "Order packing" }

  message: PaymentConfirmed
  key: order

  PackShipment.show({ order })
  Order.readyToShip()
  return Packed

  end Packed {
    title: { pt-BR: "Pronto para envio", en: "Ready to ship" }
    outcome: success
  }
}
```

*Valida: mensagem como gatilho de início; correlação pelo sujeito; processo curto e completo.*
## Document

### D.1 Cliente com identidade, PII e renomeação

```archol
document Customer {
  title: { pt-BR: "Cliente", en: "Customer" }
  sync: live
  retention: 5y

  fields {
    name: string {
      title: { pt-BR: "Nome", en: "Name" }
      maxLength: 120
      lookup
      required
      sensitive
    }
    email: Email {
      title: { pt-BR: "E-mail", en: "Email" }
      lookup
      unique
      required
    }
    phone: Phone {
      title: { pt-BR: "Telefone", en: "Phone" }
      sensitive
    }
    taxId: TaxId {
      title: { pt-BR: "CPF/CNPJ", en: "Tax ID" }
      unique
      sensitive
    }
    citizen: Citizen {
      title: { pt-BR: "Cidadão", en: "Citizen" }
    }
    private riskScore: std.Count {
      title: { pt-BR: "Score interno", en: "Internal score" }
    }
    createdAt: DateTime {
      title: { pt-BR: "Criado em", en: "Created at" }
      readonly
    }
  }

  events {
    Renamed {
      title: { pt-BR: "Cliente renomeado", en: "Customer renamed" }
      payload {
        name: string { title: { pt-BR: "Nome", en: "Name" } }
      }
    }
  }

  actions {
    rename(newName: string) {
      title: { pt-BR: "Renomear", en: "Rename" }
      roles: [Clerk]
      returns: Customer

      rule newName.length > 0 {
        pt-BR: "Informe o novo nome"
        en: "Enter the new name"
      }

      {
        name = newName
        emit Renamed { name: newName }
      }
    }
  }

  indexes {
    byEmail: [email] unique
    byTaxId: [taxId] unique
    byName: [name] search
  }

  rule email != null {
    pt-BR: "Informe o e-mail do cliente"
    en: "Enter the customer email"
  }
}
```

*Valida: unicidade por e-mail e documento fiscal, `lookup` como linha de exibição, PII em `sensitive`, campo interno em `private`, referência a `Citizen` como `DocumentRef` e comando sem transição que publica evento pelo `emit` do body (5.7).*

### D.2 Nota fiscal como agregado com estado, itens e eventos

```archol
scopable document FiscalYear {
  title: { pt-BR: "Exercício fiscal", en: "Fiscal year" }
  fields {
    year: integer {
      title: { pt-BR: "Ano", en: "Year" }
      unique
      lookup
    }
    endsAt: DateTime {
      title: { pt-BR: "Fim", en: "End" }
    }
  }

  seed {
    Y2026 { year: 2026; endsAt: 2026-12-31 }
  }
}

shape InvoiceItem {
  title: { pt-BR: "Item da nota", en: "Invoice item" }
  fields {
    description: string {
      title: { pt-BR: "Descrição", en: "Description" }
      maxLength: 200
    }
    quantity: decimal {
      title: { pt-BR: "Quantidade", en: "Quantity" }
      min: 0.001
    }
    unitPrice: std.Money {
      title: { pt-BR: "Preço unitário", en: "Unit price" }
      currency: BRL
    }
    subtotal: std.Money {
      title: { pt-BR: "Subtotal", en: "Subtotal" }
      currency: BRL
      computed: quantity * unitPrice
    }
  }
}

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
    icon: status.draft
    ordinal: 10

    transition Issued {
      title: { pt-BR: "Nota emitida", en: "Invoice issued" }
      payload {
        number: InvoiceNumber { title: { pt-BR: "Número", en: "Number" } }
        issuedAt: DateTime { title: { pt-BR: "Emitida em", en: "Issued at" } }
        total: std.Money { title: { pt-BR: "Total", en: "Total" } }
      }
    }

    transition Cancelled {
      title: { pt-BR: "Nota cancelada", en: "Invoice cancelled" }
      payload {
        reason: string { title: { pt-BR: "Motivo", en: "Reason" } }
      }
    }
  }
  value Issued {
    title: { pt-BR: "Emitida", en: "Issued" }
    icon: finance.issued
    ordinal: 20

    transition Paid {
      title: { pt-BR: "Nota paga", en: "Invoice paid" }
      payload {
        paidAt: DateTime { title: { pt-BR: "Paga em", en: "Paid at" } }
        amount: std.Money { title: { pt-BR: "Valor", en: "Amount" } }
      }
    }

    transition Cancelled {
      title: { pt-BR: "Nota cancelada", en: "Invoice cancelled" }
      payload {
        reason: string { title: { pt-BR: "Motivo", en: "Reason" } }
      }
    }
  }
  value Paid {
    title: { pt-BR: "Paga", en: "Paid" }
    icon: status.paid
    ordinal: 30
  }
  value Cancelled {
    title: { pt-BR: "Cancelada", en: "Cancelled" }
    icon: status.cancelled
    ordinal: 40
  }
}

document Invoice {
  title: { pt-BR: "Nota fiscal", en: "Invoice" }
  sync: live
  retention: 10y

  fields {
    number: InvoiceNumber {
      title: { pt-BR: "Número", en: "Number" }
      format: "{fiscalYear.year:4}-{sequence:6}"
      sequence: per(FiscalYear)
      unique
      lookup
    }
    fiscalYear: FiscalYear {
      title: { pt-BR: "Exercício", en: "Fiscal year" }
      default: FiscalYear.Y2026
      required
    }
    customer: Customer {
      title: { pt-BR: "Cliente", en: "Customer" }
      lookup
      required
    }
    items: InvoiceItem[] {
      title: { pt-BR: "Itens", en: "Items" }
    }
    subtotal: std.Money {
      title: { pt-BR: "Subtotal", en: "Subtotal" }
      currency: BRL
      computed: items.sum(subtotal)
    }
    tax: std.Money {
      title: { pt-BR: "Imposto", en: "Tax" }
      currency: BRL
      default: "0 BRL"
    }
    total: std.Money {
      title: { pt-BR: "Total", en: "Total" }
      currency: BRL
      lookup
      computed: subtotal + tax
    }
    status: InvoiceStatus {
      title: { pt-BR: "Situação", en: "Status" }
      lookup
    }
    issuedAt: DateTime {
      title: { pt-BR: "Emitida em", en: "Issued at" }
    }
    paidAt: DateTime {
      title: { pt-BR: "Paga em", en: "Paid at" }
    }
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

    issue() {
      title: { pt-BR: "Emitir", en: "Issue" }
      roles: [Manager]

      to: status.Issued

      rule items.length > 0 {
        pt-BR: "Adicione itens antes de emitir a nota"
        en: "Add items before issuing the invoice"
      }
      rule total > 0 BRL {
        pt-BR: "A nota precisa ter valor"
        en: "The invoice must have a value"
      }

      returns: Invoice
      {
        issuedAt = now()
      }
    }

    pay(settledAt: DateTime) {
      title: { pt-BR: "Pagar", en: "Pay" }
      roles: [Clerk]

      to: status.Paid
      emits { amount: total }

      {
        paidAt = settledAt
      }
    }

    cancel(reason: string) {
      title: { pt-BR: "Cancelar", en: "Cancel" }
      roles: [Manager]

      to: status.Cancelled

      rule reason.length >= 10 {
        pt-BR: "Informe um motivo com pelo menos 10 caracteres"
        en: "Enter a reason with at least 10 characters"
      }
    }
  }

  indexes {
    byNumber: [number] unique
    byStatusDate: [status, issuedAt] range
    byCustomer: [customer.id, issuedAt] range
  }

  rule total == subtotal + tax {
    pt-BR: "Total da nota não confere"
    en: "Invoice total does not match"
  }
}
```

*Valida: agregado com Shape composto (`InvoiceItem`), `computed` de subtotal e total, estado em Shape `base: state` com `initial`, evento por aresta (`Issued`/`Paid`/`Cancelled`), `from:`/`to:`/`emits:` na action e número de negócio sequenciado por exercício.*

### D.3 Remessa com dispositivo, unidade e frete derivado

```archol
// imports { service "bpcms.org/std" as std }

shape ShipmentStatus {
  title: { pt-BR: "Situação da remessa", en: "Shipment status" }
  base: state
  initial: Draft

  value Draft {
    title: { pt-BR: "Rascunho", en: "Draft" }
    icon: status.draft
    ordinal: 10

    transition InTransit {
      event: Dispatched
      title: { pt-BR: "Remessa despachada", en: "Shipment dispatched" }
      payload {
        tracking: TrackingCode { title: { pt-BR: "Rastreio", en: "Tracking" } }
        dispatchedAt: DateTime { title: { pt-BR: "Despachada em", en: "Dispatched at" } }
      }
    }
  }
  value InTransit {
    title: { pt-BR: "Em trânsito", en: "In transit" }
    icon: logistic.inTransit
    ordinal: 20

    transition Delivered {
      title: { pt-BR: "Remessa entregue", en: "Shipment delivered" }
      payload {
        deliveredAt: DateTime { title: { pt-BR: "Entregue em", en: "Delivered at" } }
      }
    }
  }
  value Delivered {
    title: { pt-BR: "Entregue", en: "Delivered" }
    icon: status.delivered
    ordinal: 30
  }
}

document Shipment {
  title: { pt-BR: "Remessa", en: "Shipment" }
  sync: live

  fields {
    order: Order {
      title: { pt-BR: "Pedido", en: "Order" }
      lookup
      required
    }
    carrier: Carrier {
      title: { pt-BR: "Transportadora", en: "Carrier" }
      lookup
    }
    packages: integer {
      title: { pt-BR: "Volumes", en: "Packages" }
      min: 1
    }
    weight: std.Mass {
      title: { pt-BR: "Peso bruto", en: "Gross weight" }
      unit: kg
      readonly
    }
    volume: std.Volume {
      title: { pt-BR: "Volume", en: "Volume" }
      unit: m3
      min: 0
      readonly
    }
    freight: std.Money {
      title: { pt-BR: "Frete cotado", en: "Quoted freight" }
      currency: BRL
      readonly
    }
    tracking: TrackingCode {
      title: { pt-BR: "Rastreio", en: "Tracking" }
      lookup
    }
    status: ShipmentStatus {
      title: { pt-BR: "Situação", en: "Status" }
      lookup
    }
    promisedAt: DateTime {
      title: { pt-BR: "Entrega prometida", en: "Promised delivery" }
    }
    dispatchedAt: DateTime {
      title: { pt-BR: "Despachada em", en: "Dispatched at" }
    }
    deliveredAt: DateTime {
      title: { pt-BR: "Entregue em", en: "Delivered at" }
    }
  }

  source: device "scale-01" for weight

  actions {
    dispatch(code: TrackingCode) {
      title: { pt-BR: "Despachar", en: "Dispatch" }
      roles: [Dispatcher]

      to: status.InTransit

      rule weight > 0 kg {
        pt-BR: "A balança precisa registrar o peso antes do despacho"
        en: "The scale must record the weight before dispatch"
      }
      rule code != null {
        pt-BR: "Informe o código de rastreio"
        en: "Enter the tracking code"
      }

      {
        tracking = code
        dispatchedAt = now()
      }
    }

    confirmDelivery(deliveredOn: DateTime) {
      title: { pt-BR: "Confirmar entrega", en: "Confirm delivery" }
      roles: [Dispatcher]

      to: status.Delivered

      {
        deliveredAt = deliveredOn
      }
    }
  }

  indexes {
    tracking: [tracking] unique
    byStatus: [status, dispatchedAt] range
    byOrder: [order.id] hash
  }

  rule status == Delivered implies deliveredAt != null {
    pt-BR: "Entrega confirmada exige data de entrega"
    en: "Confirmed delivery requires a delivery date"
  }
}

document FreightEstimate {
  title: { pt-BR: "Estimativa de frete", en: "Freight estimate" }
  sync: read
  computed from Shipment, FreightTariff
  freshness: 15m
  exposes: [freight, calculatedAt]

  fields {
    shipment: Shipment {
      title: { pt-BR: "Remessa", en: "Shipment" }
      lookup
      required
    }
    carrier: Carrier {
      title: { pt-BR: "Transportadora", en: "Carrier" }
      lookup
    }
    freight: std.Money {
      title: { pt-BR: "Frete", en: "Freight" }
      currency: BRL
    }
  }

  indexes {
    byShipment: [shipment.id] unique
  }
}
```

*Valida: `source: device` para o peso, `std.Mass`/`std.Volume` com unidade no campo, `readonly` no que vem de dispositivo/cálculo, `tracking` e `status` no lookup, transições `dispatch`/`confirmDelivery` com evento declarado e frete calculado em documento derivado (`computed from` + `freshness` + `exposes`).*

### D.4 Laudo com arquivo, assinatura e retenção

```archol
shape LabReportStatus {
  title: { pt-BR: "Situação do laudo", en: "Lab report status" }
  base: state
  initial: Draft

  value Draft {
    title: { pt-BR: "Rascunho", en: "Draft" }
    icon: status.draft
    ordinal: 10

    transition Released {
      title: { pt-BR: "Laudo liberado", en: "Lab report released" }
      payload {
        releasedAt: DateTime { title: { pt-BR: "Liberado em", en: "Released at" } }
        file: LabReportFile { title: { pt-BR: "Laudo assinado", en: "Signed report" } }
      }
    }
  }
  value Released {
    title: { pt-BR: "Liberado", en: "Released" }
    icon: lab.released
    ordinal: 20
  }
}

shape LabReportFile {
  title: { pt-BR: "Arquivo do laudo", en: "Lab report file" }
  base: attachment
  mime: [application/pdf]
}

document LabReport {
  title: { pt-BR: "Laudo laboratorial", en: "Lab report" }
  sync: live
  retention: 20y

  fields {
    patient: Citizen {
      title: { pt-BR: "Paciente", en: "Patient" }
      lookup
      required
    }
    exam: ExamType {
      title: { pt-BR: "Exame", en: "Exam" }
      lookup
    }
    status: LabReportStatus {
      title: { pt-BR: "Situação", en: "Status" }
      lookup
    }
    file: LabReportFile {
      title: { pt-BR: "Laudo assinado", en: "Signed report" }
    }
    collectedAt: DateTime {
      title: { pt-BR: "Coletado em", en: "Collected at" }
    }
    releasedAt: DateTime {
      title: { pt-BR: "Liberado em", en: "Released at" }
    }
  }

  signature {
    release: Pathologist
  }

  actions {
    release() {
      title: { pt-BR: "Liberar", en: "Release" }
      roles: [Pathologist]

      to: status.Released

      rule file != null {
        pt-BR: "Anexe o laudo antes de liberar"
        en: "Attach the report before releasing"
      }

      returns: LabReport
      {
        releasedAt = now()
      }
    }
  }

  indexes {
    byPatient: [patient.id, collectedAt] range
    byStatus: [status, releasedAt] range
  }

  rule status == Released implies releasedAt != null {
    pt-BR: "Laudo liberado exige data de liberação"
    en: "Released report requires a release date"
  }
}
```

*Valida: anexo tipado com `base: attachment` e `mime`, `retention` no documento, bloco `signature` ligando a action ao papel com autoridade de assinatura, estado `Draft`/`Released` e evento de liberação com payload de arquivo.*

### D.5 Chamado com SLA calculado e prazo

```archol
shape TicketPriority {
  title: { pt-BR: "Prioridade do chamado", en: "Ticket priority" }
  base: classification

  value Low {
    title: { pt-BR: "Baixa", en: "Low" }
    icon: priority.low
    ordinal: 10
  }
  value Normal {
    title: { pt-BR: "Normal", en: "Normal" }
    icon: priority.normal
    ordinal: 20
  }
  value High {
    title: { pt-BR: "Alta", en: "High" }
    icon: priority.high
    ordinal: 30
  }
  value Critical {
    title: { pt-BR: "Crítica", en: "Critical" }
    icon: priority.critical
    ordinal: 40
  }
}

shape TicketStatus {
  title: { pt-BR: "Situação do chamado", en: "Ticket status" }
  base: state
  initial: Open

  value Open {
    title: { pt-BR: "Aberto", en: "Open" }
    icon: status.open
    ordinal: 10

    transition InProgress {
      event: Started
      title: { pt-BR: "Atendimento iniciado", en: "Ticket started" }
      payload {
        startedAt: DateTime { title: { pt-BR: "Iniciado em", en: "Started at" } }
        dueAt: DateTime { title: { pt-BR: "Prazo do SLA", en: "SLA due at" } }
      }
    }

    transition Resolved {
      title: { pt-BR: "Chamado resolvido", en: "Ticket resolved" }
      payload {
        resolvedAt: DateTime { title: { pt-BR: "Resolvido em", en: "Resolved at" } }
        resolutionNote: string { title: { pt-BR: "Solução", en: "Resolution" } }
      }
    }
  }
  value InProgress {
    title: { pt-BR: "Em atendimento", en: "In progress" }
    icon: status.inProgress
    ordinal: 20

    transition Resolved {
      title: { pt-BR: "Chamado resolvido", en: "Ticket resolved" }
      payload {
        resolvedAt: DateTime { title: { pt-BR: "Resolvido em", en: "Resolved at" } }
        resolutionNote: string { title: { pt-BR: "Solução", en: "Resolution" } }
      }
    }
  }
  value Resolved {
    title: { pt-BR: "Resolvido", en: "Resolved" }
    icon: status.resolved
    ordinal: 30
  }
}

document Ticket {
  title: { pt-BR: "Chamado", en: "Ticket" }
  sync: live

  fields {
    subject: string {
      title: { pt-BR: "Assunto", en: "Subject" }
      maxLength: 160
      lookup
      required
    }
    customer: Customer {
      title: { pt-BR: "Cliente", en: "Customer" }
      lookup
      required
    }
    priority: TicketPriority {
      title: { pt-BR: "Prioridade", en: "Priority" }
      lookup
    }
    status: TicketStatus {
      title: { pt-BR: "Situação", en: "Status" }
      lookup
    }
    assignee: User {
      title: { pt-BR: "Responsável", en: "Assignee" }
    }
    openedAt: DateTime {
      title: { pt-BR: "Aberto em", en: "Opened at" }
      required
    }
    slaWindow: Duration {
      title: { pt-BR: "Janela de SLA", en: "SLA window" }
    }
    dueAt: DateTime {
      title: { pt-BR: "Prazo do SLA", en: "SLA due at" }
      lookup
      computed: openedAt + slaWindow
    }
    startedAt: DateTime {
      title: { pt-BR: "Iniciado em", en: "Started at" }
    }
    resolvedAt: DateTime {
      title: { pt-BR: "Resolvido em", en: "Resolved at" }
    }
    resolutionNote: string {
      title: { pt-BR: "Solução", en: "Resolution" }
      maxLength: 500
    }
  }

  actions {
    start() {
      title: { pt-BR: "Iniciar atendimento", en: "Start work" }
      roles: [Agent]

      to: status.InProgress

      {
        startedAt = now()
      }
    }

    assign(user: User) {
      title: { pt-BR: "Atribuir", en: "Assign" }
      roles: [Agent]

      rule user != null {
        pt-BR: "Selecione o responsável"
        en: "Select the assignee"
      }

      {
        assignee = user
      }
    }

    resolve(note: string) {
      title: { pt-BR: "Resolver", en: "Resolve" }
      roles: [Agent]

      to: status.Resolved

      rule note.length >= 10 {
        pt-BR: "Descreva a solução com pelo menos 10 caracteres"
        en: "Describe the resolution in at least 10 characters"
      }

      {
        resolvedAt = now()
        resolutionNote = note
      }
    }
  }

  indexes {
    byStatusDue: [status, dueAt] range
    byAssignee: [assignee.id] hash
    bySubject: [subject] search
  }

  rule priority == Critical implies slaWindow <= 4h {
    pt-BR: "Chamado crítico exige SLA de até 4 horas"
    en: "A critical ticket requires an SLA of at most 4 hours"
  }
  rule dueAt > openedAt {
    pt-BR: "O prazo do SLA precisa ser posterior à abertura"
    en: "The SLA due date must be after the ticket was opened"
  }
  rule status == Resolved implies resolvedAt != null {
    pt-BR: "Chamado resolvido exige data de resolução"
    en: "A resolved ticket requires a resolution date"
  }
}
```

*Valida: SLA calculado no campo (`computed: openedAt + slaWindow`), prioridade como `classification`, estado `Open`/`InProgress`/`Resolved`, ação `resolve` a partir de dois estados, regra de prazo por prioridade e evento com payload preenchido pelo body.*

### D.6 Solicitação de compra com alçadas

```archol
shape PurchaseRequestStatus {
  title: { pt-BR: "Situação da solicitação", en: "Purchase request status" }
  base: state
  initial: Draft

  value Draft {
    title: { pt-BR: "Rascunho", en: "Draft" }
    icon: status.draft
    ordinal: 10

    transition Submitted {
      title: { pt-BR: "Solicitação enviada", en: "Purchase request submitted" }
      payload {
        submittedAt: DateTime { title: { pt-BR: "Enviada em", en: "Submitted at" } }
        amount: std.Money { title: { pt-BR: "Valor", en: "Amount" } }
      }
    }
  }
  value Submitted {
    title: { pt-BR: "Enviada", en: "Submitted" }
    icon: status.submitted
    ordinal: 20

    transition Approved {
      title: { pt-BR: "Solicitação aprovada", en: "Purchase request approved" }
      payload {
        decidedAt: DateTime { title: { pt-BR: "Decidida em", en: "Decided at" } }
        amount: std.Money { title: { pt-BR: "Valor", en: "Amount" } }
      }
    }

    transition Rejected {
      title: { pt-BR: "Solicitação rejeitada", en: "Purchase request rejected" }
      payload {
        decidedAt: DateTime { title: { pt-BR: "Decidida em", en: "Decided at" } }
        reason: string { title: { pt-BR: "Motivo", en: "Reason" } }
      }
    }
  }
  value Approved {
    title: { pt-BR: "Aprovada", en: "Approved" }
    icon: status.approved
    ordinal: 30
  }
  value Rejected {
    title: { pt-BR: "Rejeitada", en: "Rejected" }
    icon: status.rejected
    ordinal: 40
  }
}

document PurchaseRequest {
  title: { pt-BR: "Solicitação de compra", en: "Purchase request" }
  sync: live

  fields {
    number: PurchaseRequestNumber {
      title: { pt-BR: "Número", en: "Number" }
      format: "{createdAt.year:4}-{sequence:5}"
      sequence: per(CostCenter)
      unique
      lookup
    }
    costCenter: CostCenter {
      title: { pt-BR: "Centro de custo", en: "Cost center" }
      lookup
      required
    }
    requester: User {
      title: { pt-BR: "Solicitante", en: "Requester" }
      lookup
      required
    }
    supplier: Supplier {
      title: { pt-BR: "Fornecedor", en: "Supplier" }
      lookup
    }
    amount: std.Money {
      title: { pt-BR: "Valor", en: "Amount" }
      currency: BRL
      min: 0.01
      lookup
    }
    justification: string {
      title: { pt-BR: "Justificativa", en: "Justification" }
      maxLength: 500
    }
    status: PurchaseRequestStatus {
      title: { pt-BR: "Situação", en: "Status" }
      lookup
    }
    createdAt: DateTime {
      title: { pt-BR: "Criada em", en: "Created at" }
      readonly
    }
    submittedAt: DateTime {
      title: { pt-BR: "Enviada em", en: "Submitted at" }
    }
    decidedAt: DateTime {
      title: { pt-BR: "Decidida em", en: "Decided at" }
    }
    decisionNote: string {
      title: { pt-BR: "Parecer", en: "Decision note" }
      maxLength: 500
    }
  }

  actions {
    submit() {
      title: { pt-BR: "Enviar", en: "Submit" }
      roles: [Requester]

      to: status.Submitted

      rule amount > 0 BRL {
        pt-BR: "Informe o valor da compra"
        en: "Enter the purchase amount"
      }
      rule justification.length >= 20 {
        pt-BR: "Justifique a compra com pelo menos 20 caracteres"
        en: "Justify the purchase in at least 20 characters"
      }

      returns: PurchaseRequest
      {
        submittedAt = now()
      }
    }

    approve() {
      title: { pt-BR: "Aprovar", en: "Approve" }
      roles: [Supervisor, Manager]

      to: status.Approved

      limits {
        amount <= 10000 BRL: {
          roles: [Supervisor]
          pt-BR: "Até R$ 10.000 a aprovação é do supervisor"
          en: "Up to BRL 10,000 the supervisor may approve"
        }
        amount <= 50000 BRL: {
          roles: [Manager]
          pt-BR: "Acima de R$ 10.000 a aprovação exige gerência"
          en: "Above BRL 10,000 approval requires a manager"
        }
      }

      {
        decidedAt = now()
      }
    }

    reject(reason: string) {
      title: { pt-BR: "Rejeitar", en: "Reject" }
      roles: [Supervisor, Manager]

      to: status.Rejected

      rule reason.length >= 10 {
        pt-BR: "Informe um motivo com pelo menos 10 caracteres"
        en: "Enter a reason with at least 10 characters"
      }

      {
        decidedAt = now()
        decisionNote = reason
      }
    }
  }

  segregation {
    submit != approve
  }

  indexes {
    byNumber: [number] unique
    byStatus: [status, submittedAt] range
    byCostCenter: [costCenter.id, submittedAt] range
  }

  rule status == Approved implies decidedAt != null {
    pt-BR: "Solicitação aprovada exige data de decisão"
    en: "An approved request requires a decision date"
  }
  rule status == Rejected implies decisionNote != null {
    pt-BR: "Solicitação rejeitada exige parecer"
    en: "A rejected request requires a decision note"
  }
}
```

*Valida: ciclo `Draft`/`Submitted`/`Approved`/`Rejected` com `from:`/`to:`/`emits:` em cada aresta, `limits` por papel (alçada), `segregation` entre solicitar e aprovar, número de negócio por centro de custo e `lookup` com número, centro de custo e valor.*

### D.7 Contrato com versões, assinatura e renovação

```archol
shape ContractStatus {
  title: { pt-BR: "Situação do contrato", en: "Contract status" }
  base: state
  initial: Draft

  value Draft {
    title: { pt-BR: "Rascunho", en: "Draft" }
    icon: status.draft
    ordinal: 10

    transition InReview {
      event: Submitted
      title: { pt-BR: "Contrato em revisão", en: "Contract in review" }
      payload {
        submittedAt: DateTime { title: { pt-BR: "Enviado em", en: "Submitted at" } }
      }
    }
  }
  value InReview {
    title: { pt-BR: "Em revisão", en: "In review" }
    icon: status.inReview
    ordinal: 20

    transition Active {
      event: Activated
      title: { pt-BR: "Contrato ativado", en: "Contract activated" }
      payload {
        activatedAt: DateTime { title: { pt-BR: "Ativado em", en: "Activated at" } }
        currentVersion: ContractVersion { title: { pt-BR: "Versão vigente", en: "Current version" } }
      }
    }
  }
  value Active {
    title: { pt-BR: "Vigente", en: "Active" }
    icon: status.active
    ordinal: 30

    transition Expiring {
      event: Expired
      title: { pt-BR: "Contrato a vencer", en: "Contract expiring" }
      payload {
        endsAt: DateTime { title: { pt-BR: "Término", en: "End" } }
      }
    }

    transition Terminated {
      title: { pt-BR: "Contrato encerrado", en: "Contract terminated" }
      payload {
        terminatedAt: DateTime { title: { pt-BR: "Encerrado em", en: "Terminated at" } }
        reason: string { title: { pt-BR: "Motivo", en: "Reason" } }
      }
    }
  }
  value Expiring {
    title: { pt-BR: "A vencer", en: "Expiring" }
    icon: status.expiring
    ordinal: 40

    transition Active {
      event: Renewed
      title: { pt-BR: "Contrato renovado", en: "Contract renewed" }
      payload {
        endsAt: DateTime { title: { pt-BR: "Término", en: "End" } }
      }
    }

    transition Terminated {
      title: { pt-BR: "Contrato encerrado", en: "Contract terminated" }
      payload {
        terminatedAt: DateTime { title: { pt-BR: "Encerrado em", en: "Terminated at" } }
        reason: string { title: { pt-BR: "Motivo", en: "Reason" } }
      }
    }
  }
  value Terminated {
    title: { pt-BR: "Encerrado", en: "Terminated" }
    icon: status.terminated
    ordinal: 50
  }
}

shape ContractFile {
  title: { pt-BR: "Arquivo do contrato", en: "Contract file" }
  base: attachment
  mime: [application/pdf]
}

shape ContractVersion {
  title: { pt-BR: "Versão do contrato", en: "Contract version" }
  fields {
    version: integer {
      title: { pt-BR: "Versão", en: "Version" }
      min: 1
    }
    file: ContractFile {
      title: { pt-BR: "Arquivo", en: "File" }
    }
    signedAt: DateTime {
      title: { pt-BR: "Assinado em", en: "Signed at" }
    }
    note: string {
      title: { pt-BR: "Observação", en: "Note" }
      maxLength: 500
    }
  }
}

document Contract {
  title: { pt-BR: "Contrato", en: "Contract" }
  sync: live
  retention: 10y

  fields {
    subject: string {
      title: { pt-BR: "Objeto", en: "Subject" }
      maxLength: 200
      lookup
      required
    }
    supplier: Supplier {
      title: { pt-BR: "Fornecedor", en: "Supplier" }
      lookup
      required
    }
    versions: ContractVersion[] {
      title: { pt-BR: "Versões", en: "Versions" }
    }
    currentVersion: ContractVersion {
      title: { pt-BR: "Versão vigente", en: "Current version" }
    }
    startsAt: DateTime {
      title: { pt-BR: "Início", en: "Start" }
    }
    endsAt: DateTime {
      title: { pt-BR: "Término", en: "End" }
    }
    status: ContractStatus {
      title: { pt-BR: "Situação", en: "Status" }
      lookup
    }
    submittedAt: DateTime {
      title: { pt-BR: "Enviado em", en: "Submitted at" }
    }
    activatedAt: DateTime {
      title: { pt-BR: "Ativado em", en: "Activated at" }
    }
    terminatedAt: DateTime {
      title: { pt-BR: "Encerrado em", en: "Terminated at" }
    }
    terminationReason: string {
      title: { pt-BR: "Motivo do encerramento", en: "Termination reason" }
      maxLength: 500
    }
  }

  signature {
    activate: LegalCounsel
  }

  actions {
    attachVersion(version: ContractVersion) {
      title: { pt-BR: "Anexar versão", en: "Attach version" }
      roles: [LegalCounsel]

      rule version.file != null {
        pt-BR: "A versão precisa ter o arquivo assinado"
        en: "The version must have the signed file"
      }

      {
        currentVersion = version
      }
    }

    submitReview() {
      title: { pt-BR: "Enviar para revisão", en: "Submit for review" }
      roles: [LegalCounsel]

      to: status.InReview

      rule currentVersion != null {
        pt-BR: "Anexe uma versão antes de enviar para revisão"
        en: "Attach a version before submitting for review"
      }

      {
        submittedAt = now()
      }
    }

    activate(version: ContractVersion) {
      title: { pt-BR: "Ativar", en: "Activate" }
      roles: [LegalCounsel]

      to: status.Active
      emits { currentVersion: version }

      rule version.signedAt != null {
        pt-BR: "A versão vigente precisa estar assinada"
        en: "The current version must be signed"
      }

      returns: Contract
      {
        currentVersion = version
        activatedAt = now()
      }
    }

    expire() {
      title: { pt-BR: "Marcar a vencer", en: "Mark as expiring" }
      roles: [System]

      to: status.Expiring
    }

    renew(until: DateTime) {
      title: { pt-BR: "Renovar", en: "Renew" }
      roles: [Manager]

      to: status.Active

      rule until > endsAt {
        pt-BR: "A renovação precisa estender o término"
        en: "The renewal must extend the end date"
      }

      {
        endsAt = until
      }
    }

    terminate(reason: string) {
      title: { pt-BR: "Encerrar", en: "Terminate" }
      roles: [LegalCounsel, Manager]

      to: status.Terminated

      rule reason.length >= 20 {
        pt-BR: "Informe o motivo do encerramento com pelo menos 20 caracteres"
        en: "Enter the termination reason in at least 20 characters"
      }

      {
        terminatedAt = now()
        terminationReason = reason
      }
    }
  }

  indexes {
    byStatusEndsAt: [status, endsAt] range
    bySupplier: [supplier.id] hash
    bySubject: [subject] search
  }

  rule endsAt > startsAt {
    pt-BR: "O término do contrato precisa ser posterior ao início"
    en: "The contract end must be after its start"
  }
  rule status == Terminated implies terminationReason != null {
    pt-BR: "Contrato encerrado exige motivo"
    en: "A terminated contract requires a reason"
  }
}
```

*Valida: `ContractVersion` como Shape composto (versões), bloco `signature` na ativação, ciclo `Draft` → `InReview` → `Active` → `Expiring` → `Active`/`Terminated` com arestas declaradas nas actions, `expire` automático por papel `System` e renovação que estende `endsAt`.*

### D.8 Receita mensal como documento derivado

```archol
document MonthlyRevenue {
  title: { pt-BR: "Receita mensal", en: "Monthly revenue" }
  sync: read
  retention: 10y
  computed from Invoice, Payment
  freshness: 1h
  exposes: [revenue, calculatedAt]

  fields {
    period: YearMonth {
      title: { pt-BR: "Período", en: "Period" }
      lookup
      required
    }
    revenue: std.Money {
      title: { pt-BR: "Receita", en: "Revenue" }
      currency: BRL
      lookup
    }
    orderCount: std.Count {
      title: { pt-BR: "Pedidos pagos", en: "Paid orders" }
    }
  }

  indexes {
    byPeriod: [period] unique
  }

  actions {
    refresh() {
      title: { pt-BR: "Recalcular", en: "Recalculate" }
      roles: [Manager]
    }
  }

  rule revenue >= 0 BRL {
    pt-BR: "A receita mensal não pode ser negativa"
    en: "Monthly revenue cannot be negative"
  }
}
```

*Valida: documento derivado (`computed from` + `freshness` + `exposes`) com período e receita duráveis, leitura materializada (`sync: read`), `calculatedAt` implícito no frescor e comando `refresh()` para recálculo — o número que precisa durar é documento, não relatório.*
## Shape

### H.1 Dinheiro com moeda e escala

```archol
shape PriceTag {
  title: { pt-BR: "Etiqueta de preço", en: "Price tag" }
  fields {
    amount: std.Money {
      title: { pt-BR: "Valor", en: "Amount" }
      currency: BRL
      scale: 2
    }
  }
}
```

*Valida: dinheiro é valor + moeda; escala vem da moeda ou do field.*

### H.2 Massa via std

```archol
shape Package {
  title: { pt-BR: "Pacote", en: "Package" }
  fields {
    weight: std.Mass { title: { pt-BR: "Peso", en: "Weight" } }
    volume: std.Volume { title: { pt-BR: "Volume", en: "Volume" } }
  }
}
```

*Valida: Shape de negócio usando `std.Mass`/`std.Volume`; `1 kg` não é `1 m`.*

### H.3 Comprimento via std

```archol
shape ProductDimensions {
  title: { pt-BR: "Dimensões do produto", en: "Product dimensions" }
  fields {
    length: std.Length { title: { pt-BR: "Comprimento", en: "Length" } }
    width: std.Length { title: { pt-BR: "Largura", en: "Width" } }
    height: std.Length { title: { pt-BR: "Altura", en: "Height" } }
  }
}
```

*Valida: Shape de negócio compondo `std.Length` para dimensões.*

### H.4 Temperatura via std

```archol
shape TemperatureReading {
  title: { pt-BR: "Leitura de temperatura", en: "Temperature reading" }
  fields {
    value: std.Temperature { title: { pt-BR: "Valor", en: "Value" } }
  }
}
```

*Valida: temperatura como tipo padrão da stdlib.*

### H.5 Percentual via std

```archol
shape DiscountPolicy {
  title: { pt-BR: "Política de desconto", en: "Discount policy" }
  fields {
    rate: std.Percentage { title: { pt-BR: "Taxa", en: "Rate" } }
  }
}
```

*Valida: percentual como tipo padrão, usado por um Shape de política de desconto.*

### H.6 CPF com validação e sensibilidade

```archol
shape CPF {
  title: { pt-BR: "CPF", en: "CPF" }
  base: string
  normalize: digits
  validate: cpf-check-digits
  format: masked("###.###.###-##")
  sensitive: true
  index: hash
}
```

*Valida: validação/normalização/formatação no Shape; o campo não carrega regex.*

### H.7 E-mail com regra de unicidade

```archol
shape Email {
  title: { pt-BR: "Correio eletrônico", en: "Email" }
  base: string
  normalize: lower-trim
  validate: email
  maxLength: 254
  format: plain
  index: exact
}
```

*Valida: um Shape pequeno e reutilizável, com normalização antes da unicidade.*

### H.8 Endereço composto

```archol
shape Address {
  title: { pt-BR: "Endereço", en: "Address" }
  fields {
    street: string { title: { pt-BR: "Rua", en: "Street" } }
    number: string { title: { pt-BR: "Número", en: "Number" } }
    complement: string { title: { pt-BR: "Complemento", en: "Complement" } }
    district: string { title: { pt-BR: "Bairro", en: "District" } }
    city: string { title: { pt-BR: "Cidade", en: "City" } }
    state: StateCode { title: { pt-BR: "Estado", en: "State" } }
    zip: CEP { title: { pt-BR: "CEP", en: "ZIP" } }
    country: CountryCode { title: { pt-BR: "País", en: "Country" }, default: BR }
  }

  index: [city, state]

  format(value, flags) {
    const locale = std.regionalization.current();
    const complement = value.complement ?? "";
    if (locale == "pt-BR") {
      return value.street + ", " + value.number + " " + complement + " - " + value.district + ", " + value.city + " - " + value.state + ", " + value.zip + ", " + value.country;
    }
    return value.number + " " + value.street + " " + complement + ", " + value.city + ", " + value.state + " " + value.zip + ", " + value.country;
  }
}
```

*Valida: Shape composto, default, índice e `format` customizado por regionalization.*

### H.9 Métodos com tipos explícitos e inferidos

```archol
shape Distance {
  title: { pt-BR: "Distância", en: "Distance" }
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
    km: {
      factor: 1000
      title: {
        pt-BR: { one: "quilômetro", other: "quilômetros" }
        en: { one: "kilometer", other: "kilometers" }
      }
    }
  }

  toNumber(value) {
    return value.raw;
  }

  toNumberExplicit(value: Distance): number {
    return value.raw;
  }

  format(value, flags) {
    return formatReal(value.raw, m, flags.unit ?? m, flags.style, flags.unitStyle);
  }
}
```

*Valida: argumentos obrigatórios; tipo e retorno podem ser explícitos ou inferidos.*

## bpcms.org/std

A biblioteca padrão é pseudo-serviço: precisa ser importada e usada com alias.

```text
imports {
  service "bpcms.org/std" as std
}

fields {
  weight: std.Mass
  length: std.Length
  volume: std.Volume
  discount: std.Percentage
  currency: std.Currency
}
```

Ela define 57 Shapes de medida/ciência/estatística, com unidades SI, imperial/US, CGS, astronômicas, atômicas, químicas, dados e estatística. `std.Currency` é a classificação ISO 4217, completada por `reintroduce`.

## View

### V.1 Tabela de pedidos com filtros, paginação e comandos

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

*Valida: tabela operacional com colunas herdadas do documento (`customer` traz o lookup), filtros por período e situação, página de 50 linhas e ações de linha que navegam (`open`) ou delegam o comando ao documento (`cancel`), que valida a transição.*

### V.2 Detalhe da remessa com seções, cálculo e dispositivo

```archol
view ShipmentDetail {
  title: { pt-BR: "Detalhe da remessa", en: "Shipment detail" }
  type: detail
  document: Shipment
  roles: [Dispatcher, Supervisor]

  sections { summary; items; timeline; documents; deviceReading }
  computed { freight }
  device { scale-01 }

  actions {
    openOrder -> OrderDetail
    dispatch -> Shipment.dispatch(code)
    confirmDelivery -> Shipment.confirmDelivery(deliveredOn)
  }
}
```

*Valida: ficha com seções, campo calculado de exibição (`computed`), leitura de dispositivo da balança, navegação para o pedido e comandos de transição — a View lê, quem grava é a action.*

### V.3 Relatório de vendas por região

```archol
view SalesByRegion {
  title: { pt-BR: "Vendas por região", en: "Sales by region" }
  type: report
  sources: [Invoice, Order]
  roles: [Manager, Analyst]

  filters { period; region; status }
  metrics {
    revenue: sum(Invoice.total) by region
    orders: count(Order) by region
    ticket: avg(Invoice.total) by region
    cancelled: count(Invoice) where status == Cancelled by region
  }
  units { revenue: BRL; ticket: BRL }
  freshness: 15m
  export: [pdf, csv]
}
```

*Valida: relatório como View (não documento) sobre duas fontes, métricas de agregação pura com `by` e `where`, unidade de exibição por métrica, frescor do cache de leitura e exportação — número agregado para ler, filtrar e exportar.*

### V.4 Dashboard operacional em tempo real

```archol
view OperationsControl {
  title: { pt-BR: "Controle operacional", en: "Operations control" }
  type: dashboard
  sources: [Shipment, Incident, PurchaseRequest]
  roles: [Supervisor]
  mode: live

  filters { site; carrier; status }
  metrics {
    late: count(Shipment) where status != Delivered and promisedAt < now()
    openIncidents: count(Incident) where status != Closed
    pendingApproval: count(PurchaseRequest) where status == Submitted
    pendingAmount: sum(PurchaseRequest.amount) where status == Submitted
  }
  units { pendingAmount: BRL }
  freshness: 1m
}
```

*Valida: painel com `mode: live`, métricas filtradas por `where` (atraso comparando o prazo prometido com `now()`), contagem por situação e soma monetária com unidade — painel em tempo real, sem documento de relatório.*

### V.5 Kanban de atendimento

```archol
view SupportKanban {
  title: { pt-BR: "Atendimento por situação", en: "Support board by status" }
  type: kanban
  source: Ticket
  roles: [Agent, Lead]
  groupBy: status

  card { subject; priority; dueAt; assignee }
  filter { assignee = currentUser }

  actions {
    open -> TicketDetail
    assign -> Ticket.assign(user)
    resolve -> Ticket.resolve(note)
  }
}
```

*Valida: quadro agrupado pelo campo de estado (`groupBy: status`), cartão com sujeito, prioridade, prazo de SLA e responsável, filtro contextual pelo usuário atual e ações de linha que navegam ou chamam a action do documento.*

### V.6 Inbox de minhas tarefas

```archol
view MyWork {
  title: { pt-BR: "Minhas tarefas", en: "My work" }
  type: inbox
  roles: [Clerk, Manager, SupportAgent]
  scope: mine
  groupBy: process

  columns { title; subject; role; dueAt; priority }
}
```

*Valida: inbox como projeção de tarefas (não de documentos), `scope: mine` para o que está atribuído ou pegável, agrupamento por processo e colunas derivadas do passo, da role e do prazo.*

### V.7 Andamento do processo de pedido

```archol
view OrderProgress {
  title: { pt-BR: "Andamento do pedido", en: "Order progress" }
  type: process
  process: OrderFulfillment
  roles: [Clerk, Manager, Customer]

  sections {
    current
    participants
    history
    deadlines
    subprocesses
    documents
    milestones
    next
  }
}
```

*Valida: `type: process` com `process:` obrigatório e `sections` no vocabulário fixo do contrato de progresso (4.13/7.4); a View mostra o andamento, não cria transição nem reatribui.*

### V.8 Calendário de visitas

```archol
view VisitCalendar {
  title: { pt-BR: "Agenda de visitas", en: "Visit calendar" }
  type: calendar
  source: Visit
  roles: [FieldAgent, Supervisor]

  date: scheduledAt
  filters { technician; status; region }

  actions {
    open -> VisitDetail
    reschedule -> Visit.reschedule(scheduledAt)
  }
}
```

*Valida: agenda por data (`date: scheduledAt`) sobre o documento `Visit`, filtros por técnico, situação e região, navegação para o detalhe e reagendamento como comando do documento.*
## Role / Organogram

### R.1 Papéis de loja

```archol
role Customer {
  scopes: [Store]
  title: { pt-BR: "Cliente", en: "Customer" }
  permissions: [catalog.view, order.create, order.viewOwn]
}
role Clerk {
  scopes: [Store]
  title: { pt-BR: "Atendente", en: "Clerk" }
  inherits: Customer
  permissions: [order.manage, customer.view]
}
role Manager {
  scopes: [Store]
  title: { pt-BR: "Gerente", en: "Manager" }
  inherits: Clerk
  permissions: [customer.manage, report.view, refund.approve]
}
organogram Store {
  title: { pt-BR: "Organograma da loja", en: "Store organogram" }
  hierarchy {
    Manager > Clerk > Customer
  }
  scopes: [Store]
}
```

*Valida: herança de permissão, escopo e hierarquia simples.*

### R.2 Papéis de clínica com quatro-olhos

```archol
role Reception {
  title: { pt-BR: "Recepção", en: "Reception" }
  permissions: [patient.create, appointment.schedule]
}
role Nurse {
  title: { pt-BR: "Enfermeiro", en: "Nurse" }
  inherits: Reception
  permissions: [exam.collect, patient.viewClinical]
}
role Doctor {
  title: { pt-BR: "Médico", en: "Doctor" }
  inherits: Nurse
  permissions: [prescription.create, exam.request]
}
role Manager {
  title: { pt-BR: "Gerente", en: "Manager" }
  inherits: Doctor
  permissions: [billing.view, report.view]
}
organogram Clinic {
  title: { pt-BR: "Organograma da clínica", en: "Clinic organogram" }
  hierarchy {
    Manager > Doctor > Nurse > Reception
  }
  fourEyes { prescription.create requires Doctor }
}
```

*Valida: segregação de função clínica e exigência de um segundo ator.*

### R.3 Papéis de armazém com segregação

```archol
role Picker {
  title: { pt-BR: "Separador", en: "Picker" }
  permissions: [shipment.pick]
}
role Checker {
  title: { pt-BR: "Conferente", en: "Checker" }
  permissions: [shipment.check, shipment.reject]
}
role Supervisor {
  title: { pt-BR: "Supervisor", en: "Supervisor" }
  permissions: [shipment.forceRelease, report.view]
}
organogram Warehouse {
  title: { pt-BR: "Organograma do armazém", en: "Warehouse organogram" }
  hierarchy {
    Supervisor > Checker
    Supervisor > Picker
  }
  segregation { shipment.pick != shipment.check }
  Supervisor delegates to Checker when absent for 7d
}
```

*Valida: quem separa não confere; delegação explícita.*

### R.4 Papéis financeiros com four-eyes

```archol
role Analyst {
  title: { pt-BR: "Analista", en: "Analyst" }
  permissions: [invoice.create, payment.request]
}
role Approver {
  title: { pt-BR: "Aprovador", en: "Approver" }
  permissions: [payment.approve, invoice.cancel]
}
role Controller {
  title: { pt-BR: "Controlador", en: "Controller" }
  permissions: [report.view, audit.view, limit.exceed]
}
organogram Finance {
  title: { pt-BR: "Organograma financeiro", en: "Finance organogram" }
  hierarchy {
    Controller > Approver > Analyst
  }
  fourEyes { payment.approve requires Approver }
}
```

*Valida: four-eyes e segregação de função; limites e mascaramento ficam para Document.*

### R.5 Papéis de suporte com delegação

```archol
role Agent {
  title: { pt-BR: "Agente", en: "Agent" }
  permissions: [ticket.view, ticket.comment, ticket.assign]
}
role Lead {
  title: { pt-BR: "Líder", en: "Lead" }
  inherits: Agent
  permissions: [ticket.escalate, report.view]
}
role Admin {
  title: { pt-BR: "Administrador", en: "Admin" }
  inherits: Lead
  permissions: [user.manage, ticket.delete]
}
organogram Support {
  title: { pt-BR: "Organograma de suporte", en: "Support organogram" }
  hierarchy {
    Admin > Lead > Agent
  }
  Lead delegates to Agent when absent for 7d
}
```

*Valida: herança e delegação; mascaramento fica para Document.*

### R.6 Papéis de laboratório com assinatura

```archol
role Collector {
  title: { pt-BR: "Coletador", en: "Collector" }
  scopes: [Laboratory]
  permissions: [sample.collect, exam.viewOwn]
}
role Analyst {
  title: { pt-BR: "Analista", en: "Analyst" }
  permissions: [exam.result, exam.viewClinical]
}
role Pathologist {
  scopes: [Laboratory]
  title: { pt-BR: "Patologista", en: "Pathologist" }
  permissions: [exam.review, exam.release]
}
organogram Lab {
  title: { pt-BR: "Organograma do laboratório", en: "Laboratory organogram" }
  hierarchy {
    Pathologist > Analyst > Collector
  }
  fourEyes { exam.release requires Pathologist }
  signature { Pathologist }
}
```

*Valida: autoridade de assinatura, revisão por outro ator e hierarquia.*

### R.7 Laboratório com apoio e amostras

```archol
role Collector {
  title: { pt-BR: "Coletador", en: "Collector" }
  scopes: [Laboratory]
  permissions: [
    sample.collect.hemogram,
    sample.collect.glucose,
    exam.perform.hemogram
  ]
}

role Analyst {
  scopes: [Laboratory]
  title: { pt-BR: "Analista", en: "Analyst" }
  inherits: Collector
  permissions: [
    sample.collect.culture,
    sample.collect.hba1c,
    exam.perform.culture,
    exam.perform.hba1c
  ]
}

role Pathologist {
  scopes: [Laboratory]
  title: { pt-BR: "Patologista", en: "Pathologist" }
  permissions: [exam.review, exam.release, result.release]
}

role SupportLiaison {
  scopes: [Laboratory]
  title: { pt-BR: "Ligação com apoio", en: "Support liaison" }
  permissions: [sample.receive, support.refer]
}

role LabManager {
  scopes: [Laboratory]
  title: { pt-BR: "Gerente do laboratório", en: "Laboratory manager" }
  inherits: Pathologist
  permissions: [lab.manage, support.refer]
}

organogram Laboratory {
  title: { pt-BR: "Organograma do laboratório", en: "Laboratory organogram" }
  scopes: [Laboratory]

  hierarchy {
    LabManager > Pathologist > Analyst > Collector
    LabManager > SupportLiaison
  }

  LabManager delegates to Pathologist when absent for 7d

  segregation { exam.perform != exam.release }
  signature { Pathologist }
}
```

*Valida: coletor que também executa alguns exames, analista com escopo maior, apoio externo e assinatura.*

No serviço `laboratory`, `Sample.kind` distingue `patient`, `support` e `control`. Amostras de apoio e controle não referenciam paciente; usam `supportLab`/`controlLot`.

O `LabManager` pode atribuir `Collector`, `Analyst` e `SupportLiaison` a outras pessoas, mas nunca a si mesmo. Quem recebe a role ganha as permissões da role atribuída, não as permissões do gerente; o gerente não precisa ter as permissões da role gerenciada.

```archol
// managerUser/collectorUser/regionalUser: User
// laboratoryMain/laboratorySupport: Laboratory — referências a documentos
assignRole(managerUser, LabManager, scopes: {
  Laboratory: {all}
})

assignRole(collectorUser, Collector, scopes: {
  Laboratory: {only: [laboratoryMain]}
})

assignRole(regionalUser, LabManager, scopes: {
  Laboratory: {except: [laboratoryMain]}
})
```

### R.8 Herança múltipla de roles

```archol
role Collector {
  title: { pt-BR: "Coletador", en: "Collector" }
  scopes: [Laboratory]
  permissions: [sample.collect, sample.view]
}

role Auditor {
  title: { pt-BR: "Auditor", en: "Auditor" }
  scopes: [Laboratory]
  permissions: [audit.view]
}

role Analyst {
  title: { pt-BR: "Analista", en: "Analyst" }
  scopes: [Laboratory]
  inherits: [Collector, Auditor]
  permissions: [exam.result, exam.review]
}

organogram Laboratory {
  title: { pt-BR: "Organograma do laboratório", en: "Laboratory organogram" }
  scopes: [Laboratory]

  hierarchy {
    Analyst > Collector
  }
}
```

*Valida: permissões próprias + permissões herdadas de múltiplas roles, mantendo o mesmo escopo.*

# Descoberta

## Evidence

### E.1 Formulário de compra em PDF

```archol
evidence PurchaseFormPDF {
  kind: form
  file: "evidence/purchase-form.pdf"
  hash: "sha256:7f3a..."
  confidentiality: internal
  extracted {
    document PurchaseRequest {
      fields { costCenter; amount; justification; approver }
    }
  }
  links { process PurchaseApproval.ReviewRequest }
  status: proposed
}
```

*Valida: evidência com campos extraídos, confidencialidade, vínculo com o processo e status de revisão.*

### E.2 DDL do ERP

```archol
evidence ERP_DDL {
  kind: ddl
  file: "evidence/erp.sql"
  hash: "sha256:9c1b..."
  confidentiality: internal

  extracted {
    document Supplier {
      fields { id; name; citizen; status }
    }
    document PurchaseOrder {
      fields { id; supplier; total; status; createdAt }
    }
  }
  links { service "example.com/services/billing" }
  status: proposed
}
```

*Valida: importação de DDL como proposta de documentos, sem auto-aceitar o schema.*

### E.3 MER da clínica

```archol
evidence ClinicMER {
  kind: mer
  file: "evidence/clinic-mer.pdf"
  confidentiality: restricted

  extracted {
    document Patient {
      fields { id; name; birthDate; document }
    }
    document Appointment {
      fields { id; patient; doctor; startsAt; status }
    }
    relation Appointment.patient -> Patient
  }
  status: proposed
}
```

*Valida: MER vira documentos + relações, com dados sensíveis marcados.*

### E.4 Protótipo no Figma

```archol
evidence CheckoutFigma {
  kind: figma
  file: "https://figma.com/file/checkout-v3"
  confidentiality: internal

  extracted {
    view CheckoutWizard {
      steps { Cart; Address; Payment; Review }
      integration: Payment
    }
  }
  links { process Checkout }
  status: confirmed
}
```

*Valida: Figma como evidência de view, não como fonte de verdade visual.*

### E.5 Print de sistema legado

```archol
evidence LegacyOrderScreen {
  kind: screenshot
  file: "evidence/order-screen.png"
  confidentiality: internal

  extracted {
    view OrderList {
      columns { number; customer; total; status }
      filters { status; period }
    }
  }
  links { view OrdersTable }
  status: needsReview
}
```

*Valida: print vira candidato de view, com status explícito de revisão.*

### E.6 Rascunho à mão do despacho

```archol
evidence HandDrawnDispatch {
  kind: handdrawn
  file: "evidence/dispatch-sketch.jpg"
  confidentiality: internal

  extracted {
    process OrderFulfillment {
      steps { pick; pack; weigh; quote; label; dispatch }
      actors { Picker; Packer; Carrier }
    }
  }
  links { process OrderFulfillment }
  status: proposed
}
```

*Valida: desenho à mão entra como proposta de processo, não como BPMN final.*

## Conversation

### C.1 Descoberta de compras

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
    documents: proposed
  }
  assumptions: [A-001, A-002]
  status: in_progress
  next: "Confirmar alçada do diretor"
}
```

*Valida: sessão com cobertura parcial, evidências e próxima pergunta explícita.*

### C.2 Triagem clínica com dados sensíveis

```archol
conversation ClinicIntake {
  process: PatientIntake
  goal: "Entender acolhimento e triagem"

  participants { Reception; Nurse; Doctor }
  evidence: [ClinicMER, TriageFormPDF]
  confidentiality: restricted
  coverage {
    goal: confirmed
    actors: confirmed
    clinicalRules: unknown
    consent: confirmed
  }
  assumptions: [A-004]
  status: in_progress
  next: "Validar regra de prioridade clínica"
}
```

*Valida: sessão com dado sensível, regra clínica ainda em aberto e consentimento confirmado.*

### C.3 Expedição com operador de campo

```archol
conversation LogisticsExpedition {
  process: OrderFulfillment
  goal: "Entender o despacho real, incluindo balança"

  participants { Dispatcher; Picker; Supervisor }
  evidence: [HandDrawnDispatch, ScaleManualPDF]
  coverage {
    goal: confirmed
    device: confirmed
    exceptions: proposed
    sla: unknown
  }
  assumptions: [A-001, A-005]
  status: ready_for_review
  next: "Simular pedido de 42 kg"
}
```

*Valida: sessão que combina processo, dispositivo e simulação como próximo passo.*

### C.4 Onboarding com RH e TI

```archol
conversation OnboardingHR {
  process: EmployeeOnboarding
  goal: "Mapear onboarding até o primeiro dia"

  participants { HR; IT; Manager }
  evidence: [OnboardingChecklistPDF]
  sessions: 3
  coverage {
    goal: confirmed
    documents: confirmed
    accessRules: unknown
    equipment: proposed
  }
  assumptions: [A-006]
  status: in_progress
  next: "Entrevistar TI sobre provisionamento"
}
```

*Valida: sessão multi-ator, com várias reuniões e cobertura por área.*

### C.5 Sinistro com análise de contrato

```archol
conversation ClaimsReview {
  process: InsuranceClaim
  goal: "Entender análise e decisão de cobertura"

  participants { Adjuster; Analyst; Supervisor }
  evidence: [PolicyPDF, ClaimFormPDF]
  coverage {
    goal: confirmed
    coverageRules: proposed
    fraudRules: unknown
    payment: confirmed
  }
  assumptions: [A-003]
  status: in_progress
  next: "Revisar cláusula de exclusão"
}
```

*Valida: sessão com regra de negócio extraída de contrato e decisão de cobertura.*

### C.6 War room de incidente

```archol
conversation IncidentWarRoom {
  process: IncidentManagement
  goal: "Entender triagem, escalonamento e pós-mortem"

  participants { OnCall; SRE; Support; Manager }
  evidence: [IncidentTimelineCSV, RunbookPDF]
  coverage {
    goal: confirmed
    severity: confirmed
    communication: proposed
    postmortem: unknown
  }
  assumptions: [A-005]
  status: in_progress
  next: "Validar gatilho de SEV2"
}
```

*Valida: sessão operacional com timeline, runbook e regra de severidade.*

## Decision

Decisão é uma **tabela DMN pura** (4.17 / 11): `returns` tipado (normalmente uma `classification`), `inputs` tipados e `cases` com `else` obrigatório. O processo consome com `Decision.evaluate({...})` e roteia com `if/else`. Decisão não escreve documento, não chama integração e não emite evento. Não existe `appliesTo`, `when` nem `default`: o `else` cobre o resto.

### N.1 Alçada de compra

```archol
shape ApproverTier {
  title: { pt-BR: "Alçada de aprovação", en: "Approver tier" }
  base: classification

  value auto {
    title: { pt-BR: "Automática", en: "Automatic" }
    icon: approval.auto
    ordinal: 10
  }
  value manager {
    title: { pt-BR: "Gerência", en: "Manager" }
    icon: approval.manager
    ordinal: 20
  }
  value director {
    title: { pt-BR: "Diretoria", en: "Director" }
    icon: approval.director
    ordinal: 30
  }
}

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

tier = ApprovalLimit.evaluate({
  amount: purchaseRequest.amount,
  supplier: purchaseRequest.supplier
})

if (tier == Auto) {
  purchaseRequest.approve()
} else if (tier == Manager) {
  ManagerReview.show({ purchaseRequest })
} else {
  DirectorReview.show({ purchaseRequest })
}
```

*Valida: `returns` como `classification` (valores `auto`/`manager`/`director` com `icon` e `ordinal`), `inputs` tipados, `else` obrigatório, e o processo consumindo a decisão com `evaluate` + `if/else`.*

### N.2 Elegibilidade de reembolso

```archol
shape RefundVerdict {
  title: { pt-BR: "Decisão de reembolso", en: "Refund verdict" }
  base: classification

  value approve {
    title: { pt-BR: "Aprovar", en: "Approve" }
    icon: refund.approve
    ordinal: 10
  }
  value partial {
    title: { pt-BR: "Aprovar parcial", en: "Partially approve" }
    icon: refund.partial
    ordinal: 20
  }
  value reject {
    title: { pt-BR: "Rejeitar", en: "Reject" }
    icon: refund.reject
    ordinal: 30
  }
}

decision RefundEligibility {
  title: { pt-BR: "Elegibilidade de reembolso", en: "Refund eligibility" }
  returns: RefundVerdict

  inputs {
    request: ReturnRequest
    amount: std.Money
  }

  cases {
    request.reason == damaged and request.daysSinceDelivery <= 30 -> approve
    amount <= 100 BRL -> approve
    request.reason == damaged -> partial
    else -> reject
  }

}
```

*Valida: `DocumentRef` em `inputs`, leitura de campo do documento (`request.reason`) na condição, primeira condição verdadeira vence e `else` cobrindo o resto.*

### N.3 Roteamento de chamado

```archol
shape SupportQueue {
  title: { pt-BR: "Fila de atendimento", en: "Support queue" }
  base: classification

  value sre {
    title: { pt-BR: "Plantão SRE", en: "SRE on-call" }
    icon: support.sre
    ordinal: 10
  }
  value billing {
    title: { pt-BR: "Financeiro", en: "Billing" }
    icon: support.billing
    ordinal: 20
  }
  value technical {
    title: { pt-BR: "Técnico", en: "Technical" }
    icon: support.technical
    ordinal: 30
  }
  value general {
    title: { pt-BR: "Geral", en: "General" }
    icon: support.general
    ordinal: 40
  }
}

decision TicketRouting {
  title: { pt-BR: "Roteamento de chamado", en: "Ticket routing" }
  returns: SupportQueue

  inputs {
    ticket: SupportTicket
  }

  cases {
    ticket.priority == P1 -> sre
    ticket.category == billing -> billing
    ticket.category == technical -> technical
    else -> general
  }

}
```

*Valida: roteamento por prioridade e categoria, com prioridade vencendo categoria (ordem dos `cases`) e fila final obrigatória.*

### N.4 Reposição de estoque

```archol
shape ReplenishmentAction {
  title: { pt-BR: "Ação de reposição", en: "Replenishment action" }
  base: classification

  value replenish {
    title: { pt-BR: "Repor", en: "Replenish" }
    icon: stock.replenish
    ordinal: 10
  }
  value partial {
    title: { pt-BR: "Repor parcial", en: "Partially replenish" }
    icon: stock.partial
    ordinal: 20
  }
  value backorder {
    title: { pt-BR: "Pedido pendente", en: "Backorder" }
    icon: stock.backorder
    ordinal: 30
  }
  value cancel {
    title: { pt-BR: "Cancelar item", en: "Cancel item" }
    icon: stock.cancel
    ordinal: 40
  }
}

decision ReplenishmentPolicy {
  title: { pt-BR: "Reposição de estoque", en: "Stock replenishment" }
  returns: ReplenishmentAction

  inputs {
    available: std.Count
    requested: std.Count
    backorderAllowed: boolean
  }

  cases {
    available >= requested -> replenish
    available > 0 -> partial
    backorderAllowed -> backorder
    else -> cancel
  }

}
```

*Valida: decisão com `std.Count` e `boolean`, cobrindo estoque suficiente, parcial, pendente e cancelamento — sem ação (a `action` que repõe é da task, não da decisão).*

### N.5 Risco de fraude

```archol
shape FraudBand {
  title: { pt-BR: "Faixa de risco", en: "Fraud band" }
  base: classification

  value low {
    title: { pt-BR: "Baixo", en: "Low" }
    icon: risk.low
    ordinal: 10
  }
  value medium {
    title: { pt-BR: "Médio", en: "Medium" }
    icon: risk.medium
    ordinal: 20
  }
  value high {
    title: { pt-BR: "Alto", en: "High" }
    icon: risk.high
    ordinal: 30
  }
}

decision FraudRisk {
  title: { pt-BR: "Risco de fraude", en: "Fraud risk" }
  returns: FraudBand

  inputs {
    score: std.Count
    amount: std.Money
    customer: Customer
  }

  cases {
    score >= 70 or amount > 20000 BRL -> High
    score >= 30 -> Medium
    else -> Low
  }

}
```

*Valida: condição com `or` e limiar monetário, faixas crescentes de score e `DocumentRef` (`customer`) como entrada.*

### N.6 Scoring de crédito

```archol
shape CreditGrade {
  title: { pt-BR: "Faixa de crédito", en: "Credit grade" }
  base: classification

  value prime {
    title: { pt-BR: "Prime", en: "Prime" }
    icon: credit.prime
    ordinal: 10
  }
  value standard {
    title: { pt-BR: "Padrão", en: "Standard" }
    icon: credit.standard
    ordinal: 20
  }
  value subprime {
    title: { pt-BR: "Subprime", en: "Subprime" }
    icon: credit.subprime
    ordinal: 30
  }
  value declined {
    title: { pt-BR: "Recusado", en: "Declined" }
    icon: credit.declined
    ordinal: 40
  }
}

decision CreditScoring {
  title: { pt-BR: "Scoring de crédito", en: "Credit scoring" }
  returns: CreditGrade

  inputs {
    score: std.Count
    overdueDays: std.Count
    monthlyIncome: std.Money
  }

  cases {
    overdueDays > 90 -> declined
    score >= 750 and overdueDays == 0 -> prime
    score >= 600 -> standard
    else -> subprime
  }

}
```

*Valida: `classification` de crédito com exceção primeiro (inadimplência grave), faixa `prime` exigindo score e histórico, e `else` como faixa residual.*
## Assumption

### U.1 Frete por remessa

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

*Valida: suposição com impacto, dono, prazo e vínculo com o processo.*

### U.2 Limite único de transportadora

```archol
assumption A-002 {
  statement: "Todas as transportadoras usam o mesmo limite de peso"
  impact: high
  status: rejected
  evidence: "Tabela de Frete 2026 mostra limites diferentes"
  links { decision WeightLimit }
}
```

*Valida: suposição rejeitada com evidência que a derruba.*

### U.3 Nota fiscal preserva os dados do cliente na emissão

```archol
assumption A-003 {
  statement: "A nota fiscal mantém os dados do cliente como estavam na emissão"
  impact: high
  status: confirmed
  evidence: "Contrato e requisito fiscal"
  links { document Invoice.customer }
}
```

*Valida: suposição confirmada que vira requisito de materialização do lookup na emissão.*

### U.4 Assinatura do laudo

```archol
assumption A-004 {
  statement: "O laudo só pode ser liberado com assinatura do patologista"
  impact: high
  status: open
  validate: "Validar com conselho/responsável técnico"
  due: 2026-11-25
  links { process LabExamFlow.Release }
}
```

*Valida: suposição regulatória com prazo e vínculo com a tarefa de liberação.*

### U.5 Retry de pagamento

```archol
assumption A-005 {
  statement: "Falha de banco é transitória e pode ser retentada 3 vezes"
  impact: medium
  status: proposed
  validate: "Medir falhas por 30 dias"
  links { process InsuranceClaim.RetryPayment }
}
```

*Valida: suposição operacional ligada a retry e política de erro.*

### U.6 Retenção de evidências

```archol
assumption A-006 {
  statement: "Evidências de onboarding podem ser descartadas após 2 anos"
  impact: medium
  status: confirmed
  evidence: "Política de Retenção 2026"
  links { evidence OnboardingChecklistPDF }
}
```

*Valida: suposição de retenção com evidência e vínculo à evidência.*

# Plataforma

## Environment

### N.1 Desenvolvimento local

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

*Valida: dev interpretado, sem build, com banco local, seed e mocks.*

### N.2 QA determinístico

```archol
environment qa {
  runtime: interpreted
  build: verify
  database: postgres
  data: fixtures "fixtures/qa"
  clock: virtual
  integrations: stub
  auth: oidc
  promoteTo: staging
  approvals: [QA]
  observability: test-report
}
```

*Valida: relógio virtual, fixtures, stubs e aprovação de QA.*

### N.3 Staging parecido com produção

```archol
environment staging {
  runtime: generated
  build: reproducible
  database: postgres
  data: anonymized "snapshot/2026-11"
  integrations: sandbox
  auth: oidc
  promoteTo: prod
  approvals: [QA, Owner]
  observability: full
}
```

*Valida: ambiente prod-like com dados anonimizados e aprovação dupla.*

### N.4 Produção com HA e backup

```archol
environment prod {
  runtime: generated
  build: reproducible
  database: postgres { ha: true }
  data: live
  integrations: live
  auth: oidc
  backup { rpo: 15m, rto: 1h }
  observability: otel
  approvals: [Owner, Ops]
}
```

*Valida: HA, RPO/RTO e observabilidade como parte do ambiente.*

### N.5 Sandbox de demonstração

```archol
environment sandbox {
  runtime: generated
  build: reproducible
  database: postgres
  data: demo "demo/2026"
  reset: nightly
  integrations: mock
  auth: local
  observability: minimal
}
```

*Valida: ambiente de demonstração com reset noturno e dados fictícios.*

### N.6 Recuperação de desastre

```archol
environment dr {
  runtime: generated
  database: postgres { replicaOf: prod }
  data: replica
  integrations: live
  auth: oidc
  activation: manual
  restoreFrom: "backup/prod"
  observability: otel
}
```

*Valida: ambiente de DR com réplica, ativação manual e restore explícito.*

## Parameter / Secret

> Cada bloco é um trecho de um `Service`. `Parameter` **não pode** ser declarado na `Application`; a app só faz `set` no `main`.

### PR.1 URL de configuração

```archol
parameter ErpUrl {
  title: { pt-BR: "Endereço do sistema de gestão", en: "ERP URL" }
  type: string
  default: "https://erp.example.com"
}
```

*Valida: configuração tipada, obrigatória, com default e título; valor resolvido pelo ambiente.*

### PR.2 Chave secreta

```archol
secret PaymentKey {
  title: { pt-BR: "Chave de pagamento", en: "Payment key" }
  type: string
  required
}
```

*Valida: segredo sem default, sem valor no fonte e sem serialização/log.*

### PR.3 Perfil de modelo

```archol
parameter LlmModel {
  title: { pt-BR: "Modelo de linguagem", en: "LLM model" }
  type: string
  default: "balanced"
}
```

*Valida: configuração enumerada e default; combina com a capability aberta `llm`.*

### PR.4 Limite numérico

```archol
parameter MaxRetries {
  title: { pt-BR: "Máximo de tentativas", en: "Max retries" }
  type: std.Count
  default: 3
  min: 0
  max: 10
}
```

*Valida: constraints simples no Parameter, sem validação espalhada.*

### PR.5 Janela de SLA

```archol
parameter SupportSla {
  title: { pt-BR: "Prazo de suporte", en: "Support SLA" }
  type: std.Duration
  default: "2h"
}
```

*Valida: duração tipada, com o literal entre aspas e parseado para `std.Duration`.*

### PR.6 Chave de API específica

```archol
secret GoogleMapsKey {
  title: { pt-BR: "Chave do Google Maps", en: "Google Maps key" }
  type: string
  required
}
```

*Valida: segredo para uma capability específica (`google-maps`), sem acoplar o serviço à tecnologia.*

## Serviços essenciais (usos)

> Esta seção tem exemplos de **uso** das capabilities essenciais; não conta como um dos 152 exemplos de artefatos.

### U.1 Artefato que exige capabilities

```archol
service "example.com/services/field" {
  title: { pt-BR: "Serviço de campo", en: "Field service" }
  locales: [ "pt-BR", "en" ]
}

view VisitForm {
  capabilities { location, timezone, regionalization, units }
  type: form
}
```

*Valida: a View exige capabilities; o Service não as declara.*

### U.2 View formatando data e distância

```archol
view VisitForm {
  type: form
  computed {
    scheduledLocal: regionalization.formatDate(visit.scheduledAt, timezone.current())
    distance: units.format(location.distance(tech.position, visit.position), regionalization.system())
  }
}
```

*Valida: `timezone` resolve o fuso, `regionalization` formata e `units` converte a distância para o sistema local.*

### U.3 Shape com unidades via std

```archol
shape ParcelWeight {
  title: { pt-BR: "Peso do pacote", en: "Parcel weight" }
  fields {
    gross: std.Mass
    net: std.Mass
  }
}
```

*Valida: Shape de negócio usando `std.Mass`; a stdlib resolve a unidade e a exibição.*

## Builder

### B.1 Aplicação web

```archol
builder WebApp {
  target: browser
  mode: [interpret, generate]
  capabilities: [gui, sync, identity, attachments, timezone, regionalization, units]
  consumes: [Shape, Document, View, Process, Theme]
  produces: [html, css, js]
}
```

*Valida: builder com capabilities, consumo de artefatos e dois modos.*

> Construtores podem declarar capabilities próprias, como `llm` ou `google-maps`; o contrato é documentado pelo construtor.

### B.2 API em Rust

```archol
builder ApiRust {
  target: server
  mode: [interpret, generate]
  capabilities: [oltp, sync, scheduling, identity, attachments, timezone, regionalization, units]
  consumes: [Service, Document, Action, Event]
  produces: [rust, openapi]
}
```

*Valida: builder de backend com OpenAPI e jobs.*

### B.3 Persistência Postgres

```archol
builder Postgres {
  target: database
  mode: generate
  capabilities: [oltp]
  consumes: [Document, Shape, Index]
  produces: [sql, ddl]
}
```

*Valida: builder de banco que gera DDL, índices e migrações.*

### B.4 Shell mobile

```archol
builder MobileShell {
  target: mobile
  mode: generate
  capabilities: [gui, sync, location, timezone, regionalization, units, capture, attachments, notify]
  consumes: [View, Widget, Theme]
  produces: [react-native]
  limitations {
    widgets: [SignaturePad, BarcodeScanner]
    process: readOnly
  }
}
```

*Valida: builder com limitações declaradas, em vez de prometer paridade total.*

### B.5 Terminal

```archol
builder Cli {
  target: terminal
  mode: interpret
  capabilities: [tui, attachments, timezone, regionalization, units]
  consumes: [View, Action, Report]
  produces: [text, ansi]
  limitations {
    widgets: unsupported
    images: unsupported
  }
}
```

*Valida: builder de terminal que declara o que não suporta.*

### B.6 Documentos e PDFs

```archol
builder DocsPdf {
  target: document
  mode: generate
  capabilities: [print, attachments, timezone, regionalization, units]
  consumes: [View, Report, Document]
  produces: [pdf, html]
}
```

*Valida: builder de relatórios/laudos com template e PDF.*

## Theme

> Theme é asset do construtor, não serviço nem artefato de negócio. Os exemplos testam a sintaxe que o builder consome.

### T.1 Tema claro padrão

```archol
theme NuvulaLight {
  mode: light
  supports: [WebApp, MobileShell]
  tokens {
    color.primary: "#2563eb"
    color.surface: "#ffffff"
    color.text: "#111827"
    radius.md: 8
    font.body: "Inter"
    spacing.unit: 4
  }
  icons: "lucide"
}
```

*Valida: tokens de tema, suporte por builder e ícones.*

### T.2 Tema escuro

```archol
theme NuvulaDark {
  mode: dark
  supports: [WebApp, MobileShell]
  tokens {
    color.primary: "#60a5fa"
    color.surface: "#0f172a"
    color.text: "#e5e7eb"
    radius.md: 8
    font.body: "Inter"
  }
  icons: "lucide"
}
```

*Valida: mesmo contrato de tokens, valores diferentes por modo.*

### T.3 Alto contraste

```archol
theme HighContrast {
  mode: dark
  supports: [WebApp]
  tokens {
    color.primary: "#ffff00"
    color.surface: "#000000"
    color.text: "#ffffff"
    focus.width: 3
    font.scale: 1.25
  }
  accessibility: WCAG_AA
}
```

*Valida: tema como contrato de acessibilidade, não só cores.*

### T.4 Marca da loja

```archol
theme BrandStore {
  mode: light
  supports: [WebApp, DocsPdf]
  tokens {
    color.primary: "#7c3aed"
    color.accent: "#f59e0b"
    font.heading: "Sora"
    font.body: "Inter"
    logo: "assets/logo.svg"
  }
  icons: "lucide"
}
```

*Valida: marca + logo + fontes, com suporte a documento/PDF.*

### T.5 Impressão de laudo

```archol
theme PrintFriendly {
  mode: print
  supports: [DocsPdf]
  tokens {
    color.text: "#000000"
    color.surface: "#ffffff"
    font.body: "Source Serif"
    font.size: 11
    margin: 20mm
  }
  page: A4
  header: "logo"
  footer: "page-number"
}
```

*Valida: tema específico de impressão, com página e cabeçalho/rodapé.*

### T.6 Operação compacta

```archol
theme CompactOps {
  mode: light
  supports: [WebApp, Cli]
  tokens {
    spacing.unit: 2
    font.size: 12
    radius.md: 2
    color.primary: "#0f766e"
  }
  density: compact
}
```

*Valida: tema de alta densidade para operação.*

## Locale

> Locale é configuração da app/construtor; não é serviço nem artefato de negócio.

### L.1 Português do Brasil

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

*Valida: moeda, unidades e formatos locais; fallback explícito.*

### L.2 Inglês dos Estados Unidos

```archol
locale en-US {
  currency: USD
  units: imperial
  date: "MM/dd/yyyy"
  time: "h:mm a"
  direction: ltr
  decimal: "."
  thousands: ","
  messages: "locales/en-US.json"
}
```

*Valida: unidades imperiais e formatos diferentes do pt-BR.*

### L.3 Espanhol

```archol
locale es-ES {
  fallback: en-US
  currency: EUR
  units: metric
  date: "dd/MM/yyyy"
  time: "HH:mm"
  direction: ltr
  decimal: ","
  thousands: "."
  messages: "locales/es-ES.json"
}
```

*Valida: outro idioma com a mesma estrutura, sem duplicar semântica.*

### L.4 Francês

```archol
locale fr-FR {
  fallback: en-US
  currency: EUR
  units: metric
  date: "dd/MM/yyyy"
  time: "HH:mm"
  direction: ltr
  decimal: ","
  thousands: " "
  messages: "locales/fr-FR.json"
}
```

*Valida: separador de milhar diferente e fallback.*

### L.5 Alemão

```archol
locale de-DE {
  fallback: en-US
  currency: EUR
  units: metric
  date: "dd.MM.yyyy"
  time: "HH:mm"
  direction: ltr
  decimal: ","
  thousands: "."
  messages: "locales/de-DE.json"
}
```

*Valida: formato de data próprio e unidade métrica.*

### L.6 Árabe com RTL

```archol
locale ar-SA {
  fallback: en-US
  currency: SAR
  units: metric
  date: "dd/MM/yyyy"
  time: "HH:mm"
  direction: rtl
  decimal: "."
  thousands: ","
  messages: "locales/ar-SA.json"
}
```

*Valida: direção RTL e moeda local sem alterar o modelo.*

## Widget custom

### W.1 Assinatura

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

*Valida: widget que liga um Shape, declara plataformas e fallback.*

### W.2 Mapa

```archol
widget MapPicker {
  binds: GeoPoint
  capabilities: [gui, location]
  platforms {
    web: map
    mobile: native-map
  }
  events { picked -> GeoPoint.create }
}
```

*Valida: widget com dependência HTTP e binding geográfico.*

### W.3 Leitor de código de barras

```archol
widget BarcodeScanner {
  binds: Barcode
  capabilities: [gui, capture]
  platforms {
    web: camera
    mobile: camera
    cli: stdin
  }
  events { scanned -> Barcode.create }
}
```

*Valida: widget com capacidade de câmera e fallback de terminal.*

### W.4 Entrada com unidade

```archol
widget UnitInput {
  binds: Mass | Length | Volume
  capabilities: [gui]
  platforms {
    web: input
    mobile: input
    cli: prompt
  }
  options {
    unit: required
    precision: shape
  }
  events { changed -> value }
}
```

*Valida: widget que respeita dimensão/unidade e não formata por conta própria.*

### W.5 Linha do tempo

```archol
widget Timeline {
  binds: Event[]
  capabilities: [gui]
  platforms {
    web: timeline
    mobile: list
    cli: table
  }
  options {
    order: asc
    groupBy: day
    showActor: true
  }
}
```

*Valida: widget que exibe uma coleção de eventos com adaptação por plataforma.*

### W.6 Upload de arquivo

```archol
widget FileDropzone {
  binds: FileRef
  capabilities: [gui, attachments]
  platforms {
    web: dropzone
    mobile: picker
    cli: path
  }
  options {
    maxSize: 20mb
    mime: [application/pdf, image/png, image/jpeg]
    antivirus: true
  }
  events { uploaded -> FileRef.create }
}
```

*Valida: widget de arquivo com limites, MIME e antivírus.*

## Release / Version

> `@1.4.0` é semver; `@tag("v1.4.0")` e `@commit("9f3a1c7")` também são seletores válidos. Em produção, use tag/commit imutável. O app release aponta para o **lock** que fixa uma versão por serviço.

### V.1 Release de aplicação

```archol
release "example.com/app/loja-do-joao"@1.4.0 {
  application: "example.com/app/loja-do-joao"
  lock: "locks/loja-do-joao/1.4.0.lock"
  migrations: [M-2026-11-01]
  approvals: [QA, Owner]
  strategy: rolling
  rollback: "example.com/app/loja-do-joao"@1.3.2
}
```

*Valida: release como conjunto imutável de versões + migrações + aprovação.*

### V.2 Versão de serviço

```archol
release "example.com/services/catalog"@1.2.0 {
  service: "example.com/services/catalog"
  changes: [add Product.barcode, add Product.Updated.v2]
  compatibility: backward
  migrations: [M-2026-10-12]
  approvals: [Owner]
  rollback: "example.com/services/catalog"@1.1.3
}
```

*Valida: versão de serviço com mudanças compatíveis e rollback.*

### V.3 Hotfix

```archol
release "example.com/app/loja-do-joao"@1.4.1 {
  application: "example.com/app/loja-do-joao"
  lock: "locks/loja-do-joao/1.4.1.lock"
  type: hotfix
  reason: "Corrige cálculo de frete em pedido dividido"
  approvals: [OnCall, Owner]
  strategy: immediate
  rollback: "example.com/app/loja-do-joao"@1.4.0
}
```

*Valida: hotfix com motivo, aprovação de plantão e rollback.*

### V.4 Migração de dados

```archol
release M-2026-11-01 {
  type: migration
  changes {
    document Shipment {
      add peso_normalizado: Mass
      index peso_normalizado
    }
  }
  backfill: chunked(1000)
  dryRun: required
  rollback: reverse
  approvals: [QA, DBA]
}
```

*Valida: migração como artefato versionado, com backfill e dry-run.*

### V.5 Rollback

```archol
release "example.com/app/loja-do-joao"@1.4.2 {
  type: rollback
  application: "example.com/app/loja-do-joao"
  lock: "locks/loja-do-joao/1.4.2.lock"
  to: "example.com/app/loja-do-joao"@1.4.0
  reason: "Erro de cálculo em produção"
  approvals: [Owner, Ops]
  strategy: immediate
  preserveData: true
}
```

*Valida: rollback explícito, com preservação de dados e aprovação.*

### V.6 Canary

```archol
release "example.com/app/loja-do-joao"@1.5.0 {
  application: "example.com/app/loja-do-joao"
  lock: "locks/loja-do-joao/1.5.0.lock"
  strategy: canary(10%, 30m)
  guardrails {
    errorRate: < 1%
    p95Latency: < 500ms
  }
  approvals: [QA, Owner]
  rollback: "example.com/app/loja-do-joao"@1.4.1
}
```

*Valida: canary com guardrails mensuráveis e rollback automático.*

# Operação

## Integration

### I.1 ERP por REST

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
    getPurchaseOrder(id: PurchaseOrder): PurchaseOrder
  }

  events {
    Invoice.Issued -> createInvoice
  }
}
```

*Valida: integração com operações tipadas, auth, retry e mapeamento de evento.*

### I.2 Gateway de pagamento com idempotência

```archol
integration PaymentGateway {
  protocol: rest
  baseUrl: env.PAYMENT_URL
  auth: api-key
  timeout: 15s
  idempotency: header "Idempotency-Key"

  operations {
    authorize(order: Order, amount: std.Money): Authorization
    capture(authorizationId: ExternalId): Payment
    refund(paymentId: ExternalId, amount: std.Money): Refund
  }

  webhooks {
    payment.succeeded -> Payment.succeeded
    payment.failed -> Payment.failed
  }
}
```

*Valida: idempotência explícita e webhooks como eventos de entrada.*

### I.3 Transportadora

```archol
integration Carrier {
  protocol: rest
  baseUrl: env.CARRIER_URL
  auth: oauth2
  timeout: 20s

  operations {
    quote(shipment: Shipment): FreightQuote
    createLabel(shipment: Shipment): Label
    track(tracking: TrackingCode): TrackingEvent[]
  }

  fallback {
    onUnavailable: ManualQuote
    cache: 5m
  }
}
```

*Valida: operações de frete/etiqueta/rastreio e fallback quando a transportadora cai.*

### I.4 E-mail

```archol
integration Email {
  protocol: smtp
  host: env.SMTP_HOST
  auth: password
  from: "no-reply@lojadojoao.com"

  operations {
    send(template: NotificationTemplate, to: Email, data: Map): MessageId
  }

  limits {
    rate: 100/min
    retry: exponential(5)
  }
}
```

*Valida: integração simples com rate limit e retry.*

### I.5 WhatsApp

```archol
integration WhatsApp {
  protocol: webhook
  baseUrl: env.WHATSAPP_URL
  auth: token

  operations {
    sendTemplate(to: Phone, template: NotificationTemplate, data: Map): MessageId
  }

  webhooks {
    message.status -> NotificationStatus
    message.received -> InboundMessage
  }
}
```

*Valida: canal com webhook de status e mensagem recebida.*

### I.6 Balança

```archol
integration ScaleDevice {
  protocol: mqtt
  host: env.SCALE_HOST
  topic: "scales/+/reading"
  auth: certificate

  operations {
    read(deviceId: DeviceId): Mass
  }

  events {
    reading -> Shipment.pesoBruto
  }

  quality {
    staleAfter: 5m
    calibrate: monthly
  }
}
```

*Valida: dispositivo como integração com leitura, evento e frescor.*

## Report / Dashboard

**Relatório é View, não documento** (7.3 / 21): lê, agrega, filtra e exporta — o número existe só na consulta, sem versão nem histórico.
**Documento derivado** (`computed from`) é estado durável (5.10): guarda o número calculado, o `calculatedAt` e o frescor, para fechamento, laudo e auditoria.

### R.1 Vendas por região

```archol
view SalesByRegion {
  title: { pt-BR: "Vendas por região", en: "Sales by region" }
  type: report
  sources: [Invoice, Order]
  roles: [Manager]

  filters { period; region; channel }
  metrics {
    revenue: sum(Invoice.total) by region
    orders: count(Order) by region
    avgTicket: avg(Order.total) by region
  }
  units { revenue: BRL; avgTicket: BRL }
  freshness: 15m
  export: [pdf, csv]
}
```

*Valida: `report` com `sources`, `filters`, `metrics` agregadas por dimensão (`by region`), `units`, `freshness` e `export` — a view lê dois documentos e projeta, não grava nada.*

### R.2 Envelhecimento de estoque

```archol
view InventoryAging {
  title: { pt-BR: "Envelhecimento de estoque", en: "Inventory aging" }
  type: report
  sources: [StockItem, Product]
  roles: [Supervisor, Manager]

  filters { warehouse; category; period }
  metrics {
    quantity: sum(StockItem.available) by warehouse
    value: sum(StockItem.available * Product.price) by warehouse
    aging: bucket(StockItem.lastMovementAt, [30d, 60d, 90d, 180d])
  }
  units { value: BRL }
  freshness: 1h
  export: [csv, xlsx]
}
```

*Valida: agregação `bucket` com fronteiras de tempo explícitas para envelhecer estoque, métrica derivada de dois documentos (`available * price`) e exportação em planilha.*

### R.3 Cumprimento de SLA

```archol
view SlaCompliance {
  title: { pt-BR: "Cumprimento de SLA", en: "SLA compliance" }
  type: report
  sources: [SupportTicket]
  roles: [Manager, Supervisor]

  filters { period; team; priority }
  metrics {
    compliance: percentage(SupportTicket.slaMet)
    breached: count(SupportTicket) where slaMet == false
    avgResolution: avg(SupportTicket.resolutionTime)
  }
  units { compliance: "%"; avgResolution: h }
  freshness: 5m
  export: [pdf, csv]
}
```

*Valida: `percentage` sobre condição do documento, `count ... where` para o complemento e `avg` temporal — `units` com `%` (Percentage usa `unit: "%"`, 6.1.1) e `h`.*

### R.4 Fluxo de caixa

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

*Valida: três fontes, `where` filtrando status, métrica calculada sobre outras métricas (`balance`) e unidade explícita em cada linha.*

### R.5 Qualidade laboratorial

```archol
view LabQuality {
  title: { pt-BR: "Qualidade laboratorial", en: "Laboratory quality" }
  type: report
  sources: [LabExam, NonConformity]
  roles: [Analyst, Pathologist]

  filters { period; examType; labUnit }
  metrics {
    released: count(LabExam) where status == Released
    rework: percentage(LabExam.rework)
    avgTurnaround: avg(LabExam.turnaround)
    critical: count(LabExam) where critical == true
    nonConformities: count(NonConformity) where status == Open
  }
  units { avgTurnaround: h }
  freshness: 10m
  export: [pdf]
}
```

*Valida: relatório de qualidade com `count ... where` por estado do documento, `percentage` de retrabalho, tempo médio e contagem de não conformidades — sem documento de relatório.*

### R.6 Controle operacional em tempo real

```archol
view OperationsControl {
  title: { pt-BR: "Controle operacional", en: "Operations control" }
  type: dashboard
  sources: [Shipment, Incident]
  roles: [Supervisor]
  mode: live

  filters { site; status; carrier }
  metrics {
    late: count(Shipment) where status != Delivered and dueAt < now()
    atRisk: sum(Shipment.freight) where status != Delivered and dueAt < now()
    openIncidents: count(Incident) where status != Closed
  }
  units { atRisk: BRL }
  freshness: 1m
}
```

*Valida: `dashboard` com `mode: live`, métricas condicionadas a `now()` (atraso é cálculo de leitura, não evento) e `freshness` de 1 minuto.*

### R.7 Receita mensal como documento derivado (não é view)

```archol
document MonthlyRevenue {
  title: { pt-BR: "Receita mensal", en: "Monthly revenue" }
  computed from Invoice, Payment
  freshness: 1h
  exposes: [revenue, invoiceCount, calculatedAt]

  fields {
    period: FiscalPeriod { title: { pt-BR: "Período", en: "Period" }, lookup }
    revenue: std.Money { title: { pt-BR: "Receita", en: "Revenue" }, currency: BRL }
    invoiceCount: std.Count { title: { pt-BR: "Notas", en: "Invoices" } }
  }
}
```

*Valida: número que precisa durar vira `document` derivado — `computed from` duas fontes, `freshness`, `exposes` com `calculatedAt` implícito e campos que guardam o resultado; a view R.1 continua sendo só projeção.*
## Event

**Evento de workflow fica no workflow**: é declarado na `transition` do valor de origem e publicado quando a action executa a transição. O bloco `events { }` do documento é só para eventos **sem** transição. Não existe artefato `event` de topo, nem `key`.

### E.1 Evento de transição declarado no workflow

```archol
shape OrderStatus {
  title: { pt-BR: "Situação do pedido", en: "Order status" }
  base: state
  initial: Draft

  value Draft {
    title: { pt-BR: "Rascunho", en: "Draft" }
    icon: status.draft
    ordinal: 10

    transition Placed {
      title: { pt-BR: "Pedido realizado", en: "Order placed" }
      payload {
        placedAt: DateTime { title: { pt-BR: "Realizado em" } }
        total: std.Money { title: { pt-BR: "Total" } }
        customer: User { title: { pt-BR: "Cliente" } }
      }
    }
  }

  value Placed {
    title: { pt-BR: "Realizado", en: "Placed" }
    icon: status.placed
    ordinal: 20
  }
}

document Order {
  title: { pt-BR: "Pedido", en: "Order" }

  fields {
    status: OrderStatus { title: { pt-BR: "Situação", en: "Status" }, lookup }
    placedAt: DateTime { title: { pt-BR: "Realizado em", en: "Placed at" } }
    total: std.Money { title: { pt-BR: "Total", en: "Total" } }
    customer: User { title: { pt-BR: "Cliente", en: "Customer" } }
  }

  actions {
    place() {
      title: { pt-BR: "Realizar pedido", en: "Place order" }
      roles: [Clerk]
      to: status.Placed
      {
        placedAt = now()
      }
    }
  }
}
```

*Valida: o evento é da aresta `Draft --Placed--> Placed` e está declarado no workflow; a action só aponta o destino e o payload vem por nome.*

### E.2 Evento sem transição

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

*Valida: evento que não muda estado é emitido no body; a action não declara `to:`.*

### E.3 Identidade canônica e alias

```text
SERVICO.DOCUMENTO.EVENTO
Billing.Invoice.Paid
example.com/services/billing#Invoice.Paid
```

```archol
process PaymentReconciliation(payment: Payment) {
  title: { pt-BR: "Conciliação de pagamento", en: "Payment reconciliation" }

  on [Billing.Invoice.Paid]
  key: payment

  PaymentMatch.show({ payment })
  Payment.reconcile()
  return Reconciled

  end Reconciled {
    title: { pt-BR: "Conciliado", en: "Reconciled" }
    outcome: success
  }
}
```

*Valida: processo de outro serviço assina o evento pelo nome qualificado; o alias local não muda a identidade.*

### E.4 Payload com referência a documento

```archol
events {
  Dispatched {
    title: { pt-BR: "Remessa despachada", en: "Shipment dispatched" }
    payload {
      carrier: Carrier { title: { pt-BR: "Transportadora" } }
      tracking: TrackingCode { title: { pt-BR: "Rastreio" } }
      dispatchedAt: DateTime { title: { pt-BR: "Despachado em" } }
      weight: std.Mass { title: { pt-BR: "Peso" } }
    }
  }
}
```

*Valida: payload com `DocumentRef` (`carrier` traz id + lookup), tipo de domínio (`TrackingCode`) e unidade (`std.Mass`).*

### E.5 Evento que inicia processo

```archol
process Packing(order: Order) {
  title: { pt-BR: "Separação", en: "Packing" }

  on [Order.Placed]
  key: order

  PickItems.show({ order })
  Order.readyToShip()
  return Packed

  end Packed {
    title: { pt-BR: "Pronto", en: "Ready" }
    outcome: success
  }
}
```

*Valida: o mesmo evento que fecha uma action abre um processo; correlação pela `key` (`order`).*

### E.6 O mesmo evento a partir de origens diferentes

```archol
shape InvoiceStatus {
  title: { pt-BR: "Situação da nota", en: "Invoice status" }
  base: state
  initial: Issued

  value Issued {
    title: { pt-BR: "Emitida", en: "Issued" }
    icon: status.issued
    ordinal: 10

    transition Paid {
      title: { pt-BR: "Nota paga", en: "Invoice paid" }
      payload {
        paidAt: DateTime { title: { pt-BR: "Paga em" } }
        amount: std.Money { title: { pt-BR: "Valor" } }
      }
    }
  }

  value Overdue {
    title: { pt-BR: "Vencida", en: "Overdue" }
    icon: status.overdue
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
    icon: status.paid
    ordinal: 30
  }
}

document Invoice {
  title: { pt-BR: "Nota fiscal", en: "Invoice" }

  fields {
    status: InvoiceStatus { title: { pt-BR: "Situação", en: "Status" }, lookup }
    paidAt: DateTime { title: { pt-BR: "Paga em", en: "Paid at" } }
    total: std.Money { title: { pt-BR: "Total", en: "Total" } }
  }

  actions {
    pay(paidAt: DateTime) {
      title: { pt-BR: "Pagar", en: "Pay" }
      roles: [Clerk]
      to: status.Paid
      emits { amount: total }
    }

    registerPayment(paidAt: DateTime) {
      title: { pt-BR: "Registrar pagamento", en: "Register payment" }
      roles: [Manager]
      to: status.Paid
      emits { amount: total }
    }
  }
}
```

*Valida: duas arestas (`Issued` e `Overdue`) para o mesmo evento exigem o **mesmo** payload; o compilador unifica em `Invoice.Paid` e falha com `E_EVENT_SCHEMA_CONFLICT` se divergir. Renomear o estado é breaking change.*

## Schedule

**Agendamento é um gatilho, não um artefato.** O cron é do processo; scheduler, retry e DLQ são runtime. Rotina em lote é **fan-out por documento**: `schedule` + `fanOut` inicia uma instância por documento selecionado.

### S.1 Fechamento mensal por período

```archol
process MonthClose(fiscalPeriod: FiscalPeriod) {
  title: { pt-BR: "Fechamento mensal", en: "Month close" }
  sla: "6h"

  schedule: "0 2 1 * *"
  fanOut: FiscalPeriod where status == Open
  key: fiscalPeriod

  Ledger.postPeriod({ fiscalPeriod })
  MonthCloseReview.show({ fiscalPeriod })

  if (fiscalPeriod.status == Rejected) {
    ReconciliationFix.show({ fiscalPeriod })
    Ledger.postPeriod({ fiscalPeriod })
  }

  if (fiscalPeriod.status == Open) {
    FiscalPeriod.close({ fiscalPeriod })
    return Closed
  }

  return Failed

  end Closed {
    title: { pt-BR: "Fechado", en: "Closed" }
    outcome: success
  }
  end Failed {
    title: { pt-BR: "Falhou", en: "Failed" }
    outcome: failure
  }
}
```

*Valida: cron mensal, uma instância por período aberto (`fanOut`), operação de sistema com decisão por estado e desfechos nomeados.*

### S.2 Cobrança com fan-out e espera

```archol
process Dunning(invoice: Invoice) {
  title: { pt-BR: "Cobrança", en: "Dunning" }
  sla: "45d"

  schedule: "0 6 * * *"
  fanOut: Invoice where status == Overdue and daysOverdue >= 1
  key: invoice

  send PaymentReminder {
    key: invoice
    with { number: invoice.number; total: invoice.total; dueAt: invoice.dueAt }
  }

  wait 7d

  if (invoice.daysOverdue > 30) {
    CollectionCall.show({ invoice })
  } else {
    send PaymentReminder {
      key: invoice
      with { number: invoice.number; total: invoice.total; dueAt: invoice.dueAt }
    }
  }

  cancel on Invoice.Paid {
    return Settled
  }

  return WrittenOff

  end Settled {
    title: { pt-BR: "Pago", en: "Paid" }
    outcome: success
  }
  end WrittenOff {
    title: { pt-BR: "Perda", en: "Written off" }
    outcome: failure
  }
}
```

*Valida: fan-out por fatura vencida, `send`, espera de 7 dias, decisão por `if` e `cancel on`.*

### S.3 Reposição disparada por ponto de pedido

```archol
process Replenishment(stockItem: StockItem) {
  title: { pt-BR: "Reposição", en: "Replenishment" }
  sla: "2d"

  schedule: "0 7 * * *"
  fanOut: StockItem where available < reorderPoint
  key: stockItem

  let decision: ReplenishmentDecision

  decision = ReplenishmentOrder.evaluate({
    available: stockItem.available,
    reorderPoint: stockItem.reorderPoint
  })

  if (decision.action == order) {
    PurchaseRequest.create({
      stockItem: stockItem,
      quantity: decision.quantity
    })
    return Requested
  }

  StockItem.deferReplenishment({ stockItem })
  return Deferred

  end Requested {
    title: { pt-BR: "Solicitado", en: "Requested" }
    outcome: success
  }
  end Deferred {
    title: { pt-BR: "Adiado", en: "Deferred" }
    outcome: success
  }
}
```

*Valida: decisão antes de criar a requisição, duas saídas de negócio e `fanOut` por ponto de pedido.*

### S.4 Rotina sem sujeito (digest)

```archol
process DailyDigest() {
  title: { pt-BR: "Resumo diário", en: "Daily digest" }

  schedule: "0 7 * * *"

  Digest.build()
  send DailyDigestReady {
    key: notification
  }
  ReviewDigest.show({ notification })

  return Sent

  end Sent {
    title: { pt-BR: "Enviado", en: "Sent" }
    outcome: success
  }
}
```

*Valida: processo agendado sem sujeito, com `send` e task manual de conferência; não há documento de relatório.*

## Policy / Rule

### P.1 Quatro-olhos em pagamento

```archol
policy FourEyesPayment {
  appliesTo: Payment.release
  rule: approver != creator
  severity: block
  message: "Pagamento exige segundo aprovador"
}
```

*Valida: política de segregação com bloqueio e auditoria.*

### P.2 Limite de desconto

```archol
policy DiscountLimit {
  appliesTo: Order.applyDiscount
  rule: discount <= 10% or approver.role == Manager
  severity: require
  message: "Desconto acima de 10% exige gerente"
}
```

*Valida: política que exige aprovação em vez de bloquear.*

### P.3 Retenção de dados

```archol
policy Retention {
  appliesTo: [LabExam, Evidence]
  rule: retainFor(5y) then anonymize
  severity: enforce
  exception: legalHold
}
```

*Valida: política de retenção com exceção de hold jurídico.*

### P.4 Escalonamento de SLA

```archol
policy SlaEscalation {
  appliesTo: SupportTicket
  rule: if breached then escalateTo(Lead) within 15m
  severity: enforce
  notification: [Email, WhatsApp]
}
```

*Valida: política temporal com canal de notificação.*

### P.5 Acesso a campo sensível

```archol
policy SensitiveFieldAccess {
  appliesTo: [Patient.document, LabExam.result]
  rule: role in [Doctor, Nurse] and purpose == treatment
  severity: block
  mask: default
}
```

*Valida: acesso por papel + finalidade, com mascaramento padrão.*

### P.6 Segregação de aprovação

```archol
policy ApprovalSegregation {
  appliesTo: [PurchaseOrder.approve, Payment.approve]
  rule: requester != approver and approver != payer
  severity: block
  message: "Solicitante, aprovador e pagador devem ser distintos"
}
```

*Valida: política que atravessa mais de um documento e ator.*

---

# Icon

### IC.1 Rascunho

```archol
icon Draft {
  title: { pt-BR: "Rascunho", en: "Draft" }
}
```

*Valida: ícone como token semântico; o builder/Theme resolve a imagem.*

### IC.2 Paga

```archol
icon Paid {
  title: { pt-BR: "Paga", en: "Paid" }
}
```

*Valida: ícone reutilizável por vários values.*

### IC.3 Vencida

```archol
icon Overdue {
  title: { pt-BR: "Vencida", en: "Overdue" }
}
```

*Valida: Icon sem detalhe visual; o Theme escolhe a imagem.*

### IC.4 E-mail

```archol
icon Email {
  title: { pt-BR: "Correio eletrônico", en: "Email" }
}
```

*Valida: ícone de canal definido no serviço de notificações.*

### IC.5 Prioridade

```archol
icon P1 {
  title: { pt-BR: "Prioridade 1", en: "Priority 1" }
}
```

*Valida: ícone de domínio para enum de prioridade no serviço de atendimento.*

### IC.6 Moeda

```archol
icon Brl {
  title: { pt-BR: "Real", en: "Brazilian real" }
}
```

*Valida: ícone para value de moeda; o value também tem title.*

## SVG file-based Icons

Além dos Icons declarados em `.archol`, arquivos `.svg` sob `<workspace>/services/<fullpath>/icons/` viram Icons implícitos. `icons/` é contêiner: as subpastas dentro dele são namespaces de ícone, mas `icons` não entra na referência. Exemplos em `examples/services/`:

```text
example.com/services/billing/icons/status/draft.svg              -> status.draft      (aria-label)
example.com/services/billing/icons/status/paid.svg               -> status.paid        (<title>)
example.com/services/billing/icons/status/overdue.svg            -> status.overdue     (alt)
example.com/services/billing/icons/finance/invoice/issued.svg    -> finance.invoice.issued
example.com/services/notifications/icons/channels/email.svg      -> channels.email
example.com/services/inventory/icons/stock/low-stock.svg         -> stock.lowStock     (fallback: "Low Stock")
example.com/services/payments/icons/status/refunded.svg          -> status.refunded    (aria-label)
example.com/services/fraud-checkout/icons/status/fraud-checked.svg -> status.fraudChecked
```

Uso no enum:

```text
draft { title: {...}; icon: status.draft }
paid  { title: {...}; icon: status.paid }
```

- Arquivos/pastas em kebab-case; referências em camelCase com ponto.
- Title: `aria-label` → `<title>` → `alt` → filename humanizado; sem I18N.
- Conflito com Icon declarado no mesmo serviço é erro.
- O builder resolve/sanitiza o SVG; isso é implementação, não DSL.

# O que validar nestes exemplos

Perguntas de sintaxe/modelagem que os exemplos ajudam a testar:

| # | Pergunta | Exemplos para olhar |
| :--- | :--- | :--- |
| 1 | `key` cobre identidade, correlação e dedup sem `for`? | P.1–P.4, S.1–S.4 |
| 2 | `roles` no form/manual é suficiente para execução, sem pool/lane? | F.1–F.8, P.1–P.4 |
| 3 | Variáveis só no topo e `loop`/`forEach` cobrem os fluxos reais? | P.1, P.3, S.3 |
| 4 | `parallel` + `catch` cobrem fork/join e falha de ramo? | P.3, S.1 |
| 5 | `send`/`receive` com `key`/`apply` cobrem mensagens e timeout? | M.1–M.4, P.3 |
| 6 | Decisão humana lida no estado do documento é suficiente? | P.2, F.1, F.8 |
| 7 | `timeout` duro + `sla` brando substitui `on timer`/`on breach`? | F.1, S.2 |
| 8 | `cancel on` cobre cancelamento a qualquer momento? | S.2, exemplos de shipping/support |
| 9 | `end` no fim + `return` cobre desfechos e KPIs? | todos os P |
| 10 | O que ainda parece "código demais" para o dono do processo? | todos |

## Padrões que os exemplos já sugerem

* **Processo = trabalho; documento = estado.** A tela de andamento mostra os dois no mesmo lugar.
* **Operação × task**: `action`/`call`/`compute`/`decision` são chamadas diretas; `form`/`view`/`manual`/`job`/`process` são tasks com artefato.
* **Quem executa é `roles`** no form/manual; não há pool/lane.
* **Subpasso é processo**: `Processo.run` cria outra instância, com tela, histórico e dono próprios.
* **Form é contrato de escrita/desfecho**; View é leitura; relatório é View.
* **Agendamento é gatilho**: `schedule` + `fanOut`; uma instância por documento.
* **Transição é da action**; o grafo de estados é derivado; só action muda estado.
* **Erro é falha de tentativa**: `try/catch` desvia e compensa; retry é da integração/ação.
* **Mensagem é contrato** com correlação por documento (`about`); `send`/`receive` usam `key`.
* **Desfecho é nó nomeado e classificado**; é a dimensão de KPI do processo.
* **Derivação em vez de declaração**: reads/writes, títulos herdados, grafo de estados, data objects.

## Golden examples sugeridos

Os 35 processos completos estão em [`examples/`](examples/). Comece por:

1. **Checkout** — variáveis duráveis, `try/catch`, `loop` com retry, form humano, ends.
2. **PurchaseApproval** — decisão DMN, `if/else`, form com múltiplas ações.
3. **DispatchOrder** — `parallel`, `send`/`receive`, timeout, subprocesso.
4. **OrderCancellation** — ação em outro serviço e compensação explícita.
5. **ClaimSettlement** — decisão, subprocesso, mensagem e compensação.
6. **MonthClose** — `schedule` + `fanOut`, espera e estado.
7. **DailyDigest** — rotina agendada, `send` e manual.
8. **OrderProgress** — tela de andamento com seções fixas.

## Checklist de validação

- [ ] A sintaxe parece legível para um analista, não só para um dev
- [ ] Todo passo tem operação/task e executor (`roles` ou processo)
- [ ] Todo processo tem um gatilho e ends nomeados; todo caminho chega a um end
- [ ] Variáveis estão no topo; não há declaração dentro de bloco
- [ ] O form não redeclara tipo/título do documento
- [ ] O documento não duplica estado no processo
- [ ] Agendamento aparece como `schedule`/`fanOut`, não como artefato
- [ ] Relatório aparece como View, não como documento
- [ ] Mensagens têm `about` e `key`
- [ ] Desfechos permitem calcular KPIs (ciclo, atraso, caminho, motivo)

> Se algo aqui parecer "código demais" para o dono do processo, o problema é de sintaxe/experiência — não do conceito.
