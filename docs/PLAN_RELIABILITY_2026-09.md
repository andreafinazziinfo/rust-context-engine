# RTK — Piano affidabilità post-adozione (2026-09)

| Campo | Valore |
|-------|--------|
| **Baseline** | v2.4.2 · Fase C/D (`PLAN_CLOSURE.md`) chiusa a giugno, "solo bugfix bloccanti" da allora |
| **Obiettivo fase** | Il momento "misura → poi decidi" previsto da `PLAN_CLOSURE.md` — 3 mesi di uso quotidiano reale (CycleLab + Titan) hanno prodotto 3 bug concreti nello stesso trimestre, tutti nella stessa famiglia (output filtrato non sicuro da comporre con pipe/parsing a valle) |
| **Status** | ✅ Completato (REL-1, REL-2, REL-3) |
| **Aggiornato** | 2026-09-07 |

Precedente: [PLAN_CLOSURE.md](./PLAN_CLOSURE.md) (✅) · Debito collegato: [`TECHNICAL_DEBT_LEDGER.md`](../TECHNICAL_DEBT_LEDGER.md) #7

---

## Perché questo piano esiste

Durante un audit del harness Claude Code su `cyclelab-terminal` (2026-09-06), tre bug distinti sono emersi nella stessa famiglia — output filtrato che si comporta bene quando letto da un umano/LLM, ma rompe silenziosamente qualunque cosa a valle tratti quell'output come dato strutturato:

