'use strict';

const vscode = require('vscode');
const { createDebouncer } = require('./lock-sync-support');

// Refreshes `<app>.lock` at most once every `archol.lockDebounceMs` after any
// `.archol` change/save. Selection already refreshes it immediately on the
// server side; this keeps ids in sync while editing.
function registerLockSync(context, dependencies) {
  const { session, getApplication, getDelay, log } = dependencies;
  let inFlight = false;
  let again = false;
  const pendingUris = new Set();
  const debouncer = createDebouncer(async () => {
    if (!getApplication()) return;
    if (inFlight) {
      again = true;
      return;
    }
    inFlight = true;
    const uris = [...pendingUris];
    pendingUris.clear();
    try {
      const client = await session.ensureClient();
      const relevant = uris.length === 0
        ? true
        : (await Promise.all(uris.map(async (uri) => {
          try {
            const status = await client.request('archol/inApplication', { uri });
            return Boolean(status?.inApplication);
          } catch {
            return true;
          }
        }))).some(Boolean);
      if (relevant) {
        await client.request('archol/lock', {});
        log('[lock] identifier lock refreshed');
      }
    } catch (error) {
      log(`[lock] refresh failed: ${error.message}`);
    } finally {
      inFlight = false;
      if (again) {
        again = false;
        debouncer.schedule();
      }
    }
  }, getDelay);

  const schedule = (document) => {
    if (document && document.languageId === 'archol') {
      pendingUris.add(document.uri.toString());
      debouncer.schedule();
    }
  };
  context.subscriptions.push(
    vscode.workspace.onDidChangeTextDocument((event) => schedule(event.document)),
    vscode.workspace.onDidSaveTextDocument((document) => schedule(document)),
    { dispose: () => debouncer.cancel() },
  );
}

module.exports = { registerLockSync };
