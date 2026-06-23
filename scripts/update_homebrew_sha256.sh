#!/usr/bin/env bash
# Refresh sha256 + version in rtk.rb and Formula/rtk.rb from a GitHub release tag.
set -euo pipefail
TAG="${1:-v2.3.1}"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

cd "$tmpdir"
gh release download "$TAG" --repo andreafinazziinfo/rust-context-engine --pattern 'rtk-*.tar.gz'

declare -A sums
while read -r hash file; do
  case "$file" in
    rtk-macos-arm64.tar.gz) sums[arm]=$hash ;;
    rtk-macos-amd64.tar.gz) sums[amd]=$hash ;;
    rtk-linux-amd64.tar.gz) sums[linux]=$hash ;;
  esac
done < <(sha256sum rtk-*.tar.gz)

ver="${TAG#v}"

update_formula() {
  local f="$1"
  sed -i "s/^  version \".*\"/  version \"${ver}\"/" "$f"
  sed -i "s|releases/download/v[0-9.]*/|releases/download/v${ver}/|g" "$f"
  perl -i -pe "
    if (/rtk-macos-arm64/ .. /sha256/) { s/sha256 \"[a-f0-9]+\"/sha256 \"${sums[arm]}\"/ if /sha256/; }
  " "$f"
  perl -i -pe "
    if (/rtk-macos-amd64/ .. /sha256/) { s/sha256 \"[a-f0-9]+\"/sha256 \"${sums[amd]}\"/ if /sha256/; }
  " "$f"
  perl -i -pe "
    if (/rtk-linux-amd64/ .. /sha256/) { s/sha256 \"[a-f0-9]+\"/sha256 \"${sums[linux]}\"/ if /sha256/; }
  " "$f"
}

update_formula "$ROOT/rtk.rb"
cp "$ROOT/rtk.rb" "$ROOT/Formula/rtk.rb"

echo "Updated rtk.rb + Formula/rtk.rb for $TAG"
bash "$(dirname "$0")/homebrew_smoke.sh"
