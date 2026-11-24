'use strict';

const test = require('node:test');
const assert = require('node:assert/strict');
const {
  applicationDisplayName,
  applicationNameFromUri,
  applicationUriFromSource,
  isApplicationSource,
  applicationQuickPickItem,
  browserNodeHelpersSource,
  declarationTargetId,
  isOutsideApplication,
  normalizeAstResult,
  referenceSummary,
  relativePathFromUri,
  usageDescription,
  usageLabel,
  usageLine,
  usageQuickPickItem,
  usageTargetId,
} = require('../src/app-support');

const appUri = 'file:///workspace/examples/applications/example.com/example.com-application.archol';
const rootUri = 'file:///workspace';

test('applicationNameFromUri uses the last segment without suffix and extension', () => {
  assert.equal(applicationNameFromUri(appUri), 'example.com');
  assert.equal(
    applicationNameFromUri('file:///workspace/applications/billing/billing-application.archol'),
    'billing',
  );
  assert.equal(applicationNameFromUri('file:///workspace/applications/catalog/'), 'catalog');
  assert.equal(applicationNameFromUri('untitled:Untitled-1'), 'Untitled-1');
  assert.equal(applicationNameFromUri(''), '');
});

test('applicationDisplayName prefers the server name and falls back to the URI', () => {
  assert.equal(applicationDisplayName({ name: 'Storefront', uri: appUri }), 'Storefront');
  assert.equal(applicationDisplayName({ name: '  ', uri: appUri }), 'example.com');
  assert.equal(applicationDisplayName({ uri: appUri }), 'example.com');
  assert.equal(applicationDisplayName(null), '');
});

test('applicationQuickPickItem shows the name as label and the URI as description', () => {
  const item = applicationQuickPickItem({ name: 'Storefront', uri: appUri, path: 'examples/app.archol' }, appUri);
  assert.equal(item.label, 'Storefront');
  assert.equal(item.description, appUri);
  assert.equal(item.detail, 'Current application');
  assert.equal(item.application.uri, appUri);
  const other = applicationQuickPickItem({ uri: appUri, path: 'examples/app.archol' }, 'file:///other');
  assert.equal(other.detail, 'examples/app.archol');
});

test('relativePathFromUri strips the workspace root and normalizes slashes', () => {
  assert.equal(relativePathFromUri(`${rootUri}/services/billing.archol`, rootUri), 'services/billing.archol');
  assert.equal(relativePathFromUri('file:///other/billing.archol', rootUri), 'other/billing.archol');
  assert.equal(
    relativePathFromUri('file:///c%3A/ws/services/billing.archol', 'file:///c%3A/ws'),
    'services/billing.archol',
  );
  assert.equal(relativePathFromUri('file:///workspace', rootUri), 'workspace');
  assert.equal(relativePathFromUri('', rootUri), '');
});

test('usageLabel renders one-based relative path:line and a kind description', () => {
  const location = {
    uri: `${rootUri}/services/billing.archol`,
    range: { start: { line: 11, character: 4 }, end: { line: 11, character: 10 } },
    kind: 'read',
  };
  assert.equal(usageLine(location), 12);
  assert.equal(usageLabel(location, rootUri), 'services/billing.archol:12');
  assert.equal(usageDescription(location), 'read');
  const item = usageQuickPickItem(location, rootUri);
  assert.equal(item.label, 'services/billing.archol:12');
  assert.equal(item.description, 'read');
  assert.equal(item.location, location);
});

test('usageLabel tolerates missing ranges and URIs', () => {
  assert.equal(usageLine({}), 1);
  assert.equal(usageLabel({ uri: '' }, rootUri), '(unknown):1');
  assert.equal(usageDescription({}), '');
});

test('referenceSummary shows id, ref and refKind only when present', () => {
  assert.equal(referenceSummary({ id: 12, ref: 7, refKind: 'enumValue' }), 'id 12 · ref 7 · refKind enumValue');
  assert.equal(referenceSummary({ id: 0 }), 'id 0');
  assert.equal(referenceSummary({ ref: 4 }), 'ref 4');
  assert.equal(referenceSummary({ kind: 'Literal', value: 'x' }), '');
  assert.equal(referenceSummary(null), '');
});

test('navigation targets prefer ref for declarations and require id for usages', () => {
  assert.equal(declarationTargetId({ id: 3, ref: 9 }), 9);
  assert.equal(declarationTargetId({ id: 3 }), 3);
  assert.equal(declarationTargetId({ id: 0 }), 0);
  assert.equal(declarationTargetId({}), null);
  assert.equal(usageTargetId({ id: 3, ref: 9 }), 3);
  assert.equal(usageTargetId({ ref: 9 }), null);
});

test('normalizeAstResult honors inApplication and commentedOut', () => {
  const inApp = normalizeAstResult({ ast: { kind: 'SourceFile' }, inApplication: true });
  assert.deepEqual(inApp, { ast: { kind: 'SourceFile' }, inApplication: true, commentedOut: false });
  assert.equal(isOutsideApplication(inApp), false);

  const outside = normalizeAstResult({ ast: { kind: 'SourceFile' }, inApplication: false });
  assert.equal(outside.commentedOut, true);
  assert.equal(isOutsideApplication(outside), true);

  const commented = normalizeAstResult({ ast: { kind: 'SourceFile', commentedOut: true }, inApplication: true });
  assert.equal(commented.commentedOut, true);
  assert.equal(isOutsideApplication(commented), true);

  const rootFlag = normalizeAstResult({ ast: { kind: 'SourceFile' }, inApplication: true, commentedOut: true });
  assert.equal(rootFlag.commentedOut, true);
  assert.equal(isOutsideApplication(rootFlag), true);

  assert.deepEqual(normalizeAstResult(null), { ast: null, inApplication: true, commentedOut: false });
  assert.equal(normalizeAstResult({ kind: 'SourceFile' }).ast.kind, 'SourceFile');
});

test('normalizeAstResult carries the application and error for the app AST', () => {
  const resolved = normalizeAstResult({
    ast: { kind: 'ArcholApplication' },
    application: { uri: 'example.com/app/demo', path: 'apps/demo.archol' },
    inApplication: true,
  });
  assert.equal(resolved.ast.kind, 'ArcholApplication');
  assert.deepEqual(resolved.application, {
    uri: 'example.com/app/demo',
    path: 'apps/demo.archol',
  });
  assert.equal(resolved.error, undefined);

  const failed = normalizeAstResult({ ast: null, error: 'no application' });
  assert.equal(failed.ast, null);
  assert.equal(failed.error, 'no application');
});

test('browserNodeHelpersSource exposes the same tested functions to the webview', () => {
  const source = browserNodeHelpersSource();
  assert.match(source, /function referenceSummary\(node\)/);
  assert.match(source, /function declarationTargetId\(node\)/);
  assert.match(source, /function usageTargetId\(node\)/);
});

test('isApplicationSource recognizes application declarations only', () => {
  assert.equal(isApplicationSource('application "example.com/app/demo" {\n  services {}\n}'), true);
  assert.equal(isApplicationSource('// application "example.com/app/demo"\nservice "x" {}'), false);
  assert.equal(isApplicationSource('document Invoice {}'), false);
  assert.equal(isApplicationSource(''), false);
});

test('applicationUriFromSource extracts the declared URI', () => {
  assert.equal(
    applicationUriFromSource('// header\napplication "example.com/app/demo" {\n}'),
    'example.com/app/demo',
  );
  assert.equal(applicationUriFromSource('service "example.com/services/a" {}'), '');
});
