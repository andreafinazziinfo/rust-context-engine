use regex::Regex;
use std::sync::LazyLock;

/// Filter `eslint` (stylish, the default formatter) output.
///
/// Stylish prints a file-path header, then one column-aligned line per problem
/// (`  line:col  severity  message  rule`), blank separators between files, and
/// a `... potentially fixable with the --fix option` notice. The alignment
/// padding and notice are pure noise for an agent.
///
/// Strategy:
///   - Collapse each problem to `line:col severity message (rule)`.
///   - Keep the per-file path header (once) and the `✖ N problems` summary.
///   - Drop alignment padding, blank lines, and the `--fix` notice.
///   - Fallback: return input unchanged if the filter produces empty output.
pub fn filter(input: &str) -> String {
    // `  1:7   error    'unused' ... never used  no-unused-vars`
    static PROBLEM: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"^\s+(\d+:\d+)\s+(error|warning)\s+(.+?)\s{2,}(\S+)\s*$").unwrap()
    });

    let mut out = String::with_capacity(input.len() / 2);

    for line in input.lines() {
        if let Some(caps) = PROBLEM.captures(line) {
            out.push_str(&format!(
                "{} {} {} ({})\n",
                &caps[1], &caps[2], &caps[3], &caps[4]
            ));
            continue;
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        // Indented non-problem lines are the `--fix` notice — drop them.
        if line.starts_with(char::is_whitespace) {
            continue;
        }
        // Non-indented lines are file-path headers or the summary — keep them.
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
    fn test_collapses_alignment_keeps_header_and_summary() {
        let input = concat!(
            "\n",
            "src1.js\n",
            "  1:7   error    'unused' is assigned a value but never used  no-unused-vars\n",
            "  4:9   error    Expected '===' and instead saw '=='          eqeqeq\n",
            "  5:3   warning  Unexpected console statement                 no-console\n",
            "\n",
            "✖ 3 problems (2 errors, 1 warning)\n",
            "  1 error and 0 warnings potentially fixable with the `--fix` option.\n",
        );
        let out = filter(input);
        assert!(out.contains("src1.js"), "file header missing");
        assert!(
            out.contains("1:7 error 'unused' is assigned a value but never used (no-unused-vars)"),
            "problem not collapsed: {out}"
        );
        assert!(out.contains("5:3 warning Unexpected console statement (no-console)"));
        assert!(
            out.contains("✖ 3 problems (2 errors, 1 warning)"),
            "summary missing"
        );
        assert!(!out.contains("fixable"), "fix notice leaked");
        // Alignment padding gone: no run of 2+ spaces survives in a problem line.
        assert!(!out.contains("error    "), "alignment padding leaked");
    }

    #[test]
    fn test_scoped_rule_name() {
        let input =
            "app.ts\n  10:5  error  Unexpected any. Specify a different type  @typescript-eslint/no-explicit-any\n";
        let out = filter(input);
        assert!(out.contains(
            "10:5 error Unexpected any. Specify a different type (@typescript-eslint/no-explicit-any)"
        ));
    }

    #[test]
    fn test_empty_input() {
        assert_eq!(filter(""), "");
    }

    // eslint's saving is almost entirely whitespace (alignment padding), which a
    // word-length count ignores. Character length is the whitespace-aware local
    // proxy; the real BPE saving is ~27% (verified via scripts/benchmark.py with
    // tiktoken cl100k_base).
    fn count_chars(s: &str) -> usize {
        s.len().max(1)
    }

    #[test]
    fn token_savings_stylish_run() {
        let input = include_str!("../tests/fixtures/eslint_stylish.txt");
        let out = filter(input);
        let savings = 1.0 - count_chars(&out) as f64 / count_chars(input) as f64;
        assert!(
            savings >= 0.30,
            "eslint filter: expected ≥30% char savings, got {:.1}%",
            savings * 100.0
        );
        assert!(out.contains("src1.js") && out.contains("src2.js"));
        assert!(out.contains("✖ 18 problems"));
        assert!(!out.contains("fixable"));
    }
}
