# BPCMS — Exemplos de DSL

*Sintaxe ilustrativa v0.1.* Não é gramática final nem código executável. O objetivo é **validar conceitos, ergonomia e limites** antes de fechar PRD/SPECS/PLAN.

Cobrimos os **22 tipos de artefato** do catálogo, com **6 exemplos cada** — 134 no total, porque `Shape` tem 8 para exercitar dimensões e unidades. Alguns exemplos usam o mesmo domínio de propósito, para mostrar composição e reuso; outros mudam de domínio para testar generalidade.

## Como ler

* Cada bloco `bpcms` é um exemplo independente.
* `//` é comentário.
* A linha *Valida:* diz o que aquele exemplo quer provocar.
* Nomes de artefatos são em inglês (`application`, `document`, `shape`); nomes de negócio podem ser em português (`Pedido`, `Remessa`).
* Nada aqui substitui a baseline do [BRAINSTORM.md](BRAINSTORM.md); os exemplos são um **teste de sintaxe e modelagem**.

## Convenções usadas

| Construção | Forma | Observação |
| :--- | :--- | :--- |
| Artefato | `application Nome { ... }` | bloco nomeado; o tipo é a primeira palavra |
| Campo | `nome: Shape` | `Shape` é o tipo de dado, não uma forma visual |
| Campo com parâmetros | `peso: Mass { unit: kg, min: 0 }` | o campo só passa parâmetros simples |
| Lista | `[A, B, C]` | ordem preservada quando importa |
| Opcional | `T?` | ausência é diferente de vazio |
| Referência | `Document`, `View`, `Role` | por nome; o `check` valida existência e ciclo |
| Ação | `criar(...): Retorno` | comando tipado, não método OO |
| Índice | `byStatus: [status, criadoEm] range` | `unique`, `range`, `search`, `hash` |
| Regra | `total == itens.sum(subtotal)` | expressão segura, sem código arbitrário |
| Configuração | `env.ERP_URL`, `secret.STRIPE_KEY` | ambiente e segredo são resolvidos na promoção |
| Processo | `start`, `task`, `gateway`, `end` | nós e transições; BPMN é uma visão |
| Evento de processo | `on Evento`, `on timer 2d`, `on error X` | gatilhos explícitos |
| Permissão | `purchase.approve` | namespace por capacidade |

## Cobertura

| Grupo | Tipos | Exemplos |
| :--- | :--- | :--- |
| **Núcleo** | Application, Service, Process, Document, Shape, View, Role/Organogram | 44 |
| **Descoberta** | Evidence, Conversation, Decision, Assumption | 24 |
| **Plataforma** | Environment, Builder, Theme, Locale, Widget custom, Release/Version | 36 |
| **Operação** | Integration, Report/Dashboard, Event, Job/Schedule, Policy/Rule | 30 |
| **Total** | 22 tipos | **134** |

---

# Núcleo

## Application

### A.1 Loja virtual completa

```bpcms
application LojaDoJoao {
  version: 1.4.0
  owner: "Time Loja"
  description: "Loja virtual de ponta a ponta"

  services { Catalog, Cart, Checkout, Payment, Shipping, Auth, Notifications }
  roles { Customer; Clerk; Manager; Manager > Clerk }
  theme: Nuvula
  locales: [pt-BR, en-US]
  environments: [dev, qa, prod]

  main {
    start Auth
    wire Checkout with Cart, Catalog, Payment, Shipping
    wire Notifications with Checkout, Shipping
  }
}
```

*Valida: composição por serviços, hierarquia de papéis, tema/idiomas e wiring declarativo.*

### A.2 Clínica com prontuário e faturamento

```bpcms
application ClinicaVida {
  version: 2.1.0
  owner: "Diretoria Clínica"
  description: "Atendimento, exames e faturamento"

  services { Patients, Scheduling, Encounters, Billing, Auth, Notifications }
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

  main {
    start Auth
    wire Encounters with Patients, Scheduling
    wire Billing with Encounters
  }
}
```

*Valida: papéis clínicos, um serviço de encontro que depende de pacientes e agenda, e faturamento separado.*

### A.3 Logística com despacho e rastreio

```bpcms
application RotaCerta {
  version: 1.9.2
  owner: "Operações"
  description: "Do pedido à entrega, com rastreio"

  services { Orders, Dispatch, Fleet, Tracking, Billing, Auth, Notifications }
  roles { Dispatcher; Driver; Supervisor; Supervisor > Dispatcher > Driver }
  theme: Ops
  locales: [pt-BR, en-US]
  environments: [dev, qa, prod]

  main {
    start Auth
    wire Dispatch with Orders, Fleet
    wire Tracking with Dispatch
    wire Billing with Orders
  }
}
```

*Valida: aplicação operacional, serviços de domínio distintos e um serviço de rastreio que reage ao despacho.*

### A.4 Backoffice que reusa o mesmo catálogo

```bpcms
application BackofficeLoja {
  version: 1.2.0
  owner: "Retaguarda"
  description: "Operação interna da loja"

  services { Catalog, Inventory, Orders, Billing, Auth, Reports }
  roles { Clerk; Manager; Admin; Admin > Manager > Clerk }
  theme: Backoffice
  locales: [pt-BR]
  environments: [dev, qa, prod]

  main {
    start Auth
    wire Reports with Orders, Billing, Inventory
    wire Inventory with Catalog
  }
}
```

*Valida: mesma capacidade (`Catalog`) em outra aplicação, com papéis e experiência diferentes.*

### A.5 Serviço de campo

```bpcms
application CampoAgil {
  version: 1.0.0
  owner: "Field Ops"
  description: "Ordens de serviço e técnicos em campo"

  services { WorkOrders, Scheduling, Inventory, Billing, Auth, Notifications }
  roles { Technician; Dispatcher; Manager; Manager > Dispatcher > Technician }
  theme: Field
  locales: [pt-BR]
  environments: [dev, qa, prod]

  main {
    start Auth
    wire WorkOrders with Scheduling, Inventory
    wire Notifications with WorkOrders
  }
}
```

