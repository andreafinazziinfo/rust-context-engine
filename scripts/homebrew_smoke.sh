#!/usr/bin/env bash
# Homebrew formula sanity — license, URLs, version parity with Cargo.toml
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
FORMULA="$ROOT/rtk.rb"
PKG="$ROOT/rtk/rtk-cli/Cargo.toml"

grep -q 'license "Apache-2.0"' "$FORMULA" || {
  echo "homebrew smoke FAIL: rtk.rb license must be Apache-2.0" >&2
  exit 1
}

grep -q 'andreafinazziinfo/rust-context-engine' "$FORMULA" || {
  echo "homebrew smoke FAIL: rtk.rb homepage/url mismatch" >&2
  exit 1
}

for asset in rtk-macos-arm64.tar.gz rtk-macos-amd64.tar.gz rtk-linux-amd64.tar.gz; do
  grep -q "$asset" "$FORMULA" || {
    echo "homebrew smoke FAIL: missing release asset $asset in rtk.rb" >&2
    exit 1
  }
done

grep -q 'PLACEHOLDER' "$FORMULA" && {
  echo "homebrew smoke FAIL: rtk.rb still has PLACEHOLDER sha256" >&2
  exit 1
}

grep -qE '^\s*sha256 "' "$FORMULA" || {
  echo "homebrew smoke FAIL: no sha256 lines in rtk.rb" >&2
  exit 1
}

PKG_VER="$(grep '^version' "$PKG" | head -1 | sed 's/version = "//;s/"//' | tr -d ' ')"
FORMULA_VER="$(grep '^  version "' "$FORMULA" | sed 's/.*"\(.*\)".*/\1/')"
if [ "$PKG_VER" != "$FORMULA_VER" ]; then
  echo "homebrew smoke FAIL: Cargo.toml=$PKG_VER rtk.rb=$FORMULA_VER" >&2
  exit 1
fi

echo "homebrew smoke OK (formula v$FORMULA_VER)"
