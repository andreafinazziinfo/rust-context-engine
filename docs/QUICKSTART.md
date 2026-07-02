# RTK Quickstart (~5 minutes)

Get filtered terminal output, project memory, and code index working for AI agents (Claude Code, Cursor, etc.).

## 1. Install

### macOS (Homebrew tap)

```bash
brew tap andreafinazziinfo/rust-context-engine
brew install rtk
```

Or from a [release tarball](https://github.com/andreafinazziinfo/rust-context-engine/releases/latest).

### Linux / WSL

```bash
cargo install rtk-context-engine --locked
# or from source:
git clone https://github.com/andreafinazziinfo/rust-context-engine.git
cd rust-context-engine
bash install.sh
# or prebuilt: bash install.sh --prebuilt rtk-linux-amd64.tar.gz
```

Ensure `~/.local/bin` is on your `PATH`.

### Windows (daily use)

1. Download [`rtk-windows-amd64.zip`](https://github.com/andreafinazziinfo/rust-context-engine/releases/latest) from Releases.
2. Extract `rtk.exe` to `%USERPROFILE%\.rtk-bin\`.
3. Add to PATH (PowerShell, current user):

```powershell
[Environment]::SetEnvironmentVariable(
  "Path",
  $env:Path + ";$env:USERPROFILE\.rtk-bin",
  "User"
)
```

4. Open a new terminal: `rtk --version`

**Do not** build from source with MSVC for daily use — use the prebuilt binary or WSL for development.

## 2. Initialize in your project

```bash
cd /path/to/your/project
rtk init --profile high
rtk index run
rtk doctor
```

`init` installs shell aliases and attempts PreToolUse hook setup. `doctor` should show mostly ✅; warnings link back here.

## 3. Try the core loop

```bash
# Filtered git status (compare with raw git status)
rtk git status

# Pack codebase for an agent (strip comments, optional skeleton)
rtk pack . --strip --limit 30000

# Persist a project fact across sessions
rtk memory set db_port 5432
rtk memory get db_port
```

If output is truncated, use `rtk show-log <id>` — do not re-run the same noisy command.

## 4. Status bar (optional)

[claude-statusline-pro](https://github.com/andreafinazziinfo/claude-statusline-pro) shows `💾 N% (-Xk)` from the same global tracking DB as `rtk stats` (statusline = **today only**; `rtk stats` = all-time).

```bash
bash <(curl -fsSL https://raw.githubusercontent.com/andreafinazziinfo/claude-statusline-pro/main/install.sh)
```

Pin the DB in `~/.config/claude-statusline/config.sh` if WSL/Windows paths diverge:

```bash
RTK_DB_PATH="$HOME/.local/share/rtk/rtk.db"
```

Requires the PreToolUse hook (step 2 in README) so Bash commands are rewritten to `rtk …`.

## 5. MCP (optional)

```bash
rtk mcp install --client cursor
# or: rtk mcp install --client claude
rtk mcp start
```

## 6. Verify health

```bash
rtk doctor          # exit 0 = OK, 1 = critical, 2 = warnings
rtk validate        # contributors: fmt + clippy + tests (in RTK repo)
rtk index status    # re-run `rtk index run` after large refactors (no file watcher)
```

## Troubleshooting

| Issue | Fix |
|-------|-----|
| Raw git/cargo output | Run `source ~/.bashrc` or `~/.zshrc`; check `rtk doctor` aliases |
| Empty symbols | `rtk index run` in project root |
| Hook not rewriting | Absolute path to `hooks/rtk-rewrite.sh` in Claude settings; see README |
| WSL vs Windows DB | Run RTK in one environment consistently; pin `RTK_DB_PATH` in statusline `config.sh` |
| Missing `💾` in status bar | Hook active + same DB as RTK; see [claude-statusline-pro RTK doc](https://github.com/andreafinazziinfo/claude-statusline-pro/blob/main/docs/rtk-integration.md) |

More: [README](../README.md) · [ROADMAP.md](./ROADMAP.md) · [PLAN_CLOSURE.md](./PLAN_CLOSURE.md) (project status)

## Contributors

```bash
bash scripts/setup-githooks.sh   # fmt pre-push
rtk validate                   # or: bash scripts/dev-gate.sh
```

Build from source: clone under `~/dev/` on WSL (not `/mnt/c/target`) — see `docs/archive/IMPROVEMENT_PLAN_9PLUS.md` DEV-WSL notes.
