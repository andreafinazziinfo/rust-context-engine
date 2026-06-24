# RTK — Piano chiusura operativa

| Campo | Valore |
|-------|--------|
| **Baseline** | Fase A + B completate · v2.3.1 · CI verde |
| **Obiettivo** | Considerare RTK **pronto all'uso quotidiano**; niente nuove feature finché non emerge feedback reale |
| **Aggiornato** | 2026-06-23 |

Precedente: [PLAN_NOW.md](./PLAN_NOW.md) (✅) · Roadmap: [ROADMAP.md](./ROADMAP.md)

---

## Definizione di "chiuso"

Il progetto è **chiuso per iniziare a usarlo** quando:

1. Binario installato sulla macchina di lavoro (WSL + Windows prebuilt o brew).
2. Hook/alias attivi (`rtk init`, `rtk doctor` verde o warning noti accettati).
3. CI copre anche il flusso **rewrite → filter → show-log** (`e2e_smoke`).
4. Tag release pubblicato con asset + formula Homebrew allineata.
5. Checklist adozione personale completata (sotto).
6. **Stop sviluppo feature** → solo bugfix bloccanti fino a fine Fase D.

Non significa "finito per sempre". Significa: **ship → usa → misura → poi decidi**.

---

## Legenda

| Simbolo | Significato |
|---------|-------------|
| **Effort** | XS ≤2h · S ≤1g · M 2–4g |
| **Gate** | `bash scripts/dev-gate.sh` + CI verde |

---

## Fase C — Chiusura tecnica (1–2 giorni)

Ultimi gap tra "funziona in CI" e "fidati in produzione personale".

| ID | Task | Effort | Acceptance | Note |
|----|------|:------:|------------|------|
| **CLOSE-1** | **`e2e_smoke.sh` in CI** | XS | Step Linux in `ci.yml` dopo `release_smoke`; verde su main | `scripts/e2e_smoke.sh` |
| **CLOSE-2** | **`rtk validate`** | XS | Subcomando esegue fmt check + clippy + test workspace (o invoca `dev-gate.sh`); exit ≠0 se fallisce | `rtk-cli/src/validate.rs` |
| **CLOSE-3** | **Release v2.3.1** | S | Tag `v2.3.1`, workflow release, `update_homebrew_sha256.sh`, commit formula | Patch release post Fase B |
| **CLOSE-4** | **Smoke Homebrew reale** | XS | Su macOS: `brew tap … && brew install rtk && rtk --version` | Manuale; annotare esito in registro |
| **CLOSE-5** | **Dogfood repo** | XS | `bash scripts/setup-dogfood.sh`; doc [DOGFOOD.md](./DOGFOOD.md) | RTK usa se stesso |

### Ordine

```text
CLOSE-1 → CLOSE-2 (parallelo ok)
CLOSE-3 (dopo CLOSE-1 verde)
CLOSE-4 (dopo CLOSE-3, su Mac)
CLOSE-5 (ultimo)
```

### Exit criteria Fase C

- [x] `e2e_smoke` verde in CI Linux (step aggiunto; verifica su push)
- [x] `rtk validate` disponibile
- [x] Release v2.3.1 pubblicata (GitHub + Homebrew + **crates.io** 6 crate)
- [ ] Homebrew smoke passato almeno una volta su macOS reale
- [x] Dogfood: `scripts/setup-dogfood.sh` + DOGFOOD.md
- [ ] Nessun bug **blocker** aperto (crash, data loss, rewrite sbagliato)

---

## Fase D — Adozione quotidiana (2–4 settimane)

**Zero feature nuove.** Usa RTK come stack agente reale.

### Setup una tantum

