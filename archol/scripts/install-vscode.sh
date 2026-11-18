#!/usr/bin/env bash
# Installs the Archol plugin in VS Code and ensures the LSP binary is built.
#
# Usage: bash archol/scripts/install-vscode.sh
# Uninstall: rm ~/.vscode/extensions/archol.archol-vscode-0.1.0
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
ARCHOL="$ROOT/archol"
EXT_SRC="$ARCHOL/editors/vscode"
EXTENSIONS_DIR="${VSCODE_EXTENSIONS_DIR:-$HOME/.vscode/extensions}"
EXT_TARGET="$EXTENSIONS_DIR/archol.archol-vscode-0.1.0"
BIN="$ARCHOL/target/release/archol"

case "$EXT_TARGET" in
  "$EXTENSIONS_DIR"/archol.archol-vscode-*) ;;
  *) echo "Unexpected destination: $EXT_TARGET" >&2; exit 1 ;;
esac

if ! command -v cargo >/dev/null 2>&1; then
  if [ -x "$HOME/.cargo/bin/cargo" ]; then
    export PATH="$HOME/.cargo/bin:$PATH"
  else
    echo "Rust/cargo not found. Install rustup: https://rustup.rs" >&2
    exit 1
  fi
fi

# In environments with a read-only HOME, use a temporary CARGO_HOME.
if [ -z "${CARGO_HOME:-}" ] && ! ( mkdir -p "$HOME/.cargo/registry" 2>/dev/null && : > "$HOME/.cargo/registry/.archol-write-test" 2>/dev/null ); then
  export CARGO_HOME="${TMPDIR:-/tmp}/archol-cargo-home"
  echo "==> HOME is not writable at ~/.cargo; using CARGO_HOME=$CARGO_HOME"
else
  rm -f "$HOME/.cargo/registry/.archol-write-test" 2>/dev/null || true
fi

echo "==> Building the LSP server (release)"
(cd "$ARCHOL" && cargo build --release)
test -x "$BIN"

echo "==> Installing the extension into $EXT_TARGET"
mkdir -p "$EXTENSIONS_DIR"
rm -rf -- "$EXT_TARGET"
ln -s "$EXT_SRC" "$EXT_TARGET"

# Exposes the binary as `archol` when possible.
if [ -d "$HOME/.local/bin" ]; then
  ln -sfn "$BIN" "$HOME/.local/bin/archol" 2>/dev/null || true
fi

cat <<MSG

Done.
  extension: $EXT_TARGET
  binary   : $BIN

In VS Code:
  1. Reload the window (Command Palette -> "Developer: Reload Window").
  2. Open this repository folder (the server is found at archol/target/release/archol).
  3. Open any .archol file; the status bar shows "Archol: ready".
  4. Command Palette -> "Archol: Open AST Explorer" (or Ctrl+Alt+A / Cmd+Alt+A).

If you open ANOTHER folder (outside the repository), point to the server once:
  "archol.server.path": "$BIN"
  "archol.server.args": ["lsp"]
MSG
