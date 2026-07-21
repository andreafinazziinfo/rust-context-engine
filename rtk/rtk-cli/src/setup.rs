use anyhow::Result;
use std::fs;
use std::path::Path;

/// Write only if the file doesn't already exist — never clobber a user's hand-edited
/// rule/skill file on a repeated `rtk init` run. Profile marker files (rtk-profile.*,
/// the nested AGENTS.md, CLAUDE.md) are intentionally excluded: those are meant to be
/// regenerated when `--profile` changes and already have their own overwrite guards.
fn write_if_absent(path: &Path, content: &str) -> Result<()> {
    if !path.exists() {
        fs::write(path, content)?;
    }
    Ok(())
}

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

## Memoria Persistente (`rtk memory`) & Stale Context
Utilizza la memoria SQLite isolata per il progetto per salvare e recuperare informazioni importanti tra le sessioni:
- **Salva**: `rtk memory set <chiave> <valore>` (es: `rtk memory set db_port 5432`)
- **Leggi**: `rtk memory get <chiave>`
- **Elenca**: `rtk memory list` (esegui questo comando all'inizio di una nuova sessione di chat per sincronizzare il contesto!)
- **STALE CONTEXT RULE**: Se modifichi l'architettura o le dipendenze, il database FTS diventerà stantio. **DEVI** sovrascrivere in modo proattivo le vecchie regole o logiche tramite `rtk memory set <key>` per evitare allucinazioni in futuro.

## Hidden Chain of Thought (`rtk think`)
Quando risolvi bug complessi o scrivi algoritmi intricati, la "Chain of Thought" inquinerà permanentemente la Context Window.
- **NON** scrivere lunghi ragionamenti in chat.
- Invia il tuo ragionamento a `rtk think` usando una pipe:
  ```bash
  cat << 'EOF' | rtk think
  Il mio ragionamento su come risolvere questo bug...
  EOF
  ```
- Nella chat, stampa solo un breve riassunto (es: "Ho elaborato la logica tramite rtk think, applico la soluzione.").
"#;

const PONYTAIL_CONTENT: &str = include_str!("../assets/ponytail.mdc");
const CAVEMAN_SKILL: &str = include_str!("../assets/caveman/caveman-skill.md");
const CAVEMAN_COMMIT_SKILL: &str = include_str!("../assets/caveman/caveman-commit-skill.md");
const CAVEMAN_COMPRESS_SKILL: &str = include_str!("../assets/caveman/caveman-compress-skill.md");
const CAVEMAN_REVIEW_SKILL: &str = include_str!("../assets/caveman/caveman-review-skill.md");

/// Initialize the workspace by writing custom AI agent rules, custom skills, and setting up editor integration.
/// Supports Low, Medium, High, and Max savings profiles.
pub fn run_init(profile: &str) -> Result<()> {
    println!(
        "⚙️ Bootstrapping AI Efficiency rules in the current directory (Profile: {})...",
        profile.to_uppercase()
    );
    run_init_in(Path::new("."), profile)?;

    println!("✅ Created rules inside .cursor/rules/ and .agents/rules/");
    println!();

    // Automatically try to install the shell hook into Claude/Gemini settings.json
    let _ = auto_install_hook();

    // Automatically append aliases to .bashrc / .zshrc
    let aliases_installed = auto_install_aliases().unwrap_or(false);

    // Create user default config.json
    if rtk_db::config::create_default_config().is_ok() {
        if let Some(home) = std::env::var_os("USERPROFILE").or_else(|| std::env::var_os("HOME")) {
            let config_path = Path::new(&home).join(".config/rtk/config.json");
            println!(
                "🔒 Created personal guardrails configuration template in: {}",
                config_path.display()
            );
            println!();
        }
    }

    println!("==========================================================");
    println!("🎉 RTK AI Rules Bootstrapped Successfully!");
    println!("==========================================================");

    if !aliases_installed {
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
        println!(
            "             \"command\": \"bash /path/to/rust-context-engine/hooks/rtk-rewrite.sh\","
        );
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
        println!("   alias yarn=\"rtk yarn\"");
        println!("   alias pnpm=\"rtk pnpm\"");
        println!("   alias dotnet=\"rtk dotnet\"");
        println!("==========================================================");
    }

    // Create ~/.rtk/bin wrappers
    let _ = create_path_wrappers();

    Ok(())
}

fn create_path_wrappers() -> Result<()> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|_| anyhow::anyhow!("Could not find home directory"))?;

    let rtk_bin_dir = Path::new(&home).join(".rtk").join("bin");
    if !rtk_bin_dir.exists() {
        std::fs::create_dir_all(&rtk_bin_dir)?;
    }

    let commands = vec![
        "git",
        "cargo",
        "docker",
        "npm",
        "yarn",
        "pnpm",
        "pytest",
        "ls",
        "gradle",
        "go",
        "composer",
        "terraform",
        "dotnet",
    ];

    let script_template = r#"#!/bin/bash
