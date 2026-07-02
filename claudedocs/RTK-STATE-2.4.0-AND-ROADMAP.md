# RTK — Stato v2.4.0 + Voti + Roadmap

**Data:** 2026-07-02 · **Versione:** 2.4.0 (rilasciata: GitHub Release + Homebrew + crates.io)
**Riferimenti:** audit iniziale `claudedocs/RTK-AUDIT-2026-07-01.md` · `CHANGELOG.md` [2.4.0]

Documento di handoff: dove siamo e cosa fare la prossima volta.

---

## 1. Cosa è stato fatto (sessione audit → release 2.4.0)

Tesi guida (dall'audit): **il valore unico di RTK è l'efficienza token (filtri/pack/DLP), non il grafo** (che sovrappone gitnexus). Tutto il lavoro è andato in quella direzione.

**8 filtri nuovi** (superficie da ~12 → 20 famiglie di comando):
- Python: `ruff check`, `mypy`, `pip install`
- Frontend Vue/TS: `eslint`, `tsc`, `vitest`
- Ops: `docker ps`, `gh pr checks`

**Hardening / fix:**
- DLP: +prefissi `github_pat_`/`gho_`/`glpat-`/`npm_`/`hf_`/`dop_v1_` (GCP già via PEM).
- pack `--skeleton`: supporto Vue SFC (`<script>`).
- Robustezza: rimossi panic prod (`dotnet`, `setup`), `anyhow` 1.0.103 (RUSTSEC-2026-0190).
- **Bug correttezza:** leading-flags (`rtk pytest --tb=short` ecc. rompeva clap + rewrite hook) → `allow_hyphen_values` su tutte le varianti.
- Routing `docker` (prima *tutto* andava al filtro build) → ps/build/run separati.
- Flake CI Windows (config/pricing env-race) → lock condiviso.

**Prove:** `scripts/benchmark.py` (tiktoken cl100k_base) esteso a tutti i filtri; righe REAL rinfrescate. Medie: **Phase 1 81.9%**, **overall 81.8%** (25 scenari, range 27–97%).

**Qualità:** `cargo fmt` ✅ · `clippy -D warnings` ✅ · `cargo test --workspace` **217 pass** · `cargo audit` pulito.

**Decisione registrata:** *niente feature-gate del grafo* — il peso (8.9MB) non è nel grafo ma in tree-sitter (pack) + sqlite (db), entrambi core; peso non è priorità (feature/validità sì).

---

## 2. Voti professionali per sezione (baseline per la prossima sessione)

| Sezione | Voto | Note |
|---|:---:|---|
| Filtri / efficienza token *(core)* | **9.0 — A** | 20 comandi, savings tiktoken-provati, test snapshot+savings su fixture reali, fallback. Eccellente. |
| DLP redaction | **9.0 — A** | Ampia copertura chiavi + entropia + PEM + URL-creds + audit-log + CDATA-safe + integrato in pack. |
| Pack (context packaging) | **8.5 — A-** | Ignore solidi, skeleton AST 7 lang + Vue, DLP-integrato. `--limit` è whitespace non BPE. |
| Docs | **8.5 — A-** | README con metodologia trasparente, MANUALE, CHANGELOG, QUICKSTART/USER. |
| Robustezza / error-handling | **8.5 — A-** | Zero panic prod, fallback-to-raw, exit-code propagati, leading-flags ok, no async. |
| Architettura / code quality | **8.5 — A-** | Crate separati, Rust idiomatico, regex lazy, clippy pulito, no unwrap prod. Binario 8.9MB. |
| Sicurezza | **8.5 — A-** | No shell-injection (arg arrays), deny/guardrails, hook integrity SHA-256, audit pulito. |
| Testing | **8.0 — B+** | 217 test, snapshot+savings+fixture reali, routing/rewrite. CLI glue più leggero ma migliorato. |
| CI / Release | **8.0 — B+** | Matrix 3-OS + CodeQL + cargo-audit + benchmark_gate + release/homebrew/crates.io. (CRLF script ora risolto.) |
| Memory / tracking / analytics | **8.0 — B+** | SQLite FTS, metriche token, stats/gain, modello costi. |
| MCP (13 tool) | **7.0 — B** | Completo ma legato al grafo → sovrappone gitnexus. |
| Graph / Index | **6.5 — B-** | Funziona ma duplica gitnexus, archi name-based (`callee_file_path`), deps pesanti. Non è il valore. |

### **Complessivo: 8.4 / 10 — A-** (da 8.0 all'inizio)
Core (token) = A. Freno = grafo/MCP (B-/B), coerente con la tesi audit.

---

## 3. Roadmap — prossime migliorie (in ordine di ROI)

### Alta priorità
1. **Decidere il destino del grafo** *(mai fatto — §2 del vecchio piano)*: validazione sul campo dei 13 tool MCP vs gitnexus in un worktree isolato di cyclelab. Criterio: RTK risponde offline a "chi chiama X / cosa rompo / flusso" quanto gitnexus? Esito → tenere e migliorare, oppure congelare definitivamente.
2. **Feedback reale sui filtri v2.4.0**: raccogliere savings effettivi in uso (via `rtk stats`/tracking) sui progetti veri prima di aggiungere altro.

### Media priorità
3. **`pack --limit` con tokenizer BPE reale** (oggi conteggio whitespace) — budget token più accurato. Attenzione: tokenizer aggiunge peso al binario.
4. **Precisione archi grafo** (`callee_file_path`) — *solo se* si tiene il grafo: è il residuo che abbassa impact/flow.
5. **Filtri opzionali on-demand** (se emergono dall'uso): `kubectl`, `poetry`/`uv`, `jest`, `rspec`/`rubocop`. Non aggiungere senza necessità reale.

### Bassa priorità / nice-to-have
6. **HTML graph viewer** clean-room (`rtk graph export --format html`) — solo se il grafo resta e serve visuale. NON copiare la web UI gitnexus (PolyForm Noncommercial).
7. **Ridurre iterazioni `benchmark_real`** (oggi N=10 su cargo test = 30+ min, annidato): portare a N=3 o cache, così il re-baseline è pratico.
8. **`--limit`/pack**: skeleton per altri linguaggi se richiesti.

### Vincoli fissi (non violare)
- **Legale:** GitNexus = PolyForm Noncommercial → mai leggere/portare il loro codice (anche TS→Rust = opera derivata). Solo clean-room.
- **Filtri:** ogni nuovo filtro = fixture **reale** + test snapshot + test savings + riga in `benchmark.py` + riga README + wiring (cli/dispatch/rewrite) con `allow_hyphen_values`.
- **Release:** script hanno storia CRLF (checkout `/mnt/c`); `.gitattributes` ora forza LF. Publish crates.io = `bash scripts/publish-crates.sh` (dep order); irreversibile.

---

## 4. Come aggiungere un filtro (ricetta consolidata)
1. Cattura output **reale** del tool → `rtk-filters/tests/fixtures/<tool>.txt` (sanifica path/segreti).
2. `rtk-filters/src/<tool>_filter.rs`: `filter(&str)->String`, regex lazy, fallback-to-raw, unit test + `token_savings` test.
3. Wiring: `lib.rs` (mod) · `cli.rs` (variante `#[arg(trailing_var_arg, allow_hyphen_values)]`) · `dispatch.rs` (arm, filtra solo il subcomando verboso) · `rewrite.rs` (mapping + negativi).
4. Prova: `benchmark.py` mock pair + riga README + medie.
5. `fmt` + `clippy -D warnings` + `cargo test` + e2e reale.
