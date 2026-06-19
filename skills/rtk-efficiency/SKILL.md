---
name: rtk-efficiency
description: >
  Guidance and instructions for AI agents on utilizing the RTK Token Efficiency Toolkit.
  Use to optimize context window, package codebase directory context, read SQLite virtualized logs,
  and query/set project state memory.
  Trigger when starting a workspace tasks, diagnosing failed runs with virtualized logs,
  saving project configuration notes, or analyzing directory contents.
---

# RTK CLI AI Agent Integration Skill

This skill guides you on utilizing **RTK CLI**, the token-saving developer toolkit active in this workspace.

## 1. Virtualized Output Retrieval (`rtk show-log`)
When executing standard commands (e.g., `git status`, `git diff`, `git log`, `cargo build`, `cargo test`, `pytest`, `ls`, `npm install`), the output is automatically intercepted and stripped of noise to save context tokens.
- At the end of compressed outputs, you will see: `[Full output cached. Access with: rtk show-log <id>]`.
- **Do NOT** re-run the command with extra flags to see full diagnostic logs or tracebacks.
- **Do** fetch the raw, cached log directly from SQLite:
  ```bash
  rtk show-log <id>
  ```

## 2. Token-Efficient Codebase Packing (`rtk pack`)
When you need to read or understand the contents of a directory, folder, or the entire workspace:
- **Do NOT** read multiple files individually using sequential tool calls.
- **Do NOT** output or inspect folders recursively with verbose bash scripts.
- **Do** run the packer to generate a minified XML block:
  ```bash
  rtk pack [path] --strip --skeleton --limit <token_budget>
  ```
  - Always use `--strip` (or `-s`) to strip full-line comments and empty lines.
  - Use `--skeleton` (or `-k`) to generate skeletal structures (method/function signatures) for Rust, Python, and JS/TS, allowing you to load huge files with very few tokens.
  - Always use `--limit <n>` (or `-l <n>`) to specify a safe token budget limit and avoid context blowups.

## 3. Project Context Memory Syncing (`rtk memory`)
To persist and share project settings, ports, and metadata between sessions:
- **At the start of every session**: Check if there is any active context memory saved for this project:
  ```bash
  rtk memory list
  ```
- **When discovering new setup parameters**: Store key settings (e.g., ports, test credentials, runtime settings) to prevent wasting search steps in future sessions:
  ```bash
  rtk memory set <key> <value>
  ```
  *(Example: `rtk memory set api_port 8080`)*
- **When querying configurations**: Query specific keys directly:
  ```bash
  rtk memory get <key>
  ```

## 4. Automatic Rules Synchronization (`rtk sync-rules`)
If you add or update rule files in `.cursor/rules/` or `.agents/rules/` at the workspace root, run the sync-rules command to propagate them to all project subdirectories:
```bash
rtk sync-rules
```

## 5. Data Loss Prevention (DLP) Guard
The toolkit automatically scrubs credentials, private keys, JWTs, and high-entropy secrets from command outputs and pack buffers before returning them to you.
- **Auto-Redaction**: Redacted fields appear as `[REDACTED_API_KEY]`, `[REDACTED_JWT]`, `[REDACTED_SECRET]`, or `[REDACTED_CREDENTIALS]`.
- **Zero Leakage**: Ensure you do not try to bypass this guard or log credentials, as they are securely filtered at the proxy layer.

## 6. Local Savings Dashboard (`rtk dashboard`)
If you or the user want to view the savings dashboard:
```bash
rtk dashboard
```
This compiles local statistics into an HTML dashboard showing invocations, saved tokens, and estimated financial savings in USD, opening automatically in the web browser. Or view raw statistics in-terminal with `rtk stats`.
