use lazy_static::lazy_static;
use regex::Regex;

pub fn filter(input: &str) -> String {
    let mut out = String::with_capacity(input.len());

    lazy_static! {
        // Drop test run start line (=== RUN)
        static ref RUN_LINE: Regex = Regex::new(r"^===\s*RUN\b").unwrap();
        // Drop test pass line (--- PASS)
        static ref PASS_LINE: Regex = Regex::new(r"^---\s*PASS\b").unwrap();
        // Drop ok status lines for packages with no output
        static ref PACKAGE_OK: Regex = Regex::new(r"^ok\s+[^\s]+\s+[0-9\.]+s\s*(?:\[no tests to run\])?$").unwrap();
    }

    for line in input.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if RUN_LINE.is_match(trimmed) || PASS_LINE.is_match(trimmed) || PACKAGE_OK.is_match(trimmed)
        {
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
    fn test_go_test_clean() {
        let raw = "\
=== RUN   TestAdd
--- PASS: TestAdd (0.00s)
=== RUN   TestSubtract
--- PASS: TestSubtract (0.00s)
PASS
ok      example.com/math   0.005s
";
        let filtered = filter(raw);
        assert!(filtered.contains("PASS"));
        assert!(!filtered.contains("TestAdd"));
        assert!(!filtered.contains("TestSubtract"));
    }

    #[test]
    fn test_go_test_with_failures() {
        let raw = "\
=== RUN   TestAdd
--- PASS: TestAdd (0.00s)
=== RUN   TestSubtract
    math_test.go:12: Subtract(5, 3) = 1; want 2
--- FAIL: TestSubtract (0.00s)
FAIL
FAIL    example.com/math   0.006s
FAIL
";
        let filtered = filter(raw);
        assert!(filtered.contains("FAIL: TestSubtract"));
        assert!(filtered.contains("Subtract(5, 3) = 1; want 2"));
    }
}
