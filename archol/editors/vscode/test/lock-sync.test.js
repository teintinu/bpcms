'use strict';

const test = require('node:test');
const assert = require('node:assert/strict');
const { createDebouncer } = require('../src/lock-sync-support');

function fakeTimers() {
  let now = 0;
  let nextHandle = 1;
  const pending = new Map();
  return {
    setTimeout(fn, ms) {
      const handle = nextHandle++;
      pending.set(handle, { at: now + ms, fn });
      return handle;
    },
    clearTimeout(handle) {
      pending.delete(handle);
    },
    advance(ms) {
      now += ms;
      for (const [handle, item] of [...pending]) {
        if (item.at <= now) {
          pending.delete(handle);
          item.fn();
        }
      }
    },
    get size() {
      return pending.size;
    },
  };
}

test('debouncer fires once after the delay, resetting on each schedule', () => {
  const timers = fakeTimers();
  let fired = 0;
  const debouncer = createDebouncer(() => { fired += 1; }, () => 10000, timers);
  debouncer.schedule();
  timers.advance(5000);
  debouncer.schedule();
  timers.advance(9000);
  assert.equal(fired, 0);
  timers.advance(1000);
  assert.equal(fired, 1);
  assert.equal(timers.size, 0);
  assert.equal(debouncer.pending, false);
});

test('debouncer cancel prevents the callback', () => {
  const timers = fakeTimers();
  let fired = 0;
  const debouncer = createDebouncer(() => { fired += 1; }, () => 10000, timers);
  debouncer.schedule();
  debouncer.cancel();
  timers.advance(20000);
  assert.equal(fired, 0);
});
