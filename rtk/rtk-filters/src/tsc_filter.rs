use regex::Regex;
use std::sync::LazyLock;

/// Filter TypeScript compiler (`tsc`) output.
///
/// `tsc --pretty` (common in dev/CI) emits ANSI colors, a `file:line:col -
/// error TSxxxx: message` header, an indented source code-frame with a caret
/// underline, related-information blocks, and blank separators. The colors and
/// code-frames are noise for an agent; the diagnostic text and related-info
/// locations are the signal.
///
/// Strategy:
///   - Strip ANSI escape codes.
///   - Drop the source/caret code-frames (caret-lookback).
///   - Drop blank lines; keep error headers, message continuations, and
///     related-information locations.
///   - Fallback: return input unchanged if the filter produces empty output.
pub fn filter(input: &str) -> String {
    static ANSI: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\x1b\[[0-9;]*m").unwrap());
    // A caret underline line of a code-frame, e.g. `        ~~~~~`.
    static CARET: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\s*~+\s*$").unwrap());

    let lines: Vec<String> = input
        .lines()
        .map(|l| ANSI.replace_all(l, "").into_owned())
        .collect();

    // Frames are (source, caret) pairs: mark caret lines and the source line
    // immediately above them for dropping.
    let mut drop = vec![false; lines.len()];
    for (i, line) in lines.iter().enumerate() {
        if CARET.is_match(line) {
            drop[i] = true;
            if i > 0 {
                drop[i - 1] = true;
            }
        }
    }

    let mut out = String::with_capacity(input.len() / 2);
    for (i, line) in lines.iter().enumerate() {
        if drop[i] || line.trim().is_empty() {
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
    fn test_strips_ansi_and_frames() {
        let input = concat!(
            "\x1b[96ma.ts\x1b[0m:\x1b[93m4\x1b[0m:\x1b[93m7\x1b[0m - \x1b[91merror\x1b[0m\x1b[90m TS2322: \x1b[0mType 'string' is not assignable to type 'number'.\n",
            "\n",
            "\x1b[7m4\x1b[0m const n: number = greet(42)\n",
            "\x1b[7m \x1b[0m \x1b[91m      ~\x1b[0m\n",
            "\n",
        );
        let out = filter(input);
        assert!(
            out.contains(
                "a.ts:4:7 - error TS2322: Type 'string' is not assignable to type 'number'."
            ),
            "diagnostic header wrong: {out:?}"
        );
        assert!(!out.contains('\x1b'), "ANSI leaked");
        assert!(!out.contains("const n: number"), "code-frame source leaked");
        assert!(!out.contains('~'), "caret leaked");
    }

    #[test]
    fn test_keeps_related_info() {
        let input = concat!(
            "a.ts:7:7 - error TS2741: Property 'name' is missing.\n",
            "\n",
            "7 const u: User = { id: 1 }\n",
            "        ~\n",
            "\n",
            "  a.ts:6:30\n",
            "    6 interface User { id: number; name: string }\n",
            "                                   ~~~~\n",
            "    'name' is declared here.\n",
        );
        let out = filter(input);
        assert!(out.contains("a.ts:7:7 - error TS2741: Property 'name' is missing."));
        assert!(out.contains("a.ts:6:30"), "related location dropped");
        assert!(
            out.contains("'name' is declared here."),
            "related note dropped"
        );
        assert!(!out.contains("interface User { id"), "related frame leaked");
    }

    #[test]
    fn test_empty_input() {
        assert_eq!(filter(""), "");
    }

    fn count_tokens(s: &str) -> usize {
        s.split_whitespace().map(str::len).sum::<usize>().max(1)
    }

    #[test]
    fn token_savings_pretty_run() {
        let input = include_str!("../tests/fixtures/tsc_pretty.txt");
        let out = filter(input);
        let savings = 1.0 - count_tokens(&out) as f64 / count_tokens(input) as f64;
        assert!(
            savings >= 0.30,
            "tsc filter: expected ≥30% savings, got {:.1}%",
            savings * 100.0
        );
        assert!(out.contains("error TS2322"));
        assert!(!out.contains('\x1b'), "ANSI leaked");
    }
}
