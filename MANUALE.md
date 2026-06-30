# 📖 Manuale Completo: RTK Context Engine (v2.3.2)

**RTK** (Rust Token Killer) è un toolkit proxy a riga di comando e un harness di sviluppo progettato per gli sviluppatori che lavorano con assistenti AI (Cursor, Claude Code, GitHub Copilot, ecc.). Il suo scopo è **tagliare i costi delle API LLM del 60-80%** riducendo il rumore dai log, compattando il codice sorgente con parser AST e offrendo policy di sicurezza e budget locali.

---

## 🚀 1. Setup e Inizializzazione

### `rtk init`
Inizializza la configurazione globale di RTK. Crea il file di configurazione predefinito in `~/.config/rtk/config.json`.
* **Sintassi**: `rtk init [--profile <low|medium|high|max>]`
* **Opzioni**:
  * `--profile`: Seleziona il livello di restrizione iniziale per DLP e comandi bloccati.
* **Profili disponibili**:
  * `low`: Filtri DLP minimi, nessun blocco dei comandi.
  * `medium` (Consigliato): DLP standard attivo, protezione da comandi distruttivi (es. `git push --force`).
  * `high` / `max`: Filtri di DLP aggressivi (entropia elevata), blocchi estesi sui comandi e restrizioni di budget.

### `rtk sync-rules`
Propaga in modo ricorsivo le regole dell'agente contenute in `.cursor/rules/` o `.agents/rules/` dal root del workspace a tutte le sottocartelle del progetto. Questo evita che l'agente perda l'accesso alle linee guida del progetto quando si sposta in sottocartelle.
* **Sintassi**: `rtk sync-rules`

---

## 🛡️ 2. Sicurezza, DLP e Guardrail (`rtk config`)

RTK include un motore DLP (Data Loss Prevention) locale e un gestore di comandi vietati configurabile a livello globale.

