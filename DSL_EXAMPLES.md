# BPCMS — Exemplos de DSL

*Sintaxe ilustrativa v0.2 (Process + Document).* Não é gramática final nem código executável. O objetivo é **validar conceitos, ergonomia e limites** antes de fechar PRD/SPECS/PLAN. A sintaxe e o funcionamento de cada cláusula estão em [DSL_REFERENCE.md](DSL_REFERENCE.md).

A aplicação é orientada a **Process** (trabalho) e **Document** (estado). O processo orquestra quem faz o quê; o documento guarda o dado durável; o **Form** é o contrato da tarefa humana; a **View** é tela de leitura; a **Message** é contrato entre participantes. `job` e `report` deixaram de ser artefatos (agendamento é `start`; relatório é `view type: report`), e o ciclo de vida do documento vive nas actions (`from:`/`to:`/`emits:`).

Este documento cobre o catálogo de artefatos com **186 exemplos** (e 3 usos de serviços essenciais), sendo **24 processos** — o teste de aceitação do modelo. Alguns exemplos compartilham o mesmo domínio de propósito (composição e reuso); outros mudam de domínio para testar generalidade.

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
| Processo | `start`, `task`, `gateway`, `fork`/`join`, `end` | grafo de arestas explícitas; BPMN é uma visão |
| Task | `task Review { lane: Sales; form: OrderReview }` | o tipo é inferido do contrato; não existe `type:` |
| Executor | `lane:` (ou `System`) | toda task tem lane; pool define interno/portal/parceiro/sistema |
| Subpasso | `runs: DeliveryTracking` | sempre outra instância de processo |
| Agendamento | `start { schedule: "0 2 1 * *" }` | não existe `job` |
| Relatório | `view type: report` | agregação é projeção; documento derivado é estado |
| Origem | `source: registry` / `repository` / `auto` | registry BPCMS x projeto Git |
| Versão | `version: semver("1.2.3")` / `tag("v1.2.3")` / `commit("9f3a1c")` | `1.4.0` é açúcar para `semver("1.4.0")` |
| Release | `"<uri>"@1.2.3` | URI + seletor de versão |
| Página canônica | `http://<uri>/index.html` | app = download por SO; service = docs/integração |
| Evento | `transition Paid { title; payload }` no `value` | evento do workflow definido no workflow; `initial ... with transition X` é o evento de criação; `events { }` só para evento sem transição |
| Permissão | `roles: [Manager]` na action | autoridade é do documento; lane é execução |

## Cobertura

| Grupo | Tipos | Exemplos |
| :--- | :--- | :--- |
| **Núcleo** | Application, Service, Process, Form, Message, Document, Shape, View, Role/Organogram | 85 |
| **Descoberta** | Evidence, Conversation, Decision, Assumption | 24 |
| **Plataforma** | Environment, Parameter, Builder, Theme, Locale, Widget custom, Release/Version, Icon | 48 |
| **Operação** | Integration, Report/Dashboard, Event, Schedule, Policy/Rule | 29 |
| **Total** | catálogo de artefatos | **186** |
| **Avulsos** | usos de serviços essenciais (location, timezone, regionalization, units) | 3 |

---

# Núcleo

> **Páginas canônicas.** `http://example.com/app/loja-do-joao/index.html` é a página de **download**, com opções por SO. `http://example.com/services/catalog/index.html` é a página de **documentação para integração por API**, com opções de nuvem, protocolos (sockets/REST/RPC) e iframe para microfrontend, conforme o builder. Em produção, use `https://`.

> **Duas origens.** `example.com/...` é registry BPCMS (páginas `/index.html`). `github.com/...` ou `gitlab.com/...` é repositório Git: a documentação é o **README** e o código é baixado do projeto (clone/zip/release); a versão pode ser `commit(...)` ou `tag(...)`. `source`/`version` deixam isso explícito.

## Application

### A.1 Loja virtual completa

