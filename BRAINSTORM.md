- o objetivo aqui é criar a ferramenta BPCMS que é uma mescla de BPM + CMS + SOA
- tudo que for desenvolvido aqui deverá seguir ridigamente SOLID e clean code, e algo parecido com clean arquitecture mas com artefatos e conceitos próprios. 
- Vamos adotar uma linguagem parecida com typescript para definir artefatos. Teremos cláusulas que definirao objetos, por exemplo `application A {name: 'x'}` seria equivalente a `const A=new application(); a.name = x;`. O parametro passado vai supertar YAML
- Será necessária uma linguagemSerá necessário um plugin VSCode + LSP para essa li

- artefatos:
1) Aplicacao: Sem do ponto de vista do usuário final, ou seja, mesmo que sejam varias aplicações web, em páginas e domínios diferentes, mesmo que alguns sejam aplicações windows outras android outras ios ou linux e até de linha de comando. Uma aplicação é o conceito geral disso. Ou seja, é uma abstração de que será implementado mas com informação suficiente para que seja implementado atraves dos construtores+tema. De certa forma uma aplicacao é um AST. Mas a aplicação em si apenas define os serviços a serem usados e uma initializacao. Toda lógica deverá ser escrita em artefatos dentro de um serviço.

Exemplos:

application JohnStore {
  caption:
    pt_BR: 'Loja do João'
    en: 'Jonh Store'
  services:
    - Cart
    - Checkout
    - Catalog
    - Inventory
    - Order
    - Payment
    - Shipping
    - Auth
    - Notification
}

application JohnStoreBackoffice {
  caption:
    pt_BR: 'Administração da Loja do João'
    en: 'Jonh Store Backoffice'
  services:
    - Catalog
    - Inventory
    - Order
    - Payment
    - Shipping
    - Auth
    - Notification
}

2) Serviço: É uma parte da aplicação, algo que pode ser reusado por várias aplicações e também por outros serviços. É como um package ou uma DLL mas num conceito de negócio. Ele pode importar artefatos de outros serviços, extender/sobreescrever eles e pode exportar novos artefatos. A ideia aqui é que o serviço emglobe tudo praquele servico funcionar, frontend/backend/tarefas automatizadas/manuais, etc...

* quando um servico A usa um outro serviço B, o serviço A pode sobrescrever algo em qq dos artefatos do serviço B

artefatos que podem ser definidos/importados/usados por um serviço:
  a) Shape - é como um DOMAIN do SQL mas muito mais avançado, com possibiliade de definir como armazenar/envelopar/stringify, como desenvelopar/parse, como exibir, aqui não iremos definir detalhes da UI mas, por exemplo, um número pode exibir separadores de milhares e de decimais diferentes conforme o idioma. Ou um enum pode serializar apenas 1-SIM,2-NAO mas mostrar um checkbox bonito na tela (checkbox aqui é um widget). Deve ter possibilidade também de gerar um valor random


  b) widget - é um componente visível qualquer que permita interação com o usuário. O conceito aqui é independente da plataforma e sempre no auto-nível. Então, por exemplo, um FORM será composto por INPUTs e teremos INPUTs para data/string/CPF,memo/imagem/etc... se for numa página web isso será feito com form/input,textarea e algum tratamento para colocar um IMG, num app GUI para macos/windows já será diferente e depenerá da linguagem, num app para console/terminal linux nem é possível mostrar imagem ou RTF. então poderiamos apenas selecionar um arquivo de imagem, por exemplo. O widget é por tanto uma definição conceitual, cada widget deverá ter a representaçao do que será gerado dele no construtor.

  c) document - um documento é como uma classe mas já considerando persistência, replicação de dados em multiplos ambientes, atualização sincrona/acroncrona, caching. Todas as propriedades deverão ser tipadas usando Shape, nunca tipos primitivos diretamente. Um documento pode ser uma tabela única, por exemplo um cadastro de cliente. ou pode ser uma junção de várias, por exemplo nota fiscal (já deve conter itens e os dados do cliente). É permitido duplicar dados, por exemplo pode-se duplicar os dados de cliente usados na NF e manter a referencia ao cliente. Se o cliente mudar ou for excluido a NF continua sendo um documento válido. O documento também poderá ser um state redux ou jotai na UI, por exemplo. Quando o documento for usado em modo real, a view deverá definir isso, pode-se ter um eventstream para notificar mudanças no documento, um redis para armazenar o documento em cache, etc... tudo isso é o documento. Observe que não se limita a FE ou BE, é um conceito amplo. Documentos também podem ser calculados, por exemplo, Faturamento mensagem poderá juntar dados do cliente de várias NF, enquanto o mês não fechar o documento deverá ficar como draft, isso acontece também se uma pessoa começa a cadastrar um cliente, ele fica como draft até confirmar. para confirmar é preciso passar pelas validações, primeiramente as validações dos Shapes, por exemplo um CPF inválido ou não preenchido, um nome em branco, etc... O documento também poderá ficar como valido ou como arquivado. só é permitido links a documentos válidos. caso o calculo seja muito complicado o documento poderá ter um estado de computing, é como o draft mas quer dizer que está sendo calculado naquelo momento (tipo refreshing).
  um documento poderá ter acoes, por exemplo cliente.create(name: 'maria',cpf: '123')
  algumas documentos poderão ter OLTP mas, em geral, a maioria dos documentos são assincronos, materão a informação de quando foram atualizados pela ultima vez, com possibilidade de proxima atualização/forçar a atualização naquele momento e serão sincronizados com merge (semelhante ao git), para isso será necessário um editor de conflitos para o documento e também para cada Shape

  d) view - pode ser form ou report ou table. usar widgets para exibir dados dos documentos. na verdade uma view é uma composição de widgets, com layouts, blocks, páginas, popovers, popups, etc... observe que nem todo widget poderá ser apresentado em todos os ambiente, porque há limitação para isso.

  e) organogram - define roles e hierarquia de roles. um gerente pode autorizar mas não pode fazer.  uma app poderá remapear roles, unificando ou detalhando. por examplo a role R1 se estiver duplicada no servico S1 e S2, o app poderá considerar como a mesma ou dar um alias, por exemplo R1_S1 e R1_S2.

2) construtores: são "plugins" capazes "gerar" ou "executar em runtime" os artefatos definidos acima. há ainda o conceito de tema que são opções e cores e icones que os construtores podem usar.

