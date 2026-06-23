# Contributing to RTK

## Setup

```bash
git clone https://github.com/andreafinazziinfo/rust-context-engine.git
cd rust-context-engine
bash scripts/setup-githooks.sh   # once: pre-push runs cargo fmt --check
```

**Build/test on WSL** with repo under `~/dev/rust-context-engine` (not `/mnt/c/.../target`). See [installation.md](./src/installation.md).

## Before every PR

```bash
bash scripts/dev-gate.sh   # fmt + clippy + cargo test --workspace
rtk validate               # same gate via CLI (uses dev-gate.sh when present)
```

CI runs the same checks on Linux, Windows, and macOS.

## Project layout

| Path | Role |
|------|------|
| `rtk/rtk-cli` | CLI binary, dispatch, doctor |
| `rtk/rtk-filters` | Command output filters |
| `rtk/rtk-db` | SQLite, DLP, config, tracking |
| `rtk/rtk-index` | Tree-sitter graph |
| `rtk/rtk-mcp` | MCP server |
| `rtk/fixtures/` | Golden filter fixtures |
| `Formula/rtk.rb` | Homebrew tap formula |

## Adding a filter

1. Implement `filter()` in `rtk/rtk-filters/src/`
2. Add unit test + `token_savings` ≥40% where applicable
3. Add `fixtures/<name>/input.txt` + `expected.txt`
4. Extend `rtk-cli/tests/fixtures.rs` golden test
5. Wire dispatch in `rtk-cli/src/dispatch.rs`

Refresh golden expected files:

```bash
cargo test -p rtk-context-engine --test fixtures refresh_golden_expected_files -- --ignored
```

## Release (maintainers)

See [RELEASE.md](./RELEASE.md). After tag `vX.Y.Z`:

```bash
bash scripts/update_homebrew_sha256.sh vX.Y.Z
bash scripts/release_smoke.sh
bash scripts/homebrew_smoke.sh
```

## Git hooks

| Hook | Purpose |
|------|---------|
| `.githooks/pre-push` | Block push if `cargo fmt --check` fails |

Enable: `bash scripts/setup-githooks.sh` (sets `core.hooksPath .githooks`).

## Plans & roadmap

- [PLAN_NOW.md](./PLAN_NOW.md) — active execution plan
- [ROADMAP.md](./ROADMAP.md) — public snapshot
- [archive/IMPROVEMENT_PLAN_9PLUS.md](./archive/IMPROVEMENT_PLAN_9PLUS.md) — completed quality sprint (historical)