```bpcms
application "example.com/app/loja-do-joao" {
  source: registry
  version: 1.4.0
  title: {
    pt-BR: "Loja do João"
    en-US: "João's Store"
  }
  owner: "Time Loja"
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

```bpcms
application "example.com/app/clinica-vida" {
  version: 2.1.0
  title: { pt-BR: "Clínica Vida" }
  owner: "Diretoria Clínica"

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

```bpcms
application "example.com/app/rota-certa" {
  version: 1.9.2
  title: {
    pt-BR: "Rota Certa"
    en-US: "Right Route"
  }
  owner: "Operações"

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

```bpcms
application "example.com/app/backoffice-loja" {
  version: 1.2.0
  title: { pt-BR: "Backoffice da Loja" }
  owner: "Retaguarda"

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

```bpcms
application "example.com/app/campo-agil" {
  version: 1.0.0
  title: { pt-BR: "Campo Ágil" }
  owner: "Field Ops"

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

```bpcms
application "example.com/app/mercado-aberto" {
  version: 0.8.0
  title: {
    pt-BR: "Mercado Aberto"
    en-US: "Open Market"
  }
  owner: "Marketplace"

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

```bpcms
application "github.com/acme/field-app" {
  source: repository
  version: commit("9f3a1c7")
  title: { pt-BR: "Field App" }
  owner: "Acme Field Ops"

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

```bpcms
application "example.com/app/loja-com-antifraude" {
  version: 1.0.0
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

```bpcms
service "example.com/services/catalog" {
  title: { pt-BR: "Catálogo", en: "Catalog" }
  version: 1.2.0
  owner: "Time de Catálogo"
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

```bpcms
service "example.com/services/checkout" {
  title: { pt-BR: "Finalização da compra", en: "Checkout" }
  version: 2.0.1
  owner: "Time de Conversão"
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

```bpcms
service "example.com/services/inventory" {
  title: { pt-BR: "Estoque", en: "Inventory" }
  version: 1.5.0
  owner: "Supply Chain"
  // document StockItem, document Reservation, view StockTable, event StockItem.Low: público

  imports {
    service "example.com/services/catalog"
    service "example.com/services/auth"
  }
}
```

*Valida: dois documentos, evento de alerta e imports sem versão.*

### S.4 Faturamento com job e relatório

```bpcms
service "example.com/services/billing" {
  title: { pt-BR: "Faturamento", en: "Billing" }
  version: 1.8.0
  owner: "Financeiro"
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

```bpcms
service "example.com/services/auth" {
  title: { pt-BR: "Identidade", en: "Identity" }
  version: 3.0.0
  owner: "Plataforma"
  // document User, document Session, roles Customer/Clerk, view LoginForm, event Session.LoggedIn: público

  imports { integration Email }
}
```

*Valida: identidade exporta papéis e view; sem lock/IDs dentro do serviço.*

### S.6 Notificações multicanal

```bpcms
service "example.com/services/notifications" {
  title: { pt-BR: "Notificações", en: "Notifications" }
  version: 1.1.0
  owner: "Customer Success"
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

```bpcms
service "gitlab.com/acme/lab-quality" {
  title: { pt-BR: "Qualidade laboratorial", en: "Laboratory quality" }
  version: tag("v2.3.0")
  owner: "Acme Lab"
  // document LabQualityReport, event QualityAlert: públicos

  imports {
    service "example.com/services/auth"
    integration Email
  }
}
```

*Valida: serviço em repositório, versão por tag e README como documentação.*

### S.8 Serviço que reintroduz um passo de processo

```bpcms
service "example.com/services/fraud-checkout" {
  title: { pt-BR: "Finalização da compra antifraude", en: "Fraud checkout" }
  version: 1.0.0
  owner: "Risk"

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

*Valida: o ciclo de vida e o processo são editados via `reintroduce`; `insert` insere estados/tasks sem substituir o todo.*

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

Os 24 processos abaixo são o teste de aceitação do modelo Process + Document. Cada um exercita construções reais: pools e lanes (interna, portal, parceiro, sistema), forms, decisões, paralelismo, mensagens, subprocessos, prazos, exceções, compensação, cancelamento e desfechos nomeados.

Convenções:

- `task` sempre tem `lane`; automação usa `lane: System`;
- o tipo da task é inferido do contrato (`form`, `manual`, `action`, `call`, `decision`, `sends`, `receives`, `runs`) — não existe `type:`;
- arestas são explícitas e a ordem de declaração não é semântica;
- `end` é nomeado e classificado (`outcome: success | failure | cancelled`);
- o documento só muda por `action`; a variável do processo é contexto.

### P.1 Aprovação de compra com alçada (procurement)

```bpcms
process PurchaseApproval for PurchaseRequest {
  title: { pt-BR: "Aprovação de compra", en: "Purchase approval" }
  version: 1
  goal: "Comprar com alçada e rastro, sem travar o pedido simples"
  sla: 5d

  pool Company {
    internal
    lane Procurement { role: Buyer }
    lane Management { role: Manager }
    lane Direction { role: Director }
  }

  variables { tier: ApproverTier; purchaseOrder: PurchaseOrder }

  start Submitted { event: PurchaseRequest.Submitted }
  start Manual { by: [Buyer, Manager] }
  on breach -> Escalate

  task Review { lane: Procurement; form: PurchaseReview; sla: 1d; on breach -> Remind }

  task Decide {
    lane: System
    decision: ApprovalLimit
    with { amount: purchaseRequest.amount; supplier: purchaseRequest.supplier }
    gives: tier
  }

  gateway Tier {
    if tier == Auto -> AutoApprove
    if tier == Manager -> ManagerReview
    else -> DirectorReview
  }

  task AutoApprove { lane: System; action: PurchaseRequest.approve; with { purchaseRequest } }
  task ManagerReview { lane: Management; form: ApprovalDecision; sla: 2d; assign: named(purchaseRequest.requester.manager) }
  task DirectorReview { lane: Direction; form: ApprovalDecision; sla: 2d }
  task Remind { lane: System; action: PurchaseRequest.remindApprover; with { purchaseRequest } }
  task Escalate { lane: System; action: PurchaseRequest.escalate; with { purchaseRequest } }
  task CreateOrder { lane: System; action: PurchaseOrder.create; with { purchaseRequest }; gives: purchaseOrder }

  Submitted -> Review
  Manual -> Review
  Review -> Decide
  Decide -> Tier
  AutoApprove -> CreateOrder
  ManagerReview.approved -> CreateOrder
  ManagerReview.rejected -> Rejected
  DirectorReview.approved -> CreateOrder
  DirectorReview.rejected -> Rejected
  Remind -> Review
  Escalate -> DirectorReview
  CreateOrder -> Approved

  end Approved { title: { pt-BR: "Aprovada", en: "Approved" }; outcome: success }
  end Rejected { title: { pt-BR: "Rejeitada", en: "Rejected" }; outcome: failure }
}
```

*Valida: dois starts, DMN com alçada, gateway por valor, breach do caso e da task, arestas por desfecho de form, desfechos nomeados.*

### P.2 Onboarding de fornecedor com portal externo (procurement)

```bpcms
process SupplierOnboarding for Supplier {
  title: { pt-BR: "Cadastro de fornecedor", en: "Supplier onboarding" }
  version: 1
  goal: "Cadastrar fornecedor com documento verificado e contrato assinado"
  sla: 10d

  pool Company {
    internal
    lane Procurement { role: Buyer }
    lane Compliance { role: ComplianceAnalyst }
  }
  pool Supplier {
    external
    lane SupplierPortal { role: SupplierUser }
  }
  pool Bureau {
    external
  }

  variables { profile: CompanyProfile; contract: Contract }

  start Nominated { by: [Buyer, ComplianceAnalyst] }

  task SubmitRegistration {
    lane: SupplierPortal
    form: SupplierRegistration
    sla: 5d
    on breach -> CancelOnboarding
  }

  task VerifyCompany {
    lane: System
    call: Bureau.verifyCompany
    with { supplier }
    gives: profile
  }

  gateway Profile {
    if profile.status == ok -> ComplianceReview
    else -> RequestFix
  }

  task RequestFix { lane: System; sends: RegistrationFixRequest; partner: Supplier; with { supplier; profile } }
  task ComplianceReview { lane: Compliance; form: ComplianceApproval; sla: 2d }
  task Register { lane: System; action: Supplier.approve; with { supplier; profile } }
  task OpenContract { lane: System; action: Contract.open; with { supplier }; gives: contract }
  task SignContract { lane: SupplierPortal; form: ContractSignature; sla: 5d }
  task Activate { lane: System; sends: SupplierActivated; partner: Supplier; with { supplier } }
  task CancelOnboarding { lane: System; action: Supplier.reject; with { supplier } }

  Nominated -> SubmitRegistration
  SubmitRegistration -> VerifyCompany
  VerifyCompany -> Profile
  RequestFix -> SubmitRegistration
  ComplianceReview.approved -> Register
  ComplianceReview.rejected -> CancelOnboarding
  Register -> OpenContract
  OpenContract -> SignContract
  SignContract.signed -> Activate
  SignContract.declined -> CancelOnboarding
  Activate -> Active
  CancelOnboarding -> Rejected

  end Active { title: { pt-BR: "Fornecedor ativo", en: "Supplier active" }; outcome: success }
  end Rejected { title: { pt-BR: "Cadastro recusado", en: "Registration rejected" }; outcome: failure }
}
```

*Valida: pool externo com portal (form preenchido fora), parceiro caixa-preta (Bureau), mensagens com partner, laço de correção, assinatura de contrato.*

### P.3 Recebimento de mercadoria com divergência (procurement)

```bpcms
process GoodsReceipt for GoodsReceipt {
  title: { pt-BR: "Recebimento de mercadoria", en: "Goods receipt" }
  version: 1
  goal: "Receber a mercadoria conferida e tratar divergência sem sujar o estoque"
  sla: 1d

  pool Company {
    internal
    lane Dock { role: Receiver }
    lane Quality { role: QualityInspector }
  }

  variables { reading: WeightReading; accepted: boolean }

  start Arrived { event: GoodsReceipt.Arrived }

  task CheckIn { lane: Dock; form: ReceivingForm; sla: 2h }
  task Weigh { lane: System; call: Scale.read; with { receipt }; gives: reading }

  task Decide {
    lane: System
    decision: ReceiptDiscrepancy
    with { expected: receipt.expectedWeight; measured: reading.weight }
    gives: accepted
  }

  gateway Discrepancy {
    if accepted -> Accept
    else -> Inspect
  }

  task Accept { lane: System; action: GoodsReceipt.accept; with { receipt; weight: reading.weight } }
  task Inspect { lane: Quality; form: DivergenceInspection; sla: 4h }
  task Quarantine { lane: System; action: GoodsReceipt.quarantine; with { receipt } }
  task ReverseReceipt { lane: System; action: GoodsReceipt.reverse; with { receipt }; compensates: Accept }
  rollback Undo

  Arrived -> CheckIn
  CheckIn -> Weigh
  Weigh -> Decide
  Decide -> Discrepancy
  Inspect.accepted -> Accept
  Inspect.divergent -> Quarantine
  Accept -> Accepted
  Quarantine -> Undo
  Undo -> Divergent

  end Accepted { title: { pt-BR: "Recebido", en: "Accepted" }; outcome: success }
  end Divergent { title: { pt-BR: "Divergente", en: "Divergent" }; outcome: failure }
}
```

*Valida: leitura de dispositivo como task, decisão por tolerância, form de inspeção, compensação declarada e rollback.*

### P.4 Entrada de pedido (orders)

```bpcms
process OrderIntake for Order {
  title: { pt-BR: "Entrada de pedido", en: "Order intake" }
  version: 1
  goal: "Confirmar o pedido com estoque reservado e prazo prometido"
  sla: 1d

  pool Company {
    internal
    lane Sales { role: Clerk }
    lane Management { role: Manager }
  }

  variables { reservation: Reservation }

  start Placed { event: Order.Placed }
  start Manual { by: [Clerk] }

  task Review {
    lane: Sales
    form: OrderReview
    sla: 4h
    on breach -> ManagerReview
    on error OrderInvalid -> Reject
  }

  task Reserve {
    lane: System
    action: Stock.reserve
    with { order }
    gives: reservation
    sla: 2m
    on error StockUnavailable -> Backorder
  }

  task Confirm { lane: System; action: Order.confirm; with { order; reservation } }
  task Backorder { lane: Management; form: BackorderDecision; sla: 1d }
  task Reject { lane: System; action: Order.reject; with { order } }
  task ManagerReview { lane: Management; form: OrderReview; sla: 2h }

  Placed -> Review
  Manual -> Review
  Review.approved -> Reserve
  Review.rejected -> Reject
  Review.held -> ManagerReview
  ManagerReview.approved -> Reserve
  ManagerReview.rejected -> Reject
  Reserve -> Confirm
  Backorder.partial -> Confirm
  Backorder.cancel -> Reject
  Confirm -> Confirmed
  Reject -> Rejected

  end Confirmed { title: { pt-BR: "Confirmado", en: "Confirmed" }; outcome: success }
  end Rejected { title: { pt-BR: "Recusado", en: "Rejected" }; outcome: failure }
}
```

*Valida: evento de início + início manual, form com três desfechos, erro de serviço com desvio, prazo com escalonamento, action encadeada.*

### P.5 Cancelamento de pedido com compensação (orders)

```bpcms
process OrderCancellation for Order {
  title: { pt-BR: "Cancelamento de pedido", en: "Order cancellation" }
  version: 1
  goal: "Cancelar com motivo, devolvendo reserva e pagamento"
  sla: 2d

  pool Company {
    internal
    lane Support { role: SupportAgent }
  }
  pool Customer {
    external
    lane Client { role: Customer }
  }

  variables { eligible: boolean }

  start Requested { by: [SupportAgent, Manager] }

  task Review { lane: Support; form: CancellationReview; sla: 8h }

  task Decide {
    lane: System
    decision: CancellationEligibility
    with { order; reason: review.reason }
    gives: eligible
  }

  gateway Eligible {
    if eligible -> ReleaseStock
    else -> AskCustomer
  }

  task ReleaseStock { lane: System; action: Stock.release; with { order }; compensates: Order.confirm }
  task Refund { lane: System; action: Payment.refund; with { order; amount: order.total }; compensates: Order.confirm }
  task CancelOrder { lane: System; action: Order.cancel; with { order; reason: review.reason } }
  task AskCustomer { lane: Client; form: CancellationOptions; sla: 2d }
  rollback Undo

  Requested -> Review
  Review -> Decide
  Decide -> Eligible
  AskCustomer.keep -> ReleaseStock
  AskCustomer.acceptfee -> ReleaseStock
  ReleaseStock -> Refund
  Refund -> CancelOrder
  CancelOrder -> Cancelled
  Refund -> Undo
  Undo -> Failed

  end Cancelled { title: { pt-BR: "Cancelado", en: "Cancelled" }; outcome: success }
  end Failed { title: { pt-BR: "Falha no cancelamento", en: "Cancellation failed" }; outcome: failure }
}
```

*Valida: decisão de elegibilidade, task no portal do cliente, compensação em ordem reversa, desvio de erro para rollback.*

### P.6 Despacho com paralelismo e coleta pelo parceiro (shipping)

```bpcms
process DispatchOrder for Order {
  title: { pt-BR: "Despacho do pedido", en: "Order dispatch" }
  version: 1
  goal: "Separar, embalar e despachar com frete e rastreio conhecidos"
  sla: 2d

  pool Company {
    internal
    lane Warehouse { role: Packer }
    lane Dispatch { role: Dispatcher }
    lane Management { role: Manager }
  }
  pool Carrier {
    external
  }

  variables { shipment: Shipment; quote: FreightQuote }

  start Approved { event: Order.Approved }

  fork Prepare
  join Prepared

  task Pick { lane: Warehouse; form: PickItems; sla: 2h }
  task Reserve { lane: System; action: Stock.reserve; with { order } }
  task Pack { lane: Warehouse; form: PackShipment; sla: 2h }
  task OpenShipment { lane: System; action: Shipment.open; with { order }; gives: shipment }
  task Quote { lane: System; call: Carrier.quote; with { shipment }; gives: quote; sla: 5m; on error CarrierUnavailable -> ManualQuote }

  gateway Weight {
    if shipment.grossWeight <= carrier.maxWeight -> PrintLabel
    else -> Split
  }

  task Split { lane: Management; form: SplitShipment }
  task PrintLabel { lane: System; call: Carrier.label; with { shipment; quote } }
  task RequestPickup { lane: System; sends: PickupRequest; partner: Carrier; with { shipment } }

  task AwaitPickup {
    lane: System
    receives: PickupConfirmed
    partner: Carrier
    apply: Shipment.applyPickup
    sla: 1d
    on timer 1d -> ManualPickup
  }

  task Close { lane: System; action: Order.dispatch; with { order; shipment } }
  task ManualQuote { lane: Management; form: FreightManualQuote; sla: 4h }
  task ManualPickup { lane: Dispatch; form: PickupConfirmation; sla: 4h }

  Approved -> Prepare
  Prepare -> Pick
  Prepare -> Reserve
  Pick -> Prepared
  Reserve -> Prepared
  Prepared -> Pack
  Pack -> OpenShipment
  OpenShipment -> Quote
  Quote -> Weight
  Split -> PrintLabel
  PrintLabel -> RequestPickup
  RequestPickup -> AwaitPickup
  AwaitPickup -> Close
  ManualQuote -> PrintLabel
  ManualPickup -> Close
  Close -> Dispatched

  end Dispatched { title: { pt-BR: "Despachado", en: "Dispatched" }; outcome: success }
}
```

*Valida: fork/join, gateway por peso, mensagem enviada e recebida com apply, timeouts com desvio, erro de integração.*

### P.7 Rastreamento da entrega como processo filho (shipping)

```bpcms
process DeliveryTracking for Shipment {
  title: { pt-BR: "Rastreamento da entrega", en: "Delivery tracking" }
  version: 1
  goal: "Acompanhar a entrega até confirmar ou devolver"
  sla: 30d

  pool Company {
    internal
    lane Dispatch { role: Dispatcher }
    lane CustomerService { role: SupportAgent }
  }
  pool Customer {
    external
    lane Client { role: Customer }
  }
  pool Carrier {
    external
  }

  start Begin { called }

  task AwaitUpdate {
    lane: System
    receives: TrackingUpdated
    partner: Carrier
    apply: Shipment.applyTracking
    with { shipment }
    sla: 5d
    on timer 5d keep -> RemindCarrier
    on timer 10d -> Investigate
  }

  gateway Status {
    if shipment.status == Delivered -> Confirm
    if shipment.status == Failed -> Retry
    else -> AwaitUpdate
  }

  task RemindCarrier { lane: System; sends: TrackingReminder; partner: Carrier; with { shipment } }
  task Retry { lane: CustomerService; form: RescheduleDelivery; sla: 2d }
  task Confirm { lane: System; action: Shipment.confirmDelivery; with { shipment } }
  task Investigate { lane: Dispatch; form: DeliveryInvestigation; sla: 2d }
  task Return { lane: System; action: Shipment.returnToSender; with { shipment } }

  milestone Delivered {
    title: { pt-BR: "Entrega confirmada", en: "Delivered" }
    on: Shipment.Delivered
  }

  cancel on Shipment.Cancelled -> Return

  Dispatched -> AwaitUpdate
  AwaitUpdate -> Status
  Retry.scheduled -> AwaitUpdate
  Retry.undeliverable -> Return
  Confirm -> Delivered
  Investigate.found -> AwaitUpdate
  Investigate.lost -> Return
  Return -> Returned

  end Delivered { title: { pt-BR: "Entregue", en: "Delivered" }; outcome: success }
  end Returned { title: { pt-BR: "Devolvido", en: "Returned" }; outcome: failure }
  Begin -> AwaitUpdate
}
```

*Valida: processo filho com passo próprio, recebimento contínuo de mensagem, laço com saída, milestone por evento, cancelamento a qualquer momento.*

### P.8 Cotação de frete com múltiplas transportadoras (shipping)

```bpcms
process FreightQuote for Shipment {
  title: { pt-BR: "Cotação de frete", en: "Freight quote" }
  version: 1
  goal: "Escolher transportadora com preço e prazo conhecidos"
  sla: 4h

  pool Company {
    internal
    lane Dispatch { role: Dispatcher }
    lane Management { role: Manager }
  }
  pool Carriers {
    external
  }

  variables { quotes: FreightQuote[]; chosen: FreightQuote }

  start Requested { by: [Dispatcher] }

  task Collect {
    lane: System
    call: FreightBroker.quoteAll
    with { shipment; carriers }
    gives: quotes
    sla: 15m
    on error BrokerUnavailable -> ManualQuote
  }

  task Select { lane: System; decision: CarrierSelection; with { quotes }; gives: chosen }
  task Book { lane: System; call: Carrier.book; with { shipment; quote: chosen }; sla: 10m; on error SlotUnavailable -> ManualQuote }
  task ManualQuote { lane: Management; form: FreightManualQuote; sla: 2h }

  Requested -> Collect
  Collect -> Select
  Select -> Book
  Book -> Booked
  ManualQuote.chosen -> Book
  ManualQuote.abort -> Aborted

  end Booked { title: { pt-BR: "Frete contratado", en: "Freight booked" }; outcome: success }
  end Aborted { title: { pt-BR: "Sem frete", en: "No freight" }; outcome: failure }
}
```

*Valida: integração de agregação, decisão sobre coleção, timeout com desvio, retorno manual ao fluxo.*
### P.9 Checkout com retentativa do cliente (checkout)

```bpcms
process Checkout for CheckoutSession {
  title: { pt-BR: "Finalização da compra", en: "Checkout" }
  version: 1
  goal: "Cotar frete, autorizar pagamento e confirmar o pedido"
  sla: 1h

  pool Company {
    internal
    lane Store { role: Clerk }
  }
  pool Customer {
    external
    lane Client { role: Customer }
  }
  pool Payments {
    external
  }

  start Validated { event: CheckoutSession.Validated }

  task QuoteShipping { lane: System; action: CheckoutSession.quoteShipping; with { session }; sla: 2m }
  task Authorize { lane: System; action: CheckoutSession.authorizePayment; with { session }; sla: 2m; on error GatewayDown -> RetryPayment }

  gateway Authorized {
    if session.paymentAuthorized -> ConfirmOrder
    else -> RetryPayment
  }

  task RetryPayment {
    lane: Client
    form: PaymentRetry
    sla: 30m
    on breach -> Abandon
  }

  task ConfirmOrder { lane: System; action: CheckoutSession.confirmOrder; with { session } }
  task Abandon { lane: System; action: CheckoutSession.abandon; with { session } }

  Validated -> QuoteShipping
  QuoteShipping -> Authorize
  Authorize -> Authorized
  RetryPayment.retry -> Authorize
  RetryPayment.giveup -> Abandon
  ConfirmOrder -> Confirmed
  Abandon -> Abandoned

  end Confirmed { title: { pt-BR: "Pedido confirmado", en: "Order confirmed" }; outcome: success }
  end Abandoned { title: { pt-BR: "Compra abandonada", en: "Checkout abandoned" }; outcome: cancelled }
}
```

*Valida: fluxo quase todo automático, task humana no portal do cliente, laço de retentativa, timeout como abandono.*

### P.10 Captura de pagamento com estorno (payments)

```bpcms
process PaymentCapture for Payment {
  title: { pt-BR: "Captura de pagamento", en: "Payment capture" }
  version: 1
  goal: "Autorizar e capturar o pagamento, estornando em caso de falha"
  sla: 1h

  pool Company {
    internal
    lane Finance { role: FinanceAnalyst }
  }
  pool Gateway {
    external
  }

  variables { authorization: Authorization }

  start Requested { event: Payment.Requested }

  task Authorize {
    lane: System
    call: Gateway.authorize
    with { payment }
    gives: authorization
    sla: 30s
    on error Declined -> Declined
  }

  task Capture {
    lane: System
    call: Gateway.capture
    with { payment; authorization }
    sla: 30s
    on error CaptureFailed -> Undo
  }

  task Reverse {
    lane: System
    call: Gateway.refund
    with { payment; authorization }
    compensates: Capture
  }

  rollback Undo

  Requested -> Authorize
  Authorize -> Capture
  Capture -> Captured
  Capture -> Undo
  Undo -> Failed

  end Captured { title: { pt-BR: "Capturado", en: "Captured" }; outcome: success }
  end Declined { title: { pt-BR: "Recusado", en: "Declined" }; outcome: failure }
  end Failed { title: { pt-BR: "Falha na captura", en: "Capture failed" }; outcome: failure }
}
```

*Valida: integração pura, compensação acionada por rollback, desfechos de negócio distintos.*

### P.11 Disputa de chargeback com a bandeira (payments)

```bpcms
process ChargebackDispute for Chargeback {
  title: { pt-BR: "Disputa de chargeback", en: "Chargeback dispute" }
  version: 1
  goal: "Responder a disputa no prazo com evidência"
  sla: 30d

  pool Company {
    internal
    lane Finance { role: FinanceAnalyst }
    lane Management { role: Manager }
  }
  pool Network {
    external
  }

  start Opened { event: Chargeback.Opened }

  task GatherEvidence { lane: Finance; form: ChargebackEvidence; sla: 7d; on breach -> Escalate }
  task Submit { lane: System; sends: ChargebackEvidence; partner: Network; with { chargeback } }

  task AwaitDecision {
    lane: System
    receives: ChargebackDecided
    partner: Network
    apply: Chargeback.applyDecision
    with { chargeback }
    sla: 21d
    on timer 21d -> Escalate
  }

  gateway Outcome {
    if chargeback.status == won -> Close
    else -> AcceptLoss
  }

  task Close { lane: System; action: Chargeback.close; with { chargeback } }
  task AcceptLoss { lane: System; action: Chargeback.acceptLoss; with { chargeback } }
  task Escalate { lane: Management; form: ChargebackEscalation; sla: 2d }

  Opened -> GatherEvidence
  GatherEvidence -> Submit
  Submit -> AwaitDecision
  AwaitDecision -> Outcome
  Escalate.submitted -> Submit
  Escalate.writeoff -> AcceptLoss
  Close -> Won
  AcceptLoss -> Lost

  end Won { title: { pt-BR: "Disputa ganha", en: "Dispute won" }; outcome: success }
  end Lost { title: { pt-BR: "Disputa perdida", en: "Dispute lost" }; outcome: failure }
}
```

*Valida: prazo legal com dois timers, mensagem enviada e recebida com apply, escalonamento humano, desfechos.*

### P.12 Emissão de nota fiscal (billing)

```bpcms
process InvoiceIssuance for Invoice {
  title: { pt-BR: "Emissão de nota fiscal", en: "Invoice issuance" }
  version: 1
  goal: "Emitir a nota conferida e entregar ao cliente"
  sla: 1d

  pool Company {
    internal
    lane Billing { role: BillingClerk }
    lane Management { role: Manager }
  }
  pool Customer {
    external
  }

  variables { invoice: Invoice }

  start Requested { by: [BillingClerk, Manager] }
  start FromOrder { event: Order.Invoiced }

  on breach -> Escalate

  task Prepare { lane: System; action: Invoice.prepare; with { order }; gives: invoice }
  task Review { lane: Billing; form: InvoiceReview; sla: 4h }

  gateway Reviewed {
    if review.approved -> Issue
    else -> Cancel
  }

  task Issue { lane: System; action: Invoice.issue; with { invoice } }
  task Send { lane: System; sends: InvoiceIssued; partner: Customer; with { invoice } }
  task Cancel { lane: Management; form: InvoiceCancellation; sla: 4h }
  task Escalate { lane: Management; form: InvoiceEscalation; sla: 2h }

  Requested -> Prepare
  FromOrder -> Prepare
  Prepare -> Review
  Review -> Reviewed
  Issue -> Send
  Send -> Issued
  Cancel -> Cancelled
  Escalate.proceed -> Issue
  Escalate.abort -> Cancel

  end Issued { title: { pt-BR: "Nota emitida", en: "Invoice issued" }; outcome: success }
  end Cancelled { title: { pt-BR: "Nota cancelada", en: "Invoice cancelled" }; outcome: cancelled }
}
```

*Valida: dois starts (manual e por evento de outro documento), mensagem ao cliente, aprovação e cancelamento.*

### P.13 Cobrança de faturas vencidas (billing)

```bpcms
process Dunning for Invoice {
  title: { pt-BR: "Cobrança de faturas", en: "Invoice dunning" }
  version: 1
  goal: "Cobrar sem desgastar o cliente e escalar quando necessário"
  sla: 45d

  pool Company {
    internal
    lane Collections { role: CollectionsAgent }
    lane Management { role: Manager }
  }
  pool Customer {
    external
  }

  start Daily {
    schedule: "0 6 * * *"
    select: invoice.status == Overdue and invoice.daysOverdue >= 1
  }

  task Remind { lane: System; sends: PaymentReminder; partner: Customer; with { invoice } }
  wait Grace { for: 7d }

  gateway Late {
    if invoice.daysOverdue > 30 -> Escalate
    else -> Remind
  }

  task Escalate { lane: Collections; form: CollectionCall; sla: 2d }
  task WriteOff { lane: Management; form: WriteOffApproval; sla: 3d }

  cancel on Invoice.Paid -> Settled

  Daily -> Remind
  Remind -> Grace
  Grace -> Late
  Escalate.promised -> Remind
  Escalate.unreachable -> WriteOff
  WriteOff -> WrittenOff

  end Settled { title: { pt-BR: "Fatura paga", en: "Invoice paid" }; outcome: success }
  end WrittenOff { title: { pt-BR: "Perda reconhecida", en: "Written off" }; outcome: failure }
}
```

*Valida: start agendado com fan-out por sujeito, espera por tempo, laço com saída, cancelamento por evento externo.*

### P.14 Fechamento mensal (billing)

```bpcms
process MonthClose for FiscalPeriod {
  title: { pt-BR: "Fechamento mensal", en: "Month close" }
  version: 1
  goal: "Fechar o período com números conferidos"
  owner: "Controladoria"
  sla: 6h

  pool Company {
    internal
    lane Controllership { role: Controller }
  }

  start Monthly {
    schedule: "0 2 1 * *"
    select: fiscalPeriod.status == Open
  }

  task PostEntries { lane: System; action: Ledger.postPeriod; with { fiscalPeriod }; on error Unbalanced -> Fix }
  task Reconcile { lane: System; action: Ledger.reconcile; with { fiscalPeriod }; on error Unbalanced -> Fix }
  task Review { lane: Controllership; form: MonthCloseReview; sla: 4h }
  task Fix { lane: Controllership; form: ReconciliationFix; sla: 1d }
  task Close { lane: System; action: FiscalPeriod.close; with { fiscalPeriod } }

  Monthly -> PostEntries
  PostEntries -> Reconcile
  Reconcile -> Review
  Review.approved -> Close
  Review.rejected -> Fix
  Fix -> Reconcile
  Close -> Closed

  end Closed { title: { pt-BR: "Período fechado", en: "Period closed" }; outcome: success }
  end Failed { title: { pt-BR: "Fechamento falhou", en: "Close failed" }; outcome: failure }
}
```

*Valida: rotina agendada como processo, tarefa de revisão humana, erro de sistema com correção, desfecho nomeado.*

### P.15 Reposição de estoque (inventory)

```bpcms
process StockReplenishment for StockItem {
  title: { pt-BR: "Reposição de estoque", en: "Stock replenishment" }
  version: 1
  goal: "Repor o que está abaixo do ponto de pedido sem excesso"
  sla: 2d

  pool Company {
    internal
    lane Planning { role: Planner }
    lane Management { role: Manager }
  }

  variables { decision: ReplenishmentDecision }

  start Daily {
    schedule: "0 7 * * *"
    select: stockItem.available < stockItem.reorderPoint
  }

  task Decide {
    lane: System
    decision: ReplenishmentOrder
    with { available: stockItem.available; reorderPoint: stockItem.reorderPoint; leadTime: stockItem.leadTime }
    gives: decision
  }

  gateway Action {
    if decision.action == order -> CreateRequest
    if decision.action == transfer -> Transfer
    else -> Defer
  }

  task CreateRequest {
    lane: System
    action: PurchaseRequest.create
    with { stockItem; quantity: decision.quantity }
    gives: purchaseRequest
  }

  task SubmitRequest { lane: System; action: PurchaseRequest.submit; with { purchaseRequest } }
  task Transfer { lane: Planning; form: StockTransfer; sla: 1d }
  task Defer { lane: System; action: StockItem.deferReplenishment; with { stockItem; until: decision.until } }

  Daily -> Decide
  Decide -> Action
  CreateRequest -> SubmitRequest
  SubmitRequest -> Requested
  Transfer -> Replenished
  Defer -> Deferred

  end Requested { title: { pt-BR: "Compra solicitada", en: "Purchase requested" }; outcome: success }
  end Replenished { title: { pt-BR: "Transferido", en: "Transferred" }; outcome: success }
  end Deferred { title: { pt-BR: "Adiado", en: "Deferred" }; outcome: success }
}
```

*Valida: decisão por faixa, `runs:` como subprocesso de outro serviço, três saídas de negócio.*

### P.16 Inventário cíclico com recontagem (inventory)

```bpcms
process CycleCount for CycleCount {
  title: { pt-BR: "Inventário cíclico", en: "Cycle count" }
  version: 1
  goal: "Contar, explicar divergência e ajustar o estoque"
  sla: 3d

  pool Company {
    internal
    lane Warehouse { role: Counter }
    lane Management { role: Manager }
  }

  variables { divergent: boolean }

  start Scheduled { event: CycleCount.Scheduled }

  task Count {
    lane: Warehouse
    form: CountSheet
    foreach { over: cycleCount.locations; as: location; mode: parallel }
    sla: 2h
  }

  task Compare {
    lane: System
    decision: CountVariance
    with { expected: cycleCount.expectedTotal; counted: cycleCount.countedTotal }
    gives: divergent
  }

  gateway Variance {
    if divergent -> Recount
    else -> Approve
  }

  task Recount { lane: Warehouse; form: RecountSheet; sla: 4h }

  task Adjust { lane: System; action: StockItem.adjust; with { cycleCount } }
  task Approve { lane: Management; form: CountApproval; sla: 1d }

  Scheduled -> Count
  Count -> Compare
  Compare -> Variance
  Recount.recounted -> Compare
  Recount.giveup -> Approve
  Approve.approved -> Adjust
  Approve.rejected -> Rejected
  Adjust -> Adjusted

  end Adjusted { title: { pt-BR: "Estoque ajustado", en: "Stock adjusted" }; outcome: success }
  end Rejected { title: { pt-BR: "Contagem rejeitada", en: "Count rejected" }; outcome: failure }
}
```

*Valida: multi-instância paralela, decisão de tolerância, task discricionária com condição, laço de recontagem.*
### P.17 Triagem de chamado com SLA (support)

```bpcms
process TicketTriage for Ticket {
  title: { pt-BR: "Triagem de chamado", en: "Ticket triage" }
  version: 1
  goal: "Classificar, resolver e comunicar dentro do prazo"
  sla: 3d

  pool Company {
    internal
    lane ServiceDesk { role: SupportAgent }
    lane Engineering { role: Engineer }
    lane Management { role: Manager }
  }
  pool Customer {
    external
    lane Client { role: Customer }
  }

  variables { queue: TicketQueue }

  start Opened { event: Ticket.Opened }

  task Classify {
    lane: System
    decision: TicketRouting
    with { subject: ticket.subject; tier: ticket.customerTier }
    gives: queue
  }

  gateway Route {
    if queue == serviceDesk -> FirstContact
    if queue == engineering -> Diagnose
    else -> BillingQueue
  }

  task FirstContact { lane: ServiceDesk; form: TicketTriageForm; sla: 2h; on breach -> Escalate }
  task BillingQueue { lane: ServiceDesk; form: TicketTriageForm; sla: 4h; on breach -> Escalate }
  task Diagnose { lane: Engineering; form: TicketDiagnosis; sla: 8h; on breach -> Escalate }

  fork Respond
  join Responded

  task Fix { lane: Engineering; form: FixRecord; sla: 1d }
  task UpdateCustomer { lane: ServiceDesk; form: CustomerUpdate; sla: 4h }

  task Confirm { lane: Client; form: ResolutionConfirm; sla: 2d }
  task Escalate { lane: Management; form: TicketEscalation; sla: 1h }
  task Close { lane: System; action: Ticket.close; with { ticket } }

  Opened -> Classify
  Classify -> Route
  FirstContact -> Respond
  BillingQueue -> Respond
  Diagnose -> Respond
  Respond -> Fix
  Respond -> UpdateCustomer
  Fix -> Responded
  UpdateCustomer -> Responded
  Responded -> Confirm
  Confirm.resolved -> Close
  Confirm.reopened -> FirstContact
  Escalate.resolved -> Close
  Close -> Resolved

  end Resolved { title: { pt-BR: "Resolvido", en: "Resolved" }; outcome: success }
  end Escalated { title: { pt-BR: "Escalado", en: "Escalated" }; outcome: failure }
}
```

*Valida: roteamento por decisão, SLA por fila com escalonamento, trabalho paralelo (correção + comunicação), confirmação no portal.*

### P.18 Reembolso com alçada e confirmação (support)

```bpcms
process RefundCase for ReturnRequest {
  title: { pt-BR: "Caso de reembolso", en: "Refund case" }
  version: 1
  goal: "Reembolsar quando elegível, com aprovação e rastro"
  sla: 5d

  pool Company {
    internal
    lane Support { role: SupportAgent }
    lane Finance { role: Manager }
  }
  pool Customer {
    external
    lane Client { role: Customer }
  }

  variables { eligible: boolean }

  start Requested { event: ReturnRequest.Requested }

  task Review { lane: Support; form: ReturnReview; sla: 8h }

  task Eligibility {
    lane: System
    decision: RefundEligibility
    with { amount: returnRequest.amount; tier: returnRequest.customer.tier }
    gives: eligible
  }

  gateway Eligible {
    if eligible -> Approve
    else -> Reject
  }

  task Approve { lane: Finance; form: RefundApproval; sla: 1d }
  task Refund { lane: System; action: Payment.refund; with { returnRequest } }
  task NotifyDone { lane: System; sends: RefundConfirmed; partner: Customer; with { returnRequest } }
  task Reject { lane: Support; form: RefundRejection; sla: 4h }

  milestone Refunded {
    title: { pt-BR: "Reembolso efetuado", en: "Refund completed" }
    on: Payment.Refunded
  }

  cancel on ReturnRequest.Cancelled -> Cancelled

  Requested -> Review
  Review -> Eligibility
  Eligibility -> Eligible
  Approve.approved -> Refund
  Approve.rejected -> Reject
  Refund -> NotifyDone
  NotifyDone -> Refunded
  Reject -> Rejected

  end Refunded { title: { pt-BR: "Reembolsado", en: "Refunded" }; outcome: success }
  end Rejected { title: { pt-BR: "Não elegível", en: "Not eligible" }; outcome: failure }
  end Cancelled { title: { pt-BR: "Cancelado pelo cliente", en: "Cancelled by customer" }; outcome: cancelled }
}
```

*Valida: decisão de elegibilidade, alçada por lane, mensagem ao cliente, milestone, cancelamento com desfecho próprio.*

### P.19 Exame laboratorial com assinatura (laboratory)

```bpcms
process LabExam for LabExam {
  title: { pt-BR: "Exame laboratorial", en: "Laboratory exam" }
  version: 1
  goal: "Liberar laudo confiável no prazo"
  sla: 8h

  pool Company {
    internal
    lane Reception { role: Collector }
    lane Analysis { role: Analyst }
    lane Pathology { role: Pathologist }
  }
  pool Clinic {
    external
  }

  variables { result: ExamResult }

  start Collected { event: LabExam.Collected }

  task Receive { lane: Reception; form: SampleReception; sla: 1h }

  task Analyze {
    lane: System
    call: LabAnalyzer.run
    with { labExam }
    gives: result
    sla: 2h
    on error AnalyzerOffline -> ManualAnalyze
  }

  task ManualAnalyze { lane: Analysis; form: ManualResultEntry; sla: 4h }

  task Validate {
    lane: Analysis
    form: ResultValidation
    sla: 4h
    on breach -> Escalate
  }

  gateway Critical {
    if labExam.critical -> NotifyDoctor
    else -> Sign
  }

  task NotifyDoctor { lane: System; sends: CriticalResult; partner: Clinic; with { labExam; result } }
  task Sign { lane: Pathology; form: ReportSignature; sla: 2h }
  task Release { lane: System; action: LabExam.release; with { labExam; signature: sign.signature } }
  task Escalate { lane: Pathology; form: LabEscalation; sla: 1h }

  Collected -> Receive
  Receive -> Analyze
  Analyze -> Validate
  ManualAnalyze -> Validate
  Validate -> Critical
  NotifyDoctor -> Sign
  Sign -> Release
  Release -> Released
  Escalate.released -> Release
  Escalate.rejected -> Rejected

  end Released { title: { pt-BR: "Laudo liberado", en: "Report released" }; outcome: success }
  end Rejected { title: { pt-BR: "Exame rejeitado", en: "Exam rejected" }; outcome: failure }
}
```

*Valida: dispositivo com fallback manual, caminho crítico por decisão, assinatura como task, prazo com escalonamento.*

### P.20 Onboarding de colaborador (hr)

```bpcms
process EmployeeOnboarding for Employee {
  title: { pt-BR: "Onboarding de colaborador", en: "Employee onboarding" }
  version: 1
  goal: "Novo colaborador produtivo no primeiro dia"
  sla: 15d

  pool Company {
    internal
    lane People { role: HR }
    lane IT { role: ITAnalyst }
    lane Management { role: Manager }
  }
  pool NewHire {
    external
    lane CandidatePortal { role: Candidate }
  }

  start Hired { event: Candidate.Hired }
  cancel on Candidate.Withdrawn -> Cancel

  fork Start
  join Ready

  task SubmitDocuments { lane: CandidatePortal; form: NewHireDocuments; sla: 5d }
  task CreateAccounts { lane: IT; form: AccountRequest; sla: 2d }
  task OrderEquipment { lane: IT; form: EquipmentRequest; sla: 3d }
  task Provision { lane: System; runs: AccessProvisioning; with { employee } }
  task Checkin { lane: Management; form: FirstDayCheckin; sla: 1d }
  task Welcome { lane: People; manual: "Receber o colaborador e apresentar o time"; sla: 2h }
  task Activate { lane: System; action: Employee.activate; with { employee } }
  task Cancel { lane: System; action: Employee.cancelOnboarding; with { employee } }

  task OrderSwag {
    lane: People
    form: SwagRequest
    discretionary
    when: employee.level == senior
  }

  Hired -> Start
  Start -> SubmitDocuments
  Start -> CreateAccounts
  Start -> OrderEquipment
  SubmitDocuments -> Ready
  CreateAccounts -> Ready
  OrderEquipment -> Ready
  Ready -> Provision
  Provision -> Checkin
  Checkin -> Welcome
  Welcome -> Activate
  Activate -> Active
  Cancel -> Cancelled

  end Active { title: { pt-BR: "Colaborador ativo", en: "Employee active" }; outcome: success }
  end Cancelled { title: { pt-BR: "Onboarding cancelado", en: "Onboarding cancelled" }; outcome: cancelled }
}
```

*Valida: paralelismo humano, portal do candidato, subprocesso de provisionamento, task manual, cancelamento.*

### P.21 Sinistro com score de fraude (claims)

```bpcms
process ClaimSettlement for Claim {
  title: { pt-BR: "Liquidação de sinistro", en: "Claim settlement" }
  version: 1
  goal: "Decidir cobertura e pagar com rastro"
  sla: 30d

  pool Company {
    internal
    lane Adjusters { role: Adjuster }
    lane Supervision { role: Supervisor }
  }
  pool Bank {
    external
  }

  variables { risk: RiskLevel; covered: boolean; settlement: Settlement }

  start Opened { event: Claim.Opened }

  task Register { lane: Adjusters; form: ClaimRegistration; sla: 4h }

  task Score {
    lane: System
    decision: FraudRisk
    with { claim }
    gives: risk
  }

  gateway Suspicious {
    if risk == High -> FraudReview
    else -> Coverage
  }

  task FraudReview { lane: Supervision; form: FraudReviewForm; sla: 2d; on breach -> Reject }

  task Coverage {
    lane: System
    decision: CoverageEligibility
    with { claim; policy: claim.policy }
    gives: covered
  }

  gateway Eligible {
    if covered -> Approve
    else -> Reject
  }

  task Approve { lane: Supervision; form: SettlementApproval; sla: 1d }
  task Pay { lane: System; action: Settlement.pay; with { settlement }; on error BankDown -> RetryPay }
  task RetryPay { lane: System; action: Settlement.pay; with { settlement }; sla: 1h }
  task Reject { lane: Adjusters; form: ClaimRejection; sla: 4h }

  Opened -> Register
  Register -> Score
  Score -> Suspicious
  FraudReview.cleared -> Coverage
  FraudReview.confirmed -> Reject
  Coverage -> Eligible
  Approve.approved -> Pay
  Approve.rejected -> Reject
  Pay -> Settled
  RetryPay -> Settled
  Reject -> Rejected

  end Settled { title: { pt-BR: "Sinistro liquidado", en: "Claim settled" }; outcome: success }
  end Rejected { title: { pt-BR: "Sinistro negado", en: "Claim rejected" }; outcome: failure }
}
```

*Valida: dois níveis de decisão, revisão de fraude com prazo, pagamento com recuperação de falha.*

### P.22 Visita técnica em campo (field-service)

```bpcms
process ServiceVisit for Visit {
  title: { pt-BR: "Visita técnica", en: "Field service visit" }
  version: 1
  goal: "Atender no local, registrar prova e concluir"
  sla: 2d

  pool Company {
    internal
    lane Coordination { role: Coordinator }
    lane Field { role: Technician }
  }
  pool Customer {
    external
    lane Client { role: Customer }
  }

  start Scheduled { event: Visit.Scheduled }
  cancel on Visit.Cancelled -> Cancel

  task Dispatch { lane: Coordination; form: VisitDispatch; sla: 1h }
  task Travel { lane: Field; manual: "Deslocamento até o cliente"; sla: 2h }
  task CheckIn { lane: Field; form: VisitCheckIn; sla: 30m; on error OutOfRange -> Reschedule }

  task Perform {
    lane: Field
    form: VisitReport
    sla: 3h
    on error CustomerAbsent -> Reschedule
  }

  task Accept { lane: Client; form: VisitAcceptance; sla: 2d }
  task Complete { lane: System; action: Visit.complete; with { visit; report } }
  task Reschedule { lane: Coordination; form: VisitReschedule; sla: 2h }
  task Cancel { lane: System; action: Visit.cancel; with { visit } }

  Scheduled -> Dispatch
  Dispatch -> Travel
  Travel -> CheckIn
  CheckIn -> Perform
  Perform -> Accept
  Accept.accepted -> Complete
  Accept.disputed -> Reschedule
  Reschedule.scheduled -> Dispatch
  Reschedule.abandoned -> Cancel
  Complete -> Completed
  Cancel -> Cancelled

  end Completed { title: { pt-BR: "Visita concluída", en: "Visit completed" }; outcome: success }
  end Cancelled { title: { pt-BR: "Visita cancelada", en: "Visit cancelled" }; outcome: cancelled }
}
```

*Valida: task manual, validação de localização com desvio, aceite no portal do cliente, reagendamento.*

### P.23 Análise de crédito com alçada (credit)

```bpcms
process CreditApplicationFlow for CreditApplication {
  title: { pt-BR: "Análise de crédito", en: "Credit analysis" }
  version: 1
  goal: "Conceder limite com risco calculado e decisão rastreável"
  sla: 5d

  pool Company {
    internal
    lane Analysis { role: CreditAnalyst }
    lane Management { role: Manager }
  }
  pool Bureau {
    external
  }

  variables { score: CreditScore; report: CompanyReport }

  start Submitted { event: CreditApplication.Submitted }

  task Score { lane: System; decision: CreditScoring; with { application }; gives: score }

  task CheckBureau {
    lane: System
    call: Bureau.checkCompany
    with { customer: application.customer }
    gives: report
    sla: 1m
    on error BureauDown -> ManualReview
  }

  gateway Risk {
    if score.tier == Auto -> SetLimit
    if score.tier == review -> ManualReview
    else -> Reject
  }

  task ManualReview { lane: Analysis; form: CreditReview; sla: 2d; on breach -> Escalate }
  task Approve { lane: Management; form: CreditApproval; sla: 1d }
  task SetLimit { lane: System; action: CreditLimit.set; with { application; amount: score.limit } }
  task Reject { lane: Analysis; form: CreditRejection; sla: 4h }
  task Escalate { lane: Management; form: CreditEscalation; sla: 1d }

  Submitted -> Score
  Score -> CheckBureau
  CheckBureau -> Risk
  ManualReview.recommend -> Approve
  ManualReview.decline -> Reject
  Approve.approved -> SetLimit
  Approve.rejected -> Reject
  Escalate.decided -> Approve
  SetLimit -> Approved
  Reject -> Rejected

  end Approved { title: { pt-BR: "Crédito aprovado", en: "Credit approved" }; outcome: success }
  end Rejected { title: { pt-BR: "Crédito negado", en: "Credit rejected" }; outcome: failure }
}
```

*Valida: integração de bureau com fallback, três faixas de decisão, dupla aprovação, escalonamento.*

### P.24 Renovação de contrato (contracts)

```bpcms
process ContractRenewal for Contract {
  title: { pt-BR: "Renovação de contrato", en: "Contract renewal" }
  version: 1
  goal: "Renovar antes do vencimento com contraproposta tratada"
  sla: 60d

  pool Company {
    internal
    lane Accounts { role: AccountManager }
    lane Legal { role: LegalAnalyst }
  }
  pool Counterparty {
    external
    lane Portal { role: SupplierUser }
  }

  start Expiring {
    schedule: "0 8 * * *"
    select: contract.status == Active and contract.endsAt <= today() + 60d
  }

  task Propose { lane: Accounts; form: RenewalProposal; sla: 5d }

  task SendProposal {
    lane: System
    sends: RenewalProposalSent
    partner: Counterparty
    with { contract; proposal: contract.proposal }
  }

  task AwaitCounter {
    lane: System
    receives: RenewalCounterOffer
    partner: Counterparty
    apply: Contract.applyCounterOffer
    with { contract }
    sla: 10d
    on timer 10d -> Escalate
  }

  gateway Agreed {
    if contract.counterAccepted -> LegalReview
    else -> Escalate
  }

  task LegalReview { lane: Legal; form: ContractLegalReview; sla: 3d }
  task Sign { lane: Portal; form: ContractSignature; sla: 5d }
  task Renew { lane: System; action: Contract.renew; with { contract } }
  task Escalate { lane: Accounts; form: RenewalEscalation; sla: 2d }

  Expiring -> Propose
  Propose -> SendProposal
  SendProposal -> AwaitCounter
  AwaitCounter -> Agreed
  LegalReview.approved -> Sign
  LegalReview.rejected -> Escalate
  Sign.signed -> Renew
  Sign.declined -> Escalate
  Escalate.renegotiate -> Propose
  Escalate.letlapse -> Lapsed
  Renew -> Renewed

  end Renewed { title: { pt-BR: "Contrato renovado", en: "Contract renewed" }; outcome: success }
  end Lapsed { title: { pt-BR: "Contrato vencido", en: "Contract lapsed" }; outcome: failure }
}
```

*Valida: fan-out agendado por sujeito, mensagem com contraproposta, assinatura no portal, loop de renegociação.*
## Form

O form é o **contrato da tarefa humana**: o que a pessoa vê, o que ela pode fazer e qual ação de documento cada botão executa. A tela é do builder; o form não tem `roles`, não escreve campo direto e não tem ciclo de vida.

### F.1 Aprovação com três desfechos e rascunho

```bpcms
form PurchaseReview {
  title: { pt-BR: "Revisar compra", en: "Review purchase" }
  document: PurchaseRequest

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

```bpcms
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

```bpcms
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

*Valida: o mesmo form serve a uma lane externa (portal); campo sensível declarado no form e no documento; anexos.*

### F.4 Form de campo com assinatura e prova

```bpcms
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

```bpcms
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

```bpcms
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

```bpcms
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

```bpcms
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

```bpcms
task ManagerReview {
  lane: Management
  form: ApprovalDecision
  assign: named(purchaseRequest.requester.manager)
  sla: 2d
}

task DirectorReview {
  lane: Direction
  form: ApprovalDecision
  sla: 2d
}
```

*Valida: um form reutilizado por duas tasks de lanes diferentes; o roteamento é do processo (`Task.approved -> ...`), não do form.*

## Message

Mensagem é contrato entre participantes: tem `about` (o documento que correlaciona), `payload` tipado e título. O participante é declarado na task (`partner:`), porque é a task que sabe com quem fala.

### M.1 Envio de solicitação

```bpcms
message PickupRequest {
  title: { pt-BR: "Solicitação de coleta", en: "Pickup request" }
  about: Shipment
  payload {
    requestedAt: DateTime { title: { pt-BR: "Solicitado em" } }
    window: Duration { title: { pt-BR: "Janela de coleta" } }
  }
}

task RequestPickup {
  lane: System
  sends: PickupRequest
  partner: Carrier
  with { requestedAt: now(); window: shipment.pickupWindow }
}
```

*Valida: mensagem com correlação por documento e envio por task de sistema.*

### M.2 Recebimento com aplicação no documento

```bpcms
message PickupConfirmed {
  title: { pt-BR: "Coleta confirmada", en: "Pickup confirmed" }
  about: Shipment
  payload {
    pickupAt: DateTime { title: { pt-BR: "Coletado em" } }
    tracking: TrackingCode { title: { pt-BR: "Rastreio" } }
  }
}

task AwaitPickup {
  lane: System
  receives: PickupConfirmed
  partner: Carrier
  apply: Shipment.applyPickup
  with { shipment }
  sla: 1d
  on timer 1d -> ManualPickup
}
```

*Valida: recebimento correlacionado aplica o payload numa action; timeout com desvio.*

### M.3 Atualização contínua de rastreio

```bpcms
message TrackingUpdated {
  title: { pt-BR: "Rastreio atualizado", en: "Tracking updated" }
  about: Shipment
  payload {
    status: ShipmentStatus { title: { pt-BR: "Situação" } }
    at: DateTime { title: { pt-BR: "Quando" } }
    note: string { title: { pt-BR: "Observação" } }
  }
}

task AwaitTracking {
  lane: System
  receives: TrackingUpdated
  partner: Carrier
  apply: Shipment.applyTracking
  with { shipment }
  sla: 5d
  on timer 5d keep -> RemindCarrier
}
```

*Valida: mensagem recorrente no mesmo passo; aplicação idempotente no documento; lembrete não interruptivo.*

### M.4 Mensagem que inicia processo

```bpcms
message PaymentConfirmed {
  title: { pt-BR: "Pagamento confirmado", en: "Payment confirmed" }
  about: Order
  payload {
    paidAt: DateTime { title: { pt-BR: "Pago em" } }
    amount: std.Money { title: { pt-BR: "Valor" } }
  }
}

process OrderPacking for Order {
  title: { pt-BR: "Separação do pedido", en: "Order packing" }
  version: 1

  pool Company {
    internal
    lane Warehouse { role: Packer }
  }

  start Paid { message: PaymentConfirmed }
  task Pack { lane: Warehouse; form: PackShipment; sla: 4h }
  task Ready { lane: System; action: Order.readyToShip; with { order } }

  Paid -> Pack
  Pack -> Ready
  Ready -> Packed

  end Packed { title: { pt-BR: "Pronto para envio", en: "Ready to ship" }; outcome: success }
}
```

*Valida: mensagem como gatilho de início; correlação pelo sujeito; processo curto e completo.*
## Document

### D.1 Cliente com identidade, PII e renomeação

```bpcms
document Customer {
  title: { pt-BR: "Cliente", en: "Customer" }
  version: 1
  owner: "CRM"
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

```bpcms
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
  version: 1
  owner: "Billing"
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
      default: 0 BRL
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

```bpcms
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
  version: 1
  owner: "Dispatch"
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
  version: 1
  owner: "Dispatch"
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

```bpcms
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
  version: 1
  owner: "Laboratório"
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

```bpcms
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
  version: 1
  owner: "Support"
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

```bpcms
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
  version: 1
  owner: "Procurement"
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

```bpcms
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
  version: 1
  owner: "Legal"
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

```bpcms
document MonthlyRevenue {
  title: { pt-BR: "Receita mensal", en: "Monthly revenue" }
  version: 1
  owner: "Finance"
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

```bpcms
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

```bpcms
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

```bpcms
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

```bpcms
shape TemperatureReading {
  title: { pt-BR: "Leitura de temperatura", en: "Temperature reading" }
  fields {
    value: std.Temperature { title: { pt-BR: "Valor", en: "Value" } }
  }
}
```

*Valida: temperatura como tipo padrão da stdlib.*

### H.5 Percentual via std

```bpcms
shape DiscountPolicy {
  title: { pt-BR: "Política de desconto", en: "Discount policy" }
  fields {
    rate: std.Percentage { title: { pt-BR: "Taxa", en: "Rate" } }
  }
}
```

*Valida: percentual como tipo padrão, usado por um Shape de política de desconto.*

### H.6 CPF com validação e sensibilidade

```bpcms
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

```bpcms
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

```bpcms
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

```bpcms
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

```bpcms
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

```bpcms
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

```bpcms
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

```bpcms
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

```bpcms
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

```bpcms
view MyWork {
  title: { pt-BR: "Minhas tarefas", en: "My work" }
  type: inbox
  roles: [Clerk, Manager, SupportAgent]
  scope: mine
  groupBy: process

  columns { title; subject; lane; dueAt; priority }
}
```

*Valida: inbox como projeção de tarefas (não de documentos), `scope: mine` para o que está atribuído ou pegável, agrupamento por processo e colunas derivadas do passo, da lane e do prazo.*

### V.7 Andamento do processo de pedido

```bpcms
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

```bpcms
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

```bpcms
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

```bpcms
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

```bpcms
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

```bpcms
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

```bpcms
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

```bpcms
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

```bpcms
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

  LabManager manages [Collector, Analyst, SupportLiaison]
  LabManager delegates to Pathologist when absent for 7d

  segregation { exam.perform != exam.release }
  signature { Pathologist }
}
```

*Valida: coletor que também executa alguns exames, analista com escopo maior, apoio externo e assinatura.*

No serviço `laboratory`, `Sample.kind` distingue `patient`, `support` e `control`. Amostras de apoio e controle não referenciam paciente; usam `supportLab`/`controlLot`.

O `LabManager` pode atribuir `Collector`, `Analyst` e `SupportLiaison` a outras pessoas, mas nunca a si mesmo. Quem recebe a role ganha as permissões da role atribuída, não as permissões do gerente; o gerente não precisa ter as permissões da role gerenciada.

```bpcms
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

```bpcms
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

```bpcms
evidence PurchaseFormPDF {
  kind: form
  file: "evidence/purchase-form.pdf"
  hash: "sha256:7f3a..."
  confidentiality: internal
  source: "Compras > 2026-10"

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

```bpcms
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

```bpcms
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

```bpcms
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

```bpcms
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

```bpcms
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
    documents: proposed
  }
  assumptions: [A-001, A-002]
  status: in_progress
  next: "Confirmar alçada do diretor"
}
```

*Valida: sessão com cobertura parcial, evidências e próxima pergunta explícita.*

### C.2 Triagem clínica com dados sensíveis

```bpcms
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

```bpcms
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

```bpcms
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

```bpcms
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

```bpcms
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

Decisão é uma **tabela DMN pura** (4.17 / 11): `returns` tipado (normalmente uma `classification`), `inputs` tipados e `cases` com `else` obrigatório. Quem usa é a task (`decision:` + `with` + `gives:`); o caminho é escolhido por `gateway`. Decisão não escreve documento, não chama integração e não emite evento. Não existe `appliesTo`, `when` nem `default`: o `else` cobre o resto.

### N.1 Alçada de compra

```bpcms
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

  source: "Política de Compras v3"
  owner: "Financeiro"
}

task DecideApproval {
  lane: System
  decision: ApprovalLimit
  with { amount: purchaseRequest.amount; supplier: purchaseRequest.supplier }
  gives: tier
}

gateway Tier {
  if tier == Auto -> AutoApprove
  if tier == Manager -> ManagerReview
  else -> DirectorReview
}
```

*Valida: `returns` como `classification` (valores `auto`/`manager`/`director` com `icon` e `ordinal`), `inputs` tipados, `else` obrigatório, `source` e `owner` (1.2), a task que consome a decisão (`decision:` + `with` + `gives:`) e o `gateway` roteando pelo valor retornado.*

### N.2 Elegibilidade de reembolso

```bpcms
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

  source: "Política de Trocas e Devoluções v2"
  owner: "Atendimento"
}
```

*Valida: `DocumentRef` em `inputs`, leitura de campo do documento (`request.reason`) na condição, primeira condição verdadeira vence e `else` cobrindo o resto.*

### N.3 Roteamento de chamado

```bpcms
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

  source: "Matriz de Roteamento de Suporte v5"
  owner: "Suporte"
}
```

*Valida: roteamento por prioridade e categoria, com prioridade vencendo categoria (ordem dos `cases`) e fila final obrigatória.*

### N.4 Reposição de estoque

```bpcms
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

  source: "Política de Estoque v4"
  owner: "Supply"
}
```

*Valida: decisão com `std.Count` e `boolean`, cobrindo estoque suficiente, parcial, pendente e cancelamento — sem ação (a `action` que repõe é da task, não da decisão).*

### N.5 Risco de fraude

```bpcms
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

  source: "Modelo de Risco v2"
  owner: "Risco"
}
```

*Valida: condição com `or` e limiar monetário, faixas crescentes de score e `DocumentRef` (`customer`) como entrada.*

### N.6 Scoring de crédito

```bpcms
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

  source: "Política de Crédito 2026"
  owner: "Crédito"
}
```

*Valida: `classification` de crédito com exceção primeiro (inadimplência grave), faixa `prime` exigindo score e histórico, e `else` como faixa residual.*
## Assumption

### U.1 Frete por remessa

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

*Valida: suposição com impacto, dono, prazo e vínculo com o processo.*

### U.2 Limite único de transportadora

```bpcms
assumption A-002 {
  statement: "Todas as transportadoras usam o mesmo limite de peso"
  impact: high
  status: rejected
  evidence: "Tabela de Frete 2026 mostra limites diferentes"
  owner: "Bruno"
  links { decision WeightLimit }
}
```

*Valida: suposição rejeitada com evidência que a derruba.*

### U.3 Nota fiscal preserva os dados do cliente na emissão

```bpcms
assumption A-003 {
  statement: "A nota fiscal mantém os dados do cliente como estavam na emissão"
  impact: high
  status: confirmed
  evidence: "Contrato e requisito fiscal"
  owner: "Financeiro"
  links { document Invoice.customer }
}
```

*Valida: suposição confirmada que vira requisito de materialização do lookup na emissão.*

### U.4 Assinatura do laudo

```bpcms
assumption A-004 {
  statement: "O laudo só pode ser liberado com assinatura do patologista"
  impact: high
  status: open
  validate: "Validar com conselho/responsável técnico"
  owner: "Dra. Lima"
  due: 2026-11-25
  links { process LabExamFlow.Release }
}
```

*Valida: suposição regulatória com prazo e vínculo com a tarefa de liberação.*

### U.5 Retry de pagamento

```bpcms
assumption A-005 {
  statement: "Falha de banco é transitória e pode ser retentada 3 vezes"
  impact: medium
  status: proposed
  validate: "Medir falhas por 30 dias"
  owner: "Ops"
  links { process InsuranceClaim.RetryPayment }
}
```

*Valida: suposição operacional ligada a retry e política de erro.*

### U.6 Retenção de evidências

```bpcms
assumption A-006 {
  statement: "Evidências de onboarding podem ser descartadas após 2 anos"
  impact: medium
  status: confirmed
  evidence: "Política de Retenção 2026"
  owner: "Jurídico"
  links { evidence OnboardingChecklistPDF }
}
```

*Valida: suposição de retenção com evidência e vínculo à evidência.*

# Plataforma

## Environment

### N.1 Desenvolvimento local

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

*Valida: dev interpretado, sem build, com banco local, seed e mocks.*

### N.2 QA determinístico

```bpcms
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

```bpcms
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

```bpcms
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

```bpcms
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

```bpcms
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

```bpcms
parameter ErpUrl {
  title: { pt-BR: "Endereço do sistema de gestão", en: "ERP URL" }
  type: string
  default: "https://erp.example.com"
}
```

*Valida: configuração tipada, obrigatória, com default e título; valor resolvido pelo ambiente.*

### PR.2 Chave secreta

```bpcms
secret PaymentKey {
  title: { pt-BR: "Chave de pagamento", en: "Payment key" }
  type: string
  required
}
```

*Valida: segredo sem default, sem valor no fonte e sem serialização/log.*

### PR.3 Perfil de modelo

```bpcms
parameter LlmModel {
  title: { pt-BR: "Modelo de linguagem", en: "LLM model" }
  type: string
  default: "balanced"
}
```

*Valida: configuração enumerada e default; combina com a capability aberta `llm`.*

### PR.4 Limite numérico

```bpcms
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

```bpcms
parameter SupportSla {
  title: { pt-BR: "Prazo de suporte", en: "Support SLA" }
  type: std.Duration
  default: 2h
}
```

*Valida: duração tipada com unidade, em vez de string.*

### PR.6 Chave de API específica

```bpcms
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

```bpcms
service "example.com/services/field" {
  title: { pt-BR: "Serviço de campo", en: "Field service" }
  version: 1.0.0
}

view VisitForm {
  capabilities { location, timezone, regionalization, units }
  type: form
}
```

*Valida: a View exige capabilities; o Service não as declara.*

### U.2 View formatando data e distância

```bpcms
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

```bpcms
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

```bpcms
builder WebApp {
  version: 1.0.0
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

```bpcms
builder ApiRust {
  version: 1.0.0
  target: server
  mode: [interpret, generate]
  capabilities: [oltp, sync, scheduling, identity, attachments, timezone, regionalization, units]
  consumes: [Service, Document, Action, Event]
  produces: [rust, openapi]
}
```

*Valida: builder de backend com OpenAPI e jobs.*

### B.3 Persistência Postgres

```bpcms
builder Postgres {
  version: 1.0.0
  target: database
  mode: generate
  capabilities: [oltp]
  consumes: [Document, Shape, Index]
  produces: [sql, ddl]
}
```

*Valida: builder de banco que gera DDL, índices e migrações.*

### B.4 Shell mobile

```bpcms
builder MobileShell {
  version: 0.4.0
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

```bpcms
builder Cli {
  version: 1.0.0
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

```bpcms
builder DocsPdf {
  version: 1.0.0
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

```bpcms
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

```bpcms
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

```bpcms
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

```bpcms
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

```bpcms
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

```bpcms
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

*Valida: moeda, unidades e formatos locais; fallback explícito.*

### L.2 Inglês dos Estados Unidos

```bpcms
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

```bpcms
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

```bpcms
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

```bpcms
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

```bpcms
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

*Valida: widget que liga um Shape, declara plataformas e fallback.*

### W.2 Mapa

```bpcms
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

```bpcms
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

```bpcms
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

```bpcms
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

```bpcms
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

```bpcms
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

```bpcms
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

```bpcms
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

```bpcms
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

```bpcms
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

```bpcms
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
    getPurchaseOrder(id: PurchaseOrder): PurchaseOrder
  }

  events {
    Invoice.Issued -> createInvoice
  }
}
```

*Valida: integração com operações tipadas, auth, retry e mapeamento de evento.*

### I.2 Gateway de pagamento com idempotência

```bpcms
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

```bpcms
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

```bpcms
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

```bpcms
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

```bpcms
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

```bpcms
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

```bpcms
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

```bpcms
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

```bpcms
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

```bpcms
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

```bpcms
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

```bpcms
document MonthlyRevenue {
  title: { pt-BR: "Receita mensal", en: "Monthly revenue" }
  version: 1
  owner: "Financeiro"
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

```bpcms
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

```bpcms
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

```bpcms
process PaymentReconciliation for Payment {
  title: { pt-BR: "Conciliação de pagamento", en: "Payment reconciliation" }
  version: 1

  pool Company {
    internal
    lane Finance { role: FinanceAnalyst }
  }

  start Paid { event: Billing.Invoice.Paid }

  task Match { lane: Finance; form: PaymentMatch; sla: 4h }
  task Close { lane: System; action: Payment.reconcile; with { payment } }

  Paid -> Match
  Match -> Close
  Close -> Reconciled

  end Reconciled { title: { pt-BR: "Conciliado", en: "Reconciled" }; outcome: success }
}
```

*Valida: processo de outro serviço assina o evento pelo nome qualificado; o alias local não muda a identidade.*

### E.4 Payload com referência a documento

```bpcms
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

```bpcms
process Packing for Order {
  title: { pt-BR: "Separação", en: "Packing" }
  version: 1

  pool Company {
    internal
    lane Warehouse { role: Packer }
  }

  start Placed { event: Order.Placed }
  task Pick { lane: Warehouse; form: PickItems; sla: 2h }
  task Ready { lane: System; action: Order.readyToShip; with { order } }

  Placed -> Pick
  Pick -> Ready
  Ready -> Packed

  end Packed { title: { pt-BR: "Pronto", en: "Ready" }; outcome: success }
}
```

*Valida: o mesmo evento que fecha uma action abre um processo; correlação pelo sujeito (`for Order`).*

### E.6 O mesmo evento a partir de origens diferentes

```bpcms
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

**Agendamento é um start, não um artefato.** O cron é do processo; o scheduler, o retry e a DLQ são runtime. Rotina em lote é **fan-out por sujeito**: `for` + `select` inicia uma instância por documento selecionado.

### S.1 Fechamento mensal por período

```bpcms
process MonthClose for FiscalPeriod {
  title: { pt-BR: "Fechamento mensal", en: "Month close" }
  version: 1
  owner: "Controladoria"
  sla: 6h

  pool Company {
    internal
    lane Controllership { role: Controller }
  }

  start Monthly {
    schedule: "0 2 1 * *"
    select: fiscalPeriod.status == Open
  }

  task PostEntries { lane: System; action: Ledger.postPeriod; with { fiscalPeriod }; on error Unbalanced -> Fix }
  task Review { lane: Controllership; form: MonthCloseReview; sla: 4h }
  task Fix { lane: Controllership; form: ReconciliationFix; sla: 1d }
  task Close { lane: System; action: FiscalPeriod.close; with { fiscalPeriod } }

  Monthly -> PostEntries
  PostEntries -> Review
  Review.approved -> Close
  Review.rejected -> Fix
  Fix -> PostEntries
  Close -> Closed

  end Closed { title: { pt-BR: "Fechado", en: "Closed" }; outcome: success }
  end Failed { title: { pt-BR: "Falhou", en: "Failed" }; outcome: failure }
}
```

*Valida: cron diário/mensal, seleção de sujeitos (uma instância por período aberto), tarefa de sistema com erro e revisão humana.*

### S.2 Cobrança com fan-out e espera

```bpcms
process Dunning for Invoice {
  title: { pt-BR: "Cobrança", en: "Dunning" }
  version: 1
  sla: 45d

  pool Company {
    internal
    lane Collections { role: CollectionsAgent }
  }
  pool Customer {
    external
  }

  start Daily {
    schedule: "0 6 * * *"
    select: invoice.status == Overdue and invoice.daysOverdue >= 1
  }

  task Remind { lane: System; sends: PaymentReminder; partner: Customer; with { invoice } }
  wait Grace { for: 7d }

  gateway Late {
    if invoice.daysOverdue > 30 -> Escalate
    else -> Remind
  }

  task Escalate { lane: Collections; form: CollectionCall; sla: 2d }
  cancel on Invoice.Paid -> Settled

  Daily -> Remind
  Remind -> Grace
  Grace -> Late
  Late -> Escalate
  Late -> Remind
  Escalate.promised -> Remind
  Escalate.writeoff -> WrittenOff

  end Settled { title: { pt-BR: "Pago", en: "Paid" }; outcome: success }
  end WrittenOff { title: { pt-BR: "Perda", en: "Written off" }; outcome: failure }
}
```

*Valida: fan-out por fatura vencida, espera de 7 dias, laço com saída e cancelamento pelo pagamento.*

### S.3 Reposição disparada por ponto de pedido

```bpcms
process Replenishment for StockItem {
  title: { pt-BR: "Reposição", en: "Replenishment" }
  version: 1
  sla: 2d

  pool Company {
    internal
    lane Planning { role: Planner }
  }

  variables { decision: ReplenishmentDecision }

  start Daily {
    schedule: "0 7 * * *"
    select: stockItem.available < stockItem.reorderPoint
  }

  task Decide {
    lane: System
    decision: ReplenishmentOrder
    with { available: stockItem.available; reorderPoint: stockItem.reorderPoint }
    gives: decision
  }

  gateway Action {
    if decision.action == order -> CreateRequest
    else -> Defer
  }

  task CreateRequest { lane: System; action: PurchaseRequest.create; with { stockItem; quantity: decision.quantity } }
  task Defer { lane: System; action: StockItem.deferReplenishment; with { stockItem } }

  Daily -> Decide
  Decide -> Action
  CreateRequest -> Requested
  Defer -> Deferred

  end Requested { title: { pt-BR: "Solicitado", en: "Requested" }; outcome: success }
  end Deferred { title: { pt-BR: "Adiado", en: "Deferred" }; outcome: success }
}
```

*Valida: decisão antes de criar a requisição; duas saídas de negócio; sem `batch`.*

### S.4 Rotina sem sujeito (digest)

```bpcms
process DailyDigest {
  title: { pt-BR: "Resumo diário", en: "Daily digest" }
  version: 1
  owner: "Operações"

  pool Company {
    internal
    lane Operations { role: Operator }
  }
  pool Team {
    external
  }

  start Daily { schedule: "0 7 * * *" }

  task Build { lane: System; action: Digest.build }
  task Send { lane: System; sends: DailyDigestReady; partner: Team }
  task Review { lane: Operations; manual: "Conferir o resumo antes de divulgar"; sla: 1h }

  Daily -> Build
  Build -> Send
  Send -> Review
  Review -> Sent

  end Sent { title: { pt-BR: "Enviado", en: "Sent" }; outcome: success }
}
```

*Valida: processo agendado sem sujeito (sem `for`), com tarefa manual de conferência; não há documento de relatório.*
## Policy / Rule

### P.1 Quatro-olhos em pagamento

```bpcms
policy FourEyesPayment {
  appliesTo: Payment.release
  rule: approver != creator
  severity: block
  message: "Pagamento exige segundo aprovador"
}
```

*Valida: política de segregação com bloqueio e auditoria.*

### P.2 Limite de desconto

```bpcms
policy DiscountLimit {
  appliesTo: Order.applyDiscount
  rule: discount <= 10% or approver.role == Manager
  severity: require
  message: "Desconto acima de 10% exige gerente"
}
```

*Valida: política que exige aprovação em vez de bloquear.*

### P.3 Retenção de dados

```bpcms
policy Retention {
  appliesTo: [LabExam, Evidence]
  rule: retainFor(5y) then anonymize
  severity: enforce
  exception: legalHold
}
```

*Valida: política de retenção com exceção de hold jurídico.*

### P.4 Escalonamento de SLA

```bpcms
policy SlaEscalation {
  appliesTo: SupportTicket
  rule: if breached then escalateTo(Lead) within 15m
  severity: enforce
  notification: [Email, WhatsApp]
}
```

*Valida: política temporal com canal de notificação.*

### P.5 Acesso a campo sensível

```bpcms
policy SensitiveFieldAccess {
  appliesTo: [Patient.document, LabExam.result]
  rule: role in [Doctor, Nurse] and purpose == treatment
  severity: block
  mask: default
}
```

*Valida: acesso por papel + finalidade, com mascaramento padrão.*

### P.6 Segregação de aprovação

```bpcms
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

```bpcms
icon Draft {
  title: { pt-BR: "Rascunho", en: "Draft" }
}
```

*Valida: ícone como token semântico; o builder/Theme resolve a imagem.*

### IC.2 Paga

```bpcms
icon Paid {
  title: { pt-BR: "Paga", en: "Paid" }
}
```

*Valida: ícone reutilizável por vários values.*

### IC.3 Vencida

```bpcms
icon Overdue {
  title: { pt-BR: "Vencida", en: "Overdue" }
}
```

*Valida: Icon sem detalhe visual; o Theme escolhe a imagem.*

### IC.4 E-mail

```bpcms
icon Email {
  title: { pt-BR: "Correio eletrônico", en: "Email" }
}
```

*Valida: ícone de canal definido no serviço de notificações.*

### IC.5 Prioridade

```bpcms
icon P1 {
  title: { pt-BR: "Prioridade 1", en: "Priority 1" }
}
```

*Valida: ícone de domínio para enum de prioridade no serviço de atendimento.*

### IC.6 Moeda

```bpcms
icon Brl {
  title: { pt-BR: "Real", en: "Brazilian real" }
}
```

*Valida: ícone para value de moeda; o value também tem title.*

## SVG file-based Icons

Além dos Icons declarados em `.bpcms`, arquivos `.svg` sob a raiz do serviço viram Icons implícitos. Exemplos em `examples/services/`:

```text
billing/status/draft.svg              -> status.draft      (aria-label)
billing/status/paid.svg               -> status.paid        (<title>)
billing/status/overdue.svg            -> status.overdue     (alt)
billing/finance/invoice/issued.svg    -> finance.invoice.issued
notifications/channels/email.svg      -> channels.email
inventory/stock/low-stock.svg         -> stock.lowStock     (fallback: "Low Stock")
payments/status/refunded.svg          -> status.refunded    (aria-label)
```

Uso no enum:

```text
draft { title: {...}; icon: status.draft }
paid  { title: {...}; icon: status.paid }
```

- Arquivos/pastas em kebab-case; referências em camelCase com ponto.
- Title: `aria-label` → `<title>` → `alt` → filename humanizado; sem I18n.
- Conflito com Icon declarado no mesmo serviço é erro.
- O builder resolve/sanitiza o SVG; isso é implementação, não DSL.

# O que validar nestes exemplos

As perguntas abaixo são **decisões de sintaxe e de modelagem** que os exemplos ajudam a testar. A v0.2 respondeu às antigas (tipo de task, form × view, job, report, workflow em task, reads/writes); restam as seguintes.

| # | Pergunta | Exemplos para olhar |
| :--- | :--- | :--- |
| 1 | `for <Documento>` (sujeito) deve ser obrigatório para todo processo visível? | P.1, P.13, P.24 |
| 2 | `runs:` como único mecanismo de subpasso visível é suficiente, ou falta um agrupamento local para escopo de erro/compensação? | P.6, P.20, P.15 |
| 3 | "Erro aborta a instância e compensa" é aceitável, ou precisamos de boundary local a um escopo? | P.3, P.5, P.10 |
| 4 | `fork`/`join` explícitos são mais legíveis que gateway paralelo? | P.6, P.17, P.20 |
| 5 | Form nomeado (sempre artefato) é melhor que form inline na task? | F.1–F.8 |
| 6 | `lane` obrigatória em toda task: verbosidade aceitável? | todos os P |
| 7 | `sla` deve ser erro (não aviso) para task humana? | P.5, P.9, P.22 |
| 8 | `select:` no start agendado cobre rotinas em lote, sem `batch`/`cursor`? | P.13, P.14, P.15, P.24 |
| 9 | `decision` com uma saída e primeiro-que-casa é suficiente para DMN? | P.1, P.23, N.1–N.6 |
| 10 | `message` com `about` (documento) cobre colaboração e coreografia? | M.1–M.4, P.7, P.11 |
| 11 | Form sem `roles` (autoridade na lane + action + campo) está claro? | F.3, F.8, P.18 |
| 12 | `outcome` nos `end` é suficiente para KPIs de desfecho? | P.5, P.13, P.18 |
| 13 | `foreach` sem agregação declarada é suficiente? | P.16, P.20 |
| 14 | `cancel on` cobre "cancelar a qualquer momento" sem event sub-process? | P.7, P.13, P.18, P.20 |
| 15 | `on timer ... keep` limitado a task lateral (sem aresta) é suficiente para lembretes? | P.7 |
| 16 | `assign: named(expr)` cobre atribuição dirigida sem `assignment`? | P.1, P.8 |
| 17 | Títulos herdados (task ← form/action; campo ← documento) reduzem ruído sem perder i18n? | F.1, F.8, P.19 |
| 18 | O analista consegue revisar um processo sem ler o form/documento? | P.6, P.17, P.21 |
| 19 | Falta algum artefato para representar seu domínio? | todos |
| 20 | O que ainda parece "código demais" para o dono do processo? | todos |

## Padrões que os exemplos já sugerem

* **Processo = trabalho; documento = estado.** A tela de andamento mostra os dois no mesmo lugar.
* **Tipo de task é o contrato**, não uma etiqueta: `form`, `manual`, `action`, `call`, `decision`, `sends`, `receives`, `runs`.
* **Quem executa é lane**; pool diz se é interno, portal externo, parceiro caixa-preta ou sistema.
* **Subpasso é processo**: `runs:` cria outra instância, com tela, histórico e dono próprios.
* **Form é contrato de escrita/desfecho**; View é leitura; relatório é View.
* **Agendamento é start**; rotina em lote é fan-out por sujeito com `select`.
* **Transição é da action**; o grafo de estados é derivado; só action muda estado.
* **Erro é falha de tentativa**: cancela, compensa e desvia; retry é da integração/ação.
* **Mensagem é contrato** com correlação por documento; o participante é declarado na task.
* **Desfecho é nó nomeado e classificado**; é a dimensão de KPI do processo.
* **Derivação em vez de declaração**: reads/writes, títulos herdados, grafo de estados, data objects.

## Golden examples sugeridos

Se for validar só alguns, comece por estes oito — eles cobrem as decisões mais estruturais:

1. **P.1 PurchaseApproval** — sujeito, alçada DMN, gateway, breach, desfechos.
2. **P.6 DispatchOrder** — pools interno/externo, fork/join, mensagem enviada e recebida, timeouts.
3. **P.7 DeliveryTracking** — processo filho, recebimento recorrente, laço com saída, milestone, cancel on.
4. **P.13 Dunning** — start agendado com fan-out, wait, laço, cancelamento por evento.
5. **P.18 RefundCase** — decisão, alçada, mensagem, milestone, três desfechos.
6. **F.1 + F.8** — form com três desfechos + form reutilizado por duas tasks.
7. **D.2 Invoice** — agregado, estado, eventos, ações com from/to/emits.
8. **V.7 OrderProgress** — tela de andamento com seções fixas.

## Checklist de validação

- [ ] A sintaxe parece legível para um analista, não só para um dev
- [ ] Toda task tem lane e um contrato; não há task "faz-tudo"
- [ ] Todo processo tem start e ends nomeados; todo caminho chega a um end
- [ ] O form não redeclara tipo/título do documento
- [ ] O documento não duplica estado no processo
- [ ] Agendamento aparece como start, não como artefato
- [ ] Relatório aparece como View, não como documento
- [ ] Mensagens têm `about` e participante declarado na task
- [ ] Desfechos permitem calcular KPIs (ciclo, atraso, caminho, motivo)
- [ ] Falta algum artefato para representar seu domínio

> Se algo aqui parecer "código demais" para o dono do processo, o problema é de sintaxe/experiência — não do conceito. O próximo passo é escolher 3–5 exemplos como **golden examples** e usá-los como teste de aceitação da DSL no SPECS.