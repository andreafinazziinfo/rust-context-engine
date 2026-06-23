# RTK Context Engine — Piano migliorie verso 9+

| Campo | Valore |
|-------|--------|
| **Repo** | `rust-context-engine` (workspace `rtk/`) |
| **Baseline audit** | HEAD `a3c8258` · release / crates.io **v2.3.0** |
| **Voto attuale** | **8.6 / 10** (post-S3, 2026-06-23) |
| **Sprint 3 chiuso** | 2026-06-23 — PACK/DB/FILT/GRD/ARCH-3 |
| **Obiettivo** | **≥ 9.0** su ogni sezione entro 4 sprint |
| **Creato** | 2026-06-23 |
| **Revisione piano** | 2026-06-23 (review strutturale + allineamento codebase) |
| **Sprint 1 chiuso** | 2026-06-23 — commit `s1-fondamenta` (10/11 task; CI-1 slittato S2) |

---

## Indice

1. [Executive summary](#1-executive-summary)
2. [Scorecard](#2-scorecard-attuale--target)
3. [Roadmap sprint](#3-roadmap-sprint)
4. [Prerequisiti dev (Sprint 1)](#40-prerequisiti-dev-locale--obbligatori)
5. [Dettaglio per area](#5-dettaglio-per-area)
6. [Backlog trasversale](#6-backlog-trasversale)
7. [Runbook esecuzione](#7-runbook-esecuzione)
8. [Definition of Done](#8-definition-of-done-globale)
9. [Registro task unificato](#9-registro-task-unificato)
10. [Riferimenti audit](#10-riferimenti-audit)

**Legenda effort:** S = ≤1 giorno · M = 2–4 giorni · L = ≥1 settimana  
**Legenda priorità:** P0 = blocker · P1 = alto impatto · P2 = qualità · P3 = polish

---

## 1. Executive summary

RTK è un prodotto **reale e ben archittettato** (6 crate, ~12k LOC, CI attiva, release multi-piattaforma). I gap verso 9+ non sono di visione ma di:

- **Onestà documentale** (claim ONNX/semantic vs build default)
- **Affidabilità operativa** (index lazy, guardrail, DLP)
- **Gate qualità dev** (WSL build path, test pricing fail)
- **CI cross-platform** (oggi solo Ubuntu)

### 1.1 Stato ambiente verificato (2026-06-23)

| Ambiente | Toolchain | RTK in uso | Build from source | Note |
|----------|-----------|------------|-------------------|------|
| **WSL Ubuntu** | cargo/rustc 1.95 | build `~/dev/.../target/debug/rtk` | ✅ `~/dev/rust-context-engine` | fmt ✅ · clippy ✅ · test ✅ · `dev-gate.sh` |
| **Windows** | cargo/rustc 1.96 | `~/.rtk-bin/rtk.exe` **v2.3.0** | ❌ locale (link.exe coreutils) | PATH OK · `index run` → 401 simboli |

### 1.2 Regole operative

| Attività | Dove | Obbligatorio? |
|----------|------|:-------------:|
| **Sviluppo / test / pre-merge** | WSL, repo in `~/dev/rust-context-engine` | **Sì** |
| **Uso quotidiano RTK su Windows** | Release zip → `%USERPROFILE%\.rtk-bin` | **Sì** |
| **Build nativo Windows (MSVC)** | Solo se si vuole `cargo build` fuori WSL | **No** |
| **CI Windows/macOS** | GitHub Actions (MSVC preinstallato sui runner) | Automatico in CI-1 |

> **Anti-pattern:** `cargo build` / `cargo test` con `target/` sotto `/mnt/c/...` → OOM e I/O lento.

---

## 2. Scorecard attuale → target

| Sezione | Attuale | Target | Sprint |
|---------|:-------:|:------:|:------:|
| Architettura workspace | 8.5 | 9.5 | S1 |
| Filtri input | 8.5 | 9.0 | S2–S3 ✅ |
| DLP / redaction | 7.5 | 9.0 | S1 ✅ |
| Rewrite / guardrail | 8.0 | 9.0 | S1–S3 ✅ |
| DB / tracking / memory | 8.5 | 9.0 | S3 ✅ |
| Index AST / graph | 8.5 | 9.5 | S1–S2 ✅ |
| MCP server | 8.0 | 9.0 | S2 ✅ |
| Pack / skeleton | 8.5 | 9.0 | S3 ✅ |
| Pricing / FinOps | 8.0 | 9.0 | S1 ✅ |
| Dashboard / telemetry | 8.0 | 9.0 | S2 ✅ |
| Testing | 8.5 | 9.5 | S1–S2 ✅ |
| CI/CD | 8.0 | 9.5 | S1–S2 |
| Docs / README | 8.0 | 9.5 | S1–S2 ✅ |

**Definizione “9+” per sezione:** zero bug P0; test automatici su casi critici; README/CLI allineati al build default; CI verde; nessun claim non verificabile.

---

## 3. Roadmap sprint

### 3.1 Panoramica

| Sprint | Focus | Exit criteria (sintesi) |
|--------|-------|-------------------------|
| **S1 Fondamenta** | Dev gate WSL · fix pricing · lazy index · DLP/GRD · MCP version · Cargo.lock · docs onesti | ✅ Gate verde · IDX-1/2 · FIN-1 · MCP-1 · DOC-1 · DLP-1 · GRD-1 · ARCH-1 |
| **S2 Affidabilità** | MCP tests · memory · filtri golden · CI matrix | ✅ |
| **S3 Qualità** | tiktoken · pack limits · git show/branch · GC throttle · strict_chained | ✅ PACK-1/2 · DB-2/3 · FILT-2/3 · GRD-2 · ARCH-3 |
| **S4 Polish** | doctor · benchmark gate · release smoke · graph UX | DoD §8 completo |

### 3.2 Sprint 1 — ordine di esecuzione (P0 → P1)

| Ordine | ID | Priorità | Descrizione | Status |
|:------:|----|:--------:|-------------|:------:|
| 1 | DEV-WSL-1 | P0 | Setup build WSL su `~/dev` | ✅ |
| 2 | FIN-1 | P0 | Fix `pricing::test_calculate_cost` (+ gate DEV-WSL-2) | ✅ |
| 3 | ARCH-1 | P0 | Commit `Cargo.lock` | ✅ |
| 4 | MCP-1 | P1 | Version MCP = `CARGO_PKG_VERSION` | ✅ |
| 5 | IDX-1 | P1 | Lazy auto-index | ✅ |
| 6 | IDX-2 | P1 | `rtk index status` | ✅ |
| 7 | DLP-1 | P1 | Fix bypass base64 `=` | ✅ |
| 8 | GRD-1 | P1 | Deny su comandi chained | ✅ |
| 9 | DOC-1 | P1 | README “Default vs Full build” | ✅ |
| 10 | CI-1 | P2 | Matrix OS (slittato → S2) | ✅ |

### 3.4 Sprint 3 — ordine eseguito (2026-06-23)

| Ordine | ID | Descrizione | Status |
|:------:|----|-------------|:------:|
| 1 | PACK-1 | `--limit` usa `count_tokens` centralizzato | ✅ |
| 2 | PACK-2 | Test DLP pack full + skeleton | ✅ |
| 3 | DB-3 | GC throttled 24h su `record()` | ✅ |
| 4 | DB-2 | Feature `tiktoken` opzionale su `count_tokens` | ✅ |
| 5 | FILT-2 | Filtri `git show`, `git branch -v` | ✅ |
| 6 | FILT-3 | Tabella filtered vs passthrough in `cli.md` | ✅ |
| 7 | GRD-2 | `strict_chained` in `.rtk.json` | ✅ |
| 8 | ARCH-3 | Tabella env in `configuration.md` | ✅ |
| 9 | ARCH-2 | Estrarre filter pipeline | ✅ S4 |

### 3.5 Sprint 4 — in corso (2026-06-23)

| Ordine | ID | Descrizione | Status |
|:------:|----|-------------|:------:|
| 1 | FILT-4 | `get_profile_for_cmd` in pipeline | ✅ |
| 2 | DOCTOR-1 | `rtk doctor` esteso + exit 0/1/2 | ✅ |
| 3 | ARCH-2 | `filter_pipeline.rs` estratto | ✅ |
| 4 | CI-3 | Gate `token_savings` in CI + script | ✅ |
| 5 | REL-1 | Release smoke / install parity | ✅ |
| 6 | GRAPH-1 | Graph UX export/audit polish | ✅ |

**Regola merge Sprint 1:** nessuna PR feature senza **DEV-WSL-2** verde (fmt + clippy + test).

---

## 4.0 Prerequisiti dev locale — obbligatori

Obbligatori **prima** di ogni patch Sprint 1 e **prima di ogni merge** su `main`.

### DEV-WSL-1 — Build from source WSL

| | |
|---|---|
| **Problema** | Build su `/mnt/c/.../target` → `Cannot allocate memory (os error 12)` |
| **Causa** | NTFS via 9p + target directory su mount Windows |
| **Soluzione** | Repo e `target/` su filesystem Linux ext4 WSL |

**Setup (una tantum):**

```bash
# A — clone (consigliato)
mkdir -p ~/dev && cd ~/dev
git clone https://github.com/andreafinazziinfo/rust-context-engine.git
cd rust-context-engine/rtk

# B — sync da Windows (repo già su /mnt/c)
mkdir -p ~/dev/rust-context-engine
rsync -a --exclude target --exclude .rtk \
  /mnt/c/Users/Andrea/dev/rust-context-engine/ ~/dev/rust-context-engine/
cd ~/dev/rust-context-engine/rtk
```

**Acceptance criteria:**

- [x] `cargo build --release` exit 0 in `~/dev/rust-context-engine/rtk`
- [x] `./target/release/rtk status` risponde
- [x] Nessun `target/` sotto `/mnt/c/` usato per dev/test
- [x] Nota in `docs/src/installation.md` — sezione “Sviluppo WSL”

**Effort:** S · **Blocker:** sì

---

### DEV-WSL-2 — Quality gate (fmt / clippy / test)

Allineato a `.github/workflows/ci.yml` (con `--all-targets` aggiuntivo consigliato in locale).

```bash
cd ~/dev/rust-context-engine/rtk

cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --workspace
```

| ID | Task | Acceptance |
|----|------|------------|
| DEV-WSL-2 | Gate completo | Tutti e 3 comandi exit 0 |
| FIN-1 | Fix pricing test (sotto-task obbligatorio) | `cargo test -p rtk-context-db` verde |
| DEV-WSL-2-script | (Opz.) `scripts/dev-gate.sh` | Un comando = gate completo |

**Fail attivo noto:** ~~`pricing::tests::test_calculate_cost`~~ **risolto (FIN-1)** — side-effect `.rtk_pricing.json` da `test_pricing_overrides`.

**Effort:** S · **Blocker:** sì (FIN-1)

---

## 5. Dettaglio per area

### 5.1 Architettura workspace (8.5 → 9.5)

**Problemi:** `main.rs` ~1100 LOC; `Cargo.lock` assente; env var DB duplicate.

| ID | Task | File | Acceptance |
|----|------|------|------------|
| ARCH-1 | Commit `Cargo.lock` | `rtk/Cargo.lock` | `cargo build --locked` in CI |
| ARCH-2 | Estrarre filter pipeline | `filter_pipeline.rs` | `main.rs` ≤ 800 LOC; test OK |
| ARCH-3 | Tabella env unificata | `docs/src/configuration.md` | RTK_DB_PATH, RTK_PROJECT_DB_PATH, RTK_INDEX_DB_PATH |
| ARCH-4 | Documentare feature flags | README, `rtk-index/Cargo.toml` | embeddings, ort, onnx-* |

**Effort:** M · **Sprint:** S1–S2

---

### 5.2 Filtri input (8.0 → 9.0)

**Problemi:** copertura git/cargo parziale; npm/yarn via distill generico; pochi golden test.

| ID | Task | Acceptance |
|----|------|------------|
| FILT-1 | Golden fixtures top-5 comandi (insta) | Snapshot committed |
| FILT-2 | Filter `git show`, `git branch -v` | Savings ≥ 40% su fixture |
| FILT-3 | Tabella filtered vs passthrough | `docs/src/cli.md` |
| FILT-4 | Wire `get_profile_for_cmd()` in pipeline | Profilo cambia output misurabile |

**Effort:** M · **Sprint:** S2

---

### 5.3 DLP / redaction (6.5 → 9.0)

**Evidenza** (`rtk-db/src/dlp.rs`):

```rust
let is_base64_like = word.ends_with("==") || word.ends_with('=');
if !is_git_hash && !is_base64_like { /* entropy */ }
```

Secret base64 con padding `=` → **bypass**.

| ID | Task | Acceptance |
|----|------|------------|
| DLP-1 | Regex base64 + entropy (no skip cieco su `=`) | Test secret `YWJj...=` redatto |
| DLP-2 | JWT regex `eyJ[A-Za-z0-9-_=]+` | Test varianti header |
| DLP-3 | E2E `config dlp add` → filtro output | Integration test |
| DLP-4 | Limiti documentati | `docs/src/limitations.md` |
| DLP-5 | (Opz.) `config dlp audit show` | Tail hash-only audit.log |

**Effort:** S · **Sprint:** S1

---

### 5.4 Rewrite / guardrail (6.0 → 9.0)

**Problema:** deny ancorato a `^`; chained commands → passthrough → `foo && rm -rf /` non bloccato.

| ID | Task | Acceptance |
|----|------|------------|
| GRD-1 | `split_segments()` + deny per segmento | 10 test chained |
| GRD-2 | Config `strict_chained: true` → deny passthrough | `config.json` |
| GRD-3 | Doc hook + troubleshooting | `hooks/rtk-rewrite.sh`, README |

**Effort:** M · **Sprint:** S1

---

### 5.5 DB / tracking / memory (7.5 → 9.0)

| ID | Task | Acceptance |
|----|------|------------|
| DB-1 | Preservare `created_at` su memory update | Test overwrite |
| DB-2 | Feature `tiktoken` per `count_tokens` | Stats ±5% vs benchmark.py |
| DB-3 | GC throttled (max 1/24h) | Test 100 record |
| DB-4 | CLI help: “FTS5 search” (no claim ONNX default) | README |
| DB-5 | Migration test schema legacy | Fresh + old DB |

**Effort:** M · **Sprint:** S2

---

### 5.6 Index AST / graph (7.0 → 9.5) — priorità utente

**Problema critico:** nessun lazy index → `symbols/refs/impact` vuoti finché non si esegue `rtk index run`.

| ID | Task | Acceptance |
|----|------|------------|
| IDX-1 | Lazy index se count=0 o stale >24h | Fresh clone: `symbols find` OK senza `index run` |
| IDX-2 | `rtk index status` (human + JSON) | symbols, edges, last_indexed, coverage |
| IDX-3 | Single reload in `analyze_impact` | Latency −30% |
| IDX-4 | Parser Java | Fixture `.java` |
| IDX-5 | (S3) File watcher o doc-only | YAGNI default: doc in doctor |

**Effort:** L · **Sprint:** S1 (IDX-1/2 P1)

---

### 5.7 MCP server (6.5 → 9.0)

| ID | Task | Acceptance |
|----|------|------------|
| MCP-1 | `env!("CARGO_PKG_VERSION")` in initialize | `rtk mcp ping` = 2.3.0 |
| MCP-2 | `search_code` via index DB, fallback grep | < 500ms su repo RTK |
| MCP-3 | Tool `impact_analyze` | Parity CLI |
| MCP-4 | JSON-RPC test per ogni tool | ≥8 test in CI |
| MCP-5 | README tabella tool → requisiti | No “semantic” senza feature |

**Effort:** M · **Sprint:** S1 (MCP-1) · S2 (resto)

---

### 5.8 Pack / skeleton (7.5 → 9.0)

**Nota review:** DLP **già applicato** in `rtk-pack/src/pack.rs` (`redact_with_source`). PACK-2 = estendere coverage test, non implementazione da zero.

| ID | Task | Acceptance |
|----|------|------------|
| PACK-1 | `--limit` usa `count_tokens` centralizzato | Coerente con stats |
| PACK-2 | Test DLP pack (API key fake) | No leak in stdout |
| PACK-3 | Doc `--strip` + `--skeleton` | docs |
| PACK-4 | Benchmark gate pack self | CI threshold |

**Effort:** S · **Sprint:** S3

---

### 5.9 Pricing / FinOps (7.0 → 9.0)

**Blocker attivo:** ~~`test_calculate_cost` fail~~ **risolto (FIN-1, 2026-06-23)**.

| ID | Task | Acceptance |
|----|------|------------|
| FIN-1 | Fix test o isolamento pricing load | `cargo test -p rtk-context-db` verde |
| FIN-2 | Proptest savings = cost diff | 1000 casi |
| FIN-3 | Golden test `rtk estimate` | Snapshot stable |
| FIN-4 | Warn model env assente | Doc fallback tier |
| FIN-5 | `schema_version` in pricing JSON | v1 |

**Effort:** S · **Sprint:** S1

---

### 5.10 Dashboard / telemetry (7.0 → 9.0)

**Bug confermato:** `top_saver` e `most_frequent` usano entrambi `breakdown.first()`.

| ID | Task | Acceptance |
|----|------|------------|
| DASH-1 | Fix aggregazione top_saver vs most_frequent | JSON campi distinti |
| DASH-2 | Test HTTP `/api/stats` | 200 + schema valido |
| DASH-3 | Prometheus labels opzionali | promtool check |
| DASH-4 | Doc localhost-only | CLI help |

**Effort:** S · **Sprint:** S3

---

### 5.11 Testing (7.5 → 9.5)

| ID | Task | Sprint | Acceptance |
|----|------|--------|------------|
| DEV-WSL-1 | WSL build path | S1 | §4.0 |
| DEV-WSL-2 | Gate fmt/clippy/test | S1 | §4.0 |
| TST-2 | MCP integration tests | S2 | 8 tool |
| TST-3 | `scripts/e2e_smoke.sh` | S2 | rewrite→filter→show-log |
| TST-4 | Windows CI (runner GH, non locale) | S1–S2 | Green windows-latest |
| TST-5 | Coverage opzionale 70% db+filters | S3 | Artifact CI |
| TST-6 | FTS concurrent test | S2 | No DB locked |

**Effort:** L · **Sprint:** S1–S2

---

### 5.12 CI/CD (7.0 → 9.5)

| ID | Task | Acceptance |
|----|------|------------|
| CI-1 | Matrix ubuntu + windows + macos | All green |
| CI-2 | `cargo build --release --locked` | Cargo.lock enforced |
| CI-3 | Benchmark regression job | Fail se savings < threshold |
| CI-4 | Release smoke post-upload | `rtk --help`, `mcp ping` |
| CI-5 | Ridurre RUSTSEC ignore | ≤1 temporaneo, documentato |
| CI-6 | CodeQL path filter `rtk/**` | Meno noise |

> **Nota:** CI Windows **≠** build locale Windows. I runner GitHub hanno MSVC; lo sviluppatore usa WSL o release zip.

**Effort:** M · **Sprint:** S1–S2

---

### 5.13 Docs / README (6.0 → 9.5)

| Claim README | Realtà default build |
|--------------|---------------------|
| Hybrid BM25 + ONNX memory | FTS5 only (`embeddings` feature off) |
| MCP search_code semantic | Grep lineare filesystem |
| `rtk mcp serve` | CLI: `rtk mcp start` |
| Savings ~81–82% | OK se gated in CI (CI-3) |

| ID | Task | Acceptance |
|----|------|------------|
| DOC-1 | Sezione “Default vs Full build” | Tabella feature cargo |
| DOC-2 | Fix comandi MCP | README + MANUALE.md |
| DOC-3 | Link a questo piano in Contributing | README |
| DOC-4 | limitations.md aggiornato post IDX-1 | Alias bypass, DLP, index |
| DOC-5 | Versioni allineate | crates.io = release = MCP ping |

**Effort:** S · **Sprint:** S1

---

## 6. Backlog trasversale

### 6.1 `rtk doctor` esteso (Sprint 4)

- Version RTK vs crates.io latest
- Warning WSL/Windows DB path split
- Aliases shell + hook PreToolUse
- Index freshness (`index status`)
- Memory doctor · config regex valid · disk `.rtk/`

Exit: 0 OK · 2 warnings · 1 critical

### 6.2 Policy embeddings (Sprint 3)

**Decisione adottata: Opzione A** — embeddings optional; docs onesti; MCP search via index+FTS.  
Opzione B (default ONNX) solo su richiesta esplicita (+15MB binary).

### 6.3 Release parity (Sprint 4)

- `cargo install` version = release zip = MCP initialize
- `install.sh` WSL → aggiorna `~/.local/bin/rtk` da linux-amd64
- Homebrew formula smoke

---

## 7. Runbook esecuzione

### 7.1 Sprint 1 — sequenza

```bash
# DEV-WSL-1 (una tantum)
mkdir -p ~/dev && cd ~/dev
git clone https://github.com/andreafinazziinfo/rust-context-engine.git
cd rust-context-engine/rtk

# DEV-WSL-2 (ogni PR)
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --workspace

# Proof release build
cargo build --release && ./target/release/rtk status
```

### 7.2 Windows — uso operativo

```powershell
# %USERPROFILE%\.rtk-bin\rtk.exe  (v2.3.0)
rtk index run
rtk pack rtk/rtk-filters/src --strip --limit 8000
```

### 7.3 Checklist pre-PR

- [x] Lavoro su `~/dev/rust-context-engine` (DEV-WSL-1)
- [x] Gate fmt + clippy + test verde (DEV-WSL-2)
- [x] Task ID §9 aggiornati
- [x] Nessun claim README nuovo non verificato (DOC-1)

---

## 8. Definition of Done globale

Progetto **9+** quando tutti veri:

| # | Criterio |
|---|----------|
| 0 | Gate DEV-WSL-1/2 documentato e usato | ✅ |
| 1 | `cargo test --workspace` verde su Ubuntu + Windows CI | 🔄 (locale ✅, CI-1 pending) |
| 2 | README allineato al build default | ✅ |
| 3 | Lazy index: `symbols find` out-of-box | ✅ |
| 4 | DLP base64 + deny chained verdi | ✅ |
| 5 | MCP version = crate version; 8 tool testati | 🔄 (version ✅, MCP-4 pending) |
| 6 | `Cargo.lock` + `cargo build --locked` | 🔄 (lock ✅, CI-2 pending) |
| 7 | Benchmark regression gate CI |
| 8 | `rtk doctor` OK post-`init --profile high` | 🔄 (doctor ✅, smoke manuale pending) |

---

## 9. Registro task unificato

Aggiornare a ogni merge. **FIN-1** è il fix pricing; non duplicare con alias separati in tracking.

| ID | P | Sprint | Status | Note |
|----|:-:|--------|--------|------|
| DEV-WSL-1 | 0 | S1 | ✅ | build ~/dev |
| DEV-WSL-2 | 0 | S1 | ✅ | gate fmt/clippy/test |
| FIN-1 | 0 | S1 | ✅ | pricing test tempdir |
| ARCH-1 | 0 | S1 | ✅ | Cargo.lock |
| MCP-1 | 1 | S1 | ✅ | CARGO_PKG_VERSION |
| IDX-1 | 1 | S1 | ✅ | ensure_index_fresh |
| IDX-2 | 1 | S1 | ✅ | index status |
| DLP-1 | 1 | S1 | ✅ | base64 entropy |
| GRD-1 | 1 | S1 | ✅ | chained deny |
| DOC-1 | 1 | S1 | ✅ | README default vs full |
| DEV-WSL-2-script | 1 | S1 | ✅ | scripts/dev-gate.sh |
| CI-1 | 2 | S1–S2 | ✅ | Matrix ubuntu/windows/macos |
| CI-2 | 2 | S1–S2 | ✅ | build --release --locked in CI |
| ARCH-2 | 2 | S4 | ✅ | filter_pipeline.rs |
| FILT-4 | 2 | S4 | ✅ | apply_profile_settings in pipeline |
| DOCTOR-1 | 2 | S4 | ✅ | doctor esteso exit 0/1/2 |
| FILT-1 | 2 | S2 | ✅ | Golden insta git_status/cargo_* |
| MCP-4 | 2 | S2 | ✅ | 10 test MCP (initialize + tools) |
| DB-1 | 2 | S2 | ✅ | memory_set preserves created_at |
| DASH-1 | 2 | S2 | ✅ | top_saver fix |
| DOC-4 | 2 | S2 | ✅ | limitations.md |
| DB-4 | 2 | S2 | ✅ | README FTS5 |
| IDX-3 | 2 | S2 | ✅ | analyze_impact |
| TST-3 | 2 | S2 | ✅ | e2e_smoke.sh |
| TST-6 | 2 | S2 | ✅ | FTS concurrent |
| TST-4 | 2 | S2 | 🔄 | CI push pending |
| PACK-1 | 2 | S3 | ✅ | pack --limit count_tokens |
| PACK-2 | 2 | S3 | ✅ | DLP pack test |
| DB-2 | 2 | S3 | ✅ | feature tiktoken |
| DB-3 | 2 | S3 | ✅ | GC throttle 24h |
| FILT-2 | 2 | S3 | ✅ | git show / branch -v |
| FILT-3 | 2 | S3 | ✅ | cli.md table |
| GRD-2 | 2 | S3 | ✅ | strict_chained |
| ARCH-3 | 2 | S3 | ✅ | env var table |
| CI-3 | 2 | S4 | ✅ | token_savings gate + benchmark_gate.sh |
| REL-1 | 2 | S4 | ✅ | release_smoke.sh + install --prebuilt |
| GRAPH-1 | 2 | S4 | ✅ | graph audit hints + index.md export |

⬜ TODO · 🔄 IN PROGRESS · ✅ DONE

---

## 10. Riferimenti audit

| Tipo | Dettaglio |
|------|-----------|
| Audit chat | 2026-06-23 · score 7.2/10 |
| File critici | `dlp.rs`, `rewrite.rs`, `rtk-index/lib.rs`, `rtk-mcp/lib.rs`, `pricing.rs` |
| Proof Windows | `rtk index run` → 401 symbols |
| Proof WSL | fmt ✅ clippy ✅ test ✅ · `dev-gate.sh` · `e2e_smoke.sh` |
| Piano review | 2026-06-23 — dedup FIN-1, PACK DLP verificato, CI vs locale chiarito |

---

## Changelog documento

| Data | Modifica |
|------|----------|
| 2026-06-23 | Creazione iniziale |
| 2026-06-23 | Sprint 2 chiuso (core): §3.3, scorecard 8.2, §9 aggiornato |
| 2026-06-23 | Sprint 1 chiuso: §9 ✅, scorecard aggiornato |

---

*Documento vivo — aggiornare §9 ad ogni task completato.*
