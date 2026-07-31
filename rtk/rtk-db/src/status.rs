use anyhow::Result;
use std::fs;
use std::path::Path;

/// Query and print the current RTK environment status, active savings profile, and shell hook installation state.
pub fn run_status() -> Result<()> {
    println!("==========================================================");
    println!("🔍 RTK Token Saver: ACTIVE");
    println!("==========================================================");

    // Check for CLI hooks
    let hook_installed = check_hook_installed();
    if hook_installed {
        println!("🛡️  Input Protection: ON (CLI Hooks installed)");
    } else {
        println!("⚠️  Input Protection: OFF (CLI Hooks not detected in settings.json)");
    }

    // Check for profile
    let profile_path = Path::new(".cursor/rules/rtk-profile.mdc");
    let (profile_name, profile_desc) = if profile_path.exists() {
        match fs::read_to_string(profile_path) {
            Ok(content) => match active_profile_level(&content) {
                Some("MAX") => ("MAX", "Ponytail + Caveman Ultra"),
                Some("HIGH") => ("HIGH", "Ponytail + Caveman Full"),
                Some("MEDIUM") => ("MEDIUM", "Ponytail + Caveman Lite"),
                _ => ("LOW", "Ponytail Only"),
            },
            Err(_) => ("UNKNOWN", "Could not read profile"),
        }
    } else {
        ("NONE", "No profile installed")
    };

    println!("🤖 Output Profile:   {} ({})", profile_name, profile_desc);

    if let Some(skill) = active_profile_skill(profile_name) {
        print!("🗣️  Caveman Skill:    ");
        let claude_skill = Path::new(".claude/skills").join(skill).join("SKILL.md");
        if claude_skill.exists() {
            println!("✅ \"{skill}\" found in .claude/skills/");
        } else {
            println!(
                "⚠️  \"{skill}\" referenced by profile but missing at {}",
                claude_skill.display()
            );
            println!(
                "   👉 Run `rtk init --profile <level> --force-profile` to reinstall skill files."
            );
        }
    }

    let cfg = crate::config::get_config();
    let local_exists = Path::new(".rtk.json").exists();
    let local_indicator = if local_exists {
        " (loaded from local .rtk.json)"
    } else {
        " (loaded from global config)"
    };
    println!(
        "⚙️  Savings Profile:   {}{}",
        cfg.default_profile, local_indicator
    );

    println!("==========================================================");

    if profile_name == "NONE" {
        println!("To configure output rules, run: rtk init --profile <low|medium|high|max>");
    } else {
        println!("To change output rules, run: rtk init --profile <low|medium|high|max>");
    }

    Ok(())
}

/// Whether Claude/Gemini settings reference the RTK rewrite hook.
pub fn is_rewrite_hook_installed() -> bool {
    check_hook_installed()
}

/// Parse the profile level from a generated `rtk-profile.mdc`/`AGENTS.md` block by
/// its `# RTK Output Profile: <LEVEL>` heading — the single source of truth, instead
/// of sniffing for level-specific substrings that drift when the prose wording changes.
pub fn active_profile_level(content: &str) -> Option<&'static str> {
    for level in ["MAX", "HIGH", "MEDIUM", "LOW"] {
        if content.contains(&format!("# RTK Output Profile: {level}")) {
            return Some(level);
        }
    }
    None
}

/// The skill name a given profile level auto-triggers (all levels use the single
/// level-parameterized "caveman" skill), or `None` for LOW/unset (no caveman skill).
pub fn active_profile_skill(profile_name: &str) -> Option<&'static str> {
    match profile_name {
        "MAX" | "HIGH" | "MEDIUM" => Some("caveman"),
        _ => None,
    }
}

fn check_hook_installed() -> bool {
    let home = std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(std::path::PathBuf::from);

    if let Some(h) = home {
        let dirs = vec![h.join(".gemini").join("antigravity"), h.join(".claude")];
        for dir in dirs {
            if dir.exists() {
                let path = dir.join("settings.json");
                if path.exists() {
                    if let Ok(content) = fs::read_to_string(&path) {
                        if content.contains("rtk-rewrite.sh") {
                            return true;
                        }
                    }
                }
            }
        }
    }
    false
}
