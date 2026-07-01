use regex::Regex;
use std::sync::LazyLock;

/// Filter `pip install` output.
///
/// A pip install is mostly noise for an agent: `Collecting`, `Downloading`,
/// `Using cached`, progress bars, per-dependency `Requirement already
/// satisfied`, and the `Installing collected packages` list. The signal is the
/// final `Successfully installed ...` line plus any errors.
///
/// Strategy:
///   - Keep `Successfully installed/built`, errors, warnings, deprecations.
///   - Collapse all `Requirement already satisfied` lines into one count.
///   - Drop Collecting/Downloading/Using cached/progress/Installing noise and
///     pip's self-upgrade notice.
///   - Fallback: return input unchanged if the filter produces empty output.
pub fn filter(input: &str) -> String {
    static KEEP: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(
            r"^(Successfully |ERROR|DEPRECATION|WARNING|Could not|No matching distribution|error:)",
        )
        .unwrap()
    });
    static SATISFIED: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^Requirement already satisfied:").unwrap());
    // pip's own upgrade chatter — always noise.
    static NOTICE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"A new release of pip|You are using pip version|^\s*\[notice\]").unwrap()
    });

    let mut out: Vec<String> = Vec::new();
    let mut satisfied = 0usize;
    let mut satisfied_pos: Option<usize> = None;

    for line in input.lines() {
        if SATISFIED.is_match(line) {
            if satisfied_pos.is_none() {
                satisfied_pos = Some(out.len());
            }
            satisfied += 1;
            continue;
        }
        if NOTICE.is_match(line) {
            continue;
        }
        if KEEP.is_match(line) {
            out.push(line.to_string());
        }
        // Everything else (Collecting/Downloading/Using cached/progress/
        // Installing collected packages/indented detail) is dropped.
    }

    if satisfied > 0 {
        let unit = if satisfied == 1 {
            "requirement"
        } else {
            "requirements"
        };
        let note = format!("[{satisfied} {unit} already satisfied]");
        out.insert(satisfied_pos.unwrap_or(0), note);
    }

    if out.is_empty() {
        return input.to_string(); // fallback / empty passthrough
    }
    format!("{}\n", out.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collapses_noise_keeps_result() {
        let input = include_str!("../tests/fixtures/pip_install_verbose.txt");
        let out = filter(input);
        assert!(out.contains(
            "Successfully installed anyio-4.14.1 h11-0.16.0 httpcore-1.0.9 httpx-0.28.1 typing_extensions-4.15.0"
        ));
        assert!(out.contains("[2 requirements already satisfied]"));
        assert!(!out.contains("Collecting"), "collecting leaked");
        assert!(!out.contains("Using cached"), "using-cached leaked");
        assert!(
            !out.contains("Installing collected packages"),
            "install list leaked"
        );
        assert!(!out.contains("━"), "progress bar leaked");
    }

    #[test]
    fn test_singular_requirement() {
        let input = concat!(
            "Requirement already satisfied: requests in /venv (2.34.2)\n",
            "Successfully installed flask-3.1.3\n",
        );
        let out = filter(input);
        assert!(out.contains("[1 requirement already satisfied]"));
        assert!(out.contains("Successfully installed flask-3.1.3"));
    }

    #[test]
    fn test_errors_kept() {
        let input = concat!(
            "Collecting nonexistent-pkg\n",
            "ERROR: Could not find a version that satisfies the requirement nonexistent-pkg\n",
            "ERROR: No matching distribution found for nonexistent-pkg\n",
        );
        let out = filter(input);
        assert!(out.contains("ERROR: Could not find a version"));
        assert!(out.contains("ERROR: No matching distribution found"));
        assert!(!out.contains("Collecting"));
    }

    #[test]
    fn test_pip_notice_dropped() {
        let input = concat!(
            "Successfully installed flask-3.1.3\n",
            "[notice] A new release of pip is available: 24.0 -> 25.1\n",
            "[notice] To update, run: pip install --upgrade pip\n",
        );
        let out = filter(input);
        assert_eq!(out.trim(), "Successfully installed flask-3.1.3");
    }

    #[test]
    fn test_empty_input() {
        assert_eq!(filter(""), "");
    }

    fn count_tokens(s: &str) -> usize {
        s.split_whitespace().map(str::len).sum::<usize>().max(1)
    }

    #[test]
    fn token_savings_verbose_run() {
        let input = include_str!("../tests/fixtures/pip_install_verbose.txt");
        let out = filter(input);
        let savings = 1.0 - count_tokens(&out) as f64 / count_tokens(input) as f64;
        assert!(
            savings >= 0.60,
            "pip filter: expected ≥60% savings, got {:.1}%",
            savings * 100.0
        );
    }
}
