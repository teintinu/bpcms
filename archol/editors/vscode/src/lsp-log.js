'use strict';

function shortUri(uri) {
  try {
    const parsed = new URL(uri);
    const decoded = decodeURIComponent(parsed.pathname);
    return decoded.length > 60 ? `…${decoded.slice(-59)}` : decoded;
  } catch {
    return uri.length > 60 ? `…${uri.slice(-59)}` : uri;
  }
}

function describeParams(params) {
  if (!params || typeof params !== 'object') return '';
  const bits = [];
  const uri = params.textDocument?.uri || params.uri;
  if (uri) bits.push(shortUri(uri));
  if (params.position) bits.push(`${params.position.line}:${params.position.character}`);
  if (params.textDocument?.version !== undefined) bits.push(`v${params.textDocument.version}`);
  if (Array.isArray(params.diagnostics)) bits.push(`${params.diagnostics.length} diagnóstico(s)`);
  return bits.length > 0 ? ` ${bits.join(' ')}` : '';
}

module.exports = { describeParams, shortUri };
