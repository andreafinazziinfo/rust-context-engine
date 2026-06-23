# RTK Roadmap

Public snapshot for **v2.3.0**. Detailed sprint history lives in [`archive/IMPROVEMENT_PLAN_9PLUS.md`](archive/IMPROVEMENT_PLAN_9PLUS.md).

## Shipped (release quality)

- Command filters + DLP redaction (15+ wrappers)
- `rtk pack` with `--strip`, `--skeleton`, `--limit`
- SQLite memory, artifacts, session state
- Tree-sitter code graph (`symbols`, `refs`, `impact`, lazy index)
- MCP server (8 tools, version parity with CLI)
- CI matrix (Linux / Windows / macOS), release smoke, savings regression gate

## Next (high ROI)

| Item | Why |
|------|-----|
| Homebrew tap | One-line macOS install (`rtk.rb` pinned to release checksums) |
| Pre-push fmt hook | Prevent CI failures from rustfmt drift |
| Targeted tests | `dlp`, `rewrite`, `filter_pipeline` coverage (not global 70%) |
| Golden filters | Extend pytest/docker fixtures + CI gate |

## Won't do (unless product direction changes)

- Embeddings in **default** binary (+15MB ONNX) — stays optional `--features embeddings`
- Filesystem index watcher — manual `rtk index run` after large refactors
- Chasing scorecard 9.5 everywhere without user-facing benefit

## Contributing

```bash
bash scripts/setup-githooks.sh   # once per clone
bash scripts/dev-gate.sh         # fmt + clippy + test before PR
```
