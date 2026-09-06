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
| **REL-1** | **Fix #77** (`ls` filter corrompe conteggi piped) | S–M | `ls -1 <dir con 30+ voci> \| wc -l` restituisce il conteggio reale, non il conteggio delle righe di prosa compressa | Fix proposto nell'issue: rilevare stdout non-TTY (`!isatty(1)`) e saltare la summarizzazione — stesso check presumibilmente già usato per il fix SIGPIPE (#16) |
| **REL-2** | **Sweep sistematico, non esplorativo** — stessa classe di bug su ogni comando wrappato (`git`, `cargo`, `pytest`, `ruff`, `mypy`, `pip`, `eslint`, `tsc`, `vitest`, `docker`, `gh`, `pack`) | M | Per ciascuno: un caso con output lungo abbastanza da attivare la summarizzazione, pipato in `wc -l`/`grep -c`/un parser reale — conteggio corretto o troncamento esplicito (mai un numero sbagliato silenzioso) | Regressione mirata sul difetto già dimostrato in REL-1, non un fishing generico — se un comando non ha una path di summarizzazione, si documenta "N/A" e si passa oltre |
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

## Root cause (comune a REL-1/REL-2/REL-3)

Tutti i comandi wrappati passano da un unico punto di snodo, `execute_with_filter` (`rtk-cli/src/filter_pipeline.rs`). Prima del fix, la compressione (filtro per-comando + `post_process_filter_output` con regole regex/profilo + `distiller`) veniva sempre applicata, e l'annotazione informativa `[Full output cached. Access with: rtk show-log N]` / il warning di autonomia venivano sempre appesi allo stream stampato — **indipendentemente dal fatto che quello stream fosse un terminale interattivo o una pipe**. Un terminale reale e una pipe/cattura di subprocess (esattamente come un harness agente cattura `rtk`) sono indistinguibili se non si controlla lo stato del file descriptor.

Fix (un solo punto, `filter_pipeline.rs`):

