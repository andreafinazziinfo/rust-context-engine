# Technical Debt Ledger

Seeded from the first Brownfield ASSESSMENT baseline (2026-08-04). See `5_BROWNFIELD_FRAMEWORK.md` Area A in the maintainer's operating framework for the process this follows.

| # | Item | Impact | Status | Owner |
|---|------|--------|--------|-------|
| 1 | Branch ruleset on `main` requires a status check literally named `Build and Test`, but CI only reports matrix-suffixed names (`Build and Test (ubuntu-latest)`, etc.). Every PR is permanently `BLOCKED` regardless of CI outcome. | High — blocks all delivery | Open | Repo owner |
| 2 | No `SECURITY.md` / vulnerability disclosure process existed before this ledger. | High — no clear path for reporters | Fixed (this change) | Repo owner |
| 3 | `rusqlite` was pinned to 0.32 (8 minor versions behind) because `usize` `FromSql`/`ToSql` support was removed upstream in 0.40, breaking the build. Fixed in #65. | Medium — dependency drift risk if deferred again | Fixed (#65) | Repo owner |
| 4 | No retroactive ADRs existed for `tract-onnx` and `tree-sitter`, both structural dependencies with limited alternatives. | Medium — SPOF with no documented fallback | Fixed (ADR-001, ADR-002, this change) | Repo owner |
| 5 | 15+ commits (mostly dependency bumps, including a RUSTSEC fix) accumulated on `main` after the v2.4.0 tag with no release cut. | Low — delays availability of fixes to crates.io users | Addressed by v2.4.1 | Repo owner |
| 6 | No `runbooks/` for operational issues (crates.io publish failure, CI outage, etc.) — Area E (Legacy Fit) scored 2/5. | Low — this is a CLI/library, not a hosted service, so impact is limited | Open | Repo owner |
