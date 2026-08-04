use rtk_db::{config, pricing, status, tracking};
use std::path::PathBuf;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum DoctorOutcome {
    Ok,
    Warnings,
    Critical,
}

pub fn run_doctor() -> DoctorOutcome {
    println!("🩺 Running RTK Health Check...");
    println!("==================================================");

    let mut critical = false;
    let mut warnings = false;

    print!("📦 RTK Version: ");
    println!("✅ {} (build)", env!("CARGO_PKG_VERSION"));

    print!("🗄️  Database Access: ");
    match tracking::get_recent_logs(1) {
        Ok(_) => println!("✅ OK"),
        Err(e) => {
            println!("❌ FAILED (Error: {e})");
            println!("   👉 Check RTK_DB_PATH or folder permissions.");
            critical = true;
        }
    }

    if let Ok(db_path) = std::env::var("RTK_DB_PATH") {
        let on_windows = std::env::var("USERPROFILE").is_ok();
        if on_windows && db_path.contains("/mnt/") {
            println!("⚠️  RTK_DB_PATH looks like a WSL path but shell is Windows-side");
            println!("   👉 Use a native path or run RTK inside WSL for dev/test.");
            warnings = true;
        }
    }

    print!("📁 Project .rtk/: ");
    match std::env::current_dir() {
        Ok(cwd) => {
            let rtk_dir = cwd.join(".rtk");
            if rtk_dir.is_dir() {
                println!("✅ OK ({})", rtk_dir.display());
            } else {
                println!("⚠️  missing (created on first tracked command)");
                warnings = true;
            }
        }
        Err(e) => {
            println!("❌ FAILED ({e})");
            critical = true;
        }
    }

    print!("💰 Pricing Registry: ");
    let registry = pricing::get_registry();
    if registry.models.is_empty() {
        println!("❌ FAILED (no models loaded)");
        critical = true;
    } else {
        println!(
            "✅ OK (revision: {}, {} models)",
            registry.pricing_revision,
            registry.models.len()
        );
    }

    print!("🧠 Project Memory: ");
    match tracking::memory_doctor() {
        Ok(report) => {
            if report.duplicates.is_empty()
                && report.stale.is_empty()
                && report.contradictory.is_empty()
            {
                println!("✅ OK");
            } else {
                println!("⚠️  issues detected");
                if !report.duplicates.is_empty() {
                    println!("   duplicates: {}", report.duplicates.join(", "));
                }
                if !report.stale.is_empty() {
                    println!("   stale keys (>30d): {}", report.stale.len());
                }
                if !report.contradictory.is_empty() {
                    println!("   contradictory: {}", report.contradictory.len());
                }
                warnings = true;
            }
        }
        Err(e) => {
            println!("⚠️  skipped ({e})");
            warnings = true;
        }
    }

    print!("🔍 Code Index: ");
    match rtk_index::get_index_status() {
        Ok(st) if st.symbols_count == 0 => {
            println!("⚠️  empty (run `rtk index run`)");
            warnings = true;
        }
        Ok(st) if st.stale => {
            println!(
                "⚠️  stale ({} symbols, last: {})",
                st.symbols_count,
                st.last_indexed
                    .map(|t| t.to_string())
                    .unwrap_or_else(|| "never".into())
            );
            warnings = true;
        }
        Ok(st) => {
            println!(
                "✅ OK ({} symbols, {:.0}% graph)",
                st.symbols_count, st.graph_coverage
            );
        }
        Err(e) => {
            println!("⚠️  unavailable ({e})");
            warnings = true;
        }
    }
    println!("   ℹ️  No file watcher — run `rtk index run` after large refactors");

    print!("⚙️  Config Regex: ");
    let regex_errors = config::validate_regex_config();
    if regex_errors.is_empty() {
        println!("✅ OK");
    } else {
        println!("❌ FAILED");
        for err in &regex_errors {
            println!("   - {err}");
        }
        critical = true;
    }

    print!("🪝 PreToolUse Hook: ");
    if status::is_rewrite_hook_installed() {
        println!("✅ OK");
    } else {
        println!("⚠️  not found in ~/.claude or ~/.gemini settings");
        println!("   👉 Run `rtk init` or add hooks/rtk-rewrite.sh to settings.json");
        println!("   👉 Quickstart: docs/QUICKSTART.md (install → init → index run)");
        warnings = true;
    }

    print!("🐚 Shell Aliases: ");
    if shell_aliases_installed() {
        println!("✅ OK");
    } else {
        println!("⚠️  RTK aliases not found in shell rc");
        println!("   👉 Run `rtk init` to install aliases.");
        println!("   👉 Quickstart: docs/QUICKSTART.md");
        warnings = true;
    }

    print!("🗣️  Caveman Skill: ");
    let profile_path = std::path::Path::new(".cursor/rules/rtk-profile.mdc");
    match std::fs::read_to_string(profile_path)
        .ok()
        .and_then(|c| status::active_profile_level(&c))
        .and_then(status::active_profile_skill)
    {
        None => println!("ℹ️  no caveman skill referenced (LOW profile or `rtk init` not run)"),
        Some(skill) => {
            let claude_skill = PathBuf::from(".claude/skills").join(skill).join("SKILL.md");
            if claude_skill.exists() {
                println!("✅ OK (\"{skill}\" found in .claude/skills/)");
            } else {
                println!(
                    "⚠️  \"{skill}\" referenced by profile but missing at {}",
                    claude_skill.display()
                );
                println!(
                    "   👉 Run `rtk init --profile <level> --force-profile` to reinstall skill files."
                );
                warnings = true;
            }
        }
    }

    print!("🦀 Rust Toolchain: ");
    match std::process::Command::new("rustc")
        .arg("--version")
        .output()
    {
        Ok(o) if o.status.success() => {
            println!("✅ OK ({})", String::from_utf8_lossy(&o.stdout).trim());
        }
        _ => {
            println!("⚠️  rustc not found (needed only for source builds)");
            warnings = true;
        }
    }

    print!("💾 Workspace Access: ");
    if std::fs::metadata(".").is_ok() {
        println!("✅ OK");
    } else {
        println!("❌ FAILED");
        critical = true;
    }

    println!("==================================================");
    if critical {
        println!("❌ RTK doctor found critical issues.");
        DoctorOutcome::Critical
    } else if warnings {
        println!("⚠️  RTK doctor completed with warnings.");
        DoctorOutcome::Warnings
    } else {
        println!("✅ RTK is healthy and ready to optimize your AI coding agent workflow!");
        DoctorOutcome::Ok
    }
}

fn shell_aliases_installed() -> bool {
    let home = std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from);

    if let Some(h) = home {
        for shell in [".bashrc", ".zshrc", ".profile"] {
            let path = h.join(shell);
            if path.exists() {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if content.contains("RTK AI Token Saver Aliases") {
                        return true;
                    }
                }
            }
        }
    }
    false
}
