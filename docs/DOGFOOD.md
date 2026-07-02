# Source RTK dev binary for this repo (WSL/Linux)

```bash
source scripts/rtk-dev.env.sh
rtk --version          # 2.4.x from target/debug
rtk git status
rtk cargo test --manifest-path rtk/Cargo.toml -p rtk-context-filters
rtk validate           # same as scripts/dev-gate.sh
rtk pack . --strip --limit 30000
```

## One-shot setup

```bash
bash scripts/setup-dogfood.sh
```

Creates/updates `.cursor/rules/`, runs `index run`, `doctor`, `validate`, `e2e_smoke`.

## Cursor / Claude agent hooks

1. Build once: `bash scripts/setup-dogfood.sh`
2. Hook path (WSL): `hooks/rtk-rewrite.sh` — see [hooks/HOOKS-README.md](../hooks/HOOKS-README.md)
3. Rules: `.cursor/rules/rtk-toolkit.mdc` (from `rtk init`)

When `CURSOR=1` or wrappers in `~/.rtk/bin` are on PATH, shell commands route through RTK filters.

## Status bar (`💾`)

Optional: [claude-statusline-pro](https://github.com/andreafinazziinfo/claude-statusline-pro) reads the **global** tracking DB (`~/.local/share/rtk/rtk.db` on WSL). Project `.rtk/rtk.db` has memory/index only — not savings telemetry.

```bash
# ~/.config/claude-statusline/config.sh
RTK_DB_PATH="$HOME/.local/share/rtk/rtk.db"
```

## Windows (PowerShell)

Use prebuilt or WSL for dev. From WSL path:

```bash
source ~/dev/rust-context-engine/scripts/rtk-dev.env.sh
```

Native Windows: `%USERPROFILE%\.rtk-bin\rtk.exe` per [QUICKSTART.md](./QUICKSTART.md).