*Valida: aplicação enxuta, com um serviço central (`WorkOrders`) e dependências laterais.*

### A.6 Marketplace com disputas

```bpcms
application MercadoAberto {
  version: 0.8.0
  owner: "Marketplace"
  description: "Vendedores, pedidos e mediação"

  services { Sellers, Listings, Orders, Payments, Disputes, Auth, Notifications }
  roles { Buyer; Seller; Support; Admin; Admin > Support > Seller > Buyer }
  theme: Market
  locales: [pt-BR, en-US]
  environments: [dev, qa, prod]

  main {
    start Auth
    wire Orders with Listings, Payments
    wire Disputes with Orders, Payments
    wire Notifications with Orders, Disputes
  }
}
```

*Valida: aplicação com muitos atores e um serviço de disputa que cruza pedido e pagamento.*

## Service

### S.1 Catálogo com contrato e dependências

```bpcms
service Catalog {
  version: 1.2.0
  owner: "Time de Catálogo"

  exports {
    document Product
    shape MoneyBRL
    view ProductForm
    event ProductUpdated
  }
  imports {
    service Auth@^1.0
    service Media@^1.2
  }
  requires { db, http, files, search }
  provides { catalog.api, catalog.events }
  roles { CatalogManager, CatalogViewer }
}
```

*Valida: exports/imports explícitos, versão semântica e capabilities exigidas.*

### S.2 Checkout como orquestrador

```bpcms
service Checkout {
  version: 2.0.1
  owner: "Time de Conversão"

  exports {
    document CheckoutSession
    event OrderPlaced
  }
  imports {
    service Cart@^1
    service Catalog@^1
    service Payment@^1
    service Shipping@^1
    service Auth@^1
  }
  requires { db, http, jobs, realtime }
  provides { checkout.api, checkout.events }
  roles { Customer }
}
```

*Valida: serviço que compõe outros serviços sem virar dono dos dados deles.*

### S.3 Estoque com reservas e alerta

```bpcms
service Inventory {
  version: 1.5.0
  owner: "Supply Chain"

  exports {
    document StockItem
    document Reservation
    view StockTable
    event StockLow
  }
  imports {
    service Catalog@^1
    service Auth@^1
  }
  requires { db, jobs, search }
  provides { inventory.api, inventory.events }
  roles { StockClerk, WarehouseManager }
}
```

*Valida: dois documentos no mesmo serviço, evento de alerta e view de operação.*

### S.4 Faturamento com job e relatório

```bpcms
service Billing {
  version: 1.8.0
  owner: "Financeiro"

  exports {
    document Invoice
    document Payment
    report CashFlow
    job CloseMonth
    event InvoiceIssued
    event PaymentFailed
  }
  imports {
    service Orders@^1
    service Auth@^1
    integration ERP
  }
  requires { db, http, jobs, files }
  provides { billing.api, billing.events }
  roles { FinanceAnalyst, FinanceApprover }
}
```

*Valida: serviço de negócio com job agendado, relatório e integração de ERP.*

### S.5 Identidade e sessão

```bpcms
service Auth {
  version: 3.0.0
  owner: "Plataforma"

  exports {
    document User
    document Session
    role Customer
    role Clerk
    view LoginForm
    event UserLoggedIn
  }
  imports { integration Email }
  requires { db, http, cache }
  provides { auth.api, auth.guard }
}
```

*Valida: serviço que exporta primitivas de identidade, inclusive papéis e view de login.*

### S.6 Notificações multicanal

```bpcms
service Notifications {
  version: 1.1.0
  owner: "Customer Success"

  exports {
    document NotificationTemplate
    event NotificationSent
  }
  imports {
    service Auth@^1
    integration Email
    integration WhatsApp
  }
  requires { db, http, jobs }
  provides { notifications.api, notifications.events }
  roles { NotificationAdmin }
}
```

*Valida: serviço de suporte com templates como documento e canais como integrações substituíveis.*

## Process

### P.1 Aprovação de compra com alçadas

```bpcms
process PurchaseApproval {
  version: 1
  goal: "Comprar com alçada e rastro, sem travar o simples"
  roles: [Requester, Manager, Buyer, Director]
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

  task BuyerQuote {
    role: Buyer
    form: QuoteForm
    reads: [PurchaseRequest]
    writes: [Quote]
  }

  task Approve {
    role: Manager
    form: ApprovalForm
    reads: [Quote]
    writes: [PurchaseOrder]
  }

  end on PurchaseApproved

  on timer 2d after ReviewRequest -> Escalate
  task Escalate {
    role: Director
    form: EscalationForm
  }
}
```

*Valida: gateway por valor, tarefa automática, cotação humana, SLA e escalonamento por timer.*

### P.2 Expedição com paralelismo e erro de transportadora

```bpcms
process OrderFulfillment {
  version: 2
  goal: "Entregar o pedido no prazo com frete conhecido"
  roles: [Picker, Packer, Supervisor, Carrier]
  start on OrderApproved

  parallel {
    task PickItems {
      role: Picker
      reads: [Order, StockItem]
      writes: [Shipment]
      sla: 2h
    }
    task ReserveStock {
      type: service
      action: Inventory.reserve
    }
  }

  task PackItems {
    role: Packer
    reads: [Shipment, Product]
    writes: [Shipment]
  }

  gateway WeightLimit {
    if shipment.pesoBruto <= carrier.limitDePeso -> QuoteFreight
    else -> SplitShipment
  }

  task SplitShipment {
    role: Supervisor
    form: SplitForm
    reads: [Shipment]
    writes: [Shipment]
  }

  task QuoteFreight {
    type: service
    integration: Carrier
    reads: [Shipment]
    writes: [FreightQuote]
  }

  task PrintLabel {
    type: service
    integration: Carrier
    reads: [Shipment]
    writes: [Label]
  }

  end on ShipmentDispatched

  on error CarrierUnavailable -> ManualQuote
  task ManualQuote {
    role: Supervisor
    form: FreightOverrideForm
    writes: [FreightQuote]
  }
}
```

