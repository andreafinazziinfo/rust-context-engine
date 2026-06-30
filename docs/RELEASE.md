# Release checklist (maintainer)

## v2.3.2 (Fase D — fixes & dependency alignment)

See [CHANGELOG](../CHANGELOG.md#232---2026-06-30) · [GitHub release](https://github.com/andreafinazziinfo/rust-context-engine/releases/tag/v2.3.2)

- DLP redacts generic `sk-` API keys; no SIGPIPE panic on `head`/`less`
- tree-sitter ecosystem upgraded (core 0.24 + grammars 0.23); rusqlite 0.32, toml 1.1, tokenizers 0.23
- CI fix: `e2e_ide_pipeline_flow` robust to detached-HEAD checkout

## v2.3.1 (Fase C)

See [RELEASE_v2.3.1.md](./RELEASE_v2.3.1.md) · [GitHub release](https://github.com/andreafinazziinfo/rust-context-engine/releases/tag/v2.3.1)

- `rtk validate` subcommand
- `e2e_smoke.sh` in CI Linux
- Dogfood: `scripts/setup-dogfood.sh`, `docs/DOGFOOD.md`

After pushing tag `vX.Y.Z`:

1. Wait for [Release workflow](https://github.com/andreafinazziinfo/rust-context-engine/actions/workflows/release.yml) to finish.
2. Refresh Homebrew checksums:
   ```bash
   bash scripts/update_homebrew_sha256.sh vX.Y.Z
   git add rtk.rb Formula/rtk.rb && git commit -m "chore: homebrew sha256 vX.Y.Z"
   ```
3. Verify: `bash scripts/homebrew_smoke.sh` and `bash scripts/release_smoke.sh`
4. Release assets include `*.tar.gz.sha256` sidecars (Unix builds).
5. **crates.io** (manual — not in GitHub Actions):
   ```bash
   # once: token from https://crates.io/settings/tokens
   cargo login
   bash scripts/publish-crates.sh --dry-run   # packages leaf crates only (db, filters, index)
   bash scripts/publish-crates.sh             # publishes all 6 crates in order
   ```
   Order: `rtk-context-db` → `filters` → `index` → `pack` → `mcp` → `rtk-context-engine`.
   Wait ~30–60s between crates if publish fails with “dependency not found”.

Homebrew install (in-repo tap):

```bash
brew tap andreafinazziinfo/rust-context-engine
brew install rtk
```
