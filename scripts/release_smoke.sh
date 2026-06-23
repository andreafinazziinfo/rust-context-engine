#!/usr/bin/env bash
# REL-1: release parity — CLI version == Cargo.toml == MCP initialize
set -euo pipefail
source "${HOME}/.cargo/env" 2>/dev/null || true

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
MANIFEST="$ROOT/rtk/Cargo.toml"
PKG="$ROOT/rtk/rtk-cli/Cargo.toml"

EXPECTED="$(grep '^version' "$PKG" | head -1 | sed 's/version = "//;s/"//' | tr -d ' ')"

if [ ! -x "$ROOT/rtk/target/release/rtk" ]; then
  cargo build --release --locked --manifest-path "$MANIFEST"
fi
RTK="$ROOT/rtk/target/release/rtk"

ACTUAL="$("$RTK" --version 2>&1 | awk '{print $2}')"
if [ "$EXPECTED" != "$ACTUAL" ]; then
  echo "REL-1 FAIL: Cargo.toml=$EXPECTED rtk --version=$ACTUAL" >&2
  exit 1
fi

cargo test --manifest-path "$MANIFEST" -p rtk-context-mcp initialize_reports_crate_version -q

echo "REL-1 release smoke OK (v$EXPECTED)"
