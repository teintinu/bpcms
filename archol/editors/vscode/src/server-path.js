'use strict';

const { spawn } = require('node:child_process');
const os = require('node:os');
const path = require('node:path');

function expandHome(value) {
  if (value === '~') return os.homedir();
  if (value.startsWith('~/') || value.startsWith('~\\')) {
    return path.join(os.homedir(), value.slice(2));
  }
  return value;
}

function buildServerCandidates(configuredPath, workspaceRoots = [], platform = process.platform) {
  const configured = String(configuredPath || 'archol').trim() || 'archol';
  const roots = [...new Set((workspaceRoots || []).filter(Boolean))];
  const executable = platform === 'win32' ? 'archol.exe' : 'archol';
  const candidates = [];
  const seen = new Set();
  const add = (candidate) => {
    if (!candidate) return;
    const key = platform === 'win32' ? candidate.toLowerCase() : candidate;
    if (seen.has(key)) return;
    seen.add(key);
    candidates.push(candidate);
  };

  const expanded = expandHome(configured);
  if (path.isAbsolute(expanded)) {
    add(expanded);
  } else if (expanded.includes('/') || expanded.includes('\\')) {
    for (const root of roots) add(path.join(root, expanded));
    add(expanded);
  } else {
    add(expanded);
    if (platform === 'win32') add(`${expanded}.exe`);
  }

  for (const root of roots) {
    add(path.join(root, 'archol', 'target', 'release', executable));
    add(path.join(root, 'archol', 'target', 'debug', executable));
  }
  return candidates;
}

function spawnProcess(command, args, cwd, env) {
  return new Promise((resolve, reject) => {
    let child;
    try {
      child = spawn(command, args, {
        cwd,
        env,
        stdio: ['pipe', 'pipe', 'pipe'],
        windowsHide: true,
      });
    } catch (error) {
      reject(error);
      return;
    }
    const cleanup = () => {
      child.removeListener('spawn', onSpawn);
      child.removeListener('error', onError);
    };
    const onSpawn = () => {
      cleanup();
      resolve(child);
    };
    const onError = (error) => {
      cleanup();
      reject(error);
    };
    child.once('spawn', onSpawn);
    child.once('error', onError);
  });
}

module.exports = { buildServerCandidates, expandHome, spawnProcess };
