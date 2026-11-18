'use strict';

// Small timer-based debouncer kept free of `vscode` so `node --test` can cover it.
function createDebouncer(callback, delayProvider, timers) {
  const scheduler = timers || {
    setTimeout: (fn, ms) => setTimeout(fn, ms),
    clearTimeout: (handle) => clearTimeout(handle),
  };
  let handle = null;
  const debouncer = {
    schedule() {
      if (handle !== null) scheduler.clearTimeout(handle);
      const delay = Math.max(0, Number(delayProvider()) || 0);
      handle = scheduler.setTimeout(() => {
        handle = null;
        callback();
      }, delay);
    },
    cancel() {
      if (handle !== null) scheduler.clearTimeout(handle);
      handle = null;
    },
    get pending() {
      return handle !== null;
    },
  };
  return debouncer;
}

module.exports = { createDebouncer };
