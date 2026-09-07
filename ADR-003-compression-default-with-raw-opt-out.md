# ADR-003: Compression stays on by default; `RTK_RAW=1` is the opt-out for composed output

**Status**: Accepted
**Date recorded**: 2026-09-07

## Context

Issue #77 (`docs/PLAN_RELIABILITY_2026-09.md`, REL-1/2/3) showed that RTK's per-command output compression (e.g. `ls`'s `... and N more entries ...` summary) silently corrupts anything downstream that treats the output as structured data: `ls -1 dir | wc -l` reported 19 for 36 real entries, no error, no signal.

The first fix gated compression on `std::io::IsTerminal`: raw, uncompressed output whenever the destination stream wasn't a real terminal. This correctly fixed the literal repro, but broke RTK's primary use case. An AI agent harness (Claude Code's Bash tool, or any equivalent) capturing a wrapped command's output is *also* "not a terminal" — indistinguishable, at the OS/file-descriptor level, from a shell script piping into `wc -l`. Gating on TTY therefore disabled compression for exactly the audience the product is built for, directly contradicting the README's own "AI Agent Guide" (which documents the compressed-output-plus-cache-note workflow as expected agent behavior) and the "Average 81.8% Token Savings" headline claim.

RTK cannot reliably tell "an agent about to read this as prose" apart from "a program about to parse this as data" from the OS side alone — both present identically as a non-terminal stdout/stderr.

## Decision

1. Compression (per-command filters, `apply_profile_settings`, `distiller`) is **on by default** for every wrapped command — the original pre-#77-fix behavior, restored.
2. `RTK_RAW=1` is a new, explicit per-invocation environment variable. When set, `execute_with_filter` (`rtk-cli/src/filter_pipeline.rs`) skips compression and returns the untruncated original text. The caller (typically the agent itself, per README's AI Agent Guide) sets it for the specific invocation whose output it is about to compose with another tool (`| wc -l`, `grep -c`, a parser, or reuse as another command's argument).
3. Security-motivated processing — DLP redaction (`rtk-db/src/dlp.rs`) and user-configured custom regex rules (`rtk filter add`) — always runs, `RTK_RAW` or not. It is not a token-savings heuristic and is not gated by this flag.
4. The `[Full output cached...]` cache-note and the autonomy warning always go to stderr, never mixed into stdout, unconditionally (not just under `RTK_RAW`). This closes the related REL-3 finding (a DB `tracking.cmd` corruption where cache-note text leaked into a *subsequent* command's argv via shell `$(...)` capture) structurally: `$(...)` only captures stdout, so a marker confined to stderr cannot recur regardless of flags or terminal state.

## Alternatives considered

| Option | Rejected because |
|--------|-------------------|
| Keep the TTY-gated raw fallback (first fix attempt) | Silently disables compression for the primary real-world consumer (an agent's non-interactive capture), which is indistinguishable from a human's `\| wc -l` at the OS level. Directly regresses the product's stated value proposition. |
| A trust-signal environment variable set by RTK's own installed hook/wrapper (e.g. `RTK_TRUSTED_CONSUMER=1`), defaulting to raw otherwise | Requires RTK to control the invoking harness's environment, which it doesn't for most integration paths (Claude Code's Bash tool launches the wrapped binary directly, not through an RTK-owned wrapper that could inject env vars). Also inverts the safety default: anyone piping into `wc -l` without knowing about the flag would still hit #77. |
| Per-command `--raw` CLI flag instead of an env var | Most wrapped commands (`Commands::Ls { args }`, etc.) forward `args` verbatim to the real underlying binary; injecting and stripping a `--raw` flag before that forwarding, for every wrapped command, is meaningfully more code than one `std::env::var_os` check, for the same effect. |

## Consequences

- Correctness for composed output (`| wc -l`, `$(...)` reuse) is now a documented convention the caller must opt into (`RTK_RAW=1`), not an automatic guarantee for every invocation — a deliberate trade-off favoring the primary (agent) use case over generic/manual human piping, per explicit product-owner direction (`docs/PLAN_RELIABILITY_2026-09.md`, "Root cause" section).
- The cache-note/autonomy-warning-on-stderr split is an unconditional structural guarantee (not opt-in), so the REL-3 corruption class cannot recur even if an agent forgets `RTK_RAW`.
- If a future integration path captures stdout+stderr separately and only surfaces stdout to the agent, the cache-note becomes invisible to it — worth revisiting if that integration shape appears (currently: Claude Code's Bash tool and this repo's own `.output()`-based tests both observe both streams).

## Provenance

Written during the REL-1/2/3 pipe/parsing reliability fix (`docs/PLAN_RELIABILITY_2026-09.md`), PR #78, after a design review that caught the TTY-gating regression against the README's AI Agent Guide.
