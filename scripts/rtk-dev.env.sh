# RTK dev env — source from repo root: source scripts/rtk-dev.env.sh
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export RTK="$ROOT/rtk/target/debug/rtk"
export PATH="$(dirname "$RTK"):$PATH"