*Valida: paralelismo, decisão por peso, divisão de remessa, serviço externo e boundary error.*

### P.3 Exame laboratorial com dispositivo e assinatura

```bpcms
process LabExamFlow {
  version: 1
  goal: "Liberar laudo confiável no prazo"
  roles: [Collector, Analyst, Pathologist]
  start on SampleCollected

  task ReceiveSample {
    role: Collector
    form: ReceiveForm
    reads: [LabExam]
    writes: [LabExam]
  }

  task ReadAnalyzer {
    type: service
    integration: LabAnalyzer
    reads: [LabExam]
    writes: [LabExam.result]
  }

  task ValidateResult {
    role: Analyst
    form: ValidationForm
    reads: [LabExam]
    writes: [LabExam]
  }

  gateway Critical {
    if exam.critical -> NotifyDoctor
    else -> Review
  }

  task NotifyDoctor {
    type: service
    integration: Notifications
    reads: [LabExam]
  }

  task Review {
    role: Pathologist
    form: ReviewForm
    reads: [LabExam]
    writes: [LabExam.signature]
  }

  task Release {
    type: service
    action: LabExam.release
    writes: [LabExam]
  }

  end on ExamReleased

  on timer 4h after ReceiveSample -> Escalate
  task Escalate {
    role: Pathologist
    form: EscalationForm
  }
}
```

*Valida: leitura de dispositivo, caminho crítico, assinatura e liberação como tarefas distintas.*

### P.4 Onboarding com subprocesso

```bpcms
process EmployeeOnboarding {
  version: 1
  goal: "Novo colaborador produtivo no primeiro dia"
  roles: [HR, IT, Manager, NewHire]
  start on CandidateHired

  parallel {
    task CollectDocuments {
      role: NewHire
      form: DocumentUpload
      writes: [EmployeeProfile]
    }
    task CreateAccounts {
      role: IT
      form: AccountForm
      writes: [AccessGrant]
    }
    task OrderEquipment {
      role: IT
      form: EquipmentForm
      writes: [EquipmentOrder]
    }
  }

  subprocess ITProvisioning {
    process: ProvisionAccess
  }

  task ManagerCheckin {
    role: Manager
    form: CheckinForm
    reads: [EmployeeProfile]
  }

  end on OnboardingCompleted

  on timer 3d after CandidateHired -> Escalate
  task Escalate {
    role: HR
    form: OnboardingEscalation
  }
}
```

*Valida: paralelismo de tarefas humanas, subprocesso reutilizável e prazo de onboarding.*

### P.5 Sinistro com risco, cobertura e compensação

```bpcms
process InsuranceClaim {
  version: 1
  goal: "Regular sinistro com decisão rastreável"
  roles: [Adjuster, Analyst, Supervisor]
  start on ClaimOpened

  task RegisterClaim {
    role: Analyst
    form: ClaimForm
    reads: [Claim, Policy]
    writes: [Claim]
  }

  task ScoreFraud {
    type: service
    integration: FraudEngine
    reads: [Claim]
    writes: [FraudScore]
  }

  gateway Risk {
    if fraud.score < 30 -> AutoApprove
    else -> ManualReview
  }

  task AutoApprove {
    type: service
    action: Claim.approve
  }

  task ManualReview {
    role: Adjuster
    form: ReviewForm
    reads: [Claim, FraudScore]
    writes: [Claim]
  }

  gateway Coverage {
    if claim.covered -> Pay
    else -> Reject
  }

  task Pay {
    type: service
    integration: Bank
    writes: [Payment]
  }

  task Reject {
    role: Adjuster
    form: RejectForm
    writes: [Claim]
  }

  end on ClaimSettled

  on error BankUnavailable -> RetryPayment
  task RetryPayment {
    type: service
    integration: Bank
    retry: exponential(3)
  }
}
```

*Valida: duas decisões encadeadas, integração financeira, tratamento de erro e encerramento comum.*

### P.6 Incidente com severidade e pós-mortem

```bpcms
process IncidentManagement {
  version: 1
  goal: "Restaurar o serviço e aprender com o incidente"
  roles: [OnCall, SRE, Support, Manager]
  start on IncidentReported

  task Triage {
    role: OnCall
    form: TriageForm
    reads: [Incident]
    writes: [Incident]
    sla: 15m
  }

  gateway Severity {
    if severity >= SEV2 -> PageSRE
    else -> SupportQueue
  }

  task PageSRE {
    type: service
    integration: Pager
    writes: [Incident]
  }

  task SupportQueue {
    role: Support
    form: SupportForm
    reads: [Incident]
  }

  parallel {
    task Mitigate {
      role: SRE
      form: MitigationForm
      writes: [Incident]
    }
    task Communicate {
      role: Support
      form: UpdateForm
      writes: [IncidentUpdate]
    }
  }

  task Postmortem {
    role: SRE
    form: PostmortemForm
    writes: [Postmortem]
  }

  end on IncidentClosed

  on timer 15m after Triage -> Escalate
  task Escalate {
    role: Manager
    form: EscalationForm
  }
}
```

*Valida: evento de início, severidade, comunicação em paralelo e aprendizado pós-incidente.*

## Document

### D.1 Cliente com identidade e unicidade

```bpcms
document Customer {
  version: 1
  owner: "CRM"
  sync: live
  audit: true

  fields {
    id: CustomerId
    name: Text { maxLength: 120, required: true }
    email: Email { unique: true }
    phone: Phone?
    status: CustomerStatus
    createdAt: DateTime
  }

  indexes {
    email: [email] unique
    name: [name] search
  }

  actions {
    create(name: Text, email: Email): Customer
    update(name: Text, phone: Phone?): Customer
    archive(): void
  }

  rules {
    email != null
    status in [active, inactive, archived]
  }
}
```

*Valida: unicidade por índice, campos opcionais, ações tipadas e sincronização live.*

### D.2 Nota fiscal como agregado com snapshot

