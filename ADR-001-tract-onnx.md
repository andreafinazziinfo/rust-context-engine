# ADR-001: Use `tract-onnx` for local embedding inference

**Status**: Accepted (retroactive)
**Date recorded**: 2026-08-04

## Context

RTK needs to run a small embedding model locally (no network calls) to power semantic search over the local FTS5/vector cache. This requires a pure-Rust or easily-vendored ONNX runtime, since RTK ships as a single static binary via `cargo install` / Homebrew / a release zip — no Python, no system-wide ONNX Runtime dependency can be assumed on the target machine.

## Decision

Use `tract-onnx` (Sonos' pure-Rust ONNX inference engine) rather than bindings to Microsoft's `onnxruntime` (e.g. the `ort` crate).

## Alternatives considered

| Option | Rejected because |
|--------|-------------------|
| `ort` (onnxruntime bindings) | Links a large prebuilt C++ shared library per platform; complicates static/single-binary distribution and cross-compilation (esp. `aarch64-apple-darwin` in `release.yml`). |
| Call out to a Python sidecar | Reintroduces the exact runtime dependency (Python + venv) RTK exists to avoid for its own users. |
| Skip embeddings, keyword-only search | Would regress semantic search quality; embeddings are used by `rtk-index`'s vector cache. |

## Consequences

- `tract-onnx` is a smaller, less-resourced project than `onnxruntime` upstream — slower to pick up new ONNX opset versions or new model architectures.
- Bundled model files must target ONNX opsets `tract-onnx` supports; upgrading the embedding model requires checking `tract` compatibility first.
- If `tract-onnx` stalls or is abandoned upstream, the fallback path is `ort` behind a feature flag, accepting the single-binary distribution trade-off above — no other realistic path was identified.

## Provenance

Retroactively documented during the 2026-08-04 Brownfield ASSESSMENT baseline (see `TECHNICAL_DEBT_LEDGER.md` item 4). No original design doc existed; this reflects the decision as best reconstructed from the dependency's role and RTK's single-binary distribution constraint.
