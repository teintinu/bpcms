#!/usr/bin/env bash
# Regenerates the AST snapshots from the current test output.
#
# Usage:
#   bash archol/scripts/test-update-snapshots.sh
#
# One family is updated:
#   - application: `<app>.result-ast.yml`     -> `<app>.expected-ast.yml`
#                  (resolved `ArcholApplication` from `archol application`).
#
# The snapshot tests write the result files before comparing them. This script
# removes stale results, runs both snapshot tests (differences are expected to
# fail), copies every result over its expected file, and then runs the full
# suite to confirm everything is green.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
ARCHOL="$ROOT/archol"
APPLICATIONS="$ROOT/examples/applications"

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

echo "==> Removing stale result snapshots"
find "$APPLICATIONS" -name '*.result-*.yml' -delete

for test in application_snapshots; do
  echo "==> Running $test (differences are expected)"
  if cargo test -p archol-cli --test "$test"; then
    echo "==> $test already matched; refreshing its expected files anyway"
  else
    echo "==> $test reported differences"
  fi
done

app_count="$(find "$APPLICATIONS" -name '*-application.archol' | wc -l | tr -d ' ')"

update_family() {
  local family="$1"
  local results=()
  while IFS= read -r file; do
    results+=("$file")
  done < <(find "$APPLICATIONS" -name "*.result-$family.yml" | sort)

  if [ "${#results[@]}" -eq 0 ]; then
    echo "No $family result snapshots were produced." >&2
    exit 1
  fi
  if [ "${#results[@]}" -ne "$app_count" ]; then
    echo "Expected $app_count $family result snapshots, found ${#results[@]}." >&2
    echo "The test may have failed before writing every result." >&2
    exit 1
  fi
  for result in "${results[@]}"; do
    expected="${result%.result-$family.yml}.expected-$family.yml"
    cp "$result" "$expected"
    echo "updated ${expected#"$ROOT/"}"
  done
}

update_family ast

find "$APPLICATIONS" -name '*.result-*.yml' -delete

echo "==> Running the full test suite"
exec cargo test
