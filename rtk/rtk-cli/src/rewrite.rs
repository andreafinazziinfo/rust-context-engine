/// rtk rewrite <cmd>
///
/// Exit code protocol (required by rtk-rewrite.sh hook):
///   0 + stdout  → rewrite found, auto-allow
///   1           → no RTK equivalent, pass through
///   2           → deny rule matched
///   3 + stdout  → ask rule matched (rewrite output but prompt user)
use anyhow::Result;
use regex::Regex;
use std::sync::LazyLock;

/// Execute the command rewriter check. Evaluates whether the given raw command should be
/// blocked, rewritten directly, prompt the user (ask), or passed through as-is.
pub fn run(raw: &str) -> Result<()> {
    let cmd = raw.trim();

    // Deny rules — dangerous or irreversible commands
    if is_denied(cmd) {
        std::process::exit(2);
    }

    // Security bypass: if command contains chaining/metacharacters, bypass rewriting
    if is_chained(cmd) {
        std::process::exit(1);
    }

    // Ask rules — commands that modify shared state
    if let Some(rewritten) = ask_rewrite(cmd) {
        print!("{rewritten}");
        std::process::exit(3);
    }

    // Auto-allow rewrites
    if let Some(rewritten) = auto_rewrite(cmd) {
        print!("{rewritten}");
        std::process::exit(0);
    }

    // No match
    std::process::exit(1);
}

fn is_denied(cmd: &str) -> bool {
    let config = rtk_db::config::get_config();
    is_denied_internal(cmd, &config.denied_commands)
}

fn split_segments(cmd: &str) -> Vec<String> {
    static SEGMENT_SPLIT: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"\s*&&\s*|\s*;\s*|\s*\|\|\s*|\s*\|\s*").unwrap());
    SEGMENT_SPLIT
        .split(cmd)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect()
}

fn segment_denied(segment: &str, custom_denied: &[String]) -> bool {
    static DENY: LazyLock<Vec<Regex>> = LazyLock::new(|| {
        vec![
            Regex::new(r"^rm\s+-rf?\s+/").unwrap(),
            Regex::new(r"^git\s+push\s+.*--force").unwrap(),
            Regex::new(r"^git\s+reset\s+--hard").unwrap(),
        ]
    });
    if DENY.iter().any(|re| re.is_match(segment)) {
        return true;
    }

    for pattern in custom_denied {
        if let Ok(re) = Regex::new(pattern) {
            if re.is_match(segment) {
                return true;
            }
        } else if segment.contains(pattern) {
            return true;
        }
    }

    false
}

fn is_denied_internal(cmd: &str, custom_denied: &[String]) -> bool {
    let segments = split_segments(cmd);
    if segments.is_empty() {
        return segment_denied(cmd, custom_denied);
    }
    segments
        .iter()
        .any(|seg| segment_denied(seg, custom_denied))
}

fn is_chained(cmd: &str) -> bool {
    cmd.contains("&&")
        || cmd.contains(';')
        || cmd.contains("||")
        || cmd.contains('|')
        || cmd.contains('`')
        || cmd.contains("$(")
}

fn ask_rewrite(cmd: &str) -> Option<String> {
    if is_chained(cmd) {
        return None;
    }
    let words: Vec<&str> = cmd.split_whitespace().collect();
    if words.len() >= 2 && words[0] == "git" && (words[1] == "push" || words[1] == "commit") {
        return Some(format!("rtk git {}", words[1]));
    }
    None
}

