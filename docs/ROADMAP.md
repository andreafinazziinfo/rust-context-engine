# RTK Roadmap

Public snapshot for **v2.3.1**. **Active plan:** **[PLAN_CLOSURE.md](./PLAN_CLOSURE.md)** (chiusura → uso → valutazione). Completed: [PLAN_NOW.md](./PLAN_NOW.md). Sprint audit (archived): [archive/IMPROVEMENT_PLAN_9PLUS.md](./archive/IMPROVEMENT_PLAN_9PLUS.md).

## Shipped (release quality)

- Command filters + DLP redaction (15+ wrappers)
- `rtk pack` with `--strip`, `--skeleton`, `--limit`
- SQLite memory, artifacts, session state
- Tree-sitter code graph (`symbols`, `refs`, `impact`, lazy index)
- MCP server (8 tools, version parity with CLI)
- CI matrix (Linux / Windows / macOS), release smoke, savings regression gate
- Homebrew formula (`rtk.rb`) v2.3.1 + sha256; fmt pre-push hook

## Adesso (Fase A — see PLAN_NOW.md) ✅

Completed v2.3.0+ — see [QUICKSTART.md](./QUICKSTART.md).

## Subito dopo (Fase B) ✅

Completed — native npm/yarn filters, golden ls/npm, core integration tests, USER/CONTRIBUTING docs.

## Chiusura operativa (Fase C → E) — active

See **[PLAN_CLOSURE.md](./PLAN_CLOSURE.md)**:

- **C** (1–2 gg): `e2e_smoke` in CI ✅, `rtk validate` ✅, release v2.3.1 🔄, brew smoke macOS, dogfood [DOGFOOD.md](./DOGFOOD.md) ✅
- **D** (2–4 sett): uso quotidiano, log feedback, **no new features**
- **E** (1 gg): backlog max 5 item da pain point reali

## Won't do (unless product direction changes)

- Embeddings in **default** binary (+15MB ONNX) — stays optional `--features embeddings`
- Filesystem index watcher — manual `rtk index run` after large refactors
- Chasing scorecard 9.5 everywhere without user-facing benefit

## Contributing

```bash
bash scripts/setup-githooks.sh   # once per clone
bash scripts/dev-gate.sh         # fmt + clippy + test before PR
```
