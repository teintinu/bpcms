/**
 * Browser half of the Archol preview bundle.
 *
 * Registers the `archol` file type in the right Sidebar document preview and
 * renders it with token colours, an artifact outline and the file's
 * diagnostics. The source itself comes from the document owner (the same paged
 * text the plain-text preview reads); only the compiler payload arrives through
 * the Host route, so without a compiler the file still reads as plain source.
 * Outline and diagnostic clicks go through the Sidebar's own line navigation,
 * so a declaration on a page that is not loaded yet still lands correctly.
 */

window.__ModuleLoader__.load({
  id: '@local/archol-preview',
  factory(require) {
    const React = require('react')
    const h = React.createElement
    const { useCallback, useEffect, useMemo, useRef, useState } = React

    /** Renderer identity; the document slot key is the same string. */
    const ID = '@local/archol-preview/archol'
    /** Host route served by `index.js`; keep both values in sync. */
    const ROUTE = '/archol-preview/view'
    /** Copy namespace registered into the Client locale service. */
    const NS = 'archolPreview'
    /** Grammar prefix of a `file` resource address. */
    const ADDRESS_PREFIX = 'dsh-resource://file/'

    /** English dictionary, also the fallback when the locale service is absent. */
    const en = {
      'viewer.title': 'Archol',
      'outline.title': 'Structure',
      'outline.close': 'Hide structure',
      'outline.empty': 'No top-level artifacts.',
      'outline.unavailable': 'Outline unavailable.',
      'analysis.loading': 'Analyzing with the archol compiler…',
      'analysis.loading.short': 'archol…',
      'analysis.failed': 'Plain source: the archol compiler is unavailable.',
      'analysis.error': 'Plain source: the analysis failed.',
      'diagnostics.one': 'diagnostic',
      'diagnostics.other': 'diagnostics',
      'diagnostics.toggle': 'Toggle diagnostics',
      'pending.label': 'pending comments',
      'application.label': 'Application',
      'retry': 'Retry',
      'empty.file': 'This file is empty.',
    }

    /** Brazilian Portuguese dictionary. */
    const pt = {
      'viewer.title': 'Archol',
      'outline.title': 'Estrutura',
      'outline.close': 'Ocultar estrutura',
      'outline.empty': 'Nenhum artefato de topo.',
      'outline.unavailable': 'Estrutura indisponível.',
      'analysis.loading': 'Analisando com o compilador archol…',
      'analysis.loading.short': 'archol…',
      'analysis.failed': 'Texto puro: o compilador archol não está disponível.',
      'analysis.error': 'Texto puro: a análise falhou.',
      'diagnostics.one': 'diagnóstico',
      'diagnostics.other': 'diagnósticos',
      'diagnostics.toggle': 'Alternar diagnósticos',
      'pending.label': 'comentários pendentes',
      'application.label': 'Aplicação',
      'retry': 'Tentar novamente',
      'empty.file': 'Este arquivo está vazio.',
    }

    /** Fallback copy for a slot rendered without the locale seat. */
    const fallback = key => en[key] ?? key

    /**
     * Read the session and path out of a `dsh-resource://file/session/…` address.
     * @param {unknown} address - the document owner's resource address.
     * @returns {{sessionId: string, path: string} | undefined} the address parts.
     */
    function parseFileAddress(address) {
      if (typeof address !== 'string' || !address.startsWith(ADDRESS_PREFIX)) return undefined
      const end = address.search(/[?#]/u)
      const [scope, ...rest] = address.slice(ADDRESS_PREFIX.length, end === -1 ? undefined : end).split('/')
      if (scope !== 'session') return undefined
      const [id, ...segments] = rest
      if (id === undefined || id === '' || segments.length === 0) return undefined
      try {
        return { sessionId: decodeURIComponent(id), path: segments.map(decodeURIComponent).join('/') }
      } catch {
        return undefined
      }
    }

    /**
     * UTF-8 byte length of one Unicode code point.
     * @param {number} cp - code point.
     * @returns {number} bytes the code point occupies in UTF-8.
     */
    function utf8Length(cp) {
      return cp < 0x80 ? 1 : cp < 0x800 ? 2 : cp < 0x10000 ? 3 : 4
    }

    /**
     * Map the compiler's byte spans onto JS string indices of the loaded text.
     * A token is dropped when the loaded page ends inside it.
     * @param {string} text - loaded source text.
     * @param {readonly {start: number, end: number, class: string}[]} tokens - compiler tokens.
     * @returns {{cls: string, start: number, end: number}[]} string-index spans.
     */
    function mapTokens(text, tokens) {
      const spans = []
      let byte = 0
      let index = 0
      /** Advance the cursor to a UTF-8 byte offset. @param {number} target - byte offset. */
      const advanceTo = target => {
        while (byte < target && index < text.length) {
          const cp = text.codePointAt(index)
          const size = cp > 0xffff ? 2 : 1
          byte += utf8Length(cp)
          index += size
        }
      }
      for (const token of tokens) {
        if (token.end <= byte) continue
        advanceTo(token.start)
        if (byte !== token.start) break
        const start = index
        advanceTo(token.end)
        if (byte !== token.end) break
        if (index > start) spans.push({ cls: token.class, start, end: index })
      }
      return spans
    }

    /**
     * Split the loaded text into lines of `[class, text]` segments.
     * @param {string} text - loaded source text.
     * @param {{cls: string, start: number, end: number}[]} spans - string-index token spans.
     * @returns {{cls: string, text: string}[][]} one segment array per source line.
     */
    function toLines(text, spans) {
      const lines = [[]]
      /** Append one chunk, starting a new line at every `\n`. */
      const push = (from, to, cls) => {
        const chunk = text.slice(from, to)
        let start = 0
        for (let i = 0; i < chunk.length; i += 1) {
          if (chunk.charCodeAt(i) !== 10) continue
          if (i > start) lines[lines.length - 1].push({ cls, text: chunk.slice(start, i) })
          lines.push([])
          start = i + 1
        }
        if (start < chunk.length) lines[lines.length - 1].push({ cls, text: chunk.slice(start) })
      }
      let cursor = 0
      for (const span of spans) {
        if (span.start > cursor) push(cursor, span.start, '')
        push(span.start, span.end, span.cls)
        cursor = span.end
      }
      if (cursor < text.length) push(cursor, text.length, '')
      return lines
    }

    /** Component styles; theme tokens only, so both themes follow the host. */
    const CSS = `
      .archol-root {
        min-height: 100%;
        color: var(--dsw-alias-label-primary);
        background: var(--dsw-alias-bg-base);
      }
      .archol-bar {
        position: sticky;
        top: 0;
        z-index: 2;
        display: flex;
        align-items: center;
        gap: 8px;
        padding: 5px 10px;
        border-bottom: 1px solid var(--dsw-alias-border-l1);
        background: var(--dsw-alias-bg-layer-1);
      }
      .archol-chip {
        display: inline-flex;
        align-items: baseline;
        gap: 6px;
        padding: 2px 9px;
        border: 1px solid var(--dsw-alias-border-l1);
        border-radius: 999px;
        background: transparent;
        color: var(--dsw-alias-label-secondary);
        font: inherit;
        font-size: 11.5px;
        cursor: pointer;
      }
      .archol-chip:hover { color: var(--dsw-alias-label-primary); background: var(--dsw-alias-bg-layer-2); }
      .archol-chip[aria-expanded="true"] { color: var(--dsw-alias-label-primary); }
      .archol-chip-count { color: var(--dsw-alias-label-secondary); font-variant-numeric: tabular-nums; }
      .archol-status { font-size: 11px; color: var(--dsw-alias-label-secondary); }
      .archol-diags {
        margin: 10px 14px 0;
        border: 1px solid var(--dsw-alias-border-l1);
        border-radius: 6px;
        overflow: auto;
        max-height: 200px;
        background: var(--dsw-alias-bg-layer-1);
      }
      .archol-diag {
        display: flex;
        gap: 8px;
        align-items: baseline;
        width: 100%;
        padding: 5px 9px;
        border: 0;
        border-bottom: 1px solid var(--dsw-alias-border-l1);
        background: transparent;
        color: var(--dsw-alias-label-primary);
        font: inherit;
        font-size: 12px;
        text-align: left;
        cursor: pointer;
      }
      .archol-diag:last-child { border-bottom: 0; }
      .archol-diag:hover { background: var(--dsw-alias-bg-layer-2); }
      .archol-diag-pos { flex: 0 0 auto; font-variant-numeric: tabular-nums; color: var(--dsw-alias-label-secondary); }
      .archol-diag-msg { flex: 1 1 auto; }
      .archol-diag-error .archol-diag-pos { color: var(--dsw-alias-state-error-primary); }
      .archol-diag-warning .archol-diag-pos { color: var(--dsw-alias-state-warn-primary); }
      .archol-note {
        display: flex;
        gap: 8px;
        align-items: baseline;
        margin: 10px 14px 0;
        padding: 6px 10px;
        border-radius: 6px;
        background: var(--dsw-alias-bg-layer-2);
        color: var(--dsw-alias-label-secondary);
        font-size: 12px;
      }
      .archol-note-error { background: color-mix(in srgb, var(--dsw-alias-state-error-primary) 12%, var(--dsw-alias-bg-layer-2)); }
      .archol-note-detail {
        flex: 1 1 auto;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
        color: var(--dsw-alias-state-error-primary);
      }
      .archol-retry {
        flex: 0 0 auto;
        padding: 2px 8px;
        border: 1px solid var(--dsw-alias-border-l2);
        border-radius: 5px;
        background: transparent;
        color: var(--dsw-alias-label-primary);
        font: inherit;
        font-size: 11.5px;
        cursor: pointer;
      }
      .archol-retry:hover { background: var(--dsw-alias-bg-layer-1); }
      .archol-outline {
        display: flex;
        flex-wrap: wrap;
        gap: 4px;
        padding: 8px 10px;
        border-bottom: 1px solid var(--dsw-alias-border-l1);
        background: var(--dsw-alias-bg-layer-1);
        max-height: 168px;
        overflow: auto;
      }
      .archol-artifact {
        display: inline-flex;
        align-items: baseline;
        gap: 6px;
        max-width: 100%;
        padding: 2px 9px;
        border: 1px solid var(--dsw-alias-border-l1);
        border-radius: 999px;
        background: transparent;
        color: var(--dsw-alias-label-primary);
        font: inherit;
        font-size: 11.5px;
        cursor: pointer;
      }
      .archol-artifact:hover { background: var(--dsw-alias-bg-layer-2); }
      .archol-artifact-kind { flex: 0 0 auto; font-size: 10px; text-transform: uppercase; letter-spacing: .05em; color: var(--dsw-alias-brand-primary); }
      .archol-artifact-name { min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
      .archol-artifact-line { flex: 0 0 auto; font-size: 10.5px; color: var(--dsw-alias-label-secondary); font-variant-numeric: tabular-nums; }
      .archol-outline-foot { flex-basis: 100%; margin-top: 2px; font-size: 11px; color: var(--dsw-alias-label-secondary); overflow-wrap: anywhere; }
      .archol-outline-empty { padding: 2px 0; font-size: 12px; color: var(--dsw-alias-label-secondary); }
      .archol-empty { padding: 24px 16px; color: var(--dsw-alias-label-secondary); font-size: 13px; }
      .archol-code {
        padding: 10px 0 18px;
        font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
        font-size: 12.5px;
        line-height: 1.55;
        tab-size: 2;
      }
      .archol-line { display: flex; min-height: 1.55em; }
      .archol-line:hover { background: var(--dsw-alias-bg-layer-1); }
      .archol-num {
        flex: 0 0 auto;
        min-width: 3.4em;
        padding-right: 1em;
        text-align: right;
        color: var(--dsw-alias-label-secondary);
        opacity: .65;
        user-select: none;
        font-variant-numeric: tabular-nums;
      }
      .archol-line-text { flex: 1 1 auto; white-space: pre; padding-right: 16px; }
      .archol-wrap .archol-line-text { white-space: pre-wrap; word-break: break-word; }
      .archol-tok-keyword { color: var(--dsw-alias-brand-primary); font-weight: 600; }
      .archol-tok-type { color: var(--dsw-alias-brand-primary); }
      .archol-tok-string { color: var(--dsw-alias-state-success-primary); }
      .archol-tok-number { color: var(--dsw-alias-state-warn-primary); }
      .archol-tok-comment { color: var(--dsw-alias-label-secondary); font-style: italic; }
      .archol-tok-operator, .archol-tok-punctuation { color: var(--dsw-alias-label-secondary); }
      .archol-tok-error { color: var(--dsw-alias-state-error-primary); text-decoration: underline wavy; }
    `

    /** Memoized source lines: a Sidebar scroll re-render must not rebuild every row. */
    const SourceLines = React.memo(function SourceLines({ lines }) {
      return h(
        'div',
        { className: 'archol-code', 'data-archol-code': '' },
        lines.map((segments, index) => {
          const line = index + 1
          return h(
            'div',
            { key: line, className: 'archol-line', 'data-line': line, 'data-textpreview-line': line },
            h('span', { className: 'archol-num', 'aria-hidden': true }, line),
            h(
              'span',
              { className: 'archol-line-text' },
              segments.map((segment, segmentIndex) => h(
                'span',
                {
                  key: segmentIndex,
                  className: segment.cls === '' ? undefined : `archol-tok-${segment.cls}`,
                },
                segment.text,
              )),
            ),
          )
        }),
      )
    })

    /**
     * Build the document body bound to the Client Remote.
     * @param {(sessionId: string, path: string, signal: AbortSignal) => Promise<{ok: boolean, value?: {absolutePath: string}, error?: {message: string}}>} statFile
     * @returns {function} the React body component.
     */
    function makeBody(statFile) {
      /**
       * @param {object} props - document body props from the preview owner.
       * @returns {import('react').ReactNode} the highlighted source and outline.
       */
      return function ArcholBody(props) {
        const { resourceAddress, content, wrap, t: translate, useTabInfo } = props
        const t = typeof translate === 'function' ? translate : fallback
        const tabInfo = typeof useTabInfo === 'function' ? useTabInfo() : undefined
        const openResource = tabInfo?.tab?.actions?.openResource
        const hasContent = content !== undefined && content.kind === 'text'
        const text = hasContent ? content.text : ''
        const address = useMemo(() => parseFileAddress(resourceAddress), [resourceAddress])
        const [view, setView] = useState({ phase: 'idle' })
        const [showOutline, setShowOutline] = useState(false)
        const [showDiagnostics, setShowDiagnostics] = useState(true)
        const [retry, setRetry] = useState(0)
        const codeRef = useRef(null)

        useEffect(() => {
          if (!hasContent) return undefined
          setView(previous => (previous.phase === 'ready' ? previous : { phase: 'loading' }))
          if (address === undefined) {
            setView({ phase: 'error', unavailable: false, message: `Unsupported file address: ${String(resourceAddress)}` })
            return undefined
          }
          const controller = new AbortController()
          const timer = setTimeout(() => {
            if (controller.signal.aborted) return
            const load = async () => {
              const stat = await statFile(address.sessionId, address.path, controller.signal)
              if (controller.signal.aborted) return
              if (!stat.ok) throw new Error(stat.error?.message ?? 'could not resolve the file path')
              const url = `${ROUTE}?path=${encodeURIComponent(stat.value.absolutePath)}`
              const response = await fetch(url, { signal: controller.signal, headers: { accept: 'application/json' } })
              const body = await response.json().catch(() => undefined)
              if (!response.ok) {
                const error = new Error(body?.message ?? `HTTP ${response.status}`)
                error.unavailable = response.status === 503
                throw error
              }
              if (body === undefined || body.kind !== 'ArcholView') throw new Error('unexpected compiler payload')
              setView({ phase: 'ready', payload: body })
            }
            load().catch(error => {
              if (controller.signal.aborted) return
              setView({
                phase: 'error',
                unavailable: error?.unavailable === true,
                message: error instanceof Error ? error.message : String(error),
              })
            })
          }, 300)
          return () => {
            clearTimeout(timer)
            controller.abort()
          }
        }, [address?.sessionId, address?.path, hasContent, text, retry])

        const lines = useMemo(() => {
          const tokens = view.phase === 'ready' ? view.payload.tokens : []
          return toLines(text, mapTokens(text, Array.isArray(tokens) ? tokens : []))
        }, [text, view])

        /**
         * Ask the Sidebar to reveal a source line; the owner then loads pages,
         * scrolls and marks the line. Local scroll is only the no-owner fallback.
         */
        const jumpToLine = useCallback(line => {
          if (typeof openResource === 'function') {
            openResource(resourceAddress, { params: { line } })
            return
          }
          const element = codeRef.current?.querySelector(`[data-line="${line}"]`)
          if (element !== undefined) element.scrollIntoView({ block: 'center' })
        }, [openResource, resourceAddress])

        if (!hasContent) return null

        const payload = view.phase === 'ready' ? view.payload : undefined
        const diagnostics = payload !== undefined && Array.isArray(payload.diagnostics) ? payload.diagnostics : []
        const outline = payload !== undefined && Array.isArray(payload.outline) ? payload.outline : []
        const application = payload?.application
        const version = payload?.version
        const pending = typeof payload?.pending === 'number' ? payload.pending : 0
        const unavailable = view.phase === 'error' && view.unavailable === true

        return h(
          'div',
          { className: `archol-root${wrap ? ' archol-wrap' : ''}`, 'data-archol-preview': '' },
          h('style', { key: 'style' }, CSS),
          h(
            'div',
            { className: 'archol-bar', key: 'bar' },
            h(
              'button',
              {
                type: 'button',
                className: 'archol-chip',
                'aria-expanded': showOutline,
                'aria-controls': 'archol-outline',
                title: showOutline ? t('outline.close') : t('outline.title'),
                onClick: () => setShowOutline(value => !value),
              },
              t('outline.title'),
              h('span', { className: 'archol-chip-count' }, outline.length),
            ),
            diagnostics.length > 0 && h(
              'button',
              {
                type: 'button',
                className: 'archol-chip',
                'aria-expanded': showDiagnostics,
                'aria-controls': 'archol-diagnostics',
                title: t('diagnostics.toggle'),
                onClick: () => setShowDiagnostics(value => !value),
              },
              diagnostics.length,
              ' ',
              t(diagnostics.length === 1 ? 'diagnostics.one' : 'diagnostics.other'),
            ),
            view.phase === 'loading' && h('span', { className: 'archol-status', role: 'status', 'aria-live': 'polite' }, t('analysis.loading.short')),
          ),
          view.phase === 'error' && h(
            'div',
            { className: 'archol-note archol-note-error', 'data-archol-note': 'error', role: 'alert' },
            h('span', null, unavailable ? t('analysis.failed') : t('analysis.error')),
            h('span', { className: 'archol-note-detail', title: view.message }, view.message),
            h(
              'button',
              { type: 'button', className: 'archol-retry', onClick: () => setRetry(value => value + 1) },
              t('retry'),
            ),
          ),
          showDiagnostics && diagnostics.length > 0 && h(
            'div',
            { className: 'archol-diags', id: 'archol-diagnostics', 'data-archol-diagnostics': diagnostics.length },
            diagnostics.map((diagnostic, index) => h(
              'button',
              {
                key: index,
                type: 'button',
                className: `archol-diag archol-diag-${diagnostic.severity ?? 'error'}`,
                'aria-label': `${diagnostic.severity ?? 'error'} ${diagnostic.code ?? ''} at line ${diagnostic.line}: ${diagnostic.message}`,
                title: `${diagnostic.code ?? ''} (${diagnostic.severity ?? 'error'})`,
                onClick: () => jumpToLine(diagnostic.line),
              },
              h('span', { className: 'archol-diag-pos' }, `${diagnostic.line}:${diagnostic.column} ${diagnostic.severity ?? 'error'}`),
              h('span', { className: 'archol-diag-msg' }, diagnostic.message),
            )),
          ),
          showOutline && h(
            'div',
            { className: 'archol-outline', key: 'outline', id: 'archol-outline', 'data-archol-outline': '' },
            outline.length === 0
              ? h(
                'span',
                { className: 'archol-outline-empty' },
                view.phase === 'loading'
                  ? t('analysis.loading')
                  : view.phase === 'error' ? t('outline.unavailable') : t('outline.empty'),
              )
              : outline.map((artifact, index) => h(
                'button',
                {
                  key: index,
                  type: 'button',
                  className: 'archol-artifact',
                  title: `${artifact.kind} · ${artifact.line}:${artifact.column}`,
                  onClick: () => jumpToLine(artifact.line),
                },
                h('span', { className: 'archol-artifact-kind' }, artifact.kind),
                h('span', { className: 'archol-artifact-name' }, artifact.name),
                h('span', { className: 'archol-artifact-line' }, artifact.line),
              )),
            application !== undefined && h(
              'span',
              { className: 'archol-outline-foot' },
              `${t('application.label')}: ${application.uri}`,
            ),
            pending > 0 && h('span', { className: 'archol-outline-foot' }, `${pending} ${t('pending.label')}`),
            version !== undefined && h('span', { className: 'archol-outline-foot' }, `archol ${version}`),
          ),
          text.length === 0
            ? h('div', { className: 'archol-empty', key: 'empty' }, t('empty.file'))
            : h('div', { key: 'code', ref: codeRef }, h(SourceLines, { lines })),
        )
      }
    }

    /** Required browser services: slots, copy, the preview registry and the file Remote. */
    const inject = ['slots', 'locale', 'documentPreviews', 'remote', 'remote.workspaceFiles']

    /**
     * @param {object} ctx - Client root context.
     */
    function apply(ctx) {
      ctx.effect(() => ctx.locale.register(NS, { en, pt }), 'archol-preview: dictionaries')
      const t = ctx.locale.bind(NS)
      ctx.effect(() => ctx.documentPreviews.register({
        id: ID,
        extensions: ['archol'],
        priority: 'extension',
        title: () => t('viewer.title'),
        loading: 'text-pages',
        wrap: true,
        sourceLines: true,
      }), 'archol-preview: archol file type')
      const body = makeBody((sessionId, path, signal) => ctx.remote.workspaceFiles.stat(sessionId, path, signal))
      ctx.effect(() => ctx.slots.inject('sidebar.right.tab.document', () => ctx.slots.register(
        {
          name: 'sidebar.right.tab.document',
          key: ID,
          locale: NS,
          inject: { hooks: { tabInfo: (_standard, useTabInfo) => useTabInfo } },
        },
        body,
      )), 'archol-preview: archol document body')
    }

    return { inject, apply }
  },
})
