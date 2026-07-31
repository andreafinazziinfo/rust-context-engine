# RTK — Piano: fix skill discovery Claude Code (`.claude/skills` mancante + nomi mismatch)

| Campo | Valore |
|-------|--------|
| **Tier** | STANDARD (`0_META` — bugfix > 2h, modulo persistente `rtk init`, nessun tocco a security/denaro/prod) |
| **Framework** | `1_DESIGN` Pilastri 1–2 + `2_EXECUTION` completo (fonte: [operational-engineering-framework](https://github.com/andreafinazziinfo/operational-engineering-framework)) |
| **Baseline** | v2.4.0 · bug riprodotto su `andreafinazziinfo/cyclequant-terminal` (profilo HIGH installato, skill mai discoverable) |
| **Origine** | Diagnosi sessione 2026-07-31 — vedi riproduzione sotto |
| **Owner proposto** | maintainer RTK |

---

## 0. Riproduzione del bug (evidenza)

1. `cyclequant-terminal/CLAUDE.md` contiene il blocco `# RTK Output Profile: HIGH` con `"You MUST auto-trigger the **caveman-full** skill"` — generato da `rtk init --profile high`.
2. `.claude/skills/` nel repo contiene solo `gitnexus/*` e `lead-architect-plan/` (installati da altro tooling). Nessuna cartella `caveman*`.
3. `rtk-cli/src/setup.rs:274` scrive le skill **solo** sotto `.agents/skills/`:
   ```rust
   let agents_skills_dir = base.join(".agents").join("skills");
   ```
4. Grep sull'intero crate `rtk`: `.claude/skills` non compare **mai** come target di scrittura.
5. Anche ignorando (3)/(4): il testo profilo HIGH (`setup.rs:361`) cita `caveman-full`; l'unico asset generato è `caveman-skill.md` → cartella `caveman` (`setup.rs:319-321`). L'asset stesso (`assets/caveman/caveman-skill.md`, frontmatter `name: caveman`) conferma che a monte (JuliusBrussee/caveman) esiste **una sola skill `caveman`**, selezionata via `/caveman lite|full|ultra` — non tre skill separate.

**Conseguenza**: in ogni ambiente (locale o cloud) che rispetta la convenzione Claude Code (`.claude/skills/<nome>/SKILL.md`), l'istruzione "auto-trigger caveman-full" non risolve a nessuna skill reale. Il comportamento osservato ("a volte funziona") è un modello che improvvisa uno stile compresso leggendo solo il nome+istruzione in prosa, non un'invocazione skill reale — non deterministico per costruzione.

---

## 1. Pre-Mortem (Pilastro 1 — 1_DESIGN)

Ipotesi: il fix è live, e un anno dopo l'adozione fallisce di nuovo. Scenari (≥5, soglia framework):

| # | Scenario | Impatto | Mitigazione nel piano |
|---|----------|---------|------------------------|
| 1 | Fix scrive solo `.claude/skills`, ma un altro agente (Cursor/Windsurf) si aspettava `.agents/skills` per compatibilità futura AGENTS.md-spec | P2 — regressione per agenti non-Claude | Mantenere **entrambi** i path (scrittura idempotente, nessuna rimozione di `.agents/skills`) |
| 2 | Repo esistenti (es. cyclequant-terminal) hanno già un blocco `CLAUDE.md` con "RTK Output Profile" — il guard `!existing_claude.contains("RTK Output Profile")` (`setup.rs:403`) blocca il re-init, il fix non si propaga mai a chi l'ha già installato | P1 — bug persiste su tutti gli utenti esistenti, il fix aiuta solo nuove install | Versionare il blocco profilo (`<!-- rtk-profile-version: N -->`) + `rtk doctor` che rileva versione stale e istruisce `rtk init --force-profile` |
| 3 | Skill vendorizzate (`caveman-skill.md` ecc.) driftano da upstream (JuliusBrussee/caveman aggiorna livelli/regole, RTK resta fermo alla copia embedded al build) | P3 — output caveman disallineato da quanto documentato upstream | Attribution + commento "sync source" nel file generato; task backlog separato (non in questo piano) per un comando `rtk skills sync` che rilegge upstream — **out of scope qui**, tracciato in ROADMAP |
| 4 | `rtk doctor` nuovo check rompe l'output testuale già parsato da script utente (`check_hook_installed` pattern) | P3 — CI di terzi che grepano l'output doctor | Nuovo check additivo, non modifica righe esistenti; test snapshot su `doctor.rs` |
| 5 | Utente ha già rinominato manualmente `.agents/skills/caveman` in `.agents/skills/caveman-full` per "farlo funzionare" — il fix con `write_if_absent` non tocca file esistenti, quindi coesistono due cartelle skill divergenti | P3 — confusione, non rottura | `rtk doctor` segnala **entrambe** le dir se trovate, con nota "solo `.claude/skills/<nome-atteso>` è quella letta da Claude Code" |
| 6 | Cross-platform: `base.join(".claude").join("skills")` su Windows con path già esistente come file (non dir) da install precedente rotto | P3 — `fs::create_dir_all` fallisce | `run_init_in` già propaga `Result`; aggiungere test Windows path in CI esistente (matrice già presente per release builds) |

**SPOF peggiore**: (2) — senza fix di propagazione, la correzione del bug non raggiunge nessun repo già inizializzato, incluso quello che ha innescato la diagnosi.

---

## 2. ADR-fix-001: path skill Claude Code + naming profilo

**Stato**: Proposto
**Data**: 2026-07-31

### Contesto
`rtk init` genera istruzioni in `CLAUDE.md`/`AGENTS.md` che richiedono all'agente di "auto-trigger" una skill (`caveman-full`/`-lite`/`-ultra`), ma scrive i file skill solo in `.agents/skills/`, mai in `.claude/skills/` (unica convenzione letta da Claude Code), e con un solo nome (`caveman`) invece dei tre citati nel prompt.

### Scelta effettuata
1. Scrivere le skill anche in `base.join(".claude").join("skills")`, stesso contenuto/idempotenza di `.agents/skills` (`write_if_absent`, nessun overwrite di edit utente).
2. Correggere il testo profilo per riferirsi al **nome skill reale** (`caveman`) più il **livello come parametro**, coerente con l'upstream (JuliusBrussee/caveman: `/caveman lite|full|ultra`), invece di inventare `caveman-full`/`caveman-lite`/`caveman-ultra` come nomi-skill.
3. Versionare il blocco profilo per permettere rigenerazione controllata su repo già inizializzati.
4. Aggiungere un check in `rtk doctor` che verifica l'esistenza reale della skill referenziata.

### Alternative scartate
- **Delegare interamente al plugin marketplace ufficiale** (`claude plugin marketplace add JuliusBrussee/caveman && claude plugin install caveman@caveman`): install più robusta (statusline, `.caveman-active`, hook nativi) ma richiede la CLI `claude` disponibile e un passo interattivo che `rtk init` non può eseguire in modo affidabile e non-interattivo su ogni piattaforma/agente supportato da RTK (Cursor, Windsurf, Copilot non hanno marketplace equivalente). Scartata **per ora** come sostituzione; mantenuta come suggerimento stampato a fine `rtk init` (vedi Fase C).
- **Rimuovere del tutto la persona caveman/ponytail da RTK e delegare 100% a install esterna**: elimina il bug ma rompe l'offerta "un binario, zero dipendenze esterne, funziona offline" che è il differenziatore di RTK (README: "local runtime"). Scartata.
- **Rinominare gli asset generati in `caveman-full.md`/`caveman-lite.md`/`caveman-ultra.md`** invece di correggere il testo verso `caveman`: scartata perché duplica contenuto quasi identico in 3 file (drift garantito) e diverge dalla skill upstream reale (`name: caveman`), rompendo la tracciabilità con JuliusBrussee/caveman.

### Conseguenze e trade-off
- **Guadagno**: skill effettivamente discoverable da Claude Code, coerenza col nome skill upstream, path per sanare install esistenti.
- **Costo**: doppia scrittura file (`.agents` + `.claude`) — footprint disco trascurabile; un comando doctor in più da mantenere; migrazione richiede bump di versione del marker profilo (piccola rottura di idempotenza intenzionale, mitigata da opt-in `--force-profile`).

---

## 3. Esecuzione (`2_EXECUTION` — 6 fasi)

### A. Task Breakdown

| # | Task | Output verificabile | Dipendenze | Stima |
|---|------|---------------------|------------|-------|
| A1 | Aggiungere scrittura `.claude/skills/<nome>/SKILL.md` in `run_init_in` (`setup.rs:270-333`) | 4 nuove dir create, contenuto identico a `.agents/skills` | — | XS |
| A2 | Correggere `profile_content` (`setup.rs:336-389`) per citare `caveman` + livello come parametro (non `caveman-full/-lite/-ultra`) | Testo profilo aggiornato per LOW/MEDIUM/HIGH/MAX | — | XS |
| A3 | Versionare il marker profilo (`<!-- rtk-profile-version: 2 -->`) + logica `--force-profile` per rigenerare CLAUDE.md già esistenti | Nuovo flag CLI, guard su `existing_claude.contains(...)` aggiornato | A2 | S |
| A4 | Estendere `status.rs` (righe 20-38) e `doctor.rs` per: (a) rilevare la skill citata nel profilo, (b) verificare che `.claude/skills/<nome>/SKILL.md` esista | A1, A2 | S |
| A5 | Stampa suggerimento marketplace ufficiale a fine `run_init` (`setup.rs:156-192`) quando rileva CLI `claude` in `$PATH` | A1 | XS |
| A6 | Aggiornare `rtk-cli/tests/integration_test.rs`: per ogni profilo, asserire che il nome citato nel prompt esiste come `.claude/skills/<nome>/SKILL.md` | A1, A2 | S |
| A7 | CHANGELOG + bump versione (2.4.0 → 2.4.1, patch — nessuna breaking API) | Entry `[Unreleased]` → `[2.4.1]` | tutti | XS |

Tutti i task ≤ 2h salvo A3/A4/A6 (S, stimati ≤ mezza giornata) → rispetta soglia 90% `≤2h` (Fase A framework) con 3 eccezioni giustificate qui.

### B. Test Strategy (prima del codice)

- **Unit**: `run_init_in` su `tempdir()` per ciascun profilo (`low|medium|high|max`) → asserire esistenza `.claude/skills/caveman/SKILL.md` (dove atteso) e che il testo generato in `AGENTS.md`/`CLAUDE.md` non citi nomi skill inesistenti (regex `caveman-(full|lite|ultra)\b` deve sparire dal testo di prompt, resta solo come istruzione di livello `/caveman <level>`).
- **Idempotenza**: due chiamate consecutive a `run_init_in` con lo stesso profilo → nessun file skill riscritto (mtime invariato), coerente con `write_if_absent`.
- **Regressione migrazione**: `run_init_in` su un `CLAUDE.md` che contiene già `"RTK Output Profile: HIGH"` (fixture presa da cyclequant-terminal) → senza `--force-profile` nessuna modifica; con `--force-profile` il blocco viene sostituito e la versione marker aggiornata.
- **Doctor**: test che, dato un profilo `HIGH` con `.claude/skills/caveman/` mancante, `rtk doctor` esce con warning (non critical, coerente con severità esistenti in `doctor.rs`).
- **Coverage target**: ≥80% su `setup.rs` (già toccato per intero dal fix) e sulle nuove funzioni in `status.rs`/`doctor.rs`.

### C. Implementation Loop

Ordine consigliato (dipendenze A1→A2→A3→A4, A5/A6/A7 in coda):

1. A1 — path fix, commit atomico.
2. A2 — naming fix, commit atomico (separato da A1 per bisect pulito).
3. A3 — versioning + `--force-profile`.
4. A4 — doctor/status check.
5. A6 — test di integrazione (chiude il loop rosso→verde per A1-A4).
6. A5 — hint marketplace (cosmetico, basso rischio).
7. A7 — changelog/version bump, ultimo commit prima di release.

Checkpoint dopo ogni task: `bash scripts/dev-gate.sh` (fmt + clippy + test) verde prima di procedere al successivo, come da `rtk validate` esistente.

### D. Review Gate

- [ ] `cargo clippy --all-targets -- -D warnings` pulito.
- [ ] Diff atteso ≈ 150-250 righe (setup.rs + status.rs + doctor.rs + test) → sotto soglia 400 righe/PR, nessuno split necessario.
- [ ] Second reviewer (umano o AI) legge riga per riga in particolare: guard `--force-profile` (rischio di sovrascrivere edit utente non-RTK in CLAUDE.md se il marker matcha per errore un testo utente).
- [ ] CVE/dipendenze: nessuna nuova dipendenza introdotta (fix è solo file I/O + string matching, stdlib).

### E. Release & Rollback

- Release come **patch 2.4.1** su crates.io (`cargo publish`) + tag GitHub + Homebrew formula bump (`rtk.rb`), stesso processo di `docs/RELEASE.md`.
- Rollback: `cargo yank --version 2.4.1` se emergono regressioni; nessun rollback dati necessario (comando idempotente, nessuno stato server-side).
- Finestra di osservazione: non applicabile a un CLI locale — sostituita da monitoraggio issue GitHub per 7 giorni post-release (RTK non ha telemetria remota per progetto, coerente con `SECURITY.md`/privacy-first).

### F. Post-Mortem (dopo release)

Da compilare 48h dopo pubblicazione 2.4.1 (o prima se emergono issue): confrontare scenari Pre-Mortem §1 con quanto realmente riportato dagli utenti; se lo scenario 2 (repo esistenti bloccati dal guard) genera più friction del previsto, valutare un comando dedicato `rtk migrate-profile` invece del flag `--force-profile`.

---

## 4. Definition of Done (tier STANDARD)

- [ ] Test pertinenti eseguiti (§3.B) + Review Gate minimo (§3.D)
- [ ] Execution trace prodotto (commit log 7 task §3.A)
- [ ] ADR §2 presente per la decisione architetturale
- [ ] Test strategy definita prima del codice (§3.B, questo documento precede l'implementazione)
- [ ] Coverage ≥ 80% su moduli toccati (`setup.rs`, `status.rs`, `doctor.rs`)
- [ ] Pre-mortem con ≥ 3 scenari (qui: 6, §1)
- [ ] Documentazione aggiornata: `README.md` (nessuna promessa da correggere, il comportamento descritto — "9 tools MCP", ecc. — non cambia), `CHANGELOG.md`, `docs/QUICKSTART.md` se il flag `--force-profile` va documentato

---

## 5. Migrazione per repo già inizializzati (es. cyclequant-terminal)

Dopo il rilascio 2.4.1:

```bash
rtk init --profile high --force-profile
rtk doctor   # deve riportare "Output Profile: HIGH" + skill "caveman" trovata in .claude/skills/
```

Verifica manuale in una sessione Claude Code reale: la skill `caveman` deve comparire nella lista skill disponibili di sistema (stesso meccanismo con cui è stato isolato il bug in questa diagnosi).
