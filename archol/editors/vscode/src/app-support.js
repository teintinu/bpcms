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

// True when the source declares `application "<uri>" {` (ignores comments).
function isApplicationSource(text) {
  return /^\s*application\s+"[^"]+"\s*\{/m.test(String(text || ''));
}

// URI declared by an application source, or an empty string.
function applicationUriFromSource(text) {
  const match = /^\s*application\s+"([^"]+)"/m.exec(String(text || ''));
  return match ? match[1] : '';
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

// A typed reference is either a plain lock id (`ref`) or a node
// (`{ kind: "PendingRef", id, ... }`). Both reduce to the target id.
function referenceTargetId(node) {
  if (!node || typeof node !== 'object') return null;
  const ref = node.ref;
  if (ref && typeof ref === 'object') {
    return ref.id !== undefined && ref.id !== null ? ref.id : null;
  }
  return ref !== undefined && ref !== null ? ref : null;
}

// Brief identifier information shown for the selected AST node.
function referenceSummary(node) {
  if (!node || typeof node !== 'object') return '';
  const parts = [];
  if (node.id !== undefined && node.id !== null) parts.push(`id ${node.id}`);
  const ref = referenceTargetId(node);
  if (ref !== null) parts.push(`ref ${ref}`);
  if (node.refKind) parts.push(`refKind ${node.refKind}`);
  return parts.join(' · ');
}

// `archol/declaration` accepts the reference target when present, otherwise
// the node's own declaration id.
function declarationTargetId(node) {
  if (!node || typeof node !== 'object') return null;
  const ref = referenceTargetId(node);
  if (ref !== null) return ref;
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
  const application = result.application && typeof result.application === 'object'
    ? result.application
    : null;
  const normalized = { ast, inApplication, commentedOut };
  if (application) normalized.application = application;
  const error = typeof result.error === 'string' ? result.error : '';
  if (error) normalized.error = error;
  return normalized;
}

function isOutsideApplication(result) {
  const normalized = normalizeAstResult(result);
  return !normalized.inApplication || normalized.commentedOut;
}

// Source injected into the webview script so the behavior tested here is the
// exact behavior used by the tree.
function browserNodeHelpersSource() {
  return [referenceTargetId, referenceSummary, declarationTargetId, usageTargetId]
    .map((fn) => fn.toString())
    .join('\n');
}

module.exports = {
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
  uriPathname,
  usageDescription,
  usageLabel,
  usageLine,
  usageQuickPickItem,
  usageTargetId,
};
