# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

## [2.4.2] - 2026-08-04

### Fixed
*   **`.claude/skills` never written** — `rtk init` wrote caveman/ponytail skill files only under `.agents/skills/`, a path Claude Code never reads (it discovers skills exclusively from `.claude/skills/<name>/SKILL.md`). Skills are now written to both locations.
*   **Profile text cited nonexistent skill names** — the HIGH/MEDIUM/MAX profile blocks instructed the agent to trigger `caveman-full`/`caveman-lite`/`caveman-ultra`, none of which were ever generated as assets (only the single, level-parameterized `caveman` skill exists, matching upstream [JuliusBrussee/caveman](https://github.com/JuliusBrussee/caveman)). Profile text now references `caveman` + the level as a parameter.
*   **`rtk doctor`/`rtk status`** now verify the skill referenced by the active profile actually exists under `.claude/skills/`, and warn with a fix command if not.

### Added
*   `rtk init --force-profile` — regenerates an outdated `RTK Output Profile` block in `CLAUDE.md`/`copilot-instructions.md` (versioned via an internal marker) without touching any other content in the file. Needed because the two bugs above mean every existing `rtk init` install is silently broken and the normal idempotent guard would otherwise never let the fix reach already-initialized projects.
*   `rtk init` prints a one-line hint to install the official `claude plugin marketplace add JuliusBrussee/caveman` plugin when the `claude` CLI is detected on `PATH`.

See `docs/PLAN_CLAUDE_SKILLS_FIX.md` for the full root-cause analysis and design.

## [2.4.1] - 2026-08-04

### Fixed
*   `rusqlite` bumped 0.32 → 0.40: two crates (`rtk-context-db`, `rtk-context-index`) read/bound `usize` directly against `rusqlite::Row`/`params!`, which 0.40 no longer supports (`FromSql`/`ToSql` narrowed to fixed-width integer types). Now read as `i64` and cast at the boundary; no public struct field types changed.

### Dependencies
*   Bumped `crossbeam-epoch` 0.9.18 → 0.9.20 (RUSTSEC-2026-0204), `petgraph` 0.6.5 → 0.8.3, `ndarray` 0.15.6 → 0.17.2, `tract-onnx` 0.21.17 → 0.23.4, `regex` 1.12.4 → 1.13.0, `tree-sitter-rust` 0.23.3 → 0.24.2, `tree-sitter-python` 0.23.6 → 0.25.0, `tree-sitter-java` 0.23.2 → 0.23.5, `tree-sitter` 0.24.7 → 0.26.10, `dirs` 5.0.1 → 6.0.0, `rusqlite` 0.32.0 → 0.40.1.

### Added
*   `SECURITY.md` — vulnerability disclosure process.
*   `TECHNICAL_DEBT_LEDGER.md` and `ADR-001`/`ADR-002` (retroactive) — first Brownfield ASSESSMENT baseline.

## [2.4.0] - 2026-07-02

### Added
*   **6 new command filters**, extending the token-efficiency surface across Python, the Vue/TS frontend, and container/PR ops:
    *   `rtk ruff check` — collapses ruff's "full"/pretty output (code-frames + `help:` hints) to one `file:line:col: CODE message` line per violation (~53%).
    *   `rtk mypy` — collapses `mypy --pretty` code-frames and re-joins wrapped messages into clean single-line diagnostics, notes preserved (~39%).
    *   `rtk pip install` — drops resolver noise and collapses `Requirement already satisfied` into one count, keeping only `Successfully installed` and errors (~94%).
    *   `rtk eslint` — collapses the stylish formatter's column alignment / per-file headers / `--fix` notice to `line:col severity message (rule)` (~27%).
    *   `rtk tsc` — strips ANSI colors and `--pretty` code-frames, keeping error headers, message continuations and related-information locations.
    *   `rtk vitest` — drops decorative `⎯` rules, banners and code-frames, keeping failures, assertion diffs, locations and the summary tally.
    *   `rtk docker ps` — compacts the wide table to `NAMES  IMAGE  STATUS  PORTS`.
    *   `rtk gh pr checks` — strips the long per-job URL from each check row.
*   DLP now redacts more token prefixes: GitHub fine-grained `github_pat_`, GitHub `gho_/ghs_/ghr_/ghu_`, GitLab `glpat-`, npm `npm_`, HuggingFace `hf_`, DigitalOcean `dop_v1_`.
*   `rtk pack --skeleton` supports Vue single-file components (`.vue`): keeps the `<script>` block, drops `<template>`/`<style>`.

### Fixed
*   Wrapped commands now forward leading flags: `rtk pytest --tb=short` (and every other wrapper) no longer errors on a leading `--flag`. Previously the rewrite hook could turn a valid command into a broken one.
*   `docker` subcommands are now routed correctly: `docker ps` gets its own filter instead of being fed to the `docker build` filter; unrecognized subcommands pass through.
*   `rtk dotnet` no longer panics when the `dotnet` binary is missing; it falls back gracefully.
*   `rtk setup` no longer panics on a non-standard `settings.json` (JSON manipulation hardened against malformed input).
*   Eliminated a Windows-only flaky test (`config`/`pricing` tests raced on process-global `HOME`/`USERPROFILE`; now serialized on a shared lock).

### Dependencies
*   Bumped `anyhow` 1.0.102 → 1.0.103 (clears RUSTSEC-2026-0190).

## [2.3.2] - 2026-06-30

### Fixed
*   DLP `pack --strip` now redacts generic/legacy `sk-`-prefixed API keys (OpenAI-style); previously only the `sk-proj-`, `sk-ant-`, and `sk_live_/sk_test_` forms were matched.
*   No longer panics (exit code 101) on `SIGPIPE`: piping RTK output into `head`/`less` now terminates cleanly instead of crashing.
*   Hook manual-test instructions in `README.md` corrected to the stdin-JSON form (`echo '{"tool_input":{"command":"git status"}}' | bash hooks/rtk-rewrite.sh`); the previous `RTK_REWRITE_CMD` form never produced output.
*   `e2e_ide_pipeline_flow` integration test made robust to GitHub Actions' detached-HEAD checkout, which previously failed CI on every PR.

### Dependencies
*   Upgraded the tree-sitter ecosystem to core 0.24 with all grammars on 0.23 (migrated call sites to the `LanguageFn` API: `language()` → `LANGUAGE.into()`).
*   Bumped `rusqlite` 0.31 → 0.32, `toml` 0.8 → 1.1, and `tokenizers` 0.19 → 0.23 (with a `tract-onnx` API migration in the optional `embeddings` feature).
*   Bumped CI actions: `checkout` v7, `cache` v6, `codeql-action` v4, `action-gh-release` v3.

### Added
*   Local compliance audit logger for Data Loss Prevention (DLP) redactions, recording timestamps, context/source tools, and secure cryptographic hashes of redacted secrets to `~/.config/rtk/audit.log`.
*   Troubleshooting guidelines for shell profiles, WSL pathways, and database locks in `README.md`.
*   Safety disclaimers and bypass warning documentation for transparent CLI aliases.

### Changed
*   Updated benchmark engine and cost savings projections to use real-world state-of-the-art model pricing (Claude 3.5 Sonnet, Claude 3 Opus, GPT-4o, Gemini 1.5 Pro/Flash).

---

## [0.1.0] - 2026-06-19

### Added
*   Initial release of RTK (Runtime Token Toolkit).
*   15 input virtualization CLI wrappers for noisy tool outputs (`git`, `cargo`, `pytest`, `docker`, `npm`, `yarn`, `pnpm`, `composer`, `terraform`, `dotnet`, `gradle`, `go_test`, `ls`).
*   `rtk think` reasoning offloader and persistent project memory commands (`rtk memory`).
*   `rtk pack` Tree-Sitter AST context packaging engine with minification and signature skeletal structures.
*   Data Loss Prevention (DLP) engine with regex pattern matching and Shannon-entropy scanner.
*   `rtk init` rule system bootstrapping for Claude Code, Cursor, and Windsurf AI profiles (*Caveman* and *Ponytail* response rules).