```bpcms
document Invoice {
  version: 1
  owner: "Billing"
  sync: live

  fields {
    id: InvoiceId
    number: InvoiceNumber { unique: true }
    customer: CustomerSnapshot
    items: InvoiceItem[]
    subtotal: MoneyBRL
    tax: MoneyBRL
    total: MoneyBRL
    status: InvoiceStatus
    issuedAt: DateTime?
  }

  indexes {
    number: [number] unique
    customer: [customer.id] hash
    byStatusDate: [status, issuedAt] range
  }

  actions {
    issue(): Invoice
    cancel(reason: Text): Invoice
  }

  rules {
    total == subtotal + tax
    total == items.sum(subtotal)
    status == issued implies issuedAt != null
  }
}
```

*Valida: documento como agregado, snapshot do cliente, itens e regra de consistência do total.*

### D.3 Remessa com dispositivo, unidades e cálculo

```bpcms
document Shipment {
  version: 1
  owner: "Dispatch"
  sync: live

  fields {
    id: ShipmentId
    orderId: OrderId
    items: ShipmentItem[]
    pesoBruto: Mass { unit: kg, min: 0.001 }
    volume: Volume { unit: m3, min: 0 }
    frete: MoneyBRL?
    tracking: TrackingCode?
    status: ShipmentStatus
    readAt: DateTime?
    createdAt: DateTime
  }

  source: device "scale-01" for pesoBruto

  computed frete from Shipment, CarrierTable {
    freshness: 15m
    exposes: [frete, calculatedAt]
  }

  indexes {
    order: [orderId] hash
    byStatus: [status, createdAt] range
    peso_normalizado: [pesoBruto.normalized] range
  }

  actions {
    addItem(item: ShipmentItem): Shipment
    split(): Shipment[]
    dispatch(carrier: CarrierRef): Shipment
  }

  rules {
    pesoBruto <= carrier.limitDePeso
    status == dispatched implies tracking != null
  }
}
```

*Valida: unidade com projeção normalizada, leitura de dispositivo, cálculo com frescor e regra por transportadora.*

### D.4 Laudo com arquivo, assinatura e retenção

```bpcms
document LabExam {
  version: 1
  owner: "Laboratório"
  sync: live
  retention: 5y

  fields {
    id: ExamId
    patientId: PatientId
    type: ExamType
    status: ExamStatus
    result: Text?
    resultFile: FileRef?
    signature: Signature?
    collectedAt: DateTime
    releasedAt: DateTime?
  }

  indexes {
    patient: [patientId, collectedAt] range
    byStatus: [status, collectedAt] range
  }

  actions {
    collect(sample: SampleRef): LabExam
    result(value: Text, file: FileRef): LabExam
    review(pathologist: UserId, signature: Signature): LabExam
    release(): LabExam
  }

  rules {
    status == released implies signature != null and result != null
    status == released implies releasedAt != null
  }
}
```

*Valida: anexo, assinatura, retenção e regra de liberação que exige evidência.*

### D.5 Ticket com SLA calculado

```bpcms
document SupportTicket {
  version: 1
  owner: "Support"
  sync: live

  fields {
    id: TicketId
    customerId: CustomerId
    subject: Text { maxLength: 160 }
    description: Text
    priority: TicketPriority
    status: TicketStatus
    assignee: UserId?
    slaDue: DateTime
    createdAt: DateTime
  }

  computed SlaBreach from SupportTicket, Calendar {
    freshness: 1m
    exposes: [breached, remaining]
  }

  indexes {
    byStatusPriority: [status, priority, slaDue] range
    assignee: [assignee] hash
    search: [subject, description] search
  }

  actions {
    assign(user: UserId): SupportTicket
    comment(body: Text): SupportTicket
    resolve(): SupportTicket
    reopen(reason: Text): SupportTicket
  }

  rules {
    status == resolved implies assignee != null
  }
}
```

*Valida: campo calculado de SLA, índice por prioridade e ações de atendimento.*

### D.6 Faturamento mensal calculado

```bpcms
document MonthlyRevenue {
  version: 1
  owner: "Finance"
  computed from Invoice, Payment
  freshness: 1h
  sync: read

  fields {
    month: YearMonth
    revenue: MoneyBRL
    orderCount: Count
    calculatedAt: DateTime
  }

  indexes { month: [month] unique }

  actions { refresh(): MonthlyRevenue }

  rules {
    month >= 2026-01
  }
}
```

*Valida: documento calculado, frescor de 1h, leitura sob demanda e ação de refresh.*

## Shape

### H.1 Dinheiro com moeda e escala

```bpcms
shape MoneyBRL {
  base: decimal
  currency: BRL
  scale: 2
  parse: locale-number
  format: currency
  compare: exact
  aggregate: sum
}
```

*Valida: dinheiro não é “número”; moeda, escala e arredondamento vivem no Shape.*

### H.2 Massa com dimensão e projeção

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

*Valida: dimensão, unidade, precisão e projeção para índice; `1 kg` não é `1 m`.*

### H.3 Comprimento com múltiplas unidades de exibição

```bpcms
shape Length {
  base: decimal
  dimension: length
  unit: cm
  displayUnits: [cm, m, in, ft]
  precision: 3
  parse: quantity
  convert: length
  index: normalized
}
```

*Valida: entrada em cm, exibição em in/ft e índice em unidade normalizada.*

### H.4 Temperatura com conversão afim

```bpcms
shape Temperature {
  base: decimal
  dimension: temperature
  unit: C
  min: -273.15 C
  parse: quantity
  convert: affine(C, F, K)
  format: locale-unit
}
```

*Valida: conversão que não é só multiplicação; mínimo físico e exibição por locale.*

### H.5 Percentual como razão

```bpcms
shape Percentage {
  base: decimal
  dimension: ratio
  unit: "%"
  min: 0
  max: 100
  scale: 2
  parse: percent
  format: percent
  aggregate: weighted-average
}
```

*Valida: percentual é adimensional, mas tem faixa, escala e agregação própria.*

### H.6 CPF com validação e sensibilidade

```bpcms
shape CPF {
  base: text
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
  base: text
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
  fields {
    street: Text
    number: Text
    complement: Text?
    district: Text
    city: Text
    state: StateCode
    zip: CEP
    country: CountryCode { default: BR }
  }
  format: postal
  index: [city, state]
}
```

