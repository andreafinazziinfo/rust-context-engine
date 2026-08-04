# ADR-002: Use `tree-sitter` for multi-language parsing

**Status**: Accepted (retroactive)
**Date recorded**: 2026-08-04

## Context

`rtk-index` and the filter/skeleton commands (`rtk pack --skeleton`, symbol indexing, impact analysis) need fast, incremental, error-tolerant parsing across multiple languages (Rust, Python, Java, Vue/TS SFCs, and more added over time — see `CHANGELOG.md` 2.4.0). Parsing must work on incomplete/invalid code (mid-edit files) without crashing.

## Decision

Use `tree-sitter` core plus one grammar crate per supported language (`tree-sitter-rust`, `tree-sitter-python`, `tree-sitter-java`, etc.).

## Alternatives considered

| Option | Rejected because |
|--------|-------------------|
| Language-specific parsers (e.g. `syn` for Rust, `rustpython-parser` for Python) | No single incremental/error-tolerant model across languages; would multiply parser integration work per language added and most aren't error-tolerant on invalid syntax. |
| Regex/heuristic symbol extraction | Already RTK's fallback for unsupported languages; too fragile for the primary indexing path (wrong symbol boundaries break impact analysis). |
| LSP servers per language | Requires each language's toolchain installed on the user's machine — violates the same single-binary, zero-dependency goal as ADR-001. |

## Consequences

- Adding a language means finding (and trusting) its `tree-sitter-<lang>` grammar crate; quality and maintenance vary by grammar (see recurring dependabot bumps: `tree-sitter-rust`, `-python`, `-java`).
- Grammar crates version somewhat independently of `tree-sitter` core, causing periodic compatibility bumps (e.g. #46–#50 dependency PRs) — this is expected churn, not a signal to abandon the approach.
- `tree-sitter` core itself is a de facto standard (used by GitHub, Neovim, Helix); low risk of abandonment, making this a lower-priority SPOF than ADR-001's `tract-onnx`.

## Provenance

Retroactively documented during the 2026-08-04 Brownfield ASSESSMENT baseline (see `TECHNICAL_DEBT_LEDGER.md` item 4).