# RTK transparent interceptor wrapper for COMMAND_NAME

IN_AGENT=0
if [ -n "$CLAUDE_CODE" ] || [ -n "$CURSOR" ] || [ -n "$AIDER" ] || [ -n "$ANTIGRAVITY" ] || [ -n "$GEMINI_AGENT" ] || [ -n "$AGENT_CONTEXT" ]; then
    IN_AGENT=1
fi

if [ "$IN_AGENT" -eq 1 ]; then
    exec rtk COMMAND_NAME "$@"
else
    CLEAN_PATH=$(echo "$PATH" | sed -E 's|[^:]*/.rtk/bin:?||g')
    REAL_BIN=$(PATH="$CLEAN_PATH" which COMMAND_NAME 2>/dev/null)
    if [ -n "$REAL_BIN" ]; then
        exec "$REAL_BIN" "$@"
    else
        exec COMMAND_NAME "$@"
    fi
fi
"#;

    for cmd in commands {
        let file_path = rtk_bin_dir.join(cmd);
        let content = script_template.replace("COMMAND_NAME", cmd);
        std::fs::write(&file_path, content)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(meta) = std::fs::metadata(&file_path) {
                let mut perms = meta.permissions();
                perms.set_mode(0o755);
                let _ = std::fs::set_permissions(&file_path, perms);
            }
        }
    }

    println!(
        "✅ Created PATH interceptor wrappers inside: {}",
        rtk_bin_dir.display()
    );
    Ok(())
}

