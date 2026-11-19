/**
 * Host half of the Archol preview bundle.
 *
 * The Web Client cannot run the compiler, so this route turns one `.archol`
 * path into the compact payload `archol view --json` already produces. It never
 * writes files: it locates a trusted compiler, optionally picks an application
 * whose closure contains the file (semantic diagnostics and app context), and
 * runs `view` with bounded time and output. Every failure is reported as JSON
 * so the Client can keep showing plain source.
 */

import { execFile } from 'node:child_process'
import { existsSync } from 'node:fs'
import { stat } from 'node:fs/promises'
import { dirname, extname, isAbsolute, join } from 'node:path'
import { fileURLToPath } from 'node:url'

/** Route the Client calls; keep in sync with `client.js`. */
const ROUTE = '/archol-preview/view'
/** Refuse files larger than this before running the compiler. */
const MAX_FILE_BYTES = 2 * 1024 * 1024
/** Bound one compiler run, including output buffering. */
const COMPILER_TIMEOUT_MS = 10_000
const COMPILER_MAX_OUTPUT_BYTES = 8 * 1024 * 1024
/** Application selector accepted by `archol view --app`: a URI or an absolute path. */
const APPLICATION_PATTERN = /^[A-Za-z0-9/._-]+$/u
/** How many applications one workspace may be probed with before falling back to syntax. */
const MAX_APPLICATION_PROBES = 12

/** Directory of this plugin, used to find a compiler built beside it. */
const PLUGIN_DIR = dirname(fileURLToPath(import.meta.url))

/** Required Host service: the HTTP carrier that serves this route. */
export const inject = ['webServer']

/**
 * @param {object} ctx - Host context carrying `webServer`.
 */
export function apply(ctx) {
  /** Applications discovered per workspace root. */
  const applicationsByRoot = new Map()
  /** Application whose closure accepted one file, keyed by absolute path. */
  const applicationByFile = new Map()

  /**
   * Answer one preview request.
   * @param {import('node:http').IncomingMessage} req
   * @param {import('node:http').ServerResponse} res
   */
  const handler = async (req, res) => {
    if (req.method !== 'GET' && req.method !== 'HEAD') {
      sendJson(res, 405, { error: 'method-not-allowed', message: 'use GET' })
      return
    }
    let url
    try {
      url = new URL(String(req.url), 'http://localhost')
    } catch {
      sendJson(res, 400, { error: 'bad-request', message: 'malformed URL' })
      return
    }
    const target = url.searchParams.get('path') ?? ''
    const requestedApp = url.searchParams.get('app') ?? ''
    if (target === '' || !isAbsolute(target)) {
      sendJson(res, 400, { error: 'bad-request', message: 'path must be an absolute file path' })
      return
    }
    if (extname(target).toLowerCase() !== '.archol') {
      sendJson(res, 400, { error: 'bad-request', message: 'only .archol files are previewed' })
      return
    }
    if (requestedApp !== '' && !APPLICATION_PATTERN.test(requestedApp)) {
      sendJson(res, 400, { error: 'bad-request', message: 'invalid app selector' })
      return
    }
    let info
    try {
      info = await stat(target)
    } catch {
      sendJson(res, 404, { error: 'not-found', message: `no file at ${target}` })
      return
    }
    if (!info.isFile()) {
      sendJson(res, 400, { error: 'bad-request', message: `${target} is not a regular file` })
      return
    }
    if (info.size > MAX_FILE_BYTES) {
      sendJson(res, 413, { error: 'too-large', message: `file exceeds ${MAX_FILE_BYTES} bytes` })
      return
    }

    const outcome = await preview(target, requestedApp, applicationsByRoot, applicationByFile)
    if (outcome.kind === 'missing') {
      sendJson(res, 503, {
        error: 'compiler-not-found',
        message: 'Could not find the archol compiler. Build it with `cargo build --release` in archol/ or set ARCHOL_COMPILER.',
      })
      return
    }
    if (outcome.kind === 'failed') {
      sendJson(res, 500, { error: 'compiler-failed', message: outcome.message })
      return
    }
    let payload
    try {
      payload = JSON.parse(outcome.stdout)
    } catch {
      sendJson(res, 502, { error: 'compiler-output', message: 'compiler did not return JSON' })
      return
    }
    sendJson(res, 200, payload)
  }

  ctx.effect(
    () => ctx.webServer.register({ kind: 'exact', path: ROUTE, handler }),
    `archol-preview: GET ${ROUTE}`,
  )
}

/**
 * Run `view` with the most specific application that accepts the file, falling
 * back to the syntax-only payload when no application does.
 * @param {string} target - absolute `.archol` file.
 * @param {string} requestedApp - application from the query, or empty.
 * @param {Map<string, string[]>} applicationsByRoot - per-root application cache.
 * @param {Map<string, string>} applicationByFile - per-file application cache.
 * @returns {Promise<{kind: 'ok', stdout: string} | {kind: 'missing'} | {kind: 'failed', message: string}>}
 */
