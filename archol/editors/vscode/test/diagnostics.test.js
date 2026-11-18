'use strict';

const test = require('node:test');
const assert = require('node:assert/strict');
const { hasOutsideMarker, OUTSIDE_APPLICATION_CODE } = require('../src/diagnostics-support');

test('detects the outside-application marker', () => {
  assert.equal(hasOutsideMarker([{ code: OUTSIDE_APPLICATION_CODE }]), true);
  assert.equal(hasOutsideMarker([{ code: 'E_ENUM_VALUE' }]), false);
  assert.equal(hasOutsideMarker([]), false);
  assert.equal(hasOutsideMarker(null), false);
});
