use anyhow::Result;
use std::fs;
use std::path::Path;

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
            Ok(content) => {
                if content.contains("caveman-ultra") {
                    ("MAX", "Ponytail + Caveman Ultra")
                } else if content.contains("caveman-full") {
                    ("HIGH", "Ponytail + Caveman Full")
                } else if content.contains("caveman-lite") {
                    ("MEDIUM", "Ponytail + Caveman Lite")
                } else {
                    ("LOW", "Ponytail Only")
                }
            }
            Err(_) => ("UNKNOWN", "Could not read profile"),
        }
    } else {
        ("NONE", "No profile installed")
    };

    println!("🤖 Output Profile:   {} ({})", profile_name, profile_desc);
    println!("==========================================================");
    
    if profile_name == "NONE" {
        println!("To configure output rules, run: rtk init --profile <low|medium|high|max>");
    } else {
        println!("To change output rules, run: rtk init --profile <low|medium|high|max>");
    }

    Ok(())
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