fn run_init_in(base: &Path, profile: &str) -> Result<()> {
    let cursor_rules_dir = base.join(".cursor").join("rules");
    let windsurf_rules_dir = base.join(".windsurf").join("rules");
    let agents_rules_dir = base.join(".agents").join("rules");
    let agents_skills_dir = base.join(".agents").join("skills");
    let github_dir = base.join(".github");

    // Create directories
    let dirs = [
        &cursor_rules_dir,
        &windsurf_rules_dir,
        &agents_rules_dir,
        &agents_skills_dir.join("caveman"),
        &agents_skills_dir.join("caveman-commit"),
        &agents_skills_dir.join("caveman-compress"),
        &agents_skills_dir.join("caveman-review"),
        &github_dir,
    ];
    for dir in &dirs {
        fs::create_dir_all(dir)?;
    }

    // Write input rules (skip if already present — preserve user edits)
    write_if_absent(&cursor_rules_dir.join("lazy-dev.mdc"), LAZY_DEV_CONTENT)?;
    write_if_absent(
        &cursor_rules_dir.join("token-efficiency.mdc"),
        TOKEN_EFFICIENCY_CONTENT,
    )?;
    write_if_absent(
        &cursor_rules_dir.join("rtk-toolkit.mdc"),
        RTK_TOOLKIT_CONTENT,
    )?;

    write_if_absent(&agents_rules_dir.join("lazy-dev.mdc"), LAZY_DEV_CONTENT)?;
    write_if_absent(
        &agents_rules_dir.join("token-efficiency.mdc"),
        TOKEN_EFFICIENCY_CONTENT,
    )?;
    write_if_absent(
        &agents_rules_dir.join("rtk-toolkit.mdc"),
        RTK_TOOLKIT_CONTENT,
    )?;

    // Write ponytail logic
    write_if_absent(&cursor_rules_dir.join("ponytail.mdc"), PONYTAIL_CONTENT)?;
    write_if_absent(&agents_rules_dir.join("ponytail.md"), PONYTAIL_CONTENT)?;

    // Write caveman skills
    write_if_absent(
        &agents_skills_dir.join("caveman").join("SKILL.md"),
        CAVEMAN_SKILL,
    )?;
    write_if_absent(
        &agents_skills_dir.join("caveman-commit").join("SKILL.md"),
        CAVEMAN_COMMIT_SKILL,
    )?;
    write_if_absent(
        &agents_skills_dir.join("caveman-compress").join("SKILL.md"),
        CAVEMAN_COMPRESS_SKILL,
    )?;
    write_if_absent(
        &agents_skills_dir.join("caveman-review").join("SKILL.md"),
        CAVEMAN_REVIEW_SKILL,
    )?;

    // Generate output profile rule
    let profile_content = match profile.to_lowercase().as_str() {
        "max" => {
            r#"---
description: RTK Output Autonomy Profile
alwaysApply: true
---
# RTK Output Profile: MAX

You are operating under the RTK MAX profile for maximum token efficiency.
1. Always apply the Ponytail philosophy (YAGNI, minimal code, deletion over addition).
2. You MUST auto-trigger the **caveman-ultra** skill for every response (no articles, heavy abbreviation).
3. Auto-trigger **caveman-commit** for all git commit operations.
4. Auto-trigger **caveman-review** for all code reviews.
5. Auto-trigger **caveman-compress** when writing persistent memories or documentation.
"#
        }
        "high" => {
            r#"---
description: RTK Output Autonomy Profile
alwaysApply: true
---
# RTK Output Profile: HIGH

You are operating under the RTK HIGH profile for strict token efficiency.
1. Always apply the Ponytail philosophy (YAGNI, minimal code, deletion over addition).
2. You MUST auto-trigger the **caveman-full** skill for every response (no articles, short phrasing).
3. Auto-trigger **caveman-commit** for all git commit operations.
"#
        }
        "medium" => {
            r#"---
description: RTK Output Autonomy Profile
alwaysApply: true
---
# RTK Output Profile: MEDIUM

You are operating under the RTK MEDIUM profile for balanced token efficiency.
1. Always apply the Ponytail philosophy (YAGNI, minimal code, deletion over addition).
2. You MUST auto-trigger the **caveman-lite** skill for every response (complete sentences but no filler/hedging).
"#
        }
        _ => {
            r#"---
description: RTK Output Autonomy Profile
alwaysApply: true
---
# RTK Output Profile: LOW

You are operating under the RTK LOW profile for safe efficiency.
1. Always apply the Ponytail philosophy (YAGNI, minimal code, deletion over addition).
2. Write concise, standard technical language without unnecessary conversational filler.
"#
        }
    };

    // Distribute the profile universally
    fs::write(cursor_rules_dir.join("rtk-profile.mdc"), profile_content)?;
    fs::write(windsurf_rules_dir.join("rtk-profile.md"), profile_content)?;
    fs::write(agents_rules_dir.join("AGENTS.md"), profile_content)?;

    // CLAUDE.md is a user-curated file — never overwrite it. Append the RTK
    // profile once, preserving existing content (same guard pattern as the
    // copilot-instructions append below). Empty/absent file is written fresh.
    let claude_file = base.join("CLAUDE.md");
    let existing_claude = fs::read_to_string(&claude_file).unwrap_or_default();
    if existing_claude.trim().is_empty() {
        fs::write(&claude_file, profile_content)?;
    } else if !existing_claude.contains("RTK Output Profile") {
        fs::write(
            &claude_file,
            format!("{}\n\n{}", existing_claude, profile_content),
        )?;
    }

    // Append to copilot instructions
    let copilot_file = github_dir.join("copilot-instructions.md");
    let existing = fs::read_to_string(&copilot_file).unwrap_or_default();
    if !existing.contains("RTK Output Profile") {
        fs::write(
            &copilot_file,
            format!("{}\n\n{}", existing, profile_content),
        )?;
    }

    Ok(())
}

const ALIASES_BLOCK: &str = r#"
# RTK AI Token Saver Aliases
alias git="rtk git"
alias cargo="rtk cargo"
alias pytest="rtk pytest"
alias ls="rtk ls"
alias npm="rtk npm"
alias yarn="rtk yarn"
alias pnpm="rtk pnpm"
alias dotnet="rtk dotnet"
alias composer="rtk composer"
alias terraform="rtk terraform"
"#;