| Step | Comando / azione | Verifica |
|------|------------------|----------|
| D-1 | Install da release o prebuilt (`QUICKSTART.md`) | `rtk --version` |
| D-2 | `rtk init --profile high` nel repo principale (es. `cyclelab-terminal`) | `rtk doctor` |
| D-3 | MCP server in Cursor/Claude (se usi MCP) | tool list visibile |
| D-4 | `rtk index run` su repo grande | `rtk symbols` / `impact` |
| D-5 | Githooks fmt pre-push sul clone RTK | push senza surprise CI |

### Uso minimo settimanale

Ogni settimana, almeno:

- [ ] Comandi agente passano da RTK (`git status`, `cargo test`, `npm install`, `pack`)
- [ ] Almeno 1× `rtk pack . --strip --limit 30000` al posto di dump grezzo
- [ ] Almeno 1× `rtk memory set` / `get` per decisioni che vuoi persistere
- [ ] Note friction in file locale (sotto)

### Log feedback (copia e compila)

Crea `docs/USAGE_FEEDBACK.md` (locale, **non** committare se contiene path sensibili) oppure usa:

```bash
rtk memory set closure_week1 "filtri ok; npm lento; doctor warning X"
```

Template settimanale:

```markdown
## Settimana N (data)
- Repo usati:
- Comandi più filtrati:
- Cosa ha risparmiato token (sì/no, quanto a occhio):
- Bug / falsi positivi DLP / rewrite:
- Comando mancante o distill debole:
- Vale la pena aprire issue? (sì/no)
```

### Exit criteria Fase D

- [ ] ≥2 settimane uso reale su ≥1 repo non-RTK
- [ ] Log feedback con almeno 2 entry
- [ ] Lista max 5 pain point prioritizzati (non 50 idee)

---

## Fase E — Valutazione post-uso (1 giorno, dopo Fase D)

Solo **dopo** Fase D. Decisione guidata da dati, non da repo CodeWhale/ECC.

| Domanda | Se sì → | Se no → |
|---------|---------|---------|
| Filtro distill debole su comando frequente? | Issue + golden fixture (pattern Fase B) | Ignora |
| Crash o exit code rewrite sbagliato? | Bugfix P0 | — |
| Serve `session fork` / agent OS? | Valuta pivot P2 (doc separato) | Resta context engine |
| Embeddings utili in pratica? | Prova `--features embeddings` | Resta optional |
| MCP tool mancante? | 1 tool mirato | — |
| Tutto ok, pochi pain point? | **Congela v1 uso**; release solo bugfix | — |

Output atteso: **backlog max 5 item** oppure dichiarazione esplicita:

> RTK chiuso come v1 operativa; prossimo lavoro solo su feedback raccolto.

---

## Cosa NON fare prima di Fase E

| Item | Motivo |
|------|--------|
| P2 harness completo (`mode`, `replay`, LSP) | Pivot prodotto; zero evidenza uso |
| Scorecard 9.5 | Vanity |
| Coverage 70% | Costo >> beneficio |
| Index watcher | YAGNI finché `index run` basta |
| Copia superficie ECC / CodeWhale | Scope creep |

---

## Registro avanzamento

| ID | Status | Note |
|----|:------:|------|
| CLOSE-1 | ✅ | e2e_smoke in ci.yml |
| CLOSE-2 | ✅ | rtk validate |
| CLOSE-3 | ✅ | release v2.3.1 GitHub + Homebrew + crates.io |
| CLOSE-4 | ⬜ | brew smoke macOS (manuale) |
| CLOSE-5 | ✅ | setup-dogfood + DOGFOOD.md |
| Fase D | 🔄 | adozione 2–4 sett — **active now** |
| Fase E | ⬜ | valutazione post-uso |

⬜ TODO · 🔄 IN PROGRESS · ✅ DONE

---

## Riferimento rapido comandi

```bash
bash scripts/dev-gate.sh          # pre-PR
bash scripts/e2e_smoke.sh         # smoke end-to-end locale
bash scripts/homebrew_smoke.sh    # formula dry-run
rtk doctor                        # setup agente
rtk validate                      # dopo CLOSE-2
```
