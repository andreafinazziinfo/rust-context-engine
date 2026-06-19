use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

const LAZY_DEV_CONTENT: &str = r#"---
description: Principi Senior Developer — YAGNI, codice minimo, modifiche mirate e diff corti per efficienza token.
alwaysApply: true
---

# Lazy Developer (YAGNI & Efficiency)

Adotta la mentalità del "miglior codice è quello che non è stato scritto" (YAGNI - You Aren't Gonna Need It) per massimizzare la chiarezza e ridurre il consumo di token.

## Regole Comportamentali

1. **Codice Minimo**: Scrivi solo le righe necessarie per implementare la feature o risolvere il bug. Evita di aggiungere astrazioni premature, classi helper o logica "per il futuro".
2. **Modifiche Mirate (Diff Corti)**: Quando modifichi file esistenti, mantieni il delta il più piccolo possibile.
   - Non riscrivere interi file o intere funzioni per modesti cambiamenti (es. ≤3 righe). Usa modifiche mirate.
   - Non formattare parti del file non correlate alla tua patch.
3. **Fiducia nella Standard Library**: Usa funzioni native del linguaggio e framework correnti. Non aggiungere nuove dipendenze o librerie esterne a meno che non sia esplicitamente richiesto o bloccante.
4. **Nessun Recap o Narrazione**: Procedi direttamente alle modifiche. Non riassumere i tuoi compiti né ripetere le istruzioni dell'utente.

## Ladder of Laziness
- Livello 1: Risolvi il problema con il minor numero di righe modificate.
- Livello 2: Usa le funzioni già esistenti nel codebase.
- Livello 3: Evita modifiche strutturali se una semplice correzione locale è sufficiente.
"#;

const TOKEN_EFFICIENCY_CONTENT: &str = r#"---
description: Token caps misurabili — BRIEF default, batch letture, no policy spam per efficienza token.
alwaysApply: true
---

# Token efficiency

## Modalità

- **BRIEF** — default operativo (patch, bugfix, routine).
- **STANDARD** — solo con **superfici multiple** o spiegazione necessaria (non audit formale).
- **AUDIT** — solo audit / security / architecture review **dichiarati**.

## Caps (operativi)

- Prima riga di ogni risposta: `Mode: BRIEF|STANDARD|AUDIT | Repo: <attivo>`.
- Max **3** file letti per batch; stop e rivaluta salvo **AUDIT** esplicito.
- Summary finale post-patch: **≤ 8 righe** (file toccati + comando test o checklist).
- **GRAPH_REPORT** / grafi: **vietato** wall of text; estratti brevi (max 10 righe).
- **Vietata** la duplicazione in chat di lunghi tratti di policy o regole.

## Skill

- **Minimizzare** stacking; una primaria + secondaria solo se a valore verificabile.
- **Non caricare** skill lunghe senza trigger chiaro.
- Planning (sparc, sequential, hermes): **no** per patch banali / one-liner.
- **codeburn**: per audit / perf / review pesante — non per ogni fix.
- **context-mode** (`skills/context-mode-routing.md`): raccomandato per molte letture.
- **caveman/brief**: (`skills/caveman/SKILL.md`).

## MCP

- I server MCP (come GitNexus / Graphify o altri indicizzatori) possono be **stale** rispetto allo stato corrente di HEAD.
- Non fare affidamento solo sulle risposte del grafo se una ricerca locale (`grep` / `git diff`) può confermare lo stato dei file.
"#;

const RTK_TOOLKIT_CONTENT: &str = r#"---
description: Guida all'uso di RTK CLI per ottimizzare il contesto e risparmiare token.
alwaysApply: true
---

# RTK CLI & Context Optimization

Questo repository utilizza **RTK CLI**, uno strumento per ottimizzare la verbosità dei comandi e archiviare i log nel database locale SQLite.

## Comandi Intercettati (Log Virtualizzati)
Quando esegui comandi standard come `git status`, `git diff`, `git log`, `cargo build`, `cargo test`, `pytest`, `ls` o `npm install`, l'output viene intercettato e filtrato per risparmiare token.
- Se l'output viene compresso, vedrai un messaggio alla fine: `[Full output cached. Access with: rtk show-log <id>]`.
- **IMPORTANTE**: Non rieseguire il comando per vedere i dettagli! Recupera il log completo usando:
  ```bash
  rtk show-log <id>
  ```

## Esplorazione Directory (`rtk pack`)
Non importare intere cartelle o leggere file multipli consecutivamente. Usa `rtk pack` per generare una rappresentazione XML compatta:
- Usa sempre `--strip` (o `-s`) per rimuovere commenti su riga singola e collassare righe vuote consecutive.
- Usa `--limit <max_tokens>` (o `-l`) per specificare un budget di token ed evitare overflow del contesto.
- Esempio:
  ```bash
  rtk pack . --strip --limit 30000
  ```

## Memoria Persistente (`rtk memory`)
Utilizza la memoria SQLite isolata per il progetto per salvare e recuperare informazioni importanti tra le sessioni (come porte database, versioni di runtime, endpoint di test):
- **Salva**: `rtk memory set <chiave> <valore>` (es: `rtk memory set db_port 5432`)
- **Leggi**: `rtk memory get <chiave>`
- **Elenca**: `rtk memory list` (esegui questo comando all'inizio di una nuova sessione di chat per sincronizzare il contesto!)
"#;

pub fn run_init() -> Result<()> {
    println!("⚙️ Bootstrapping AI Efficiency rules in the current directory...");
    run_init_in(Path::new("."))?;

    println!("✅ Created rules inside .cursor/rules/ and .agents/rules/");
    println!();

    // Automatically try to install the shell hook into Claude/Gemini settings.json
    let _ = auto_install_hook();

    println!("==========================================================");
    println!("🎉 RTK AI Rules Bootstrapped Successfully!");
    println!("==========================================================");
    println!("To complete your setup:");
    println!("1. Activate transparent terminal filtering in Claude Code by adding");
    println!("   the PreToolUse hook to your .claude/settings.json:");
    println!();
    println!("   \"hooks\": {{");
    println!("     \"PreToolUse\": [");
    println!("       {{");
    println!("         \"matcher\": \"Bash\",");
    println!("         \"hooks\": [");
    println!("           {{");
    println!("             \"type\": \"command\",");
    println!("             \"command\": \"bash /path/to/ai-token-saver/hooks/rtk-rewrite.sh\",");
    println!("             \"timeout\": 5000");
    println!("           }}");
    println!("         ]]");
    println!("       }}");
    println!("     ]");
    println!("   }}");
    println!();
    println!("2. Add shell aliases to your ~/.bashrc or ~/.zshrc for CLI wrappers:");
    println!("   alias git=\"rtk git\"");
    println!("   alias cargo=\"rtk cargo\"");
    println!("   alias pytest=\"rtk pytest\"");
    println!("   alias ls=\"rtk ls\"");
    println!("   alias npm=\"rtk npm\"");
    println!("==========================================================");

    Ok(())
}

fn run_init_in(base: &Path) -> Result<()> {
    let cursor_rules_dir = base.join(".cursor").join("rules");
    let agents_rules_dir = base.join(".agents").join("rules");

    // Create directories
    fs::create_dir_all(&cursor_rules_dir).context("failed to create .cursor/rules directory")?;
    fs::create_dir_all(&agents_rules_dir).context("failed to create .agents/rules directory")?;

    // Write rule files
    fs::write(cursor_rules_dir.join("lazy-dev.mdc"), LAZY_DEV_CONTENT)
        .context("failed to write lazy-dev.mdc to .cursor/rules")?;
    fs::write(
        cursor_rules_dir.join("token-efficiency.mdc"),
        TOKEN_EFFICIENCY_CONTENT,
    )
    .context("failed to write token-efficiency.mdc to .cursor/rules")?;
    fs::write(
        cursor_rules_dir.join("rtk-toolkit.mdc"),
        RTK_TOOLKIT_CONTENT,
    )
    .context("failed to write rtk-toolkit.mdc to .cursor/rules")?;

    fs::write(agents_rules_dir.join("lazy-dev.mdc"), LAZY_DEV_CONTENT)
        .context("failed to write lazy-dev.mdc to .agents/rules")?;
    fs::write(
        agents_rules_dir.join("token-efficiency.mdc"),
        TOKEN_EFFICIENCY_CONTENT,
    )
    .context("failed to write token-efficiency.mdc to .agents/rules")?;
    fs::write(
        agents_rules_dir.join("rtk-toolkit.mdc"),
        RTK_TOOLKIT_CONTENT,
    )
    .context("failed to write rtk-toolkit.mdc to .agents/rules")?;

    Ok(())
}

fn auto_install_hook() -> Result<()> {
    let current_dir = std::env::current_dir()?;
    let hook_path = current_dir.join("hooks").join("rtk-rewrite.sh");
    if !hook_path.exists() {
        return Ok(()); // Not in the main repository, skip auto-installation
    }
    let hook_path_str = hook_path
        .canonicalize()?
        .to_string_lossy()
        .replace('\\', "/");

    let home = std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(std::path::PathBuf::from);

    if let Some(h) = home {
        let dirs = vec![h.join(".gemini").join("antigravity"), h.join(".claude")];
        for dir in dirs {
            if dir.exists() {
                let path = dir.join("settings.json");
                let mut json = if path.exists() {
                    let content = fs::read_to_string(&path).unwrap_or_default();
                    serde_json::from_str::<serde_json::Value>(&content)
                        .unwrap_or_else(|_| serde_json::Value::Object(serde_json::Map::new()))
                } else {
                    serde_json::Value::Object(serde_json::Map::new())
                };

                inject_hook_value(&mut json, &hook_path_str);

                if let Ok(pretty) = serde_json::to_string_pretty(&json) {
                    if fs::write(&path, pretty).is_ok() {
                        println!(
                            "🎉 Automatically configured Claude/Gemini settings hook in: {}",
                            path.display()
                        );
                    }
                }
            }
        }
    }
    Ok(())
}

fn inject_hook_value(json: &mut serde_json::Value, hook_path: &str) {
    if !json.is_object() {
        *json = serde_json::Value::Object(serde_json::Map::new());
    }
    let obj = json.as_object_mut().unwrap();
    if !obj.contains_key("hooks") || !obj["hooks"].is_object() {
        obj.insert(
            "hooks".to_string(),
            serde_json::Value::Object(serde_json::Map::new()),
        );
    }
    let hooks_obj = obj.get_mut("hooks").unwrap().as_object_mut().unwrap();

    if !hooks_obj.contains_key("PreToolUse") || !hooks_obj["PreToolUse"].is_array() {
        hooks_obj.insert(
            "PreToolUse".to_string(),
            serde_json::Value::Array(Vec::new()),
        );
    }
    let pre_tool_array = hooks_obj
        .get_mut("PreToolUse")
        .unwrap()
        .as_array_mut()
        .unwrap();

    let mut bash_entry_idx = None;
    for (idx, val) in pre_tool_array.iter().enumerate() {
        if let Some(val_obj) = val.as_object() {
            if val_obj.get("matcher").and_then(|m| m.as_str()) == Some("Bash") {
                bash_entry_idx = Some(idx);
                break;
            }
        }
    }

    let hook_entry = serde_json::json!({
        "type": "command",
        "command": format!("bash {}", hook_path),
        "timeout": 5000
    });

    if let Some(idx) = bash_entry_idx {
        let bash_obj = pre_tool_array[idx].as_object_mut().unwrap();
        if !bash_obj.contains_key("hooks") || !bash_obj["hooks"].is_array() {
            bash_obj.insert("hooks".to_string(), serde_json::Value::Array(Vec::new()));
        }
        let inner_hooks = bash_obj.get_mut("hooks").unwrap().as_array_mut().unwrap();

        inner_hooks.retain(|h| {
            h.get("command")
                .and_then(|c| c.as_str())
                .map(|s| !s.contains("rtk-rewrite.sh"))
                .unwrap_or(true)
        });

        inner_hooks.push(hook_entry);
    } else {
        let new_bash_entry = serde_json::json!({
            "matcher": "Bash",
            "hooks": [hook_entry]
        });
        pre_tool_array.push(new_bash_entry);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_init_in() {
        let temp_dir = std::env::temp_dir().join(format!("rtk_init_test_{}", rand_suffix()));
        fs::create_dir_all(&temp_dir).unwrap();

        run_init_in(&temp_dir).unwrap();

        assert!(temp_dir.join(".cursor/rules/lazy-dev.mdc").exists());
        assert!(temp_dir.join(".cursor/rules/token-efficiency.mdc").exists());
        assert!(temp_dir.join(".cursor/rules/rtk-toolkit.mdc").exists());

        assert!(temp_dir.join(".agents/rules/lazy-dev.mdc").exists());
        assert!(temp_dir.join(".agents/rules/token-efficiency.mdc").exists());
        assert!(temp_dir.join(".agents/rules/rtk-toolkit.mdc").exists());

        let lazy_dev = fs::read_to_string(temp_dir.join(".cursor/rules/lazy-dev.mdc")).unwrap();
        assert_eq!(lazy_dev, LAZY_DEV_CONTENT);

        let rtk_rules = fs::read_to_string(temp_dir.join(".agents/rules/rtk-toolkit.mdc")).unwrap();
        assert_eq!(rtk_rules, RTK_TOOLKIT_CONTENT);

        fs::remove_dir_all(temp_dir).unwrap();
    }

    #[test]
    fn test_inject_hook_value() {
        let mut json = serde_json::json!({
            "existing_setting": true,
            "hooks": {
                "PreToolUse": [
                    {
                        "matcher": "Bash",
                        "hooks": [
                            {
                                "type": "command",
                                "command": "echo old",
                                "timeout": 1000
                            }
                        ]
                    }
                ]
            }
        });

        inject_hook_value(&mut json, "/path/to/rtk-rewrite.sh");

        let pre_tool = json["hooks"]["PreToolUse"].as_array().unwrap();
        assert_eq!(pre_tool.len(), 1);
        let bash_entry = &pre_tool[0];
        assert_eq!(bash_entry["matcher"], "Bash");
        let inner_hooks = bash_entry["hooks"].as_array().unwrap();
        assert_eq!(inner_hooks.len(), 2);
        assert_eq!(inner_hooks[1]["command"], "bash /path/to/rtk-rewrite.sh");
        assert_eq!(json["existing_setting"], true);
    }

    fn rand_suffix() -> u32 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .subsec_nanos()
    }
}
