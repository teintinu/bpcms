# BPCMS — Exemplos de DSL

*Sintaxe ilustrativa v0.1.* Não é gramática final nem código executável. O objetivo é **validar conceitos, ergonomia e limites** antes de fechar PRD/SPECS/PLAN. A sintaxe e o funcionamento de cada cláusula estão em [DSL_REFERENCE.md](DSL_REFERENCE.md).

Cobrimos os **24 tipos de artefato** do catálogo, com **6 exemplos cada** — 152 exemplos de artefatos, porque `Shape` tem 9, Application/Service têm 8 e Process tem 7 (compensação); há também 3 exemplos de uso de serviços essenciais. Alguns exemplos usam o mesmo domínio de propósito, para mostrar composição e reuso; outros mudam de domínio para testar generalidade.

## Como ler

* Cada bloco `bpcms` é um exemplo independente.
* `//` é comentário.
* A linha *Valida:* diz o que aquele exemplo quer provocar.
* Nomes de artefatos são em inglês (`application`, `document`, `shape`); nomes de negócio podem ser em português (`Pedido`, `Remessa`).
* Nada aqui substitui a baseline do [BRAINSTORM.md](BRAINSTORM.md); os exemplos são um **teste de sintaxe e modelagem**.
* **Application e Service usam URI web sem protocolo** como identificador: `"example.com/app/..."` e `"example.com/services/..."`. A URL `http://<uri>/index.html` é a página de download (app) ou de documentação (serviço); em produção, `https://`.

## Convenções usadas

| Construção | Forma | Observação |
| :--- | :--- | :--- |
| Artefato | `application "example.com/app/x" { ... }` | Application/Service usam URI; os demais usam nome simples |
| Campo | `nome: Shape` | `Shape` é o tipo de dado, não uma forma visual |
| Campo com parâmetros | `peso: Mass { unit: kg, min: 0 }` | o campo só passa parâmetros simples |
| Lista | `[A, B, C]` | ordem preservada quando importa |
| Opcional | `T` | ausência é diferente de vazio |
| Referência | `Document`, `View`, `Role` | por nome; o `check` valida existência e ciclo |
| Ação | `criar(...)` | comando tipado, sem retorno, não método OO |
| Índice | `byStatus: [status, criadoEm] range` | `unique`, `range`, `search`, `hash` |
| Regra | `total == itens.sum(subtotal)` | expressão segura, sem código arbitrário |
| Configuração | `env.ERP_URL`, `secret.STRIPE_KEY` | ambiente e segredo são resolvidos na promoção |
| Processo | `start`, `task`, `gateway`, `end` | nós e transições; BPMN é uma visão |
| Origem | `source: registry` / `repository` / `auto` | registry BPCMS x projeto Git |
| Versão | `version: semver("1.2.3")` / `tag("v1.2.3")` / `commit("9f3a1c")` | `1.4.0` é açúcar para `semver("1.4.0")` |
| Release | `"<uri>"@1.2.3` | URI + seletor de versão |
| Página canônica | `http://<uri>/index.html` | app = download por SO; service = docs/integração |
| Evento de processo | `on Evento`, `on timer 2d`, `on error X` | gatilhos explícitos |
| Permissão | `purchase.approve` | namespace por capacidade |

## Cobertura