*Valida: Shape composto, campos aninhados, default e índice derivado.*

## View

### V.1 Formulário de cliente

```bpcms
view CustomerForm {
  type: form
  document: Customer
  roles: [Clerk, Manager]

  fields {
    name: Text { required: true }
    email: Email { required: true }
    phone: Phone?
    status: CustomerStatus { readonly: true }
  }

  actions {
    save -> Customer.create
    cancel -> none
  }
}
```

*Valida: form ligado a documento, campos com parâmetros e ações explícitas.*

### V.2 Tabela de pedidos com filtros

```bpcms
view OrdersTable {
  type: table
  source: Order
  roles: [Clerk, Manager]

  columns { number; customer.name; total; status; createdAt }
  filters { status; period; customer }
  pagination: 50
  actions {
    open -> OrderDetail
    export -> Csv
  }
}
```

*Valida: tabela operacional, filtros, paginação e ações de linha.*

### V.3 Dashboard de vendas

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

*Valida: dashboard como view sobre documentos, com frescor, unidade e exportação.*

### V.4 Checkout em wizard

```bpcms
view CheckoutWizard {
  type: wizard
  roles: [Customer]
  steps { Cart -> Address -> Payment -> Review }
  data { Cart; Address; Payment }

  actions {
    next -> validateCurrentStep
    back -> previousStep
    confirm -> Checkout.placeOrder
  }
  integration: Payment
}
```

*Valida: wizard com etapas, dados compartilhados e integração de pagamento.*

### V.5 Detalhe de remessa com dispositivo

```bpcms
view ShipmentDetail {
  type: detail
  document: Shipment
  roles: [Dispatcher, Supervisor]

  sections { summary; items; timeline; documents; deviceReading }
  actions { dispatch; split; printLabel }
  computed { frete }
  device { scale-01 }
}
```

*Valida: detalhe com seções, ações, cálculo e leitura de dispositivo.*

### V.6 Kanban de atendimento

```bpcms
view SupportKanban {
  type: kanban
  source: SupportTicket
  roles: [Agent, Lead]
  groupBy: status

  columns { open; in_progress; waiting; resolved }
  card { subject; priority; slaDue; assignee }
  filter { assignee = currentUser }
  actions {
    move -> SupportTicket.changeStatus
    assign -> SupportTicket.assign
  }
}
```

*Valida: agrupamento por estado, cartão com SLA e ação de mover.*

## Role / Organogram

### R.1 Papéis de loja

```bpcms
role Customer {
  permissions: [catalog.view, order.create, order.viewOwn]
}
role Clerk {
  inherits: Customer
  permissions: [order.manage, customer.view]
}
role Manager {
  inherits: Clerk
  permissions: [customer.manage, report.view, refund.approve]
}
organogram Store {
  hierarchy { Manager > Clerk > Customer }
  scope: store
}
```

*Valida: herança de permissão, escopo e hierarquia simples.*

### R.2 Papéis de clínica com quatro-olhos

```bpcms
role Reception {
  permissions: [patient.create, appointment.schedule]
}
role Nurse {
  inherits: Reception
  permissions: [exam.collect, patient.viewClinical]
}
role Doctor {
  inherits: Nurse
  permissions: [prescription.create, exam.request]
}
role Manager {
  inherits: Doctor
  permissions: [billing.view, report.view]
}
organogram Clinic {
  hierarchy { Manager > Doctor > Nurse > Reception }
  fourEyes { prescription.create requires Doctor }
}
```

*Valida: segregação de função clínica e exigência de um segundo ator.*

### R.3 Papéis de armazém com segregação

```bpcms
role Picker {
  permissions: [shipment.pick]
}
role Checker {
  permissions: [shipment.check, shipment.reject]
}
role Supervisor {
  permissions: [shipment.override, report.view]
}
organogram Warehouse {
  hierarchy { Supervisor > Checker; Supervisor > Picker }
  segregation { shipment.pick != shipment.check }
  delegation { Supervisor -> Checker when absent }
}
```

*Valida: quem separa não confere; delegação explícita.*

### R.4 Papéis financeiros com limites

```bpcms
role Analyst {
  permissions: [invoice.create, payment.request]
}
role Approver {
  permissions: [payment.approve, invoice.cancel]
}
role Controller {
  permissions: [report.view, audit.view, limit.override]
}
organogram Finance {
  hierarchy { Controller > Approver > Analyst }
  fourEyes { payment.approve requires Approver and Approver != Analyst }
  limits {
    Approver: 50000 BRL
    Controller: unlimited
  }
}
```

*Valida: alçada como dado do papel, segregação e limite em unidade monetária.*

### R.5 Papéis de suporte com mascaramento

```bpcms
role Agent {
  permissions: [ticket.view, ticket.comment, ticket.assign]
}
role Lead {
  inherits: Agent
  permissions: [ticket.escalate, report.view]
}
role Admin {
  inherits: Lead
  permissions: [user.manage, ticket.delete]
}
organogram Support {
  hierarchy { Admin > Lead > Agent }
  delegation { Lead -> Agent when absent }
  dataMasking { Agent: customer.phone masked }
}
```

*Valida: herança, delegação e mascaramento por papel.*

### R.6 Papéis de laboratório com assinatura

```bpcms
role Collector {
  permissions: [sample.collect, exam.viewOwn]
}
role Analyst {
  permissions: [exam.result, exam.viewClinical]
}
role Pathologist {
  permissions: [exam.review, exam.release]
}
organogram Lab {
  hierarchy { Pathologist > Analyst > Collector }
  fourEyes { exam.release requires Pathologist and Pathologist != Analyst }
  signature { Pathologist }
}
```

*Valida: autoridade de assinatura, revisão por outro ator e hierarquia.*

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
      fields { id; name; taxId; status }
    }
    document PurchaseOrder {
      fields { id; supplierId; total; status; createdAt }
    }
  }
  links { service Billing }
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
      fields { id; patientId; doctorId; startsAt; status }
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