1. `compress_for_display()` — se lo stream di destinazione non è un terminale reale (`std::io::IsTerminal`), la compressione viene **saltata del tutto** e si restituisce l'output grezzo (comunque passato da redazione DLP, che è un controllo di sicurezza indipendente e non tocca il conteggio righe). Se è un terminale reale, comportamento identico a prima.
2. L'annotazione `[Full output cached...]` e il warning di autonomia vengono appesi **solo** se lo stream ricevente è un terminale reale — altrimenti non vengono mai scritti nello stream piped.
3. `join_streams()` — il join di stderr+stdout in modalità `Combined` non inserisce più un `\n` spurio quando uno dei due lati è vuoto o già terminato da newline (bug minore trovato durante lo sweep REL-2, stessa famiglia, vedi ledger #8).

Questo copre meccanicamente REL-1 e l'intero sweep REL-2 (stesso codice per tutti i comandi), e spiega/risolve REL-3 (vedi sotto) come conseguenza diretta, non come fix separato.

### Note dalla review (`/code-review high`) e trade-off accettato

Una review dedicata sul diff ha trovato e fatto correggere due bug reali nella stessa famiglia, oltre a un miglioramento di manutenibilità:

- I filtri regex custom (`rtk filter add --pattern ... --action strip|collapse`) sono un controllo di sicurezza scelto dall'utente, non un'euristica di compressione — venivano saltati insieme al resto quando lo stream non era una TTY, esattamente il canale (agente/pipe) dove un segreto ha più probabilità di finire in un log. Corretto: ora girano sempre (come la redazione DLP), indipendentemente dal terminale. Test aggiunto: `test_custom_regex_filter_still_applies_when_piped`.
- La redazione DLP delle chiavi private (`dlp.rs`) collassava un blocco PEM multi-riga in un'unica riga `[REDACTED_PRIVATE_KEY]`, cambiando il conteggio righe di qualunque output piped che contenesse per caso una chiave privata — la stessa classe di bug di #77, sul path pensato per garantirne la correttezza. Corretto: il numero di newline del blocco originale viene preservato nel replacement. Test aggiunto: `test_redact_private_key_preserves_line_count`.
- Duplicazione minore della guardia tty sui due punti di append (annotazione cache + warning di autonomia): accorpata in `append_marker_if_tty()`.

La review ha inoltre segnalato che, sul percorso non-TTY, `filtered_db == raw_db` azzera la metrica "token risparmiati" per ogni invocazione non interattiva (il caso d'uso reale prevalente, cioè esattamente l'automazione/harness agente). Questo **non è un difetto**: è la conseguenza diretta e onesta della scelta di design di questo piano — se la compressione non viene applicata, non ci sono token risparmiati da dichiarare, e la metrica precedente (che continuava a contare "risparmi" su un output che in realtà veniva già mostrato corrotto quando piped) era quella fuorviante. Non è stata reintrodotta compressione fittizia solo per gonfiare la statistica. Effetto collaterale noto e accettato, non nascosto: `rtk stats`/`rtk audit` mostreranno risparmi vicini a zero per l'uso non interattivo dopo questo fix.

### REL-1 — prima/dopo (repro esatto dell'issue #77)

Ambiente: directory con 36 voci reali, `rtk` invocato come sottoprocesso con stdout catturato via pipe (`.output()` / `$(...)` — esattamente come lo cattura un harness agente).

| | `ls -1 <dir> \| wc -l` |
|---|---|
| Ground truth (`command ls`) | 36 |
| Binario `rtk` pre-fix (installato in `~/.local/bin/rtk`) | **19** (bug riprodotto dal vivo) |
| Binario `rtk` post-fix (questo branch) | **36** |

Test automatico aggiunto: `rtk-cli/tests/integration_test.rs::ls_filter_piped_reports_true_entry_count_issue_77` (asserisce il conteggio reale e l'assenza del marker `more entries` su output catturato via `.output()`).

Comportamento interattivo invariato: con stdout attaccato a un vero terminale (verificato con `script -qec ... /dev/null` per simulare una tty), `rtk ls` continua a comprimere l'elenco lungo e ad appendere `[Full output cached...]` esattamente come prima — il fix non tocca l'esperienza umana a terminale, solo la composizione via pipe.

### REL-2 — sweep sui 12 comandi wrappati

Tutti passano dallo stesso snodo (`execute_with_filter`), quindi la garanzia è meccanica per tutti; dove il binario reale era disponibile in sandbox è stato eseguito un repro dal vivo (output reale vs `rtk`-piped, diff a livello di riga). Dove il binario non era disponibile, verificato che il comando passa dallo stesso codice (lettura `dispatch.rs`) più test a livello di funzione di filtro sulle fixture esistenti.

| # | Comando | Repro | Esito |
|---|---------|-------|-------|
| 1 | `git diff` | diff sintetico 400 righe modificate → piped, confronto riga-per-riga con `git diff` reale | ✅ 805/805 identiche |
| 2 | `git log --oneline` | 30 commit reali del repo, piped in `wc -l` | ✅ 30/30 |
| 3 | `git status` | 40 file non tracciati, piped in `wc -l` | ✅ 48/48 |
| 4 | `git show` | commit reale, piped in `wc -l` | ✅ 286/286 |
| 5 | `git branch -v` | 40 branch, piped in `wc -l` | ✅ 41/41 |
| 6 | `cargo test` | 30 test sintetici, `grep -c '^test '` | ✅ 31/31 (conteggio strutturato); vedi ledger #8 per l'off-by-one di 1 riga vuota nel join stderr+stdout, risolto in questo stesso fix |
| 7 | `cargo build`/`check` | 60 warning sintetici, diff riga-per-riga vs `cargo build` reale | ✅ 365/365 identiche |
| 8 | `pytest` | binario non presente in sandbox | ⚠️ N/A live; stesso codepath (`run_filtered`, `pytest_filter::filter`), verificato via fixture `fixtures/pytest` |
| 9 | `ruff check` | 50 violazioni sintetiche (import inutilizzati), diff riga-per-riga | ✅ 843/843 identiche |
| 10 | `mypy` | binario non presente in sandbox | ⚠️ N/A live; stesso codepath (`run_filtered`, `mypy_filter::filter`) |
| 11 | `pip install`/`pip list` | `pip list` reale, piped in `wc -l` | ✅ 136/136 |
| 12 | `eslint` | binario non presente in sandbox | ⚠️ N/A live; stesso codepath (`run_filtered`, `eslint_filter::filter`) |
| 13 | `tsc` | binario non presente in sandbox | ⚠️ N/A live; stesso codepath (`run_filtered`, `tsc_filter::filter`) |
| 14 | `vitest` | binario non presente in sandbox | ⚠️ N/A live; stesso codepath (`Combined`, stesso `join_streams` di `cargo test` — già verificato) |
| 15 | `docker build`/`run`/`ps` | `docker ps -a` reale, piped in `wc -l` | ✅ 28/28 identiche |
| 16 | `gh pr checks` | richiede repo con PR/CI reali, non disponibile in questo sandbox isolato | ⚠️ N/A live; stesso codepath (`run_filtered`, `gh_filter::filter`) |
| 17 | `pack` | ispezionato `rtk-pack/src/pack.rs`: nessun path di collasso/troncamento riga — emette ogni file per intero o fallisce esplicitamente su `--limit` superato | **N/A** — nessuna classe di bug applicabile (nessuna estimazione, mai un errore silenzioso) |

(Nota: la tabella conta 17 righe perché `git` e `docker` coprono più sotto-comandi ciascuno dei 12 comandi elencati nel piano — 12 "famiglie" di comando, 17 varianti testate.)

Nessun conteggio silenziosamente sbagliato residuo dopo il fix; l'unico problema trovato oltre a REL-1 (join_streams, riga 6) è stato risolto nello stesso commit.

### REL-3 — causa trovata e riprodotta

Riprodotto deliberatamente con un repro minimo (non il caso originale id 7472, ma lo stesso meccanismo):

1. Con il binario pre-fix, catturare l'output di un comando che attiva la compressione via `$(...)` (una pipe, esattamente come l'harness Claude Code cattura `rtk`): la variabile catturata contiene *anche* il testo letterale `[Full output cached. Access with: rtk show-log N]` (e, con `ls`, anche `... and N more entries ...`) — non solo l'output "vero".
2. Passare quella variabile come argomento a un secondo comando wrappato da `rtk` (es. `rtk git log --oneline -- "$CATTURATO"`).
3. Ispezionando la tabella `tracking` del DB SQLite, il campo `cmd` della seconda riga contiene, embedded, `... and N more entries ...` e `[Full output cached. Access with: rtk show-log 1]` — **esattamente** la firma descritta per l'anomalia id 7472 (testo di servizio nel campo `cmd`, non in `raw_output`).

Causa: `execute_with_filter` appendeva l'annotazione di cache e il warning di autonomia allo stream stampato **senza controllare se quello stream fosse un terminale**. Qualunque consumatore non interattivo (pipe, `$(...)`, cattura di subprocess) riceveva quel testo di servizio come parte dell'output "reale", e se quel testo veniva poi riusato come argomento di un comando successivo passato da `rtk`, finiva registrato dentro il campo `cmd` di quella riga.

Fix: stesso cambiamento di REL-1/REL-2 — l'annotazione e il warning ora vengono scritti solo quando lo stream di destinazione è un terminale reale (`stdout_is_tty` / `stderr_is_tty`). Con il binario post-fix, lo stesso repro (passi 1–3) non produce alcuna corruzione: la variabile catturata non contiene mai il marker, quindi non può propagarsi nel campo `cmd` di un comando successivo. Verificato dal vivo con lo stesso script di repro, prima (corruzione presente) e dopo (assente).

Non è stata trovata una causa "esogena" (es. history/cache leak nel senso di un bug di libreria esterna) — è lo stesso difetto architetturale di REL-1/REL-2 applicato al proprio output di servizio invece che all'output del comando wrappato.

## Deferred (non ora)

| Item | Motivo |
|------|--------|
| Fishing generico per bug non correlati alla famiglia pipe/parsing | Fuori perimetro di questo piano — se emerge qualcos'altro, è un piano a sé |
| Confronto con tool alternativi (headroom, tokf, lean-ctx) | Già valutato altrove (`cyclelab-terminal` `EXTERNAL_TOOLS_EVALUATION.md`); nessuno fa lo stesso lavoro di RTK, non è una decisione "sostituire vs riparare" |