async function preview(target, requestedApp, applicationsByRoot, applicationByFile) {
  const apps = []
  if (requestedApp !== '') {
    apps.push(requestedApp)
  } else {
    const remembered = applicationByFile.get(target)
    if (remembered !== undefined) apps.push(remembered)
    const configured = process.env.ARCHOL_APPLICATION
    if (configured !== undefined && configured !== '') apps.push(configured)
    for (const root of workspaceRoots(target)) {
      const discovered = await applicationsIn(root, applicationsByRoot)
      apps.push(...discovered)
    }
  }
  const tried = new Set()
  for (const app of apps.slice(0, MAX_APPLICATION_PROBES)) {
    if (app === '' || tried.has(app)) continue
    tried.add(app)
    const outcome = await runCompiler(target, ['view', target, '--json', '--app', app])
    if (outcome.kind === 'ok') {
      applicationByFile.set(target, app)
      return outcome
    }
    if (outcome.kind === 'missing') return outcome
    // A file outside this closure or an invalid selector: try the next app.
  }
  return runCompiler(target, ['view', target, '--json'])
}

/**
 * List application URIs discovered in one workspace root.
 * @param {string} root - workspace root candidate.
 * @param {Map<string, string[]>} cache - per-root cache.
 * @returns {Promise<string[]>} application URIs in compiler order.
 */
async function applicationsIn(root, cache) {
  const cached = cache.get(root)
  if (cached !== undefined) return cached
  const outcome = await runCompiler(root, ['applications', root, '--json'], root)
  if (outcome.kind !== 'ok') return []
  try {
    const list = JSON.parse(outcome.stdout)
    const uris = Array.isArray(list)
      ? list.map(entry => entry?.uri).filter(uri => typeof uri === 'string' && uri !== '')
      : []
    cache.set(root, uris)
    return uris
  } catch {
    return []
  }
}

/**
 * Workspace roots above a file, nearest first: directories that carry both
 * `applications/` and `services/`, the layout the compiler discovers.
 * @param {string} target - absolute file path.
 * @returns {string[]} candidate roots.
 */
function workspaceRoots(target) {
  const roots = []
  let dir = dirname(target)
  for (;;) {
    if (existsSync(join(dir, 'applications')) && existsSync(join(dir, 'services'))) roots.push(dir)
    const parent = dirname(dir)
    if (parent === dir) break
    dir = parent
  }
  return roots
}

/**
 * Locate a trusted compiler: configured, built beside this plugin, or on PATH.
 * It never executes a binary found under the previewed workspace.
 * @param {string} _target - absolute file being previewed (kept for call symmetry).
 * @param {string[]} args - compiler arguments.
 * @param {string} cwd - directory the compiler resolves relative paths from.
 * @returns {Promise<{kind: 'ok', stdout: string} | {kind: 'missing'} | {kind: 'failed', message: string}>}
 */
async function runCompiler(_target, args, cwd) {
  const failures = []
  for (const candidate of compilerCandidates()) {
    if (candidate.includes('/') && !existsSync(candidate)) continue
    try {
      return { kind: 'ok', stdout: await execCompiler(candidate, args, cwd) }
    } catch (error) {
      const stderr = typeof error?.stderr === 'string' ? error.stderr : ''
      if (error?.code === 'ENOENT') continue
      // An older binary without `view`: try the next candidate, no permanent poison.
      if (stderr.includes('unknown command: view')) continue
      failures.push(`${candidate}: ${stderr.trim() || error?.message || 'compiler failed'}`)
    }
  }
  if (failures.length > 0) return { kind: 'failed', message: failures.join('; ') }
  return { kind: 'missing' }
}

/**
 * Compiler candidates, in trust order.
 * @returns {string[]} executable paths or names.
 */
function compilerCandidates() {
  const candidates = []
  if (process.env.ARCHOL_COMPILER) candidates.push(process.env.ARCHOL_COMPILER)
  // This bundle lives in `archol/editors/dsh`; the crate's target dir sits two levels up.
  candidates.push(
    join(PLUGIN_DIR, '..', '..', 'target', 'release', 'archol'),
    join(PLUGIN_DIR, '..', '..', 'target', 'debug', 'archol'),
  )
  candidates.push('archol')
  return [...new Set(candidates)]
}

/**
 * Run one compiler process with bounded time and captured output.
 * @param {string} command - executable path or name.
 * @param {string[]} args - arguments.
 * @param {string} cwd - directory the compiler resolves relative paths from.
 * @returns {Promise<string>} stdout.
 */
function execCompiler(command, args, cwd) {
  return new Promise((resolve, reject) => {
    execFile(
      command,
      args,
      { cwd, timeout: COMPILER_TIMEOUT_MS, maxBuffer: COMPILER_MAX_OUTPUT_BYTES, windowsHide: true },
      (error, stdout) => {
        if (error) reject(error)
        else resolve(stdout)
      },
    )
  })
}

/**
 * Write one JSON response without caching, since the source changes on disk.
 * @param {import('node:http').ServerResponse} res
 * @param {number} status
 * @param {unknown} body
 */
function sendJson(res, status, body) {
  const text = JSON.stringify(body)
  res.statusCode = status
  res.setHeader('content-type', 'application/json; charset=utf-8')
  res.setHeader('cache-control', 'no-store')
  res.setHeader('content-length', Buffer.byteLength(text))
  res.end(text)
}
