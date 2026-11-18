'use strict';

// Permite executar `node --test archol/editors/vscode/test` (sem glob):
// o Node resolve o diretório como módulo e este índice carrega os testes.
require('./protocol.test');
require('./ast-tree.test');
require('./server-path.test');