### N.1 Limite de aprovação de compra

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

*Valida: decisão com faixas monetárias, caso default e fonte explícita.*

### N.2 Desconto por cliente e margem

```bpcms
decision DiscountPolicy {
  appliesTo: Order.applyDiscount
  when: [customer.tier, order.margin]
  cases {
    tier == GOLD and margin >= 30% -> 10%
    tier == GOLD and margin >= 20% -> 5%
    tier == SILVER and margin >= 25% -> 5%
    else -> 0%
  }
  source: "Política Comercial 2026"
  owner: Sales
}
```

*Valida: decisão multidimensional com percentual e margem.*

### N.3 Roteamento de atendimento

```bpcms
decision TicketRouting {
  appliesTo: SupportTicket.assign
  when: [priority, category]
  cases {
    priority == P1 -> SREOnCall
    category == BILLING -> FinanceQueue
    category == TECHNICAL -> SupportQueue
    else -> GeneralQueue
  }
  default: GeneralQueue
  owner: Support
}
```

*Valida: roteamento por prioridade e categoria, com default explícito.*

### N.4 Falta de estoque

```bpcms
decision StockShortage {
  appliesTo: OrderFulfillment.ReserveStock
  when: [requested, available]
  cases {
    available >= requested -> Reserve
    available > 0 and available < requested -> PartialReserve
    available == 0 and backorder.allowed -> Backorder
    else -> CancelItem
  }
  source: "Política de Estoque"
  owner: SupplyChain
}
```

*Valida: decisão de exceção com ação parcial e backorder.*

### N.5 Faixa de risco de fraude

```bpcms
decision FraudRisk {
  appliesTo: InsuranceClaim.ScoreFraud
  when: fraud.score
  cases {
    < 30 -> AutoApprove
    >= 30 and < 70 -> ManualReview
    >= 70 -> BlockAndInvestigate
  }
  source: "Modelo de Risco v2"
  owner: Risk
}
```

*Valida: faixas de score e ações progressivas.*

### N.6 Escalonamento de SLA

```bpcms
decision SlaEscalation {
  appliesTo: SupportTicket.slaDue
  when: [remaining, priority]
  cases {
    remaining <= 0 and priority == P1 -> PageManager
    remaining <= 0 -> Reassign
    remaining <= 30m -> NotifyAssignee
    else -> none
  }
  default: none
  owner: Support
}
```

*Valida: decisão temporal com notificação, reatribuição e page.*

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

### U.3 Nota fiscal preserva snapshot do cliente

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

*Valida: suposição confirmada que vira requisito de snapshot.*

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

## Builder

### B.1 Aplicação web

```bpcms
builder WebApp {
  version: 1.0.0
  target: browser
  mode: [interpret, generate]
  capabilities: [ui, http, auth, realtime, files]
  consumes: [Shape, Document, View, Process, Theme]
  produces: [html, css, js]
}
```

*Valida: builder com capabilities, consumo de artefatos e dois modos.*

### B.2 API em Rust

```bpcms
builder ApiRust {
  version: 1.0.0
  target: server
  mode: [interpret, generate]
  capabilities: [http, db, auth, jobs, realtime]
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
  capabilities: [db, migrations, indexes, constraints]
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
  capabilities: [ui, files, camera, push]
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
  capabilities: [http, files]
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
  capabilities: [pdf, files, templates]
  consumes: [View, Report, Document]
  produces: [pdf, html]
}
```

*Valida: builder de relatórios/laudos com template e PDF.*

