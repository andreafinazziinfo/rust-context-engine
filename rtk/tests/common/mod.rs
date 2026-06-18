/// Shared test utilities — imported by integration tests via `mod common`.

/// Counts whitespace-delimited tokens (proxy for LLM token count).
pub fn count_tokens(text: &str) -> usize {
    text.split_whitespace().count()
}

/// Returns (savings_pct, passes_threshold).
pub fn token_savings(original: &str, filtered: &str) -> (f64, bool) {
    let orig = count_tokens(original);
    let filt = count_tokens(filtered);
    if orig == 0 {
        return (0.0, false);
    }
    let savings = 1.0 - (filt as f64 / orig as f64);
    (savings * 100.0, savings >= 0.60)
}
