#!/usr/bin/env bash
# Dogfood RTK on this repo (build, init, index, doctor, validate, e2e smoke).
set -euo pipefail
source "${HOME}/.cargo/env" 2>/dev/null || true

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

echo "== build =="
cargo build --manifest-path "$ROOT/rtk/Cargo.toml" -p rtk-context-engine

RTK="$ROOT/rtk/target/debug/rtk"
export RTK
export PATH="$(dirname "$RTK"):$PATH"

echo "== version =="
"$RTK" --version

echo "== init (profile high) =="
"$RTK" init --profile high

echo "== index =="
"$RTK" index run || echo "warn: index run failed (non-fatal for smoke)"

echo "== doctor =="
set +e
"$RTK" doctor
doc=$?
set -e
echo "doctor exit: $doc (0=ok, 2=warnings ok for dev)"

echo "== validate =="
"$RTK" validate

echo "== e2e smoke =="
bash "$ROOT/scripts/e2e_smoke.sh"

echo ""
echo "Dogfood OK. For daily dev in this shell:"
echo "  source scripts/rtk-dev.env.sh"
echo "  rtk git status"
echo "  rtk cargo test --manifest-path rtk/Cargo.toml --workspace"
