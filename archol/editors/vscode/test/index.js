'use strict';

// Allows running `node --test archol/editors/vscode/test` (without a glob):
// Node resolves the directory as a module and this index loads the tests.
require('./protocol.test');
require('./ast-tree.test');
require('./server-path.test');
