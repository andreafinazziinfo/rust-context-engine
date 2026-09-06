# RTK — Piano affidabilità post-adozione (2026-09)

| Campo | Valore |
|-------|--------|
| **Baseline** | v2.4.2 · Fase C/D (`PLAN_CLOSURE.md`) chiusa a giugno, "solo bugfix bloccanti" da allora |
| **Obiettivo fase** | Il momento "misura → poi decidi" previsto da `PLAN_CLOSURE.md` — 3 mesi di uso quotidiano reale (CycleLab + Titan) hanno prodotto 3 bug concreti nello stesso trimestre, tutti nella stessa famiglia (output filtrato non sicuro da comporre con pipe/parsing a valle) |
| **Status** | ⬜ Non iniziato |
| **Aggiornato** | 2026-09-06 |

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

- [ ] REL-1: test reale (non solo lettura del codice) — stesso repro dell'issue, prima e dopo il fix
- [ ] REL-2: tabella di 12 righe (uno per comando wrappato), esito per ciascuno, non un "sembra a posto" generico
- [ ] REL-3: causa capita, o esplicitamente chiusa come "non riproducibile" con l'evidenza grezza allegata — non lasciata a metà senza una delle due
- [ ] Issue #77 chiusa con riferimento al fix; nuove issue aperte per ogni bug REL-2 trovi, con lo stesso standard di evidenza di #77 (repro minimo, non solo descrizione)

## Deferred (non ora)

| Item | Motivo |
|------|--------|
| Fishing generico per bug non correlati alla famiglia pipe/parsing | Fuori perimetro di questo piano — se emerge qualcos'altro, è un piano a sé |
| Confronto con tool alternativi (headroom, tokf, lean-ctx) | Già valutato altrove (`cyclelab-terminal` `EXTERNAL_TOOLS_EVALUATION.md`); nessuno fa lo stesso lavoro di RTK, non è una decisione "sostituire vs riparare" |
