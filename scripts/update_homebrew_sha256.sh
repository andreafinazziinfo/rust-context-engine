#!/usr/bin/env bash
# Refresh sha256 lines in rtk.rb from a GitHub release tag.
set -euo pipefail
TAG="${1:-v2.3.0}"
FORMULA="$(cd "$(dirname "$0")/.." && pwd)/rtk.rb"
tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

cd "$tmpdir"
gh release download "$TAG" --repo andreafinazziinfo/rust-context-engine --pattern 'rtk-*.tar.gz'

declare -A sums
while read -r hash file; do
  case "$file" in
    rtk-macos-arm64.tar.gz) key=arm ;;
    rtk-macos-amd64.tar.gz) key=amd ;;
    rtk-linux-amd64.tar.gz) key=linux ;;
  esac
  sums[$key]="$hash"
done < <(sha256sum rtk-*.tar.gz)

ver="${TAG#v}"
sed -i "s/^  version \".*\"/  version \"${ver}\"/" "$FORMULA"

perl -i -0pe "
  s|(url \".*rtk-macos-arm64.tar.gz\"\n)\s*# sha256.*|\1      sha256 \"${sums[arm]}\"|;
  s|(url \".*rtk-macos-amd64.tar.gz\"\n)\s*# sha256.*|\1      sha256 \"${sums[amd]}\"|;
  s|(url \".*rtk-linux-amd64.tar.gz\"\n)\s*# sha256.*|\1      sha256 \"${sums[linux]}\"|;
" "$FORMULA"

echo "Updated $FORMULA for $TAG"
bash "$(dirname "$0")/homebrew_smoke.sh"
