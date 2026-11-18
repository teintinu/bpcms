'use strict';

// Pure helpers shared by the extension and the AST explorer webview.
// This module must not require `vscode` so `node --test` can load it directly.

function uriPathname(uri) {
  const text = typeof uri === 'string' ? uri : '';
  if (!text) return '';
  try {
    const parsed = new URL(text);
    return decodeURIComponent(parsed.pathname || '');
  } catch {
    return text;
  }
}

function normalizeSlashes(value) {
  return String(value || '').replace(/\\/g, '/');
}

// Derives a readable application name from its URI without touching the
// filesystem: the last path segment, without `.archol` or `-application`.
function applicationNameFromUri(uri) {
  const text = typeof uri === 'string' ? uri : '';
  const pathname = normalizeSlashes(uriPathname(text)).replace(/\/+$/, '');
  const segments = pathname.split('/').filter(Boolean);
  if (segments.length === 0) return text;
  let name = segments[segments.length - 1];
  if (/\.archol$/i.test(name)) name = name.slice(0, -'.archol'.length);
  if (/-application$/i.test(name)) name = name.slice(0, -'-application'.length);
  return name || text;
}

function applicationDisplayName(application) {
  if (!application || typeof application !== 'object') return '';
  const name = typeof application.name === 'string' ? application.name.trim() : '';
  if (name) return name;
  return applicationNameFromUri(application.uri);
}

function applicationQuickPickItem(application, currentUri) {
  const uri = String(application?.uri || '');
  const current = Boolean(uri) && uri === String(currentUri || '');
  let detail = '';
  if (current) {
    detail = 'Current application';
  } else if (application?.path) {
    detail = String(application.path);
  }
  return {
    label: applicationDisplayName(application) || 'Application',
    description: uri,
    detail,
    application,
  };
}

// Relative path for a URI inside `rootUri`; falls back to the URI pathname.
function relativePathFromUri(uri, rootUri) {
  const pathname = normalizeSlashes(uriPathname(uri));
  const root = normalizeSlashes(uriPathname(rootUri)).replace(/\/+$/, '');
  if (pathname && root && pathname !== root && pathname.startsWith(`${root}/`)) {
    return pathname.slice(root.length).replace(/^\/+/, '');
  }
  if (pathname) return pathname.replace(/^\/+/, '');
  return typeof uri === 'string' ? uri : '';
}

// `archol/usages` ranges follow LSP (zero-based line); the label is one-based.
function usageLine(location) {
  const line = location?.range?.start?.line;
  const value = Number(line);
  return Number.isFinite(value) ? Math.max(1, Math.trunc(value) + 1) : 1;
}

function usageLabel(location, rootUri) {
  const path = relativePathFromUri(location?.uri, rootUri) || '(unknown)';
  return `${path}:${usageLine(location)}`;
}

function usageDescription(location) {
  return location?.kind ? String(location.kind) : '';
}

function usageQuickPickItem(location, rootUri) {
  return {
    label: usageLabel(location, rootUri),
    description: usageDescription(location),
    location,
  };
}

// Brief identifier information shown for the selected AST node.
function referenceSummary(node) {
  if (!node || typeof node !== 'object') return '';
  const parts = [];
  if (node.id !== undefined && node.id !== null) parts.push(`id ${node.id}`);
  if (node.ref !== undefined && node.ref !== null) parts.push(`ref ${node.ref}`);
  if (node.refKind) parts.push(`refKind ${node.refKind}`);
  return parts.join(' · ');
}

// `archol/declaration` accepts the reference target when present, otherwise
// the node's own declaration id.
function declarationTargetId(node) {
  if (!node || typeof node !== 'object') return null;
  if (node.ref !== undefined && node.ref !== null) return node.ref;
  if (node.id !== undefined && node.id !== null) return node.id;
  return null;
}

function usageTargetId(node) {
  if (!node || typeof node !== 'object') return null;
  return node.id !== undefined && node.id !== null ? node.id : null;
}

// `archol/ast` may return `{ ast, inApplication }`; an AST outside the
// application closure has `commentedOut: true`.
function normalizeAstResult(result) {
  if (!result || typeof result !== 'object') {
    return { ast: null, inApplication: true, commentedOut: false };
  }
  const ast = result.ast !== undefined
    ? (result.ast && typeof result.ast === 'object' ? result.ast : null)
    : (typeof result.kind === 'string' ? result : null);
  const inApplication = result.inApplication !== false;
  const commentedOut = Boolean(result.commentedOut)
    || Boolean(ast && ast.commentedOut)
    || !inApplication;
  return { ast, inApplication, commentedOut };
}

function isOutsideApplication(result) {
  const normalized = normalizeAstResult(result);
  return !normalized.inApplication || normalized.commentedOut;
}

// Source injected into the webview script so the behavior tested here is the
// exact behavior used by the tree.
function browserNodeHelpersSource() {
  return [referenceSummary, declarationTargetId, usageTargetId]
    .map((fn) => fn.toString())
    .join('\n');
}

module.exports = {
  applicationDisplayName,
  applicationNameFromUri,
  applicationQuickPickItem,
  browserNodeHelpersSource,
  declarationTargetId,
  isOutsideApplication,
  normalizeAstResult,
  referenceSummary,
  relativePathFromUri,
  uriPathname,
  usageDescription,
  usageLabel,
  usageLine,
  usageQuickPickItem,
  usageTargetId,
};
