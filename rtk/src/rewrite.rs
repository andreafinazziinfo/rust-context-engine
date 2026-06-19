/// rtk rewrite <cmd>
///
/// Exit code protocol (required by rtk-rewrite.sh hook):
///   0 + stdout  → rewrite found, auto-allow
///   1           → no RTK equivalent, pass through
///   2           → deny rule matched
///   3 + stdout  → ask rule matched (rewrite output but prompt user)
use anyhow::Result;
use lazy_static::lazy_static;
use regex::Regex;

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
    lazy_static! {
        static ref DENY: Vec<Regex> = vec![
            Regex::new(r"^rm\s+-rf?\s+/").unwrap(),
            Regex::new(r"^git\s+push\s+.*--force").unwrap(),
            Regex::new(r"^git\s+reset\s+--hard").unwrap(),
        ];
    }
    DENY.iter().any(|re| re.is_match(cmd))
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
    lazy_static! {
        static ref ASK: Vec<(Regex, &'static str)> = vec![
            (Regex::new(r"^git\s+push(\s|$)").unwrap(), "rtk git push"),
            (
                Regex::new(r"^git\s+commit(\s|$)").unwrap(),
                "rtk git commit"
            ),
        ];
    }
    for (re, replacement) in ASK.iter() {
        if re.is_match(cmd) {
            return Some(replacement.to_string());
        }
    }
    None
}

#[allow(clippy::type_complexity)]
fn auto_rewrite(cmd: &str) -> Option<String> {
    if is_chained(cmd) {
        return None;
    }
    lazy_static! {
        static ref AUTO: Vec<(Regex, Box<dyn Fn(&str) -> String + Send + Sync>)> = vec![
            (
                Regex::new(r"^git\s+status(\s|$)").unwrap(),
                Box::new(|_| "rtk git status".into())
            ),
            (
                Regex::new(r"^git\s+diff(\s|$)").unwrap(),
                Box::new(|c| c.replacen("git", "rtk git", 1))
            ),
            (
                Regex::new(r"^git\s+log(\s|$)").unwrap(),
                Box::new(|c| c.replacen("git", "rtk git", 1))
            ),
            (
                Regex::new(r"^git\s+branch(\s|$)").unwrap(),
                Box::new(|c| c.replacen("git", "rtk git", 1))
            ),
            (
                Regex::new(r"^git\s+stash(\s|$)").unwrap(),
                Box::new(|c| c.replacen("git", "rtk git", 1))
            ),
            (
                Regex::new(r"^git\s+show(\s|$)").unwrap(),
                Box::new(|c| c.replacen("git", "rtk git", 1))
            ),
            (
                Regex::new(r"^cargo\s+test(\s|$)").unwrap(),
                Box::new(|c| c.replacen("cargo", "rtk cargo", 1))
            ),
            (
                Regex::new(r"^cargo\s+build(\s|$)").unwrap(),
                Box::new(|c| c.replacen("cargo", "rtk cargo", 1))
            ),
            (
                Regex::new(r"^cargo\s+check(\s|$)").unwrap(),
                Box::new(|c| c.replacen("cargo", "rtk cargo", 1))
            ),
            (
                Regex::new(r"^cargo\s+clippy(\s|$)").unwrap(),
                Box::new(|c| c.replacen("cargo", "rtk cargo", 1))
            ),
            (
                Regex::new(r"^npm\s+install(\s|$)").unwrap(),
                Box::new(|c| c.replacen("npm", "rtk npm", 1))
            ),
            (
                Regex::new(r"^pytest(\s|$)").unwrap(),
                Box::new(|c| format!("rtk pytest{}", &c[6..]))
            ),
            (
                Regex::new(r"^ls(\s|$)").unwrap(),
                Box::new(|c| format!("rtk ls{}", &c[2..]))
            ),
            (
                Regex::new(r"^(?:\./)?gradlew?(\s|$)").unwrap(),
                Box::new(|c| {
                    if c.starts_with("./gradlew") {
                        c.replacen("./gradlew", "rtk gradle", 1)
                    } else if c.starts_with("gradlew") {
                        c.replacen("gradlew", "rtk gradle", 1)
                    } else {
                        c.replacen("gradle", "rtk gradle", 1)
                    }
                })
            ),
            (
                Regex::new(r"^go\s+test(\s|$)").unwrap(),
                Box::new(|c| c.replacen("go test", "rtk go_test", 1))
            ),
            (
                Regex::new(r"^docker\s+(build|run)(\s|$)").unwrap(),
                Box::new(|c| c.replacen("docker", "rtk docker", 1))
            ),
        ];
    }
    for (re, rewriter) in AUTO.iter() {
        if re.is_match(cmd) {
            return Some(rewriter(cmd));
        }
    }
    None
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
}
