#!/usr/bin/env bash
# Instala o plugin Archol no VS Code e garante o binário LSP compilado.
#
# Uso: bash archol/scripts/install-vscode.sh
# Desinstalar: rm ~/.vscode/extensions/archol.archol-vscode-0.1.0
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
ARCHOL="$ROOT/archol"
EXT_SRC="$ARCHOL/editors/vscode"
EXTENSIONS_DIR="${VSCODE_EXTENSIONS_DIR:-$HOME/.vscode/extensions}"
EXT_TARGET="$EXTENSIONS_DIR/archol.archol-vscode-0.1.0"
BIN="$ARCHOL/target/release/archol"

case "$EXT_TARGET" in
  "$EXTENSIONS_DIR"/archol.archol-vscode-*) ;;
  *) echo "Destino inesperado: $EXT_TARGET" >&2; exit 1 ;;
esac

if ! command -v cargo >/dev/null 2>&1; then
  if [ -x "$HOME/.cargo/bin/cargo" ]; then
    export PATH="$HOME/.cargo/bin:$PATH"
  else
    echo "Rust/cargo não encontrado. Instale o rustup: https://rustup.rs" >&2
    exit 1
  fi
fi

# Em ambientes com HOME somente leitura, usa um CARGO_HOME temporário.
if [ -z "${CARGO_HOME:-}" ] && ! ( mkdir -p "$HOME/.cargo/registry" 2>/dev/null && : > "$HOME/.cargo/registry/.archol-write-test" 2>/dev/null ); then
  export CARGO_HOME="${TMPDIR:-/tmp}/archol-cargo-home"
  echo "==> HOME sem escrita em ~/.cargo; usando CARGO_HOME=$CARGO_HOME"
else
  rm -f "$HOME/.cargo/registry/.archol-write-test" 2>/dev/null || true
fi

echo "==> Compilando o servidor LSP (release)"
(cd "$ARCHOL" && cargo build --release)
test -x "$BIN"

echo "==> Instalando a extensão em $EXT_TARGET"
mkdir -p "$EXTENSIONS_DIR"
rm -rf -- "$EXT_TARGET"
ln -s "$EXT_SRC" "$EXT_TARGET"

# Deixa o binário disponível como `archol` quando possível.
if [ -d "$HOME/.local/bin" ]; then
  ln -sfn "$BIN" "$HOME/.local/bin/archol" 2>/dev/null || true
fi

cat <<MSG

Pronto.
  extensão: $EXT_TARGET
  binário : $BIN

No VS Code:
  1. Recarregue a janela (Command Palette -> "Developer: Reload Window").
  2. Abra esta pasta do repositório (o servidor é achado em archol/target/release/archol).
  3. Abra qualquer arquivo .archol; a barra de status mostra "Archol: pronto".
  4. Command Palette -> "Archol: Abrir AST" (ou Ctrl+Alt+A / Cmd+Alt+A).

Se abrir OUTRA pasta (fora do repositório), aponte o servidor uma vez:
  "archol.server.path": "$BIN"
  "archol.server.args": ["lsp"]
MSG
