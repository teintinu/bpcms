'use strict';

const LANGUAGE_ID = 'archol';

function isArcholDocument(document) {
  if (!document) return false;
  return document.languageId === LANGUAGE_ID || document.uri.path.toLowerCase().endsWith('.archol');
}

module.exports = { LANGUAGE_ID, isArcholDocument };