fn auto_install_aliases() -> Result<bool> {
    let home = std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(std::path::PathBuf::from);

    if let Some(h) = home {
        let mut installed = false;
        let shells = [".bashrc", ".zshrc", ".profile"];

        for shell in shells {
            let path = h.join(shell);
            if path.exists() {
                let content = fs::read_to_string(&path).unwrap_or_default();
                if !content.contains("RTK AI Token Saver Aliases") {
                    let mut file = fs::OpenOptions::new().append(true).open(&path)?;
                    use std::io::Write;
                    writeln!(file, "{}", ALIASES_BLOCK)?;
                    println!("🎉 Added RTK aliases to: {}", path.display());
                    installed = true;
                } else {
                    println!("ℹ️ RTK aliases already present in: {}", path.display());
                    installed = true;
                }
            }
        }
        return Ok(installed);
    }
    Ok(false)
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
    let Some(obj) = json.as_object_mut() else {
        return;
    };
    if !obj.contains_key("hooks") || !obj["hooks"].is_object() {
        obj.insert(
            "hooks".to_string(),
            serde_json::Value::Object(serde_json::Map::new()),
        );
    }
    let Some(hooks_obj) = obj.get_mut("hooks").and_then(|h| h.as_object_mut()) else {
        return;
    };

    if !hooks_obj.contains_key("PreToolUse") || !hooks_obj["PreToolUse"].is_array() {
        hooks_obj.insert(
            "PreToolUse".to_string(),
            serde_json::Value::Array(Vec::new()),
        );
    }
    let Some(pre_tool_array) = hooks_obj
        .get_mut("PreToolUse")
        .and_then(|p| p.as_array_mut())
    else {
        return;
    };

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
        let Some(bash_obj) = pre_tool_array[idx].as_object_mut() else {
            return;
        };
        if !bash_obj.contains_key("hooks") || !bash_obj["hooks"].is_array() {
            bash_obj.insert("hooks".to_string(), serde_json::Value::Array(Vec::new()));
        }
        let Some(inner_hooks) = bash_obj.get_mut("hooks").and_then(|h| h.as_array_mut()) else {
            return;
        };

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
    fn test_inject_hook_value_survives_malformed_settings() {
        // Non-object root: replaced with an object, then the hook is injected.
        let mut json = serde_json::json!("not an object");
        inject_hook_value(&mut json, "/path/to/rtk-rewrite.sh");
        assert!(json["hooks"]["PreToolUse"].is_array());

        // `hooks` present but the wrong type must be overwritten, not panic.
        let mut json = serde_json::json!({ "hooks": 42 });
        inject_hook_value(&mut json, "/path/to/rtk-rewrite.sh");
        let arr = json["hooks"]["PreToolUse"].as_array().unwrap();
        assert_eq!(arr[0]["matcher"], "Bash");
    }

    #[test]
    fn test_run_init_in() {
        let temp_dir = std::env::temp_dir().join(format!("rtk_init_test_{}", rand_suffix()));
        fs::create_dir_all(&temp_dir).unwrap();

        run_init_in(&temp_dir, "high").unwrap();

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
    fn test_init_preserves_existing_claude_md() {
        let temp_dir = std::env::temp_dir().join(format!("rtk_init_preserve_{}", rand_suffix()));
        fs::create_dir_all(&temp_dir).unwrap();

        let curated = "# My Project\n\n## Stack\ndocker compose up -d\n";
        let claude = temp_dir.join("CLAUDE.md");
        fs::write(&claude, curated).unwrap();

        run_init_in(&temp_dir, "high").unwrap();

        let after = fs::read_to_string(&claude).unwrap();
        // Existing curated content is preserved...
        assert!(after.contains("## Stack"), "curated content was destroyed");
        assert!(after.contains("docker compose up -d"));
        // ...and the RTK profile is appended once.
        assert!(after.contains("RTK Output Profile"));

        // Idempotent: a second init must not duplicate the profile block.
        run_init_in(&temp_dir, "high").unwrap();
        let after2 = fs::read_to_string(&claude).unwrap();
        assert_eq!(
            after2.matches("RTK Output Profile").count(),
            1,
            "RTK profile appended more than once"
        );

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

    #[test]
    fn test_create_default_config() {
        let temp_dir = std::env::temp_dir().join(format!("rtk_config_test_{}", rand_suffix()));
        fs::create_dir_all(&temp_dir).unwrap();

        // Temporarily override HOME and USERPROFILE env vars
        let original_home = std::env::var_os("HOME");
        let original_userprofile = std::env::var_os("USERPROFILE");
        std::env::set_var("HOME", &temp_dir);
        std::env::set_var("USERPROFILE", &temp_dir);

        // Call our creation helper
        let res = rtk_db::config::create_default_config();
        assert!(res.is_ok());

        // Verify that the file was created in <temp_dir>/.config/rtk/config.json
        let expected_path = temp_dir.join(".config/rtk/config.json");
        assert!(expected_path.exists());

        let content = fs::read_to_string(&expected_path).unwrap();
        assert!(content.contains("denied_commands"));
        assert!(content.contains("custom_patterns"));

        // Restore env vars
        if let Some(h) = original_home {
            std::env::set_var("HOME", h);
        } else {
            std::env::remove_var("HOME");
        }
        if let Some(up) = original_userprofile {
            std::env::set_var("USERPROFILE", up);
        } else {
            std::env::remove_var("USERPROFILE");
        }

        fs::remove_dir_all(temp_dir).unwrap();
    }

    fn rand_suffix() -> u32 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .subsec_nanos()
    }
}
