# RTK → Rust Context Engine: Piano di Implementazione Completo

Questo piano traduce la visione di [rtk_complete_plan_june_2026.md](file:///C:/Users/Andrea/dev/rust-context-engine/rtk_complete_plan_june_2026.md) in azioni concrete e verificabili sul codebase RTK. È organizzato per priorità (P0→P3) e suddiviso in moduli.

---

## Stato Attuale vs Target

### Cosa esiste già ✅

| Componente | File | Stato |
|---|---|---|
| Shell wrappers (git, cargo, npm, pytest, docker, go, gradle, dotnet, ls) | [main.rs](file:///c:/Users/Andrea/dev/rust-context-engine/rtk/rtk-cli/src/main.rs) | ✅ Completo |
| Content-aware filters (10 filtri) | [rtk-filters/src/](file:///c:/Users/Andrea/dev/rust-context-engine/rtk/rtk-filters/src) | ✅ Completo |
| DLP redaction engine | [dlp.rs](file:///c:/Users/Andrea/dev/rust-context-engine/rtk/rtk-memory/src/dlp.rs) | ✅ Completo |
| `rtk think` (hidden reasoning) | [think.rs](file:///c:/Users/Andrea/dev/rust-context-engine/rtk/rtk-memory/src/think.rs) | ✅ Base |
| `rtk memory` (FTS5 project memory) | [tracking.rs](file:///c:/Users/Andrea/dev/rust-context-engine/rtk/rtk-memory/src/tracking.rs) L303-L377 | ✅ Completo |
| `rtk pack` + `rtk skeleton` | [rtk-pack/src/](file:///c:/Users/Andrea/dev/rust-context-engine/rtk/rtk-pack/src) | ✅ Completo |
| `rtk audit` (cost projection report) | [tracking.rs](file:///c:/Users/Andrea/dev/rust-context-engine/rtk/rtk-memory/src/tracking.rs) L379-L474 | ✅ Completo |
| `rtk stats` | [tracking.rs](file:///c:/Users/Andrea/dev/rust-context-engine/rtk/rtk-memory/src/tracking.rs) L200-L231 | ✅ Completo |
| `rtk dashboard --live` (glassmorphic web UI) | [dashboard.rs](file:///c:/Users/Andrea/dev/rust-context-engine/rtk/rtk-cli/src/dashboard.rs) | ✅ Completo |
| `rtk rewrite` (command rewriter) | [rewrite.rs](file:///c:/Users/Andrea/dev/rust-context-engine/rtk/rtk-cli/src/rewrite.rs) | ✅ Completo |
| Plugin system (TOML-based) | [plugins.rs](file:///c:/Users/Andrea/dev/rust-context-engine/rtk/rtk-cli/src/plugins.rs) | ✅ Completo |
| `rtk init` + `rtk config` | [setup.rs](file:///c:/Users/Andrea/dev/rust-context-engine/rtk/rtk-cli/src/setup.rs) | ✅ Completo |
| Telemetry: model, project, branch, duration_ms | [tracking.rs](file:///c:/Users/Andrea/dev/rust-context-engine/rtk/rtk-memory/src/tracking.rs) L70-L75 | ✅ Completo |
| Benchmark tool (Python, multi-model pricing) | [benchmark.py](file:///c:/Users/Andrea/dev/rust-context-engine/scripts/benchmark.py) | ✅ Completo |
| mdBook documentation site | [docs/](file:///c:/Users/Andrea/dev/rust-context-engine/docs) | ✅ Base |
| CI/CD (GitHub Actions) | [.github/](file:///c:/Users/Andrea/dev/rust-context-engine/.github) | ✅ Completo |

### Gap da colmare 🔴

| Gap | Richiesto dal piano | Priorità |
|---|---|---|
| **Pricing registry** (`data/model_pricing.json`) | Sezione D — single source of truth per pricing | P0 |
| **Cost calculator condiviso** (Rust module) | Sezione D — evita duplicazione pricing in audit/stats/dashboard | P0 |
| **Benchmark JSON/CSV export** | Sezione D — output strutturato per prove | P0 |
| **`rtk doctor`** (health check) | Sezione F — verifica installazione e configurazione | P0 |
| **`rtk session-state`** (init/get/update/export) | Sezione C — session handoff e anchored summaries | P0 |
| **`rtk agents init/doctor/compact`** | Sezione C — manutenzione AGENTS.md/CLAUDE.md | P0 |
| **Output profiles** (`.rtk.json` presets) | Sezione B — strict/balanced/developer/audit/json-only | P1 |
| **`rtk artifact` manager** | Sezione B — artifact spillover generalizzato | P1 |
| **Crate `rtk-index`** (Tree-sitter + petgraph) | Sezione B — code graph ibrido, AST indexing | P1 |
| **`rtk symbols/deps/refs/impact/search`** | Sezione B — symbol navigation CLI | P1 |
| **`docs/pricing.md`** | Sezione E — pricing methodology doc | P1 |
| **`docs/benchmarks.md`** | Sezione E — benchmark results doc | P1 |
| **`docs/use-cases.md`** | Sezione E — scenario documentation | P1 |
| **`docs/limitations.md`** | Sezione E — honest limitations | P1 |
| **Fixtures directory** (`fixtures/`) | Sezione E — reproducible golden test data | P1 |
| **MCP server** (crate `rtk-mcp`) | Sezione B — tool surface minima in Rust | P2 |
| **Hybrid retrieval** (BM25 + optional ONNX embeddings) | Sezione C — semantic search layer | P2 |
| **Budget caps & alerts** | Sezione D — spending thresholds | P2 |
| **Model routing helpers** | Sezione D — smart model selection | P2 |
| **`rtk memory doctor/overwrite`** | Sezione C — memory discipline | P2 |
| **Optional ONNX embeddings** | Aggiunta 1.1 — semantic similarity | P3 |
| **Obsidian graph export** | Aggiunta 1.2 — graph visualization | P3 |
| **`rtk audit graph`** | Aggiunta 1.3 — index audit metrics | P3 |

---

## P0 — Primi 30 giorni (Foundation & Data Layer)

> [!IMPORTANT]
> P0 è la fase critica: stabilisce la single source of truth per il pricing, consolida il data layer, e aggiunge i comandi che trasformano RTK da "token saver" a "context engine". Senza P0, tutto il resto crolla.

### Component 1: Pricing Registry & Cost Calculator

#### [NEW] [model_pricing.json](file:///c:/Users/Andrea/dev/rust-context-engine/data/model_pricing.json)
- JSON registry con tutti i modelli LLM e i rispettivi prezzi input/output per MTok
- Campi: `model_id`, `provider`, `display_name`, `input_price_per_mtok`, `output_price_per_mtok`, `pricing_revision`, `source_url`, `last_verified`
- Modelli: Claude 4 Opus/Sonnet, Claude 3.5 Sonnet/Opus, GPT-4o/mini, Gemini 2.5 Pro/Flash, DeepSeek V3, Llama 4

#### [MODIFY] [tracking.rs](file:///c:/Users/Andrea/dev/rust-context-engine/rtk/rtk-memory/src/tracking.rs)
- Refactor `get_savings_data()` e `run_audit()` per usare il pricing registry invece di hardcoded `$3.00/MTok`
- Nuovo modulo `pricing.rs` in `rtk-memory` che carica e deserializza `model_pricing.json`
- Funzione `calculate_cost(tokens: i64, model: &str) -> f64` condivisa

#### [NEW] [pricing.rs](file:///c:/Users/Andrea/dev/rust-context-engine/rtk/rtk-memory/src/pricing.rs)
- `load_pricing() -> Result<Vec<ModelPrice>>`
- `get_model_price(model_id: &str) -> Option<ModelPrice>`
- `calculate_savings(tokens_saved: i64, model_id: &str) -> f64`
- Embedded JSON via `include_str!` per zero-dependency at runtime

---

### Component 2: Benchmark Export (JSON/CSV)

#### [MODIFY] [main.rs](file:///c:/Users/Andrea/dev/rust-context-engine/rtk/rtk-cli/src/main.rs)
- Nuovo subcommand: `rtk benchmark export --format json|csv --output <path>`
- Legge dati dal DB e genera export strutturato con metadata (timestamp, commit SHA, tokenizer, pricing_revision)

#### [NEW] [benchmark.rs](file:///c:/Users/Andrea/dev/rust-context-engine/rtk/rtk-cli/src/benchmark.rs)
- `export_json(path: &str) -> Result<()>`
- `export_csv(path: &str) -> Result<()>`
- Schema: `{ metadata: { timestamp, commit_sha, rtk_version, pricing_revision }, results: [...] }`

---

### Component 3: `rtk doctor`

#### [MODIFY] [main.rs](file:///c:/Users/Andrea/dev/rust-context-engine/rtk/rtk-cli/src/main.rs)
- Nuovo subcommand: `rtk doctor`

#### [NEW] [doctor.rs](file:///c:/Users/Andrea/dev/rust-context-engine/rtk/rtk-cli/src/doctor.rs)
- Verifica: DB accessibile, shell hooks installati, profilo attivo, pricing registry presente, versione Rust, disco
- Output: checklist verde/rosso con suggerimenti fix
- `run_doctor() -> Result<()>`

---

### Component 4: `rtk session-state`

#### [MODIFY] [main.rs](file:///c:/Users/Andrea/dev/rust-context-engine/rtk/rtk-cli/src/main.rs)
- Nuovo subcommand group: `rtk session-state init|get|update|export`

#### [NEW] [session.rs](file:///c:/Users/Andrea/dev/rust-context-engine/rtk/rtk-memory/src/session.rs)
- SQLite table `session_state` (project_path, key, value, updated_at)
- `session_init()` — crea session state documento per progetto corrente
- `session_get()` — dump dello stato corrente in JSON
- `session_update(key, value)` — aggiorna un campo specifico
- `session_export()` — produce un documento di handoff completo per nuovo agente/sessione
- Formato output: JSON strutturato con sezione `decisions`, `active_tasks`, `context_files`, `warnings`

---

### Component 5: `rtk agents`

#### [MODIFY] [main.rs](file:///c:/Users/Andrea/dev/rust-context-engine/rtk/rtk-cli/src/main.rs)
- Nuovo subcommand group: `rtk agents init|doctor|compact`

#### [NEW] [agents.rs](file:///c:/Users/Andrea/dev/rust-context-engine/rtk/rtk-cli/src/agents.rs)
- `agents_init(template: &str)` — genera AGENTS.md/CLAUDE.md da template (solo-dev, team, oss, mono-repo)
- `agents_doctor()` — verifica struttura, lunghezza, stale references, chiavi duplicate
- `agents_compact()` — comprime file di regole eliminando ridondanze e linee vuote eccessive

---

### Component 6: `rtk think` enhancements

#### [MODIFY] [think.rs](file:///c:/Users/Andrea/dev/rust-context-engine/rtk/rtk-memory/src/think.rs)
- Aggiungere `think_inspect()` — mostra ultimi N pensieri salvati
- Aggiungere `think_gc()` — elimina pensieri più vecchi di N giorni

#### [MODIFY] [main.rs](file:///c:/Users/Andrea/dev/rust-context-engine/rtk/rtk-cli/src/main.rs)
- Espandere subcommand `Think` con subcommands `inspect` e `gc`

---

## P1 — 30/60 giorni (Indexing, Docs, Credibility)

### Component 7: Crate `rtk-index`

#### [NEW] Crate `rtk-index/`
- Tree-sitter parser per Rust, TypeScript, Python, Go (le 4 lingue più usate dagli agenti)
- Schema petgraph: `Symbol { name, kind, file, line_start, line_end }` + edges `CALLS`, `IMPORTS`, `DEFINES`
- DB persistence in SQLite (separato dal tracking DB)
- Comandi esposti: `index_directory()`, `query_symbols()`, `query_deps()`, `query_refs()`, `query_impact()`

#### [MODIFY] [Cargo.toml (workspace)](file:///c:/Users/Andrea/dev/rust-context-engine/rtk/Cargo.toml)
- Aggiungere `rtk-index` ai workspace members

#### [MODIFY] [main.rs](file:///c:/Users/Andrea/dev/rust-context-engine/rtk/rtk-cli/src/main.rs)
- Nuovi subcommands: `rtk symbols`, `rtk deps`, `rtk refs`, `rtk impact`, `rtk search`

---

### Component 8: Documentation Suite

#### [NEW] [pricing.md](file:///c:/Users/Andrea/dev/rust-context-engine/docs/src/pricing.md)
- Methodology: come RTK calcola i risparmi, quale tokenizer, quali modelli, da dove vengono i prezzi
- Link al `model_pricing.json` e alla pricing_revision

#### [NEW] [benchmarks.md](file:///c:/Users/Andrea/dev/rust-context-engine/docs/src/benchmarks.md)
- Risultati benchmark con fixtures, grafici, metadata di fiducia
- "How to reproduce" section

#### [NEW] [use-cases.md](file:///c:/Users/Andrea/dev/rust-context-engine/docs/src/use-cases.md)
- Scenari: single-file edit, bug investigation, PR review, refactor, CI pipeline

#### [NEW] [limitations.md](file:///c:/Users/Andrea/dev/rust-context-engine/docs/src/limitations.md)
- Honest limitations: cosa RTK non fa, edge cases, falsi positivi noti

#### [MODIFY] [SUMMARY.md](file:///c:/Users/Andrea/dev/rust-context-engine/docs/src/SUMMARY.md)
- Aggiungere le 4 nuove pagine

#### [NEW] [fixtures/](file:///c:/Users/Andrea/dev/rust-context-engine/fixtures/)
- Golden test fixtures ufficiali per ogni filtro
- Struttura: `fixtures/{filter_name}/input.txt` + `fixtures/{filter_name}/expected.txt`

---

### Component 9: `rtk artifact` Manager

#### [NEW] [artifact.rs](file:///c:/Users/Andrea/dev/rust-context-engine/rtk/rtk-cli/src/artifact.rs)
- `artifact_list()` — lista artifacts salvati (cli-log, reasoning, summary, pack, report)
- `artifact_get(id)` — recupera un artifact specifico
- `artifact_gc()` — pulizia artifacts vecchi

#### DB table in tracking.rs:
- `artifacts (id, type, content, metadata_json, created_at)`

---

## P2 — 60/90 giorni (MCP, Routing, Budget)

### Component 10: Crate `rtk-mcp`

#### [NEW] Crate `rtk-mcp/`
- Server MCP stdio-based in Rust puro
- Tool surface minima (7 tool): `search_code`, `find_symbols`, `find_refs`, `project_memory`, `artifact_get`, `context_pack`, `session_state`
- Install helper: `rtk mcp install --client claude|cursor|gemini`

---

### Component 11: Budget & Model Routing

#### [MODIFY] [pricing.rs](file:///c:/Users/Andrea/dev/rust-context-engine/rtk/rtk-memory/src/pricing.rs)
- `check_budget(limit_usd: f64) -> BudgetStatus`
- Alerting quando il consumo supera soglia configurabile
- `suggest_model(task_type: &str) -> &str` — routing hint basato su tipo di task

---

### Component 12: Memory Discipline

#### [MODIFY] [tracking.rs](file:///c:/Users/Andrea/dev/rust-context-engine/rtk/rtk-memory/src/tracking.rs)
- `memory_overwrite(key, new_val)` con log di overwrite
- `memory_doctor()` — trova chiavi duplicate, stale (>30 giorni senza accesso), contraddittorie
- Output: report di salute della memoria

---

## P3 — Oltre 90 giorni (Advanced)

### Component 13: Hybrid Retrieval & Embeddings
- Modulo opzionale ONNX embeddings in Rust
- Score combinato: `α * BM25_score + β * embedding_score + γ * metadata_score`

### Component 14: Obsidian Graph Export
- `rtk graph export --format obsidian` → genera file `.md` per ogni simbolo con backlinks
- Plugin Obsidian separato (fuori dal core)

### Component 15: `rtk audit graph`
- Metriche: symbols count, edges count, query latency, graph coverage %

---

## Proof of Work & Verification Plan

### Principio Chiave
La roadmap **NON** è un documento da scrivere e dimenticare. È un documento vivo che:
- Costringe a fare le cose in ordine sequenziale e logico.
- Traccia in tempo reale cosa è stato completato e validato.
- Valida che il progetto e le sue evoluzioni siano reali e funzionanti.
- Comunica a investitori, utenti e al team che lo sviluppo segue criteri di engineering seri e misurabili.

---

### Strategia di Proof of Work per Fase

#### Fase 1: P0 = Foundation (30 giorni)
**Output concreti:**
- [NEW] [model_pricing.json](file:///c:/Users/Andrea/dev/rust-context-engine/data/model_pricing.json) aggiornato
- [NEW] [pricing.rs](file:///c:/Users/Andrea/dev/rust-context-engine/rtk/rtk-memory/src/pricing.rs) funzionante e integrato
- [NEW] [benchmark.rs](file:///c:/Users/Andrea/dev/rust-context-engine/rtk/rtk-cli/src/benchmark.rs) per export JSON/CSV
- [NEW] [doctor.rs](file:///c:/Users/Andrea/dev/rust-context-engine/rtk/rtk-cli/src/doctor.rs) che passa (`rtk doctor`)
- [NEW] [session.rs](file:///c:/Users/Andrea/dev/rust-context-engine/rtk/rtk-memory/src/session.rs) per `rtk session-state` che salva
- [NEW] [agents.rs](file:///c:/Users/Andrea/dev/rust-context-engine/rtk/rtk-cli/src/agents.rs) che genera e valida file
- [MODIFY] [think.rs](file:///c:/Users/Andrea/dev/rust-context-engine/rtk/rtk-memory/src/think.rs) con subcommands `inspect` e `gc` funzionanti

**Validazione Automatica & Manuale:**
```bash
# Test automatici
wsl ~/.cargo/bin/cargo test

# Build release
wsl ~/.cargo/bin/cargo build --release

# Verifica dimensione binario (Deve essere < 5MB)
wsl ls -la target/release/rtk

# Verifica manuale dei comandi
rtk doctor
rtk benchmark export --format json --output bench.json
rtk session-state init
rtk session-state get
rtk agents init --template solo-dev
rtk think inspect
```

#### Fase 2: P1 = Indexing + Docs (30-60 giorni)
**Output concreti:**
- Crate `rtk-index` funzionante (Tree-sitter + petgraph)
- Comandi `rtk symbols/deps/refs/impact/search` che interrogano il DB dell'indice
- Documentazione:
  - `docs/pricing.md` con metodologia di calcolo
  - `docs/benchmarks.md` con i risultati dei test
  - `docs/use-cases.md` con scenari d'uso reali
  - `docs/limitations.md` con limitazioni oneste
- Fixtures ufficiali in `fixtures/` con golden test data

**Validazione:**
```bash
# Test index crate
wsl ~/.cargo/bin/cargo test --package rtk-index

# Query demo per verifica indexing
rtk symbols find --name "main"
rtk deps show --file "src/main.rs"
rtk refs find --symbol "calculate_cost"
rtk impact analyze --file "src/dlp.rs"

# Verifica build documentazione
mdbook serve docs/

# Verifica fixtures
wsl ~/.cargo/bin/cargo test --test fixtures
```

#### Fase 3: P2 = MCP + Budget (60-90 giorni)
**Output concreti:**
- Crate `rtk-mcp` compilata
- MCP server funzionante stdio-based che risponde ai 7 tool core
- Installazione `rtk mcp install --client claude` funzionante
- Budget caps che triggerano alert al superamento soglie
- Model routing con suggeritore modelli per compiti specifici
- `rtk memory doctor` e `rtk memory overwrite` funzionanti

**Validazione:**
```bash
# Test MCP crate
wsl ~/.cargo/bin/cargo test --package rtk-mcp

# Installazione client
rtk mcp install --client claude

# Verify tools
rtk mcp call search_code --query "token"
rtk mcp call find_symbols --name "calculate"
rtk mcp call artifact_get --id "abc123"

# Verify budget & routing
rtk budget check --limit 50.00
rtk model suggest --task "single-file-edit"
```

#### Fase 4: P3 = Advanced (90+ giorni)
**Output concreti:**
- Retrieval Ibrido attivo (BM25 + ONNX embeddings opzionali)
- Obsidian graph export (.md compatibili con Obsidian graph)
- `rtk audit graph` con visualizzazione metriche di copertura e latenza

**Validazione:**
```bash
# Test retrieval con feature embeddings abilitata
wsl ~/.cargo/bin/cargo test --package rtk-index --features embeddings

# Esportazione grafo
rtk graph export --format obsidian --output obsidian/

# Verifica metriche audit grafo
rtk audit graph
```

---

### Timeline di Proof of Work

| Giorno | Milestone | Proof Artifact / Verifica |
|---|---|---|
| **Day 7** | Pricing registry loaded | `rtk doctor` → ✅ Pricing OK |
| **Day 14** | Benchmark export working | `bench.json` generato con metadata completi |
| **Day 21** | Session state working | `rtk session-state get` → export JSON valido |
| **Day 30** | **P0 Completed** | `cargo test` passa al 100% + release build < 5MB |
| **Day 45** | Index working | Symbol queries + dependency graph queryable |
| **Day 60** | **P1 Completed** | mdBook build senza errori + fixtures golden test superati |
| **Day 75** | MCP working | Claude Code / Cursor integration + test tool calls |
| **Day 90** | **P2 Completed** | Budget alerts triggerati + model routing suggerimenti attivi |
| **Day 120** | **P3 Completed** | Embeddings locali attivi + export Obsidian funzionante |

---

### Metriche di Validazione e Qualità

| Metrica | Target | Come misurare / Strumento |
|---|---|---|
| **Tests pass rate** | 100% | `wsl ~/.cargo/bin/cargo test` |
| **Release build size** | < 5MB | `wsl ls -la target/release/rtk` |
| **Docs build** | No errors | `mdbook build docs/` |
| **Fixtures pass** | 100% | `wsl ~/.cargo/bin/cargo test --test fixtures` |
| **Benchmark export** | JSON valido | `jq . bench.json` o validazione schema |
| **MCP tools** | 7 tool attivi | `rtk mcp call <tool_name>` |
| **Budget alert** | Trigger < limite | Simulazione superamento consumi e controllo log |
| **GitNexus pre-commit** | Pass | `node .gitnexus/run.cjs detect-changes -r rust-context-engine` |

---

## Decisions & Consensus

Le seguenti decisioni strategiche ed operative sono state concordate con l'utente e integrate nel piano d'azione:

1. **Ordine di esecuzione**: Sequenziale e lineare per la fase P0: `pricing` → `benchmark` → `doctor` → `session-state` → `agents` → `think`.
2. **Ridenominazione crate (`rtk-memory` → `rtk-db`)**: Eseguita immediatamente in P0 per impostare l'architettura target definitiva, eliminando futuri breaking changes.
3. **Architettura crate**: Mantenimento della separazione netta tra `rtk-filters` (stateless filters) e `rtk-cli` (CLI logic), senza procedere alla fusione in un crate `rtk-core`. Verranno aggiunti i due nuovi crate `rtk-index` (in P1) e `rtk-mcp` (in P2).
4. **Pianificazione MCP**: Il server MCP rimane programmato per P2 (60-90 giorni), garantendo la presenza di un database di indicizzazione (`rtk-index`) completo e funzionante.
