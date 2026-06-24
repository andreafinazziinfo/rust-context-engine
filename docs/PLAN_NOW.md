# RTK — Piano esecuzione (post 9.0)

| Campo | Valore |
|-------|--------|
| **Baseline** | v2.3.0 · CI verde · score qualità 9.0 |
| **Obiettivo fase** | Adozione + uniformità filtri (non nuove feature pesanti) |
| **Status** | ✅ Completato (Fase A–B) — attivo: [PLAN_CLOSURE.md](./PLAN_CLOSURE.md) v2.3.1 |
| **Aggiornato** | 2026-06-23 |

Roadmap pubblica breve: [ROADMAP.md](./ROADMAP.md) · Audit completato: [archive/IMPROVEMENT_PLAN_9PLUS.md](./archive/IMPROVEMENT_PLAN_9PLUS.md)

---

## Legenda

| Simbolo | Significato |
|---------|-------------|
| **Effort** | XS ≤2h · S ≤1g · M 2–4g · L ≥1 sett |
| **Gate merge** | `bash scripts/dev-gate.sh` + CI verde |

---

## Fase A — Adesso (1–2 settimane)

Focus: **install macOS**, **onboarding 5 min**, **filtri golden estesi**, **release ↔ Homebrew allineati**.

| ID | Task | Effort | Acceptance | File / note |
|----|------|:------:|------------|-------------|
| **NOW-1** | **Homebrew tap** | S | `brew install <tap>/rtk` installa v pinata; test `rtk --version` | Repo `homebrew-tap` o tap org; formula da `rtk.rb` |
| **NOW-2** | **QUICKSTART 5 min** | S | Doc: install → `init --profile high` → `index run` → demo `git status` + `pack . --strip` | `docs/QUICKSTART.md` + link in README (in cima) |
| **NOW-3** | **Doctor → onboarding** | XS | Se hook/alias mancanti, doctor stampa link a QUICKSTART | `rtk-cli/src/doctor.rs` |
| **NOW-4** | **Golden pytest** | M | Fixture + test `token_savings` ≥ soglia; CI Linux gate | `rtk-filters`, `tests/fixtures/`, `ci.yml` |
| **NOW-5** | **Golden docker build** | M | Idem NOW-4 per output docker | come NOW-4 |
| **NOW-6** | **Release → sha256** | S | Tag `v*` → script aggiorna `rtk.rb` o genera `SHA256SUMS`; doc in ROADMAP | `release.yml` o checklist release + `update_homebrew_sha256.sh` |
| **NOW-7** | **README slim (user)** | S | README: hero + quickstart link; dettagli arch → docs/ | `README.md` |

### Ordine consigliato

```text
NOW-2 + NOW-3  (onboarding, parallel)
NOW-1 + NOW-6  (distribuzione macOS)
NOW-4 → NOW-5  (qualità filtri)
NOW-7          (pulizia doc, fine fase)
```

### Exit criteria Fase A

- [x] Tap Homebrew funzionante o PR tap aperta con formula testata
- [x] QUICKSTART verificabile da zero su WSL in ≤10 min
- [x] ≥2 nuovi golden filter in CI (pytest + docker)
- [x] Processo release documentato con checksum formula
- [x] Nessun task NOW-* senza test o smoke associato

---

## Fase B — Subito dopo (settimane 3–4)

Focus: **stack JS**, **regressioni silenziose**, **doc contributor vs user**.

| ID | Task | Effort | Acceptance | File / note |
|----|------|:------:|------------|-------------|
| **NEXT-1** | **Filtri npm/yarn nativi** | M | Filter module dedicato (non solo distill); savings ≥40% su fixture | `rtk-filters/src/npm*.rs` o estensione distill |
| **NEXT-2** | **Test mirati core** | S–M | Test integration/e2e su `dlp`, `rewrite`, `filter_pipeline`, `config` regex | `rtk-cli/tests/`, `rtk-db` |
| **NEXT-3** | **Golden ls / npm** | M | Fixture + gate CI per comandi ad alto volume agent | estensione pattern NOW-4 |
| **NEXT-4** | **Split docs** | S | `docs/USER.md` vs `docs/CONTRIBUTING.md`; README punta lì | riduce cognitive load |
| **NEXT-5** | **Pre-commit opzionale** | XS | `setup-githooks.sh` menzionato in CONTRIBUTING; hook documentato | già `.githooks/pre-push` |
| **NEXT-6** | **Windows prebuilt path** | S | QUICKSTART sezione Windows: `%USERPROFILE%\.rtk-bin`, no build MSVC | `docs/QUICKSTART.md` |

### Exit criteria Fase B

- [x] npm/yarn misurati con benchmark o golden (non solo distill generico)
- [x] Coverage mirata su moduli critici (no target 70% globale)
- [x] README Contributing → docs/; USER.md + CONTRIBUTING.md creati

---

## Deferred (non ora)

| Item | Motivo |
|------|--------|
| Index file watcher | YAGNI; `rtk index run` + nota doctor sufficiente |
| Embeddings in default binary | +15MB; resta `--features embeddings` |
| Dashboard v2 / plugin marketplace | Non sblocca adozione core |
| Coverage 70% workspace | Costo >> beneficio |
| Scorecard 9.5 ovunque | Solo vanity metric |

---

## Checklist pre-merge (ogni task)

- [ ] Lavoro su WSL `~/dev/rust-context-engine` (non `/mnt/c/target`)
- [ ] `bash scripts/setup-githooks.sh` (fmt pre-push)
- [ ] `bash scripts/dev-gate.sh` verde
- [ ] Aggiornare tabella status in questo file (✅ / 🔄 / ⬜)
- [ ] ROADMAP.md allineato se cambia scope pubblico

---

## Registro avanzamento

| ID | Status | Note |
|----|:------:|------|
| NOW-1 | ✅ | `Formula/rtk.rb` in-repo tap |
| NOW-2 | ✅ | `docs/QUICKSTART.md` |
| NOW-3 | ✅ | doctor → QUICKSTART hints |
| NOW-4 | ✅ | golden pytest + token_savings ≥40% |
| NOW-5 | ✅ | golden docker + token_savings ≥40% |
| NOW-6 | ✅ | release `.sha256` sidecars + `docs/RELEASE.md` |
| NOW-7 | ✅ | README quickstart banner + brew tap |
| NEXT-1 | ✅ | `npm_filter.rs` + dispatch combined |
| NEXT-2 | ✅ | integration: strict_chained, deny, JWT pack |
| NEXT-3 | ✅ | golden ls + npm fixtures |
| NEXT-4 | ✅ | `docs/USER.md` + `docs/CONTRIBUTING.md` |
| NEXT-5 | ✅ | githooks in CONTRIBUTING |
| NEXT-6 | ✅ | QUICKSTART Windows PowerShell PATH |

⬜ TODO · 🔄 IN PROGRESS · ✅ DONE