### `rtk config show`
Mostra la configurazione attualmente attiva (risultato dell'unione tra la configurazione globale e quella locale del progetto).
* **Sintassi**: `rtk config show`

### `rtk config deny add`
Aggiunge una regola regex di sicurezza per impedire agli agenti AI di eseguire comandi dannosi o non desiderati. Se un agente tenta di eseguirli, RTK ne blocca l'esecuzione ritornando l'exit code `2`.
* **Sintassi**: `rtk config deny add "<regex>"`
* **Esempio**: `rtk config deny add "git push.*--force"`

### `rtk config dlp add`
Aggiunge una regex DLP personalizzata per censurare dati sensibili (come token proprietari, password o dati sensibili) prima che vengano passati all'AI.
* **Sintassi**: `rtk config dlp add "<regex>"`
* **Esempio**: `rtk config dlp add "MY_API_KEY_[0-9a-zA-Z]{32}"`

### `rtk config export`
Esporta la configurazione globale corrente in formato JSON nello stdout, ideale per backup o per sincronizzare i propri dotfiles.
* **Sintassi**: `rtk config export > backup.json`

### `rtk config import`
Importa e sovrascrive la configurazione globale da un file JSON o direttamente dallo stdin.
* **Sintassi**:
  * Da file: `rtk config import --path backup.json`
  * Da stdin: `cat backup.json | rtk config import`

---

## 📊 3. Stima dei Costi e Risparmio Token

### `rtk estimate`
Analizza il `git diff` corrente e calcola la quantità esatta di token originale e compressa da RTK. Mostra una tabella comparativa con il costo monetario stimato (in USD) su diversi LLM (Claude Opus, Claude Sonnet, GPT-5, Gemini) e calcola il risparmio potenziale.
* **Sintassi**: `rtk estimate` *(alias: `rtk est`)*

### `rtk stats` / `rtk gain`
Mostra un riepilogo dettagliato del risparmio cumulativo dei token, il numero di comandi intercettati e i dollari risparmiati in API. Con l'opzione `--chart`, genera un grafico ASCII dell'andamento dei costi degli ultimi 10 giorni.
* **Sintassi**: `rtk stats [--chart]`

### `rtk dashboard`
Avvia un server web locale leggero e apre automaticamente una dashboard HTML interattiva nel browser per visualizzare graficamente l'utilizzo delle risorse, il trend di risparmio e l'utilizzo del budget.
* **Sintassi**: `rtk dashboard`

### `rtk telemetry export`
Esporta le metriche accumulate in formato Prometheus-compatible per consentire il monitoraggio centralizzato.
* **Sintassi**: `rtk telemetry export`

---

## 🗜️ 4. Compressione del Codice e Struttura AST

### `rtk pack`
Esporta una directory strutturata in un blocco XML ottimizzato e super-compresso per l'AI.
* **Sintassi**: `rtk pack [percorso] [opzioni]`
* **Opzioni principali**:
  * `-s, --strip`: Rimuove tutti i commenti a riga singola e gli spazi vuoti superflui per tagliare i token inutili.
  * `-k, --skeleton`: Utilizza parser AST integrati (via `tree-sitter`) per estrarre solo le firme delle funzioni/metodi/classi (supporta Rust, JS, TS, Python, Go, Java), rimuovendo il corpo del codice e riducendo le dimensioni dei file fino all'85%.
  * `-l, --limit <tokens>`: Imposta una quota massima di token oltre la quale interrompere l'impacchettamento, prevenendo sovraccarichi del contesto.

---

## 🧠 5. Gestione Memoria e Logs Virtuali

### `rtk memory`
Consente all'agente di memorizzare parametri operativi del progetto (come porte di rete, flag di debug, indirizzi database di test) in un database SQLite locale (`.rtk/rtk.db`), evitando di ripetere ricerche nei file.
* **Sintassi**:
  * Salvataggio: `rtk memory set <chiave> <valore>`
  * Lettura: `rtk memory get <chiave>`
  * Elenco: `rtk memory list`

### `rtk show-log`
Quando RTK intercetta ed esegue comandi con output lunghi (come `cargo test`), restituisce all'AI solo un output ultra-compresso (es. i soli test falliti e il riassunto) per risparmiare token, memorizzando il log completo nel database locale. Se l'agente ha bisogno di ispezionare il log grezzo e completo, può richiamarlo istantaneamente tramite ID senza rieseguire il comando.
* **Sintassi**: `rtk show-log <log_id>`

### `rtk think`
Legge da stdin e memorizza riflessioni complesse o catene di pensiero (Chain-of-Thought) dell'agente all'interno del database FTS5 locale del progetto, tenendole fuori dal contesto della conversazione corrente dell'editor.
* **Sintassi**: `echo "ragionamento..." | rtk think`

---

## 🤖 6. Funzionalità di Integrazione MCP ed Esecuzione

### `rtk rewrite`
La funzionalità core che agisce come proxy di comando. Intercetta ed esegue un comando CLI applicando i filtri di formattazione dell'output in tempo reale prima di restituire il controllo all'AI.
* **Sintassi**: `rtk rewrite "<command>"`

### MCP Server: `get_budget_status`
Se eseguito in modalità MCP (Model Context Protocol), RTK espone all'AI il tool `get_budget_status`. Questo tool consente agli agenti AI autonomi di auto-regolarsi controllando autonomamente il limite di spesa impostato e bloccando le proprie catene di esecuzione se il budget è esaurito.

---

## 📦 Ecosistema dei Moduli Rust (`rtk-context-*`)

Se decidi di importare le funzionalità di RTK all'interno di altri tuoi applicativi Rust, i moduli del workspace sono suddivisi come segue:

| Pacchetto | Scopo |
| :--- | :--- |
| **`rtk-context-filters`** | Filtri regex e compressione di output standard (`cargo`, `git`, `pytest`, `docker`). |
| **`rtk-context-db`** | Database SQLite FTS5 locale, telemetria, storage dei log e motore DLP. |
| **`rtk-context-index`** | Analizzatore del grafo del codice, navigatore simboli e blast-radius. |
| **`rtk-context-pack`** | Modulo per comprimere e impacchettare codebase usando parser AST `tree-sitter`. |
| **`rtk-context-mcp`** | Server MCP compatibile per esporre i tool agli agenti AI. |
| **`rtk-context-engine`** | **L'eseguibile CLI principale** (`rtk`). Collega tutti i moduli e gestisce la CLI. |
