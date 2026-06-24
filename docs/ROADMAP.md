# RTK Roadmap

Public snapshot for **v2.3.1**. **Active plan:** **[PLAN_CLOSURE.md](./PLAN_CLOSURE.md)** (Fase D → real-world use). Completed sprints: [PLAN_NOW.md](./PLAN_NOW.md). Audit (archived): [archive/IMPROVEMENT_PLAN_9PLUS.md](./archive/IMPROVEMENT_PLAN_9PLUS.md).

## Shipped (v2.3.1)

- Command filters + DLP redaction (15+ wrappers)
- Native npm/yarn/pnpm filters + golden regression fixtures
- `rtk pack` with `--strip`, `--skeleton`, `--limit`
- SQLite memory, artifacts, session state
- Tree-sitter code graph (`symbols`, `refs`, `impact`, lazy index)
- MCP server (9 tools, version parity with CLI)
- `rtk validate` quality gate · `e2e_smoke` in CI
- CI matrix (Linux / Windows / macOS), release smoke, savings regression gate
- GitHub release + Homebrew formula + **crates.io** (6 workspace crates)
- Dogfood: [DOGFOOD.md](./DOGFOOD.md)

## Fase C — Chiusura tecnica ✅

See [PLAN_CLOSURE.md](./PLAN_CLOSURE.md). All items done except optional macOS `brew install` smoke (CLOSE-4).

## Fase D — Active now (2–4 weeks)

**No new features.** Use RTK daily on real repos; log friction with `rtk memory` or local notes. Then Fase E: max 5 backlog items from pain points.

## Won't do (unless product direction changes)

- Embeddings in **default** binary (+15MB ONNX) — stays optional `--features embeddings`
- Filesystem index watcher — manual `rtk index run` after large refactors
- Agent OS / harness layer (CodeWhale-style) — deferred until Fase E feedback
- Chasing scorecard 9.5 everywhere without user-facing benefit

## Contributing

```bash
bash scripts/setup-githooks.sh   # once per clone
rtk validate                   # or: bash scripts/dev-gate.sh
```

Release: [RELEASE.md](./RELEASE.md) · crates.io: `bash scripts/publish-crates.sh`