#[allow(clippy::type_complexity)]
fn auto_rewrite(cmd: &str) -> Option<String> {
    if is_chained(cmd) {
        return None;
    }
    let words: Vec<&str> = cmd.split_whitespace().collect();
    if words.is_empty() {
        return None;
    }

    // Dynamic plugins match
    let plugins_cfg = crate::plugins::load_plugins();
    for plugin in &plugins_cfg.plugins {
        if words[0] == plugin.bin {
            let matches_args = if let Some(ref p_args) = plugin.args {
                words.len() > p_args.len() && words[1..=p_args.len()] == p_args[..]
            } else {
                true
            };
            if matches_args {
                return Some(cmd.replacen(&plugin.bin, &format!("rtk plugin {}", plugin.name), 1));
            }
        }
    }

    match words[0] {
        "git" if words.len() >= 2 => match words[1] {
            "status" => Some("rtk git status".to_string()),
            "diff" | "log" | "branch" | "stash" | "show" => Some(cmd.replacen("git", "rtk git", 1)),
            _ => None,
        },
        "cargo" if words.len() >= 2 => match words[1] {
            "test" | "build" | "check" | "clippy" => Some(cmd.replacen("cargo", "rtk cargo", 1)),
            _ => None,
        },
        "go" if words.len() >= 2 => match words[1] {
            "test" => Some(cmd.replacen("go test", "rtk go_test", 1)),
            _ => None,
        },
        "npm" if words.len() >= 2 => match words[1] {
            "install" => Some(cmd.replacen("npm", "rtk npm", 1)),
            _ => None,
        },
        "docker" if words.len() >= 2 => match words[1] {
            "build" | "run" => Some(cmd.replacen("docker", "rtk docker", 1)),
            _ => None,
        },
        "dotnet" if words.len() >= 2 => match words[1] {
            "build" | "run" | "test" => Some(cmd.replacen("dotnet", "rtk dotnet", 1)),
            _ => None,
        },
        "yarn" | "pnpm" if words.len() >= 2 => match words[1] {
            "install" => Some(cmd.replacen(words[0], &format!("rtk {}", words[0]), 1)),
            _ => None,
        },
        "composer" if words.len() >= 2 => match words[1] {
            "install" | "update" => Some(cmd.replacen("composer", "rtk composer", 1)),
            _ => None,
        },
        "terraform" if words.len() >= 2 => match words[1] {
            "plan" | "apply" => Some(cmd.replacen("terraform", "rtk terraform", 1)),
            _ => None,
        },
        "pytest" => Some(cmd.replacen("pytest", "rtk pytest", 1)),
        "ls" => Some(cmd.replacen("ls", "rtk ls", 1)),
        "gradle" | "./gradlew" | "gradlew" => Some(cmd.replacen(words[0], "rtk gradle", 1)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_status_rewrite() {
        assert_eq!(auto_rewrite("git status"), Some("rtk git status".into()));
    }

    #[test]
    fn test_git_diff_passthrough_args() {
        let result = auto_rewrite("git diff HEAD~1 HEAD --stat");
        assert_eq!(result, Some("rtk git diff HEAD~1 HEAD --stat".into()));
    }

    #[test]
    fn test_cargo_test_rewrite() {
        assert_eq!(auto_rewrite("cargo test"), Some("rtk cargo test".into()));
    }

    #[test]
    fn test_gradle_rewrite() {
        assert_eq!(
            auto_rewrite("./gradlew build"),
            Some("rtk gradle build".into())
        );
        assert_eq!(auto_rewrite("gradle test"), Some("rtk gradle test".into()));
    }

    #[test]
    fn test_go_test_rewrite() {
        assert_eq!(
            auto_rewrite("go test ./..."),
            Some("rtk go_test ./...".into())
        );
    }

    #[test]
    fn test_docker_rewrite() {
        assert_eq!(
            auto_rewrite("docker build -t app ."),
            Some("rtk docker build -t app .".into())
        );
        assert_eq!(
            auto_rewrite("docker run -it app"),
            Some("rtk docker run -it app".into())
        );
    }

    #[test]
    fn test_no_match_returns_none() {
        assert_eq!(auto_rewrite("python manage.py runserver"), None);
    }

    #[test]
    fn test_deny_force_push() {
        assert!(is_denied("git push origin main --force"));
    }

    #[test]
    fn test_git_push_is_ask_not_auto() {
        assert!(auto_rewrite("git push").is_none());
        assert!(ask_rewrite("git push").is_some());
    }

    #[test]
    fn test_chained_commands_bypassed() {
        assert_eq!(auto_rewrite("git status && echo 1"), None);
        assert_eq!(auto_rewrite("git diff; ls"), None);
        assert_eq!(auto_rewrite("ls | grep foo"), None);
        assert_eq!(auto_rewrite("pytest || exit 1"), None);
        assert_eq!(ask_rewrite("git push && echo ok"), None);
    }

    #[test]
    fn test_chained_deny_rules() {
        assert!(is_denied("git status && git push origin main --force"));
        assert!(is_denied("echo ok; rm -rf /"));
        assert!(is_denied("true || git reset --hard"));
    }

    #[test]
    fn test_custom_denied_commands() {
        let custom_denied = vec![
            "git push.*--force-with-lease".to_string(),
            "secret-utility".to_string(),
        ];
        assert!(is_denied_internal(
            "git push origin main --force-with-lease",
            &custom_denied
        ));
        assert!(is_denied_internal("secret-utility run", &custom_denied));
        assert!(!is_denied_internal("git push origin main", &custom_denied));
    }

    #[test]
    fn test_dynamic_plugin_rewrite() {
        let toml_content = r#"
[[plugins]]
name = "ruff-check"
bin = "ruff"
args = ["check"]
filter_mode = "stdout"
"#;
        std::fs::write("plugins.toml", toml_content).unwrap();

        let result = auto_rewrite("ruff check src/main.rs");
        assert_eq!(
            result,
            Some("rtk plugin ruff-check check src/main.rs".to_string())
        );

        std::fs::remove_file("plugins.toml").ok();
    }
}