| Grupo | Tipos | Exemplos |
| :--- | :--- | :--- |
| **Núcleo** | Application, Service, Process, Document, Shape, View, Role/Organogram | 50 |
| **Descoberta** | Evidence, Conversation, Decision, Assumption | 24 |
| **Plataforma** | Environment, Parameter, Builder, Theme, Locale, Widget custom, Release/Version, Icon | 48 |
| **Operação** | Integration, Report/Dashboard, Event, Job/Schedule, Policy/Rule | 30 |
| **Total** | 24 tipos | **152** |

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

  reintroduce Checkout.Checkout {
    task FraudCheck {
      type: service
      action: FraudCheck.evaluate
    }

    insert FraudCheck before AuthorizePayment
  }
}
```

*Valida: cada passo do checkout é um nó reutilizável; o fraud-checkout acrescenta `FraudCheck` antes de `AuthorizePayment` sem substituir o processo inteiro.*

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

### P.1 Aprovação de compra com alçadas

```bpcms
process PurchaseApproval {
  version: 1
  goal: "Comprar com alçada e rastro, sem travar o simples"
  roles: [Requester, Manager, Buyer, Director]
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

  end on PurchaseRequest.Approved

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
  start on Order.Approved

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

  end on Shipment.Dispatched

  on error CarrierUnavailable -> ManualQuote
  task ManualQuote {
    role: Supervisor
    form: FreightManualQuoteForm
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
  start on Sample.Collected

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

  end on LabExam.Released

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
  start on Candidate.Hired

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

  end on Onboarding.Completed

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
  start on Claim.Opened

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

  end on Claim.Settled

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
  start on Incident.Reported

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

  end on Incident.Closed

  on timer 15m after Triage -> Escalate
  task Escalate {
    role: Manager
    form: EscalationForm
  }
}
```

*Valida: evento de início, severidade, comunicação em paralelo e aprendizado pós-incidente.*

### P.7 Compensação de pagamento

```bpcms
process PaymentCompensation {
  version: 1
  goal: "Reverter a cobrança quando a reserva de estoque falha"
  roles: [Clerk, Finance]
  start on Order.Approved

  task ChargePayment {
    type: service
    integration: Payment
    action: Payment.charge
  }

  task ReserveStock {
    type: service
    action: Inventory.reserve
  }

  on error StockUnavailable -> RefundPayment

  task RefundPayment {
    type: compensation
    for: ChargePayment
    action: Payment.refund
  }

  end on Order.Cancelled
}
```

*Valida: compensação como passo de processo, alvo `for`, disparo por erro e execução idempotente em ordem reversa.*

## Document

### D.1 Cliente com identidade e unicidade

```bpcms
document Customer {
  title: { pt-BR: "Cliente", en: "Customer" }
  version: 1
  owner: "CRM"
  sync: live
  audit: true

  fields {
    name: string {
      title: { pt-BR: "Nome", en: "Name" }
      maxLength: 120
      lookup
      required,
    }
    email: Email { unique, lookup }
    phone: Phone
    status: CustomerStatus
    private internalScore: Score
    createdAt: DateTime
  }

  indexes {
    email: [email] unique
    name: [name] search
  }

  actions {
    create(name: string, email: Email) {
      title: { pt-BR: "Criar", en: "Create" }
      roles: [Clerk]
      {
      }
    }
    update(name: string, phone: Phone?) {
      title: { pt-BR: "Atualizar", en: "Update" }
      roles: [Clerk]
      {
      }
    }
    archive() {
      title: { pt-BR: "Arquivar", en: "Archive" }
      roles: [Manager]
      {
      }
    }
    private recalcScore() {
      title: { pt-BR: "Recalcular", en: "Recalculate" }
      roles: [System]
    }
  }

  rules {
    email != null: {
      fields: []
      pt-BR: "Regra do documento não atendida"
      en: "Document rule not met"
    }
    status in [active, inactive, archived]: {
      fields: []
      pt-BR: "Regra do documento não atendida"
      en: "Document rule not met"
    }
  }
}
```

*Valida: unicidade por índice, campos opcionais, ações tipadas, `private` em campo/ação e sincronização live.*

### D.2 Nota fiscal com escopos e links

```bpcms
scopable document Store {
  title: { pt-BR: "Loja", en: "Store" }
  fields {
    code: string { title: { pt-BR: "Código", en: "Code" }, unique, lookup }
    name: string { title: { pt-BR: "Nome", en: "Name" }, lookup }
  }
}

scopable document FiscalYear {
  title: { pt-BR: "Exercício fiscal", en: "Fiscal year" }
  fields {
    year: integer { title: { pt-BR: "Ano", en: "Year" }, unique, lookup }
    startsAt: DateTime { title: { pt-BR: "Início", en: "Start" } }
    endsAt: DateTime { title: { pt-BR: "Fim", en: "End" } }
  }
}

document Invoice {
  title: { pt-BR: "Nota fiscal", en: "Invoice" }
  version: 1
  owner: "Billing"
  sync: live

  fields {
    store: Store {
      title: { pt-BR: "Loja", en: "Store" }
    }
    fiscalYear: FiscalYear {
      title: { pt-BR: "Exercício", en: "Fiscal year" }
    }
    customer: Customer {
      title: { pt-BR: "Cliente", en: "Customer" }
      lookup
    }
    number: InvoiceNumber {
      format: "{fiscalYear.year:yy}-{store.code:4}-{sequence:6}"
      sequence: per(Store, FiscalYear)
      unique
      lookup
    }
    items: InvoiceItem[]
    subtotal: std.Money
    tax: std.Money
    total: std.Money {
      title: { pt-BR: "Total", en: "Total" }
      lookup
      read: [Clerk, Manager]
      edit: [Manager]
    }
    status: InvoiceStatus
    issuedAt: DateTime
  }

  indexes {
    number: [number] unique
    store: [store.id] hash
    fiscalYear: [fiscalYear.id] hash
    byStatusDate: [status, issuedAt] range
  }

  actions {
    issue() {
      title: { pt-BR: "Emitir", en: "Issue" }
      roles: [Manager]
      transition: Issued

      rules {
        items.length > 0: {
          fields: [items]
          pt-BR: "Adicione itens antes de emitir a fatura"
          en: "Add items before issuing the invoice"
        }
        total > 0 BRL: {
          fields: [total]
          pt-BR: "Fatura sem valor"
          en: "Invoice has no value"
        }
      }

      {
        issuedAt = now()
        audit("invoice.issued")
      }
    }

    cancel(reason: string) {
      title: { pt-BR: "Cancelar", en: "Cancel" }
      roles: [Manager]
      transition: Cancelled

      rules {
        reason.length >= 10: {
          fields: [reason]
          pt-BR: "Informe um motivo com pelo menos 10 caracteres"
          en: "Enter a reason with at least 10 characters"
        }
        status == issued: {
          fields: [status]
          pt-BR: "A nota precisa estar emitida"
          en: "Invoice must be issued"
        }
      }

      {
        audit("invoice.cancelled", reason)
      }
    }
  }

  rules {
    total == subtotal + tax: {
      fields: []
      pt-BR: "Regra do documento não atendida"
      en: "Document rule not met"
    }
    total == items.sum(subtotal): {
      fields: []
      pt-BR: "Regra do documento não atendida"
      en: "Document rule not met"
    }
    status == issued implies issuedAt != null: {
      fields: []
      pt-BR: "Regra do documento não atendida"
      en: "Document rule not met"
    }
  }
}
```

*Valida: `scopable`, escopo inferido por `link`, ID interno, número alternativo, lookup automático do cliente, ação com `transition` e regra de total.*

### D.3 Remessa com dispositivo, unidades e cálculo

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
    items: ShipmentItem[]
    pesoBruto: Mass { unit: kg, min: 0.001 }
    volume: Volume { unit: m3, min: 0 }
    frete: std.Money
    tracking: TrackingCode { lookup }
    status: ShipmentStatus { lookup }
    readAt: DateTime
    createdAt: DateTime
  }

  source: device "scale-01" for pesoBruto

  computed frete from Shipment, CarrierTable {
    freshness: 15m
    exposes: [frete, calculatedAt]
  }

  indexes {
    order: [order.id] hash
    byStatus: [status, createdAt] range
    peso_normalizado: [pesoBruto.normalized] range
  }

  actions {
    addItem(item: ShipmentItem) {
      title: { pt-BR: "Adicionar item", en: "Add item" }
      roles: [Clerk]
      {
      }
    }
    split() {
      title: { pt-BR: "Dividir", en: "Split" }
      roles: [Manager]
      {
      }
    }
    dispatch(carrier: CarrierRef) {
      title: { pt-BR: "Despachar", en: "Dispatch" }
      roles: [Manager]
      {
      }
    }
  }

  rules {
    pesoBruto <= carrier.limitDePeso: {
      fields: []
      pt-BR: "Regra do documento não atendida"
      en: "Document rule not met"
    }
    status == dispatched implies tracking != null: {
      fields: []
      pt-BR: "Regra do documento não atendida"
      en: "Document rule not met"
    }
  }
}
```

*Valida: unidade com projeção normalizada, leitura de dispositivo, cálculo com frescor e regra por transportadora.*

### D.4 Laudo com arquivo, assinatura e retenção

```bpcms
document LabExam {
  title: { pt-BR: "Exame laboratorial", en: "Lab exam" }
  version: 1
  owner: "Laboratório"
  sync: live
  retention: 5y

  fields {
    patient: Citizen {
      title: { pt-BR: "Paciente", en: "Patient" }
      lookup
    }
    type: ExamType { lookup }
    status: ExamStatus { lookup }
    result: string
    resultFile: FileRef
    signature: Signature
    collectedAt: DateTime
    releasedAt: DateTime
  }

  indexes {
    patient: [patient.id, collectedAt] range
    byStatus: [status, collectedAt] range
  }

  actions {
    collect(sample: SampleRef) {
      title: { pt-BR: "Coletar", en: "Collect" }
      roles: [Analyst]
      {
      }
    }
    result(value: string, file: FileRef) {
      title: { pt-BR: "Registrar resultado", en: "Record result" }
      roles: [Analyst]
      {
      }
    }
    review(pathologist: User, signature: Signature) {
      title: { pt-BR: "Revisar", en: "Review" }
      roles: [Pathologist]
      {
      }
    }
    release() {
      title: { pt-BR: "Liberar", en: "Release" }
      roles: [Pathologist]
      {
      }
    }
  }

  rules {
    status == released implies signature != null and result != null: {
      fields: []
      pt-BR: "Regra do documento não atendida"
      en: "Document rule not met"
    }
    status == released implies releasedAt != null: {
      fields: []
      pt-BR: "Regra do documento não atendida"
      en: "Document rule not met"
    }
  }
}
```

*Valida: anexo, assinatura, retenção e regra de liberação que exige evidência.*

### D.5 Ticket com SLA calculado

```bpcms
document SupportTicket {
  title: { pt-BR: "Chamado de atendimento", en: "Support ticket" }
  version: 1
  owner: "Support"
  sync: live

  fields {
    customer: Customer {
      title: { pt-BR: "Cliente", en: "Customer" }
      lookup
    }
    subject: string { maxLength: 160, lookup }
    description: string
    priority: TicketPriority { lookup }
    status: TicketStatus { lookup }
    assignee: User {
      title: { pt-BR: "Responsável", en: "Assignee" }
    }
    slaDue: DateTime
    createdAt: DateTime
  }

  computed SlaBreach from SupportTicket {
    freshness: 1m
    exposes: [breached, remaining]
  }

  indexes {
    byStatusPriority: [status, priority, slaDue] range
    assignee: [assignee.id] hash
    search: [subject, description] search
  }

  actions {
    assign(user: User) {
      title: { pt-BR: "Atribuir", en: "Assign" }
      roles: [Agent]

      rules {
        user != null: {
          fields: [assignee]
          pt-BR: "Selecione o responsável"
          en: "Select the assignee"
        }
      }

      {
        assignee = user
        audit("ticket.assigned", user)
      }
    }

    comment(body: string) {
      title: { pt-BR: "Comentar", en: "Comment" }
      roles: [Agent]

      rules {
        body.length > 0: {
          fields: []
          pt-BR: "Escreva um comentário"
          en: "Write a comment"
        }
      }

      {
        audit("ticket.commented", body)
      }
    }

    resolve() {
      title: { pt-BR: "Resolver", en: "Resolve" }
      roles: [Agent]
      transition: Resolved

      rules {
        assignee != null: {
          fields: [assignee]
          pt-BR: "Atribua o chamado antes de resolver"
          en: "Assign the ticket before resolving"
        }
      }

      {
        audit("ticket.resolved")
      }
    }

    reopen(reason: string) {
      title: { pt-BR: "Reabrir", en: "Reopen" }
      roles: [Manager]
      transition: Reopened

      rules {
        reason.length >= 10: {
          fields: [reason]
          pt-BR: "Informe um motivo com pelo menos 10 caracteres"
          en: "Enter a reason with at least 10 characters"
        }
      }

      {
        audit("ticket.reopened", reason)
      }
    }
  }

  rules {
    status == resolved implies assignee != null: {
      fields: []
      pt-BR: "Regra do documento não atendida"
      en: "Document rule not met"
    }
  }
}
```

*Valida: campo calculado de SLA, índice por prioridade e ações de atendimento.*

### D.6 Faturamento mensal calculado

```bpcms
document MonthlyRevenue {
  title: { pt-BR: "Receita mensal", en: "Monthly revenue" }
  version: 1
  owner: "Finance"
  computed from Invoice, Payment
  freshness: 1h
  sync: read

  fields {
    month: YearMonth { lookup }
    revenue: std.Money { lookup }
    orderCount: Count
    calculatedAt: DateTime
  }

  indexes { month: [month] unique }

  actions {
    refresh() {
      title: { pt-BR: "Atualizar", en: "Refresh" }
      roles: [Manager]
      {
      }
    }
  }

  rules {
    month >= 2026-01: {
      fields: []
      pt-BR: "Regra do documento não atendida"
      en: "Document rule not met"
    }
  }
}
```

*Valida: documento calculado, frescor de 1h, leitura sob demanda e ação de refresh.*

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

### V.1 Formulário de cliente

```bpcms
view CustomerForm {
  type: form
  document: Customer
  roles: [Clerk, Manager]

  fields {
    name: string { required }
    email: Email { required }
    phone: Phone
    status: CustomerStatus { readonly }
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
    confirm -> Checkout.confirmOrder
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

### R.1 Vendas por região

```bpcms
document SalesByRegion {
  title: { pt-BR: "Vendas por região", en: "Sales by region" }
  type: report
  computed from Invoice, Customer
  freshness: 15m

  report {
    filters { period; region }
    metrics {
    revenue: sum(Invoice.total) by region
    orders: count(Invoice) by region
    }
    units { revenue: BRL }
    export: [pdf, csv]
  }
}
```

*Valida: dashboard com métricas, dimensão, unidade e frescor.*

### R.2 Envelhecimento de estoque

```bpcms
document InventoryAging {
  title: { pt-BR: "Envelhecimento de estoque", en: "Inventory aging" }
  type: report
  computed from StockItem, Product
  freshness: 15m

  report {
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
}
```

*Valida: relatório com bucket temporal e múltiplas métricas.*

### R.3 Cumprimento de SLA

```bpcms
document SlaCompliance {
  title: { pt-BR: "Cumprimento de prazo", en: "SLA compliance" }
  type: report
  computed from SupportTicket
  freshness: 5m

  report {
    filters { period; team; priority }
    metrics {
    compliance: percentage(SupportTicket.slaMet)
    breached: count(SupportTicket.breached)
    avgResolution: avg(SupportTicket.resolutionTime)
    }
    alert { compliance < 90% -> NotifyLead }
  }
}
```

*Valida: dashboard com percentual, alerta e métrica temporal.*

### R.4 Fluxo de caixa

```bpcms
document CashFlow {
  title: { pt-BR: "Fluxo de caixa", en: "Cash flow" }
  type: report
  computed from Invoice, Payment, Bill
  freshness: 1h

  report {
    filters { period; account }
    dimensions { month; account }
    metrics {
    inflow: sum(Payment.received)
    outflow: sum(Bill.paid)
    balance: inflow - outflow
    }
    units { inflow: BRL; outflow: BRL; balance: BRL }
    export: [pdf, csv]
  }
}
```

*Valida: relatório financeiro com unidades explícitas e saldo calculado.*

### R.5 Qualidade laboratorial

```bpcms
document LabQuality {
  title: { pt-BR: "Qualidade laboratorial", en: "Lab quality" }
  type: report
  computed from LabExam, LabNonConformity
  freshness: 10m

  report {
    filters { period; examType }
    metrics {
    released: count(LabExam.released)
    rework: percentage(LabExam.rework)
    avgTurnaround: avg(LabExam.turnaround)
    critical: count(LabExam.critical)
    }
    export: [pdf]
  }
}
```

*Valida: dashboard de qualidade com tempo de resposta e não conformidade.*

### R.6 Controle operacional em tempo real

```bpcms
document OperationsControl {
  title: { pt-BR: "Controle operacional", en: "Operations control" }
  type: report
  computed from Shipment, Incident, DeviceReading
  freshness: live

  report {
    mode: live
    filters { site; status }
    metrics {
    openShipments: count(Shipment.open)
    lateShipments: count(Shipment.late)
    deviceOffline: count(DeviceReading.offline)
    incidents: count(Incident.open)
    }
    alert {
    lateShipments > 20 -> PageOps
    deviceOffline > 5 -> NotifyOps
    }
  }
}
```

*Valida: dashboard live com alertas operacionais e múltiplas fontes.*

## Event

### E.1 Evento de workflow

```bpcms
shape OrderStatus {
  base: workflow
  initial: draft

  value draft {
    title: { pt-BR: "Rascunho", en: "Draft" }

    transition Placed {
      title: { pt-BR: "Pedido realizado", en: "Order placed" }
      to: placed
      payload {
        customer: User
        total: std.Money
      }
    }
  }

  value placed {
    title: { pt-BR: "Realizado", en: "Placed" }
  }
}
```

*Valida: evento dentro da transição, sem `events`/`emit` na action.*

### E.2 Evento sem workflow

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

*Valida: evento sem workflow em `events`, emitido pela action.*

### E.3 FQN cross-service

```bpcms
process PaymentSaga {
  start on Order.Placed
  end on Payment.Succeeded
}
```

*Valida: assinatura `SERVICO.DOCUMENTO.EVENTO`.*

### E.4 Payload com DocumentRef

```bpcms
transition Placed {
  title: { pt-BR: "Pedido realizado", en: "Order placed" }
  to: placed
  payload {
    customer: User
    total: std.Money
  }
}
```

*Valida: payload com `DocumentRef` e valor monetário.*

### E.5 Transição sem payload

```bpcms
transition Closed {
  title: { pt-BR: "Fechado", en: "Closed" }
  to: closed
}
```

*Valida: payload vazio permitido.*

### E.6 Eventos não são artefatos de topo

Não existe `event <Name> { ... }`.

Eventos existem em:

- `transition` — workflow;
- `events` — sem workflow.

## Job / Schedule

### J.1 Fechamento mensal

```bpcms
job CloseMonth {
  capabilities { scheduling }
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
  capabilities { scheduling }
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
  capabilities { scheduling }
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
  capabilities { scheduling }
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
  capabilities { scheduling }
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
  capabilities { scheduling }
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

Estas não são pendências da baseline; são **decisões de sintaxe** que os exemplos ajudam a testar. Marque o que parece natural e o que parece burocrático.

| # | Pergunta | Exemplos para olhar |
| :--- | :--- | :--- |
| 1 | `shape` deve ser nomeado por dimensão (`Mass`, `Length`) ou genérico (`Quantity<Mass>`)? | H.2, H.3, H.4 |
| 2 | O campo deve carregar `unit` ou o `Shape` já define a unidade | H.2, D.3, W.4 |
| 3 | Documento agregado deve listar itens como `Document[]` ou ter um bloco `items` próprio | D.2, D.3 |
| 4 | O `lookup` automático cobre a projeção manual antiga | D.1, D.2 |
| 5 | O processo deve ser declarado por nós (`task`, `gateway`) ou por XML BPMN com extensões? | P.1–P.6 |
| 6 | `reads`/`writes` por tarefa são úteis ou ruído | P.1–P.6 |
| 7 | Expressões (`sum`, `count`, `by`, `bucket`, `percentage`) são suficientes? | R.1–R.6, N.1–N.6 |
| 8 | `view` deve listar campos ou compor widgets | V.1–V.6 |
| 9 | `role` deve ter permissões inline ou só referenciar um catálogo | R.1–R.6 |
| 10 | `evidence` deve carregar o conteúdo extraído ou só o vínculo e a confiança | E.1–E.6 |
| 11 | `environment` deve declarar `promoteTo` ou a promoção é definida no `release`? | N.1–N.6, V.1–V.6 |
| 12 | `builder` deve declarar `limitations` por plataforma | B.1–B.6 |
| 13 | `widget` deve mapear `platforms` ou só declarar `capabilities`? | W.1–W.6 |
| 14 | `conversation` deve ter `coverage` por dimensão ou por artefato | C.1–C.6 |
| 15 | `release` deve apontar para o lock do app (uma versão/serviço) em vez de listar includes? | V.1–V.6 |
| 16 | Cálculo deve ser campo computado (`computed frete`) ou documento (`document FreightQuote`)? | D.3, D.6 |
| 17 | Shape composto (`fields`) é Shape ou Document embutido? | H.8, D.2 |
| 18 | URI literal em todas as referências é verboso; quer alias local | A.1, S.2 |
| 19 | A página `/index.html` deve listar SOs/APIs/iframe a partir do builder | A.1, S.1, B.1–B.6 |
| 20 | URI entre aspas é aceitável ou prefere uma sintaxe sem aspas (ex.: `@example.com/...`)? | A.1, S.1 |
| 21 | Repositório Git deve ser detectado por host (`source: auto`) ou sempre explícito? | A.7, S.7 |
| 22 | Versão default de repo: commit, tag ou branch Produção deve exigir pin | S.7, V.1–V.6 |
| 23 | A app deve listar só entry points e derivar a closure dos imports | A.1, S.2 |
| 24 | Uma versão por serviço por app é suficiente Como tratar conflito de contrato | lock, V.1–V.6 |
| 25 | IDs no lock devem cobrir todos os identificadores ou só os serializados | lock |
| 26 | `reintroduce` ativo por closure é suficiente, ou precisa de prioridade explícita | A.8, S.8 |
| 27 | Serviço sem `exports` e privacidade só por membro (`private`) está certo? | D.1, S.1–S.8 |
| 28 | Alias local (`"uri" as Nome`) ajuda ou polui? | A.1, S.2 |
| 29 | Sem `extends` na v1 e reuso só por composição/reintroduce está bom | S.8 |
| 30 | Migração como `release` do tipo `migration` é suficiente | V.4 |
| 31 | Capabilities de alto nível (`gui`, `tui`, `print`, `oltp`, `sync`, `location`…) cobrem os casos? | S.1–S.8, B.1–B.6 |
| 32 | Compensação como `type: compensation` + `for` está correta | P.7 |
| 33 | `Parameter` cobre configuração/segredo com `type`, `source`, `required` e `default`? | PR.1–PR.6 |
| 34 | Capabilities customizadas (`llm`, `google-maps`) devem ser nomes abertos/contrato do construtor? | B.1, S.1 |
| 35 | `main` deve ser só `set` de Parameter de serviço e `start`, sem `wire`? | A.1 |
| 36 | `location`, `timezone`, `regionalization` e `units` devem ser capabilities essenciais, não imports | U.1–U.3 |
| 37 | Shapes de unidades métrico/imperial com dependência de regionalization estão corretos | U.3 |
| 38 | `title` localizado por locale (`pt-BR`, `en-US`) e erro/fallback para tradução faltante está correto? | A.1–A.8 |
| 39 | Todo document tem id implícito e `lookup`; `Category` não declara `id`/`categoryId` e `Product.categoryId` é FK | D.1–D.3 |
| 40 | A projeção `User(citizenId)` (só o extra) e o lookup automático de `User` (`nickname`, `avatar`) estão claros? | D.2, árvores |
| 41 | `Consumer extends User` compartilha `User`; `ConsumerId` é alias (mesmo valor) e `citizenId` é o extra? | árvores |
| 42 | O que falta para você dizer “isso eu consigo revisar sem ser programador”? | todos |

## Padrões que os exemplos já sugerem

* **Shape** é o único lugar com validação/formatação/conversão; o campo só parametriza.
* **Document** agrega, referencia ou materializa lookup; computed/device são variações do mesmo conceito.
* **Process** orquestra, mas não implementa regra: chama `action`, `view`, `integration` e `role`.
* **View/Report** são projeções; não são donas de dados.
* **Environment/Builder/Release** são artefatos de plataforma, não de negócio.
* **Dependências são implícitas:** o app lista entry points; `Service.imports` define a closure.
* **Sem `exports`:** todo artefato de topo é público; `private` só em membros internos.
* **Lock do app:** uma versão por serviço e um ID por identificador; o serviço não carrega nenhum dos dois.
* **`reintroduce` é composição:** ativo quando o serviço que reintroduz está na closure; senão, o original vale.
* **Capabilities são de alto nível:** `gui/tui/print`, `oltp/sync`, `location`; o construtor resolve a tecnologia.
* **Parameter** é o contrato de configuração/segredo; o valor é resolvido pelo ambiente e nunca entra no lock.
* **Capabilities são abertas:** construtores podem declarar `llm`, `google-maps`, etc.; o serviço só declara `capabilities`.
* **Compensação é passo:** `type: compensation` + `for` + `action`, em ordem reversa e idempotente.
* **Serviços essenciais são capabilities:** `location`, `timezone`, `regionalization` e `units`; não são imports de app.
* **Repositório Git é origem de primeira classe:** README é documentação, clone/zip/release é distribuição, commit/tag é versão.
* **Evidence/Conversation/Decision/Assumption** são a memória da descoberta; precisam ser consultáveis e versionadas.

### Golden examples sugeridos

Se for validar só alguns, comece por estes seis — eles cobrem as decisões mais estruturais:

1. **H.2 Mass** — dimensão, unidade, projeção e agregação.
2. **D.2 Invoice** — agregado, lookup, itens e regra de total.
3. **P.2 OrderFulfillment** — processo, paralelismo, integração e erro.
4. **V.3 SalesDashboard** — métricas, frescor, unidade e exportação.
5. **R.4 FinanceRoles** — permissões, quatro-olhos e limite.
6. **N.1 PurchaseApproval** — decisão por faixa e caso default.
7. **A.8 + S.8** — lock, closure e reintroduce ativo.
8. **P.7** — compensação como passo de BPMN.

### Checklist de validação

- [ ] A sintaxe parece legível para um analista, não só para um dev
- [ ] `Shape` resolve validação/unidade sem espalhar regra pelo campo
- [ ] Documento agregado com lookup automático faz sentido para o negócio
- [ ] O processo orquestra sem virar código
- [ ] `View` e `Report` são claramente projeções, não donas do dado
- [ ] `Role`/`Policy` expressam quatro-olhos, alçada e retenção
- [ ] `Environment`/`Builder`/`Release` dão segurança para dev/QA/prod
- [ ] Falta algum artefato para representar seu domínio

> Se algo aqui parecer “código demais” para o dono do processo, o problema é de sintaxe/experiência — não do conceito. O próximo passo é escolher 3–5 exemplos como **golden examples** e usá-los como teste de aceitação da DSL no SPECS.
