# RTK User Guide

Start here after install. **5-minute setup:** [QUICKSTART.md](./QUICKSTART.md).

## What RTK does

RTK is a **local context engine** for AI coding agents. It:

1. **Filters** noisy command output (git, cargo, pytest, npm, docker, …)
2. **Redacts** secrets (DLP) before they reach the agent
3. **Caches** full logs locally (`rtk show-log <id>`)
4. **Packs** directories for agents (`rtk pack --strip --skeleton --limit`)
5. **Remembers** project state (`rtk memory`, `rtk think`)
6. **Indexes** code for symbols, refs, and impact analysis
7. **Exposes MCP tools** for Cursor / Claude Code

## Core commands

| Task | Command |
|------|---------|
| Filter git | `rtk git status` / `diff` / `log` |
| Filter Rust | `rtk cargo test` / `build` |
| Filter Python | `rtk pytest` |
| Filter JS | `rtk npm install` / `yarn` / `pnpm` |
| Pack repo | `rtk pack . --strip --limit 30000` |
| Memory | `rtk memory set/get/search` |
| Index | `rtk index run` · `rtk symbols find <name>` |
| Health | `rtk doctor` |
| Quality gate | `rtk validate` (RTK repo / projects with `scripts/dev-gate.sh`) |

## Profiles & hooks

```bash
rtk init --profile high   # aliases + Caveman/Ponytail rules + hook setup
```

PreToolUse hook (Claude Code): see [README § AI Integration](../README.md) or `hooks/rtk-rewrite.sh`.

## Default vs full build

| Feature | Default install | `--features embeddings` |
|---------|-----------------|-------------------------|
| Filters + DLP + pack | yes | yes |
| Code graph + MCP | yes | yes |
| ONNX hybrid search | no | yes |

## Limitations

Best-effort DLP, alias-based wrappers (not a sandbox), manual `rtk index run` after large refactors. Details: [limitations.md](./src/limitations.md).

## More documentation

- [README](../README.md) — benchmarks, architecture diagrams, full reference
- [cli.md](./src/cli.md) — command table
- [installation.md](./src/installation.md) — WSL dev notes
- [ROADMAP.md](./ROADMAP.md) — what's shipped / active plan
- [PLAN_CLOSURE.md](./PLAN_CLOSURE.md) — v2.3.1 closure, Fase D real-world use
