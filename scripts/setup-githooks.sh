#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
chmod +x "$root/.githooks/pre-push"
git -C "$root" config core.hooksPath .githooks
echo "Git hooksPath -> .githooks (pre-push runs cargo fmt --check)"
