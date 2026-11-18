'use strict';

const vscode = require('vscode');

const COMPLETION_KINDS = [
  vscode.CompletionItemKind.Text,
  vscode.CompletionItemKind.Method,
  vscode.CompletionItemKind.Function,
  vscode.CompletionItemKind.Constructor,
  vscode.CompletionItemKind.Field,
  vscode.CompletionItemKind.Variable,
  vscode.CompletionItemKind.Class,
  vscode.CompletionItemKind.Interface,
  vscode.CompletionItemKind.Module,
  vscode.CompletionItemKind.Property,
  vscode.CompletionItemKind.Unit,
  vscode.CompletionItemKind.Value,
  vscode.CompletionItemKind.Enum,
  vscode.CompletionItemKind.Keyword,
  vscode.CompletionItemKind.Snippet,
  vscode.CompletionItemKind.Color,
  vscode.CompletionItemKind.File,
  vscode.CompletionItemKind.Reference,
  vscode.CompletionItemKind.Folder,
  vscode.CompletionItemKind.EnumMember,
  vscode.CompletionItemKind.Constant,
  vscode.CompletionItemKind.Struct,
  vscode.CompletionItemKind.Event,
  vscode.CompletionItemKind.Operator,
  vscode.CompletionItemKind.TypeParameter,
];

const SYMBOL_KINDS = [
  vscode.SymbolKind.File,
  vscode.SymbolKind.File,
  vscode.SymbolKind.Module,
  vscode.SymbolKind.Namespace,
  vscode.SymbolKind.Package,
  vscode.SymbolKind.Class,
  vscode.SymbolKind.Method,
  vscode.SymbolKind.Property,
  vscode.SymbolKind.Field,
  vscode.SymbolKind.Constructor,
  vscode.SymbolKind.Enum,
  vscode.SymbolKind.Interface,
  vscode.SymbolKind.Function,
  vscode.SymbolKind.Variable,
  vscode.SymbolKind.Constant,
  vscode.SymbolKind.String,
  vscode.SymbolKind.Number,
  vscode.SymbolKind.Boolean,
  vscode.SymbolKind.Array,
  vscode.SymbolKind.Object,
  vscode.SymbolKind.Key,
  vscode.SymbolKind.Null,
  vscode.SymbolKind.EnumMember,
  vscode.SymbolKind.Struct,
  vscode.SymbolKind.Event,
  vscode.SymbolKind.Operator,
  vscode.SymbolKind.TypeParameter,
];

function toRange(range) {
  if (!range || !range.start || !range.end) return new vscode.Range(0, 0, 0, 0);
  return new vscode.Range(
    range.start.line,
    range.start.character,
    range.end.line,
    range.end.character,
  );
}

function toLocation(location) {
  if (!location) return null;
  if (location.targetUri) {
    return new vscode.Location(
      vscode.Uri.parse(location.targetUri),
      toRange(location.targetSelectionRange || location.targetRange),
    );
  }
  if (!location.uri) return null;
  return new vscode.Location(vscode.Uri.parse(location.uri), toRange(location.range));
}

function toMarkdown(value) {
  const markdown = new vscode.MarkdownString(value);
  markdown.isTrusted = { enabledCommands: ['archol.showAst'] };
  return markdown;
}

function toHoverContents(contents) {
  if (!contents) return null;
  const list = Array.isArray(contents) ? contents : [contents];
  const converted = [];
  for (const item of list) {
    if (typeof item === 'string') {
      converted.push(toMarkdown(item));
      continue;
    }
    if (!item) continue;
    if (item.kind === 'markdown') {
      converted.push(toMarkdown(item.value || ''));
    } else if (item.kind === 'plaintext') {
      const markdown = new vscode.MarkdownString();
      markdown.appendText(item.value || '');
      converted.push(markdown);
    } else if (item.language) {
      converted.push(new vscode.MarkdownString().appendCodeblock(item.value || '', item.language));
    } else {
      const markdown = new vscode.MarkdownString();
      markdown.appendText(String(item.value || ''));
      converted.push(markdown);
    }
  }
  return converted.length > 0 ? converted : null;
}

function toDocumentation(documentation) {
  if (documentation === undefined || documentation === null) return undefined;
  if (typeof documentation === 'string') return documentation;
  if (documentation.kind === 'markdown') return toMarkdown(documentation.value || '');
  if (documentation.kind === 'plaintext') {
    const markdown = new vscode.MarkdownString();
    markdown.appendText(documentation.value || '');
    return markdown;
  }
  return documentation.value;
}

