use regex::Regex;
use std::sync::LazyLock;

/// Filter `vitest run` output.
///
/// Vitest's default reporter frames each failure with a numbered source
/// code-frame and a caret, draws long `⎯⎯⎯` separator rules, and prints a run
/// banner and a timing breakdown. The signal for an agent is: which tests
/// failed, the assertion diff, the failure location, and the final tally.
///
/// Strategy:
///   - Drop decorative `⎯` separator rules and the `RUN`/`Start at` banners.
///   - Drop source code-frames (`  4| ...`) and caret/gutter (`   | ^`) lines.
///   - Keep `FAIL`/`✗` lines, assertion diffs, `❯ file:line:col` locations,
///     and the `Test Files` / `Tests` / `Duration` summary.
///   - Fallback: return input unchanged if the filter produces empty output.
pub fn filter(input: &str) -> String {
    // Numbered source line of a code-frame, e.g. `      6|   it('x', ...)`.
    static SRC: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\s*\d+\|").unwrap());

    let mut out = String::with_capacity(input.len() / 2);

    for line in input.lines() {
        let trimmed = line.trim_start();

        // Separator rule: strip the decorative `⎯`. A pure rule (or one that
        // only carries a `[n/m]` progress marker) is dropped; a titled rule like
        // `⎯⎯ Failed Tests 2 ⎯⎯` keeps just its title text.
        if line.contains('⎯') {
            let title = line.replace('⎯', "");
            let title = title.trim();
            let marker: String = title
                .chars()
                .filter(|c| !"[]/".contains(*c) && !c.is_whitespace())
                .collect();
            if title.is_empty()
                || (!marker.is_empty() && marker.chars().all(|c| c.is_ascii_digit()))
            {
                continue; // pure rule or `[1/2]` progress marker
            }
            out.push_str(title);
            out.push('\n');
            continue;
        }

        // Run banner and start-time line — pure chrome.
        if trimmed.starts_with("RUN ") || trimmed.starts_with("Start at") {
            continue;
        }

        // Code-frame: numbered source line or a caret/gutter line.
        if SRC.is_match(line) || trimmed.starts_with('|') {
            continue;
        }

        if line.trim().is_empty() {
            continue;
        }

        out.push_str(line);
        out.push('\n');
    }

    let trimmed = out.trim_end();
    if trimmed.is_empty() {
        return input.to_string(); // fallback / empty passthrough
    }
    format!("{trimmed}\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drops_frames_and_rules_keeps_failures() {
        let input = concat!(
            "⎯⎯⎯⎯⎯⎯⎯ Failed Tests 2 ⎯⎯⎯⎯⎯⎯⎯\n",
            "\n",
            " FAIL  sum.test.js > add > fails on wrong\n",
            "AssertionError: expected 4 to be 5 // Object.is equality\n",
            "\n",
            "- Expected\n",
            "+ Received\n",
            "\n",
            "- 5\n",
            "+ 4\n",
            "\n",
            " ❯ sum.test.js:6:50\n",
            "      6|   it('fails on wrong', () => { expect(add(2, 2)).toBe(5) })\n",
            "       |                                                  ^\n",
            "\n",
            "⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯[1/2]⎯\n",
        );
        let out = filter(input);
        assert!(out.contains("Failed Tests 2"), "titled rule dropped");
        assert!(out.contains(" FAIL  sum.test.js > add > fails on wrong"));
        assert!(out.contains("AssertionError: expected 4 to be 5"));
        assert!(out.contains("- 5") && out.contains("+ 4"), "diff dropped");
        assert!(out.contains("❯ sum.test.js:6:50"), "location dropped");
        assert!(
            !out.contains("it('fails on wrong'"),
            "code-frame source leaked"
        );
        assert!(!out.contains('^'), "caret leaked");
        assert!(!out.contains("[1/2]"), "progress rule leaked");
    }

    #[test]
    fn test_keeps_summary() {
        let input = concat!(
            " Test Files  1 failed (1)\n",
            "      Tests  2 failed | 3 passed (5)\n",
            "   Start at  19:58:47\n",
            "   Duration  1.03s (transform 66ms, setup 0ms)\n",
        );
        let out = filter(input);
        assert!(out.contains("Test Files  1 failed (1)"));
        assert!(out.contains("Tests  2 failed | 3 passed (5)"));
        assert!(out.contains("Duration  1.03s"));
        assert!(!out.contains("Start at"), "start-at chrome leaked");
    }

    #[test]
    fn test_empty_input() {
        assert_eq!(filter(""), "");
    }

    fn count_tokens(s: &str) -> usize {
        s.split_whitespace().map(str::len).sum::<usize>().max(1)
    }

    #[test]
    fn token_savings_run() {
        let input = include_str!("../tests/fixtures/vitest_run.txt");
        let out = filter(input);
        let savings = 1.0 - count_tokens(&out) as f64 / count_tokens(input) as f64;
        assert!(
            savings >= 0.30,
            "vitest filter: expected ≥30% savings, got {:.1}%",
            savings * 100.0
        );
        assert!(out.contains("Tests  2 failed | 3 passed (5)"));
        assert!(!out.contains('⎯'), "separator rule leaked");
    }
}
