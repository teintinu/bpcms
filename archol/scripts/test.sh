#!/usr/bin/env bash
# Runs the Archol test suite.
#
# Usage:
#   bash archol/scripts/test.sh
#   bash archol/scripts/test.sh -p archol-cli --test application_ast
#   bash archol/scripts/test.sh -p archol-cli --test application_ast -- --nocapture
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
ARCHOL="$ROOT/archol"

if ! command -v cargo >/dev/null 2>&1; then
  if [ -x "$HOME/.cargo/bin/cargo" ]; then
    export PATH="$HOME/.cargo/bin:$PATH"
  else
    echo "Rust/cargo not found. Install rustup: https://rustup.rs" >&2
    exit 1
  fi
fi

if [ -z "${CARGO_HOME:-}" ] && ! ( mkdir -p "$HOME/.cargo/registry" && : > "$HOME/.cargo/registry/.archol-write-test" ) 2>/dev/null; then
  export CARGO_HOME="${TMPDIR:-/tmp}/archol-cargo-home"
  echo "==> HOME is not writable at ~/.cargo; using CARGO_HOME=$CARGO_HOME"
else
  rm -f "$HOME/.cargo/registry/.archol-write-test" 2>/dev/null || true
fi

cd "$ARCHOL"
echo "==> cargo test $*"
exec cargo test "$@"
