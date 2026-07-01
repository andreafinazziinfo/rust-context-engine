use regex::Regex;
use std::sync::LazyLock;

/// Filter `ruff check` output.
///
/// Ruff's default ("full") format prints, for every violation, a rule header,
/// a `-->` location line, a multi-line source code-frame, and a `help:` hint.
/// For an agent the actionable signal is just `file:line:col CODE message`; the
/// code-frame is redundant with the file the agent can already read.
///
/// Strategy:
///   - Collapse each violation to a single `file:line:col CODE [*] message` line.
///   - Drop the source code-frame (gutter `|`, caret `^`) and `help:` hints.
///   - Pass through already-concise diagnostics (`file:line:col: CODE ...`).
///   - Keep the trailing summary (`Found N errors`, `[*] ... fixable`,
///     `All checks passed!`).
///   - Fallback: return input unchanged if the filter produces empty output.
pub fn filter(input: &str) -> String {
    // Rule header, e.g. `F401 [*] `os` imported but unused` or `E741 Ambiguous...`.
    static RULE_HEADER: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^([A-Z]{1,4}\d{1,4})\s+(\[\*\]\s+)?(.*)$").unwrap());
    // Location line, e.g. ` --> mod_a.py:1:8`.
    static LOCATION: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^\s*-->\s+(\S+:\d+:\d+)\s*$").unwrap());
    // Already-concise diagnostic, e.g. `mod_a.py:1:8: F401 ...`.
    static CONCISE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^\S+:\d+:\d+:\s+[A-Z]{1,4}\d{1,4}\b").unwrap());
    // Summary / status lines worth keeping.
    static SUMMARY: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"^(Found \d+ error|\[\*\] \d+ fixable|All checks passed|warning:|error:)")
            .unwrap()
    });

    let mut out = String::with_capacity(input.len() / 3);
    // Pending violation header waiting for its `-->` location line.
    let mut pending: Option<(String, bool, String)> = None; // (code, fixable, message)

    for line in input.lines() {
        // A location line completes the pending header.
        if let Some(caps) = LOCATION.captures(line) {
            if let Some((code, fixable, message)) = pending.take() {
                let star = if fixable { " [*]" } else { "" };
                out.push_str(&format!("{}: {}{} {}\n", &caps[1], code, star, message));
            }
            continue;
        }

        if let Some(caps) = RULE_HEADER.captures(line) {
            pending = Some((
                caps[1].to_string(),
                caps.get(2).is_some(),
                caps[3].trim().to_string(),
            ));
            continue;
        }

        // Concise-format diagnostics and summary lines pass through verbatim.
        if CONCISE.is_match(line) || SUMMARY.is_match(line.trim_start()) {
            pending = None;
            out.push_str(line);
            out.push('\n');
        }
        // Everything else (code-frame, help:, blank lines) is dropped.
    }

    let trimmed = out.trim_end();
    if trimmed.is_empty() {
        // Nothing matched: pass the raw input through (or empty in, empty out).
        return input.to_string();
    }
    format!("{trimmed}\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collapses_full_format() {
        let input = concat!(
            "F401 [*] `os` imported but unused\n",
            " --> mod_a.py:1:8\n",
            "  |\n",
            "1 | import os\n",
            "  |        ^^\n",
            "  |\n",
            "help: Remove unused import: `os`\n",
            "\n",
            "Found 1 error.\n",
        );
        let out = filter(input);
        assert!(
            out.contains("mod_a.py:1:8: F401 [*] `os` imported but unused"),
            "collapsed diagnostic missing: {out}"
        );
        assert!(!out.contains("import os"), "code-frame leaked");
        assert!(!out.contains("help:"), "help hint leaked");
        assert!(out.contains("Found 1 error."), "summary missing");
    }

    #[test]
    fn test_non_fixable_has_no_star() {
        let input = concat!(
            "E741 Ambiguous variable name: `l`\n",
            "  --> mod_a.py:14:9\n",
            "   |\n",
            "14 |         l = [1,2,3]\n",
            "   |         ^\n",
        );
        let out = filter(input);
        assert_eq!(
            out.trim(),
            "mod_a.py:14:9: E741 Ambiguous variable name: `l`"
        );
    }

    #[test]
    fn test_concise_format_passthrough() {
        let input = "mod_a.py:1:8: F401 `os` imported but unused\nFound 1 error.\n";
        let out = filter(input);
        assert!(out.contains("mod_a.py:1:8: F401 `os` imported but unused"));
        assert!(out.contains("Found 1 error."));
    }

    #[test]
    fn test_clean_run_passthrough() {
        let input = "All checks passed!\n";
        assert_eq!(filter(input).trim(), "All checks passed!");
    }

    #[test]
    fn test_empty_input_fallback() {
        assert_eq!(filter(""), "");
    }

    fn count_tokens(s: &str) -> usize {
        s.split_whitespace().map(str::len).sum::<usize>().max(1)
    }

    #[test]
    fn token_savings_full_run() {
        let input = include_str!("../tests/fixtures/ruff_check_full.txt");
        let out = filter(input);
        let savings = 1.0 - count_tokens(&out) as f64 / count_tokens(input) as f64;
        // This fixture is `--select ALL` on real code, dominated by 1-line
        // docstring (D-code) frames — the worst case for a code-frame filter.
        // Typical `ruff check` runs (E/F/W with multi-line frames) save more.
        assert!(
            savings >= 0.55,
            "ruff filter: expected ≥55% savings, got {:.1}%",
            savings * 100.0
        );
        // Diagnostics survive the collapse; code-frames do not.
        assert!(out.contains("backend/app/api/health.py:7:11: D103"));
        assert!(out.contains("S105 Possible hardcoded password"));
        assert!(out.contains("COM812 [*] Trailing comma missing"));
        assert!(out.contains("Found 118 errors."));
        assert!(!out.contains("async def health"), "code-frame leaked");
        assert!(!out.contains("help:"), "help hint leaked");
    }
}
