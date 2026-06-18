# AI Efficiency Toolkit 🚀

A high-performance, token-efficient developer toolchain designed to optimize context windows, cut API costs, and improve speed for AI coding assistants (such as Claude Code, Cursor, Antigravity, and other agents).

By filtering verbose terminal outputs, caching logs, packing files, and enforcing YAGNI developer behaviors, the toolkit saves **60% to 95% of tokens** in common coding operations.

---

## Architecture & Core Components

1. **`rtk`**: A lightweight CLI wrapper written in Rust that intercepts and filters diagnostics, build outputs, and testing logs (sub-10ms startup time, sub-5MB memory footprint).
2. **Context Virtualization**: Automatically caches large outputs in a local SQLite database (`rtk.db`) and returns short, token-friendly reference links (`[Full output cached. Access with: rtk show-log <id>]`) to the LLM instead of massive tracebacks.
3. **Claude Code hooks**: Shell integration scripts (`rtk-rewrite.sh`, `rtk-suggest.sh`) that hook into Claude's `PreToolUse` phase to auto-route shell commands.
4. **MDC Rules**: System instructions (`lazy-dev.mdc`, `token-efficiency.mdc`) that direct the AI to follow YAGNI (You Aren't Gonna Need It) principles and keep diff scopes minimal.
5. **Caveman Presets**: Compression instructions that guide the AI to strip filler words and grammatical padding, saving up to 75% of output tokens.

---

## Features

### 1. High-Efficiency CLI Filters (`rtk`)

The `rtk` binary intercepts developer commands and filters out noise while keeping 100% of the relevant technical warnings and errors:

| Command | Noise Removed | Token Savings |
| :--- | :--- | :--- |
| `git status` | Dropped section headers, hint lines `(use "git add"...)`, and collapsed untracked file listings. | ~60% |
| `git diff` | Dropped context lines (unchanged), headers, and collapsed hunks with >8 changed lines. | 60% – 85% |
| `git log` | Reduced commits to a single line: `<hash>  <subject>`. | ~70% |
| `cargo test` | Dropped passing test lines `test ... ok` and progress lines, retaining only failures and summaries. | 70% – 95% |
| `cargo build/check` | Intercepted stderr, stripped compiling progress lines, and retained finished summary + compiler errors. | 60% – 90% |
| `pytest` | Dropped platform/plugin preambles and collapsed long deprecation/warning blocks. | 70% – 90% |
| `ls` | Stripped owner, group, and link counts. Collapsed folder listings longer than 20 files. | 50% – 70% |
| `npm install` | Uses log distillation to keep first/last 15 lines and keep error lines. | 75% – 95% |

### 2. Context Virtualization (`rtk show-log`)
If a command output is compressed, `rtk` stores the **entire raw output** in the SQLite database and appends a tiny message. If the AI specifically needs the raw traceback later, it can query:
```bash
rtk show-log <id>
```

### 3. Context Directory Packer (`rtk pack`)
Walks a folder, ignores binary files and standard heavy directories (like `.git`, `node_modules`, `target`), and outputs a highly compressed XML block:
```bash
rtk pack [path]
```

### 4. Rule Synchronization (`rtk sync-rules`)
Synchronizes `.cursor/rules/` and `.agents/rules/` from your root workspace recursively to all sub-project folders so the AI rules apply even when you open subfolders directly in Cursor or Claude Code:
```bash
rtk sync-rules
```

### 5. gamified Token Savings Stats (`rtk stats`)
Queries your database and outputs a summary of your performance:
```bash
$ rtk stats
========================================
          RTK TOKEN SAVINGS STATS       
========================================
Total Commands Run:       142
Original Tokens:          1,280,000
Filtered Tokens:          185,000
Tokens Saved:             1,095,000 (85.5%)
Estimated API Cost Saved: $3.2850 USD
========================================
```

---

## Installation

### Prerequisites
- Rust and Cargo (`cargo install` / `rustup`)
- Bash and `jq`

### Install
Run the installation script from the repository root:
```bash
bash install.sh
```

---

## License
Apache License 2.0. Free to use, modify, and distribute. See the [LICENSE](LICENSE) file for details.
