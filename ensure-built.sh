#!/usr/bin/env bash
# Lazy Rust compilation helper. Source this, don't execute directly.
#
# Before sourcing, set:
#   PLUGIN_ROOT  — path to the plugin directory containing Cargo.toml
#   BIN_DIR      — path to the compiled binaries (default: $PLUGIN_ROOT/target/release)
#
# Usage:
#   PLUGIN_ROOT="$my_plugin_dir"
#   source "$FORGE_LIB/ensure-built.sh"
#   ensure_built "my-binary"

: "${PLUGIN_ROOT:?ensure-built.sh: PLUGIN_ROOT must be set before sourcing}"
: "${BIN_DIR:=$PLUGIN_ROOT/target/release}"

ensure_built() {
  local binary="$1"
  if [ -x "$BIN_DIR/$binary" ]; then return 0; fi

  local CARGO=""
  if command -v cargo >/dev/null 2>&1; then
    CARGO=cargo
  else
    for candidate in "$HOME/.cargo/bin/cargo" /opt/homebrew/bin/cargo /usr/local/bin/cargo; do
      if [ -x "$candidate" ]; then
        CARGO="$candidate"
        break
      fi
    done
  fi

  if [ -z "$CARGO" ]; then
    echo "$(basename "$PLUGIN_ROOT"): cargo not found — install Rust: https://rustup.rs" >&2
    return 1
  fi

  "$CARGO" build --release --manifest-path "$PLUGIN_ROOT/Cargo.toml" >&2
}
