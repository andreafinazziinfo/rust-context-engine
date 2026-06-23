#!/usr/bin/env bash
# Publish all RTK workspace crates to crates.io (dependency order).
# Prereq: cargo login <token from https://crates.io/settings/tokens>
set -euo pipefail
source "${HOME}/.cargo/env" 2>/dev/null || true

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
RTK="$ROOT/rtk"

# member dir : crates.io name
CRATES=(
  rtk-db:rtk-context-db
  rtk-filters:rtk-context-filters
  rtk-index:rtk-context-index
  rtk-pack:rtk-context-pack
  rtk-mcp:rtk-context-mcp
  rtk-cli:rtk-context-engine
)

DRY="${DRY:-0}"
SKIP_TESTS="${SKIP_TESTS:-0}"
if [ "${1:-}" = "--dry-run" ]; then
  DRY=1
  shift
fi

cd "$RTK"
cargo fmt --check
if [ "$SKIP_TESTS" != "1" ]; then
  cargo test --workspace
fi

# pack/mcp/cli need upstream rtk crates on crates.io for `cargo package`
needs_registry_deps() {
  case "$1" in
    rtk-pack|rtk-mcp|rtk-cli) return 0 ;;
    *) return 1 ;;
  esac
}

publish_one() {
  local dir="$1"
  local attempt
  for attempt in 1 2 3 4 5; do
    if (cd "$RTK/$dir" && cargo publish --allow-dirty "$@"); then
      return 0
    fi
    if [ "$attempt" -lt 5 ]; then
      echo "retry $dir in 60s (crates.io index may lag)..."
      sleep 60
    fi
  done
  return 1
}

for entry in "${CRATES[@]}"; do
  dir="${entry%%:*}"
  name="${entry##*:}"
  echo "== publish $name ($dir) =="

  if [ "$DRY" = "1" ]; then
    if needs_registry_deps "$dir"; then
      echo "skip dry-run: $name needs upstream rtk crates on crates.io first"
      echo "  → run without --dry-run after cargo login"
      continue
    fi
    (cd "$RTK/$dir" && cargo package --allow-dirty "$@")
    continue
  fi

  publish_one "$dir" "$@"
  echo "waiting for index..."
  sleep 45
done

echo "Done. Verify: https://crates.io/crates/rtk-context-engine"