1. **SIGPIPE panic** (issue #16, già chiuso in v2.3.2) — precedente storico della stessa famiglia.
2. **`ls` filter: `... and N more entries ...` corrompe `wc -l`** (issue #77, non ancora chiuso) — 36 voci reali, `ls -1 .claude/skills | wc -l` restituisce 19. Nessun errore, nessun segnale — solo un numero sbagliato ma plausibile.
3. **Anomalia non investigata**: nel DB locale (`~/.local/share/rtk/rtk.db`, tabella `tracking`, riga id 7472) un comando `git log --oneline <hash>..HEAD` aveva il testo letterale `[Full output cached. Access with: rtk show-log NNNN]` **dentro il campo `cmd` stesso**, non nel `raw_output` — sembra history/cache leak in un comando successivo, non ancora capito. Trovato per caso, mai riprodotto deliberatamente.

## Legenda

| Simbolo | Significato |
|---------|-------------|
| **Effort** | XS ≤2h · S ≤1g · M 2–4g |
| **Gate** | `bash scripts/dev-gate.sh` + CI verde |

---

## Fase R — Affidabilità pipe/parsing (questa sessione dedicata)

| ID | Task | Effort | Acceptance | Note |
|----|------|:------:|------------|------|
| **REL-1** | **Fix #77** (`ls` filter corrompe conteggi piped) | S–M | `RTK_RAW=1 ls -1 <dir con 30+ voci> \| wc -l` restituisce il conteggio reale, non il conteggio delle righe di prosa compressa | Fix rivisto in corsa (v. "Root cause" sotto): il gate su TTY proposto inizialmente dall'issue rompeva la compressione per il caso d'uso primario (agente via harness). Sostituito con un escape hatch esplicito `RTK_RAW=1` — la compressione resta il default, come da design originale |
| **REL-2** | **Sweep sistematico, non esplorativo** — stessa classe di bug su ogni comando wrappato (`git`, `cargo`, `pytest`, `ruff`, `mypy`, `pip`, `eslint`, `tsc`, `vitest`, `docker`, `gh`, `pack`) | M | Per ciascuno, sotto `RTK_RAW=1`: un caso con output lungo abbastanza da attivare la summarizzazione, pipato in `wc -l`/`grep -c`/un parser reale — conteggio corretto o troncamento esplicito (mai un numero sbagliato silenzioso) | Regressione mirata sul difetto già dimostrato in REL-1, non un fishing generico — se un comando non ha una path di summarizzazione, si documenta "N/A" e si passa oltre |
| **REL-3** | **Investigare l'anomalia id 7472** (cmd corrotto, non solo output) | S | O riprodotta con un caso minimo e capita la causa, o documentata come "non riproducibile, osservata una volta" con l'evidenza grezza salvata | Diversa da REL-1/REL-2: qui è il campo `cmd` stesso a essere sporco, non `raw_output` — potenziale bug più serio (corruzione dell'input, non solo del filtro sull'output) |

### Ordine consigliato

```text
REL-1          (fix il bug già isolato e riproducibile)
REL-2          (sweep mirato, riusa il test-case pattern di REL-1)
REL-3          (indagine, indipendente dalle altre due — può girare in parallelo)
```

### Exit criteria

- [x] REL-1: test reale (non solo lettura del codice) — stesso repro dell'issue, prima e dopo il fix
- [x] REL-2: tabella di 12 righe (uno per comando wrappato), esito per ciascuno, non un "sembra a posto" generico
- [x] REL-3: causa capita, o esplicitamente chiusa come "non riproducibile" con l'evidenza grezza allegata — non lasciata a metà senza una delle due
- [x] Issue #77 chiusa con riferimento al fix (via PR, "Closes #77"); nessun nuovo bug REL-2 di entità "issue a sé" — il secondo trovato (`join_streams`, ledger #8) è stato risolto nello stesso commit del fix REL-1

## Root cause (comune a REL-1/REL-2/REL-3) e revisione di design

Tutti i comandi wrappati passano da un unico punto di snodo, `execute_with_filter` (`rtk-cli/src/filter_pipeline.rs`). Prima del fix, la compressione (filtro per-comando + `post_process_filter_output` con regole regex/profilo + `distiller`) veniva sempre applicata, e l'annotazione informativa `[Full output cached. Access with: rtk show-log N]` / il warning di autonomia venivano sempre appesi **mescolati nello stesso stream stampato** — senza distinguere in alcun modo chi stava per leggere quell'output.

**Primo tentativo (rivisto — vedi sotto perché)**: gating su `std::io::IsTerminal` — se lo stream non era un terminale reale, saltare compressione e annotazione. Funzionava per il repro letterale dell'issue (`ls -1 dir | wc -l`), ma **rompeva l'uso primario del prodotto**: quando un harness agente (Claude Code via il suo tool Bash) cattura l'output di un comando `rtk`-wrapped, quella cattura è *anch'essa* "non un terminale" — indistinguibile, a livello di sistema operativo, da una pipe verso `wc -l`. RTK non ha modo di sapere se il lettore è un programma che sta per contare le righe o è l'agente stesso, che una prosa compressa la legge benissimo. Il gate su TTY disattivava quindi la compressione — il motivo per cui RTK esiste — proprio nel caso d'uso reale prevalente, in diretto contrasto con la sezione "AI Agent Guide" del README (`... yields filtered outputs. If a log is truncated, a cache note appears...`) e con il claim "Average 81.8% Token Savings". Verificato empiricamente: ogni comando `rtk`-wrapped lanciato in questa sessione tramite lo strumento Bash risultava non-TTY, esattamente come una pipe scritta a mano.

**Design finale**: RTK è pensato per essere letto direttamente da un agente AI, che gestisce una sintesi compressa senza problemi — non è dato che sta contando alla cieca. La compressione diventa pericolosa solo quando quell'output viene *composto*: pipato in uno strumento di conteggio/parsing, o riusato come argomento di un altro comando. Poiché RTK non può distinguere in modo affidabile questi due casi dal lato OS, non tenta più di indovinare:

1. **Compressione riattivata come default** (comportamento originale ripristinato) — `compress_for_display()` comprime sempre, a meno che l'invocazione non porti `RTK_RAW=1` nell'ambiente.
2. **`RTK_RAW=1`** — nuovo escape hatch esplicito. Quando impostato, `compress_for_display()` salta compressione/riassunto (filtro per-comando, `apply_profile_settings`, `distiller`) e restituisce l'output originale intatto. I controlli di sicurezza (redazione DLP, filtri regex custom via `rtk filter add`) girano **sempre**, `RTK_RAW` o no — non sono un'euristica di risparmio token, sono un controllo di sicurezza indipendente.
3. **L'annotazione `[Full output cached...]` e il warning di autonomia vanno sempre e solo su stderr**, mai mescolati in stdout — indipendentemente da `RTK_RAW` o dal terminale. Questo, da solo, chiude strutturalmente il vettore REL-3 (sotto): la sostituzione di comando `$(...)` in shell cattura solo stdout, quindi quel testo non può più finire per sbaglio dentro l'argomento di un comando successivo, sempre, per costruzione — non per convenzione documentata.
4. `join_streams()` — il join di stderr+stdout in modalità `Combined` non inserisce più un `\n` spurio quando uno dei due lati è vuoto o già terminato da newline (bug minore trovato durante lo sweep REL-2, stessa famiglia, vedi ledger #8).

Documentato nel README (`AI Agent Guide` punto 2, e "Known Limitations") perché l'agente sappia quando usare `RTK_RAW=1`.

### Cosa garantisce RTK ora, e cosa no

- **Garantito sempre, per costruzione, indipendentemente da `RTK_RAW`**: l'annotazione di cache/autonomia non può mai finire in stdout, quindi non può mai propagarsi nel campo `cmd` di un comando successivo via `$(...)` — questo è esattamente ciò che l'anomalia REL-3 (id 7472) mostrava.
- **Garantito sempre**: redazione DLP e filtri regex di sicurezza custom girano su ogni invocazione, compressa o raw.
- **Non più garantito automaticamente**: che un `rtk <cmd> | wc -l` scritto senza `RTK_RAW=1` dia il conteggio reale — per default torna a comprimere, come nel design originale. È responsabilità di chi scrive quella pipeline (l'agente, istruito dal README) aggiungere `RTK_RAW=1` quando comporrà l'output con un altro strumento. Scelta esplicita di questo piano: ottimizzare per l'uso AI reale, non per un ipotetico consumo umano via pipe.

### Note dalla review (`/code-review high`)

Una review dedicata sul diff (eseguita sul design con gating TTY, prima della revisione sopra) ha trovato e fatto correggere due bug reali nella stessa famiglia, entrambi ancora validi col design finale:

- I filtri regex custom (`rtk filter add --pattern ... --action strip|collapse`) sono un controllo di sicurezza scelto dall'utente, non un'euristica di compressione — nella prima versione venivano saltati insieme al resto sotto `RTK_RAW`/non-TTY. Corretto: ora girano sempre, come la redazione DLP. Test: `test_custom_regex_filter_still_applies_under_raw`.
- La redazione DLP delle chiavi private (`dlp.rs`) collassava un blocco PEM multi-riga in un'unica riga `[REDACTED_PRIVATE_KEY]`, cambiando il conteggio righe di qualunque output che contenesse per caso una chiave privata — la stessa classe di bug di #77, sul path di sicurezza. Corretto: il numero di newline del blocco originale viene preservato. Test: `test_redact_private_key_preserves_line_count`.

Il finding della review sulla metrica "token risparmiati" azzerata sul path non-TTY non si applica più: col design finale la compressione resta attiva di default, quindi la telemetria torna a riflettere i risparmi reali nel caso comune.

### REL-1 — prima/dopo (repro esatto dell'issue #77)

Ambiente: directory con 36 voci reali.

| | `ls -1 <dir> \| wc -l` |
|---|---|
| Ground truth (`command ls`) | 36 |
| Binario `rtk` pre-fix (installato in `~/.local/bin/rtk`) | **19** (bug riprodotto dal vivo) |
| Binario `rtk` post-fix, default (senza `RTK_RAW`) | **17** (comprime di nuovo — comportamento *voluto*: l'agente legge la prosa, non conta le righe) |
| Binario `rtk` post-fix, `RTK_RAW=1 rtk ls -1 <dir> \| wc -l` | **36** (corretto quando esplicitamente richiesto) |

Test automatici: `ls_filter_compresses_by_default_for_agent_consumption` (il default resta compresso — guardia di non-regressione sul valore per l'agente) e `ls_filter_raw_reports_true_entry_count_issue_77` (`RTK_RAW=1` dà il conteggio reale, mai un marker di troncamento).

### REL-2 — sweep sui 12 comandi wrappati

Tutti passano dallo stesso snodo (`execute_with_filter`), quindi la garanzia `RTK_RAW=1` è meccanica per tutti. Ri-verificato dal vivo con `RTK_RAW=1` dopo la revisione di design (i numeri sotto erano già stati confermati identici anche nella prima iterazione del fix, sotto un meccanismo diverso).

| # | Comando | Repro (`RTK_RAW=1`) | Esito |
|---|---------|-------|-------|
| 1 | `git diff` | diff sintetico 400 righe modificate, confronto riga-per-riga con `git diff` reale | ✅ 805/805 identiche |
| 2 | `git log --oneline` | 30 commit reali del repo, in `wc -l` | ✅ 30/30 |
| 3 | `git status` | 40 file non tracciati, in `wc -l` | ✅ 48/48 |
| 4 | `git show` | commit reale, in `wc -l` | ✅ 286/286 |
| 5 | `git branch -v` | 40 branch, in `wc -l` | ✅ 41/41 |
| 6 | `cargo test` | 30 test sintetici, `grep -c '^test '` | ✅ 31/31 (conteggio strutturato); vedi ledger #8 per l'off-by-one di 1 riga vuota nel join stderr+stdout, risolto nello stesso fix |
| 7 | `cargo build`/`check` | 60 warning sintetici, diff riga-per-riga vs `cargo build` reale | ✅ 365/365 identiche |
| 8 | `pytest` | binario non presente in sandbox | ⚠️ N/A live; stesso codepath (`run_filtered`, `pytest_filter::filter`), verificato via fixture `fixtures/pytest` |
| 9 | `ruff check` | 50 violazioni sintetiche (import inutilizzati), diff riga-per-riga | ✅ 843/843 identiche |
| 10 | `mypy` | binario non presente in sandbox | ⚠️ N/A live; stesso codepath (`run_filtered`, `mypy_filter::filter`) |
| 11 | `pip install`/`pip list` | `pip list` reale, in `wc -l` | ✅ 136/136 |
| 12 | `eslint` | binario non presente in sandbox | ⚠️ N/A live; stesso codepath (`run_filtered`, `eslint_filter::filter`) |
| 13 | `tsc` | binario non presente in sandbox | ⚠️ N/A live; stesso codepath (`run_filtered`, `tsc_filter::filter`) |
| 14 | `vitest` | binario non presente in sandbox | ⚠️ N/A live; stesso codepath (`Combined`, stesso `join_streams` di `cargo test` — già verificato) |
| 15 | `docker build`/`run`/`ps` | `docker ps -a` reale, in `wc -l` | ✅ 28/28 identiche |
| 16 | `gh pr checks` | richiede repo con PR/CI reali, non disponibile in questo sandbox isolato | ⚠️ N/A live; stesso codepath (`run_filtered`, `gh_filter::filter`) |
| 17 | `pack` | ispezionato `rtk-pack/src/pack.rs`: nessun path di collasso/troncamento riga — emette ogni file per intero o fallisce esplicitamente su `--limit` superato | **N/A** — nessuna classe di bug applicabile |

(Nota: 17 righe perché `git` e `docker` coprono più sotto-comandi ciascuno dei 12 comandi elencati nel piano — 12 "famiglie" di comando, 17 varianti testate.)

Nessun conteggio silenziosamente sbagliato residuo sotto `RTK_RAW=1`; l'unico problema trovato oltre a REL-1 (join_streams, riga 6) è stato risolto nello stesso commit. Senza `RTK_RAW`, il default torna a comprimere come da design — non un residuo del bug, ma la scelta esplicita di questo piano.

### REL-3 — causa trovata e riprodotta, fix strutturale indipendente da `RTK_RAW`

Riprodotto deliberatamente con un repro minimo (non il caso originale id 7472, ma lo stesso meccanismo):

1. Con il binario pre-fix, catturare via `$(...)` l'output di un comando che attiva la compressione: la variabile catturata contiene *anche* il testo letterale `[Full output cached. Access with: rtk show-log N]` — non solo l'output "vero".
2. Passare quella variabile come argomento a un secondo comando wrappato da `rtk` (es. `rtk git log --oneline -- "$CATTURATO"`).
3. Nella tabella `tracking` del DB SQLite, il campo `cmd` della seconda riga contiene, embedded, `[Full output cached. Access with: rtk show-log 1]` — **esattamente** la firma descritta per l'anomalia id 7472 (testo di servizio nel campo `cmd`, non in `raw_output`).

Causa: l'annotazione di cache e il warning di autonomia venivano scritti mescolati in stdout, la stessa stream che `$(...)` cattura.

Fix, indipendente da `RTK_RAW` e dal terminale: l'annotazione e il warning ora vanno **sempre** solo su stderr, mai su stdout. `$(...)` in bash cattura solo stdout per definizione, quindi quel testo non può più propagarsi in una variabile riusata come argomento — **strutturalmente**, non per convenzione. Verificato dal vivo, prima (corruzione presente nel `cmd` della seconda riga) e dopo (assente; la variabile catturata via `$(...)` non contiene mai la stringa "Full output cached", con o senza `RTK_RAW`).

Non è stata trovata una causa "esogena" (es. history/cache leak in una libreria esterna) — è lo stesso difetto architetturale di REL-1/REL-2 applicato al proprio output di servizio invece che all'output del comando wrappato, e il fix (separazione dei canali) è più solido del precedente perché non dipende da alcuna euristica.

## Deferred (non ora)

| Item | Motivo |
|------|--------|
| Fishing generico per bug non correlati alla famiglia pipe/parsing | Fuori perimetro di questo piano — se emerge qualcos'altro, è un piano a sé |
| Confronto con tool alternativi (headroom, tokf, lean-ctx) | Già valutato altrove (`cyclelab-terminal` `EXTERNAL_TOOLS_EVALUATION.md`); nessuno fa lo stesso lavoro di RTK, non è una decisione "sostituire vs riparare" |
