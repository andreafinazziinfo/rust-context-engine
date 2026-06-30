# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

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
