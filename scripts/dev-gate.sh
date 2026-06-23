#!/usr/bin/env bash
set -euo pipefail
source "${HOME}/.cargo/env" 2>/dev/null || true
cd "$(dirname "$0")/../rtk"
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