function toCompletionItem(item) {
  const completion = new vscode.CompletionItem(
    String(item.label ?? ''),
    COMPLETION_KINDS[Number(item.kind) - 1] || vscode.CompletionItemKind.Text,
  );
  completion.detail = item.detail;
  completion.documentation = toDocumentation(item.documentation);
  if (typeof item.insertText === 'string') {
    completion.insertText = item.kind === 15
      ? new vscode.SnippetString(item.insertText)
      : item.insertText;
  }
  if (typeof item.filterText === 'string') completion.filterText = item.filterText;
  if (typeof item.sortText === 'string') completion.sortText = item.sortText;
  if (typeof item.preselect === 'boolean') completion.preselect = item.preselect;
  return completion;
}

function toDocumentSymbol(item) {
  const range = toRange(item.range);
  const selectionRange = toRange(item.selectionRange || item.range);
  const symbol = new vscode.DocumentSymbol(
    String(item.name ?? ''),
    String(item.detail ?? ''),
    SYMBOL_KINDS[item.kind] || vscode.SymbolKind.Variable,
    range,
    selectionRange,
  );
  if (Array.isArray(item.children)) {
    symbol.children = item.children
      .filter((child) => child && child.location === undefined)
      .map(toDocumentSymbol);
  }
  return symbol;
}

function toSymbolInformation(item) {
  return new vscode.SymbolInformation(
    String(item.name ?? ''),
    SYMBOL_KINDS[item.kind] || vscode.SymbolKind.Variable,
    String(item.containerName ?? ''),
    toLocation(item.location),
  );
}

function registerLanguageFeatures(context, dependencies) {
  const { getClient, log } = dependencies;

  const request = async (method, params) => {
    const client = getClient();
    if (!client) return null;
    try {
      return await client.request(method, params);
    } catch (error) {
      log(`falha em ${method}: ${error.message}`);
      return null;
    }
  };

  const documentParams = (document, position) => ({
    textDocument: { uri: document.uri.toString() },
    position: { line: position.line, character: position.character },
  });

  const hoverProvider = {
    async provideHover(document, position) {
      const result = await request('textDocument/hover', documentParams(document, position));
      if (!result) return null;
      const contents = toHoverContents(result.contents);
      if (!contents) return null;
      return new vscode.Hover(contents, result.range ? toRange(result.range) : undefined);
    },
  };

  const completionProvider = {
    triggerCharacters: ['.', ':', '[', '='],
    async provideCompletionItems(document, position) {
      const result = await request('textDocument/completion', documentParams(document, position));
      const items = Array.isArray(result) ? result : result?.items;
      if (!Array.isArray(items)) return [];
      return items.map(toCompletionItem);
    },
  };

  const definitionProvider = {
    async provideDefinition(document, position) {
      const result = await request('textDocument/definition', documentParams(document, position));
      if (!result) return null;
      const list = Array.isArray(result) ? result : [result];
      return list.map(toLocation).filter(Boolean);
    },
  };

  const documentSymbolProvider = {
    async provideDocumentSymbols(document) {
      const result = await request('textDocument/documentSymbol', {
        textDocument: { uri: document.uri.toString() },
      });
      if (!Array.isArray(result)) return [];
      if (result.some((item) => item && item.location)) {
        return result.map(toSymbolInformation);
      }
      return result.filter(Boolean).map(toDocumentSymbol);
    },
  };

  const referenceProvider = {
    async provideReferences(document, position, options) {
      const result = await request('textDocument/references', {
        ...documentParams(document, position),
        context: { includeDeclaration: Boolean(options?.includeDeclaration) },
      });
      if (!Array.isArray(result)) return [];
      return result.map(toLocation).filter(Boolean);
    },
  };

  context.subscriptions.push(
    vscode.languages.registerHoverProvider({ language: 'archol' }, hoverProvider),
    vscode.languages.registerCompletionItemProvider({ language: 'archol' }, completionProvider, '.', ':', '[', '='),
    vscode.languages.registerDefinitionProvider({ language: 'archol' }, definitionProvider),
    vscode.languages.registerDocumentSymbolProvider({ language: 'archol' }, documentSymbolProvider),
    vscode.languages.registerReferenceProvider({ language: 'archol' }, referenceProvider),
  );
}

module.exports = { registerLanguageFeatures };