## Theme

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
  capabilities: [ui, files]
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
  capabilities: [ui, http]
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
  capabilities: [ui, camera]
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
  capabilities: [ui]
  platforms {
    web: input
    mobile: input
    cli: prompt
  }
  options {
    unit: required
    convert: display
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
  capabilities: [ui]
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
  capabilities: [ui, files]
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

### V.1 Release de aplicação

```bpcms
release JohnStore@1.4.0 {
  application: JohnStore
  includes {
    Catalog@1.2.0
    Checkout@2.0.1
    Payment@1.8.0
    Shipping@1.3.4
  }
  migrations: [M-2026-11-01]
  approvals: [QA, Owner]
  strategy: rolling
  rollback: JohnStore@1.3.2
}
```

*Valida: release como conjunto imutável de versões + migrações + aprovação.*

### V.2 Versão de serviço

```bpcms
release Catalog@1.2.0 {
  service: Catalog
  changes: [add Product.barcode, add ProductUpdated.v2]
  compatibility: backward
  migrations: [M-2026-10-12]
  approvals: [Owner]
  rollback: Catalog@1.1.3
}
```

*Valida: versão de serviço com mudanças compatíveis e rollback.*

### V.3 Hotfix

```bpcms
release JohnStore@1.4.1 {
  application: JohnStore
  includes { Checkout@2.0.2 }
  type: hotfix
  reason: "Corrige cálculo de frete em pedido dividido"
  approvals: [OnCall, Owner]
  strategy: immediate
  rollback: JohnStore@1.4.0
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
release Rollback@1.4.2 {
  type: rollback
  application: JohnStore
  to: JohnStore@1.4.0
  reason: "Erro de cálculo em produção"
  approvals: [Owner, Ops]
  strategy: immediate
  preserveData: true
}
```

*Valida: rollback explícito, com preservação de dados e aprovação.*

### V.6 Canary

```bpcms
release JohnStore@1.5.0 {
  application: JohnStore
  includes { Catalog@1.3.0; Checkout@2.1.0 }
  strategy: canary(10%, 30m)
  guardrails {
    errorRate: < 1%
    p95Latency: < 500ms
  }
  approvals: [QA, Owner]
  rollback: JohnStore@1.4.1
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
    getSupplier(id: SupplierId): Supplier
    createInvoice(invoice: Invoice): ExternalId
    getPurchaseOrder(id: PurchaseOrderId): PurchaseOrder
  }

  events {
    InvoiceIssued -> createInvoice
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
    authorize(order: Order, amount: MoneyBRL): Authorization
    capture(authorizationId: ExternalId): Payment
    refund(paymentId: ExternalId, amount: MoneyBRL): Refund
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

### R.1 Vendas por região

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

*Valida: dashboard com métricas, dimensão, unidade e frescor.*

### R.2 Envelhecimento de estoque

```bpcms
report InventoryAging {
  type: report
  sources: [StockItem, Product]
  roles: [WarehouseManager]

  filters { warehouse; category }
  dimensions { product; warehouse; ageBucket }
  metrics {
    quantity: sum(StockItem.quantity)
    value: sum(StockItem.value)
    aging: bucket(StockItem.lastMovement, [30d, 60d, 90d, 180d])
  }
  sort { value: desc }
  export: [csv, xlsx]
}
```

*Valida: relatório com bucket temporal e múltiplas métricas.*

### R.3 Cumprimento de SLA

```bpcms
report SlaCompliance {
  type: dashboard
  sources: [SupportTicket]
  roles: [SupportLead, Manager]

  filters { period; team; priority }
  metrics {
    compliance: percentage(SupportTicket.slaMet)
    breached: count(SupportTicket.breached)
    avgResolution: avg(SupportTicket.resolutionTime)
  }
  freshness: 5m
  alert { compliance < 90% -> NotifyLead }
}
```

*Valida: dashboard com percentual, alerta e métrica temporal.*

### R.4 Fluxo de caixa

```bpcms
report CashFlow {
  type: report
  sources: [Invoice, Payment, Bill]
  roles: [FinanceAnalyst, Controller]

  filters { period; account }
  dimensions { month; account }
  metrics {
    inflow: sum(Payment.received)
    outflow: sum(Bill.paid)
    balance: inflow - outflow
  }
  units { inflow: BRL; outflow: BRL; balance: BRL }
  freshness: 1h
  export: [pdf, csv]
}
```

*Valida: relatório financeiro com unidades explícitas e saldo calculado.*

### R.5 Qualidade laboratorial

```bpcms
report LabQuality {
  type: dashboard
  sources: [LabExam, LabNonConformity]
  roles: [Pathologist, Manager]

  filters { period; examType }
  metrics {
    released: count(LabExam.released)
    rework: percentage(LabExam.rework)
    avgTurnaround: avg(LabExam.turnaround)
    critical: count(LabExam.critical)
  }
  freshness: 10m
  export: [pdf]
}
```

*Valida: dashboard de qualidade com tempo de resposta e não conformidade.*

### R.6 Controle operacional em tempo real

```bpcms
report OperationsControl {
  type: dashboard
  sources: [Shipment, Incident, DeviceReading]
  roles: [Ops, Manager]
  mode: live

  filters { site; status }
  metrics {
    openShipments: count(Shipment.open)
    lateShipments: count(Shipment.late)
    deviceOffline: count(DeviceReading.offline)
    incidents: count(Incident.open)
  }
  freshness: live
  alert {
    lateShipments > 20 -> PageOps
    deviceOffline > 5 -> NotifyOps
  }
}
```

*Valida: dashboard live com alertas operacionais e múltiplas fontes.*

## Event

### E.1 Pedido aprovado

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

*Valida: evento de negócio com payload, produtor, consumidores e chave de partição.*

### E.2 Remessa despachada

```bpcms
event ShipmentDispatched {
  version: 1
  payload {
    shipmentId: ShipmentId
    orderId: OrderId
    carrier: CarrierRef
    tracking: TrackingCode
    dispatchedAt: DateTime
  }
  producer: Shipment
  consumers: [Tracking, Notifications, Billing]
  key: shipmentId
}
```

*Valida: evento que cruza serviços e alimenta rastreio/notificação.*

### E.3 Exame liberado

```bpcms
event ExamReleased {
  version: 1
  payload {
    examId: ExamId
    patientId: PatientId
    releasedBy: UserId
    releasedAt: DateTime
    critical: Boolean
  }
  producer: LabExam
  consumers: [Notifications, PatientPortal]
  key: examId
  sensitivity: restricted
}
```

*Valida: evento com dado sensível e consumidor de portal do paciente.*

### E.4 Pagamento falhou

```bpcms
event PaymentFailed {
  version: 1
  payload {
    paymentId: PaymentId
    orderId: OrderId
    reason: PaymentFailureReason
    failedAt: DateTime
  }
  producer: Payment
  consumers: [Checkout, Notifications, Support]
  key: paymentId
}
```

*Valida: evento de falha com consumidores de recuperação e atendimento.*

### E.5 Estoque baixo

```bpcms
event StockLow {
  version: 1
  payload {
    stockItemId: StockItemId
    productId: ProductId
    available: Quantity
    threshold: Quantity
  }
  producer: StockItem
  consumers: [Purchasing, Notifications]
  key: stockItemId
}
```

*Valida: evento de alerta com quantidades e limiar.*

### E.6 Dispositivo offline

```bpcms
event DeviceOffline {
  version: 1
  payload {
    deviceId: DeviceId
    lastReadAt: DateTime
    site: SiteId
  }
  producer: DeviceReading
  consumers: [OperationsControl, Notifications]
  key: deviceId
  severity: warning
}
```

*Valida: evento de infraestrutura/operação com severidade.*

## Job / Schedule

### J.1 Fechamento mensal

```bpcms
job CloseMonth {
  schedule: "0 2 1 * *"
  timezone: America/Fortaleza
  action: Billing.closeMonth()
  timeout: 30m
  retry: 3
  idempotent: true
  deadLetter: true
}
```

*Valida: job mensal com timezone, timeout, retry e DLQ.*

### J.2 Resumo diário

```bpcms
job DailyDigest {
  schedule: "0 7 * * *"
  timezone: America/Fortaleza
  action: Notifications.sendDigest()
  input { audience: [Manager, SupportLead] }
  timeout: 5m
  retry: 2
}
```

*Valida: job de comunicação com input e retry.*

### J.3 Limpeza de evidências

```bpcms
job PurgeExpiredEvidence {
  schedule: "0 3 * * 0"
  timezone: America/Fortaleza
  action: Evidence.purgeExpired()
  batch: 500
  dryRun: env == "qa"
  timeout: 20m
  retry: 1
}
```

*Valida: job de retenção com batch e dry-run em QA.*

### J.4 Retry de integrações

```bpcms
job RetryFailedIntegrations {
  schedule: "*/15 * * * *"
  timezone: America/Fortaleza
  action: Integration.retryFailed()
  filter { failedAt > now() - 24h }
  maxAttempts: 5
  backoff: exponential
  deadLetter: true
}
```

*Valida: job de recuperação com filtro temporal e backoff.*

### J.5 Sincronização com ERP

```bpcms
job SyncERP {
  schedule: "*/30 * * * *"
  timezone: America/Fortaleza
  action: ERP.syncSuppliers()
  cursor: "erp.supplier.cursor"
  batch: 200
  idempotent: true
  timeout: 15m
}
```

*Valida: job incremental com cursor e idempotência.*

### J.6 Atualização de dashboard

```bpcms
job RefreshSalesDashboard {
  schedule: "*/5 * * * *"
  timezone: America/Fortaleza
  action: MonthlyRevenue.refresh()
  condition: SalesDashboard.viewers > 0
  timeout: 10m
  retry: 1
}
```

*Valida: job condicionado à audiência e ligado a documento calculado.*

## Policy / Rule

### P.1 Quatro-olhos em pagamento

```bpcms
policy FourEyesPayment {
  appliesTo: Payment.release
  rule: approver != creator
  severity: block
  message: "Pagamento exige segundo aprovador"
  audit: true
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
  audit: true
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
  audit: true
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
  audit: true
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
  audit: true
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
  audit: true
}
```

*Valida: política que atravessa mais de um documento e ator.*

---

# O que validar nestes exemplos

Estas não são pendências da baseline; são **decisões de sintaxe** que os exemplos ajudam a testar. Marque o que parece natural e o que parece burocrático.

| # | Pergunta | Exemplos para olhar |
| :--- | :--- | :--- |
| 1 | `shape` deve ser nomeado por dimensão (`Mass`, `Length`) ou genérico (`Quantity<Mass>`)? | H.2, H.3, H.4 |
| 2 | O campo deve carregar `unit` ou o `Shape` já define a unidade? | H.2, D.3, W.4 |
| 3 | Documento agregado deve listar itens como `Document[]` ou ter um bloco `items` próprio? | D.2, D.3 |
| 4 | Snapshot (`CustomerSnapshot`) é um `Shape`, um `Document` ou um operador (`snapshot Customer`)? | D.2, U.3 |
| 5 | O processo deve ser declarado por nós (`task`, `gateway`) ou por XML BPMN com extensões? | P.1–P.6 |
| 6 | `reads`/`writes` por tarefa são úteis ou ruído? | P.1–P.6 |
| 7 | Expressões (`sum`, `count`, `by`, `bucket`, `percentage`) são suficientes? | R.1–R.6, N.1–N.6 |
| 8 | `view` deve listar campos ou compor widgets? | V.1–V.6 |
| 9 | `role` deve ter permissões inline ou só referenciar um catálogo? | R.1–R.6 |
| 10 | `evidence` deve carregar o conteúdo extraído ou só o vínculo e a confiança? | E.1–E.6 |
| 11 | `environment` deve declarar `promoteTo` ou a promoção é definida no `release`? | N.1–N.6, V.1–V.6 |
| 12 | `builder` deve declarar `limitations` por plataforma? | B.1–B.6 |
| 13 | `widget` deve mapear `platforms` ou só declarar `capabilities`? | W.1–W.6 |
| 14 | `conversation` deve ter `coverage` por dimensão ou por artefato? | C.1–C.6 |
| 15 | `release` deve listar `includes` ou usar lockfile da aplicação? | V.1–V.6 |
| 16 | Cálculo deve ser campo computado (`computed frete`) ou documento (`document FreightQuote`)? | D.3, D.6 |
| 17 | Shape composto (`fields`) é Shape ou Document embutido? | H.8, D.2 |
| 18 | O que falta para você dizer “isso eu consigo revisar sem ser programador”? | todos |

## Padrões que os exemplos já sugerem

* **Shape** é o único lugar com validação/formatação/conversão; o campo só parametriza.
* **Document** agrega, referencia ou congela (snapshot); computed/device são variações do mesmo conceito.
* **Process** orquestra, mas não implementa regra: chama `action`, `view`, `integration` e `role`.
* **View/Report** são projeções; não são donas de dados.
* **Environment/Builder/Release** são artefatos de plataforma, não de negócio.
* **Evidence/Conversation/Decision/Assumption** são a memória da descoberta; precisam ser consultáveis e versionadas.

### Golden examples sugeridos

Se for validar só alguns, comece por estes seis — eles cobrem as decisões mais estruturais:

1. **H.2 Mass** — dimensão, unidade, projeção e agregação.
2. **D.2 Invoice** — agregado, snapshot, itens e regra de total.
3. **P.2 OrderFulfillment** — processo, paralelismo, integração e erro.
4. **V.3 SalesDashboard** — métricas, frescor, unidade e exportação.
5. **R.4 FinanceRoles** — permissões, quatro-olhos e limite.
6. **N.1 PurchaseApproval** — decisão por faixa e caso default.

### Checklist de validação

- [ ] A sintaxe parece legível para um analista, não só para um dev?
- [ ] `Shape` resolve validação/unidade sem espalhar regra pelo campo?
- [ ] Documento agregado com snapshot faz sentido para o negócio?
- [ ] O processo orquestra sem virar código?
- [ ] `View` e `Report` são claramente projeções, não donas do dado?
- [ ] `Role`/`Policy` expressam quatro-olhos, alçada e retenção?
- [ ] `Environment`/`Builder`/`Release` dão segurança para dev/QA/prod?
- [ ] Falta algum artefato para representar seu domínio?

> Se algo aqui parecer “código demais” para o dono do processo, o problema é de sintaxe/experiência — não do conceito. O próximo passo é escolher 3–5 exemplos como **golden examples** e usá-los como teste de aceitação da DSL no SPECS.
