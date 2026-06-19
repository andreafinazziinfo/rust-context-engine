use lazy_static::lazy_static;
use regex::Regex;

pub fn filter(input: &str) -> String {
    let mut out = String::with_capacity(input.len());

    lazy_static! {
        // Drop task execution lines like:
        // > Task :compileJava
        // > Task :processResources UP-TO-DATE
        static ref TASK_LINE: Regex = Regex::new(r"^>\s*Task\s+:").unwrap();
        // Drop progress/download status lines
        static ref PROGRESS_LINE: Regex = Regex::new(r"^(Download|Caching|Download progress|Preparing)\b").unwrap();
    }

    for line in input.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if TASK_LINE.is_match(trimmed) || PROGRESS_LINE.is_match(trimmed) {
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }

    if out.trim().is_empty() {
        input.to_string()
    } else {
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gradle_clean() {
        let raw = "\
> Task :compileJava
> Task :processResources UP-TO-DATE
> Task :classes
> Task :jar

BUILD SUCCESSFUL in 1s
2 actionable tasks: 1 executed, 1 up-to-date
";
        let filtered = filter(raw);
        assert!(filtered.contains("BUILD SUCCESSFUL"));
        assert!(!filtered.contains(":compileJava"));
    }

    #[test]
    fn test_gradle_with_errors() {
        let raw = "\
> Task :compileJava
/src/main/java/App.java:5: error: ';' expected
        System.out.println(\"Hello\")
                                   ^
1 error
> Task :compileJava FAILED

BUILD FAILED in 2s
";
        let filtered = filter(raw);
        assert!(filtered.contains("error: ';' expected"));
        assert!(filtered.contains("BUILD FAILED"));
    }
}
