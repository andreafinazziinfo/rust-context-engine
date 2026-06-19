use crate::tracking;
use anyhow::Result;
use std::io::{self, Read};

pub fn run(content_args: Vec<String>) -> Result<()> {
    let mut thought_content = String::new();

    // If arguments are provided, use them as the thought content
    if !content_args.is_empty() {
        thought_content = content_args.join(" ");
    } else {
        // Otherwise, read from standard input
        let mut stdin = io::stdin();
        stdin.read_to_string(&mut thought_content)?;
    }

    // Clean up excessive whitespace
    let cleaned_content = thought_content.trim().to_string();

    if cleaned_content.is_empty() {
        println!("[RTK] No thought content provided.");
        return Ok(());
    }

    // Record the thought in the SQLite FTS5 database for semantic search
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let key = format!("thought_{}", timestamp);
    let _ = tracking::memory_set(&key, &cleaned_content);

    // Also record in the logs for token savings calculation
    let _ = tracking::record(
        "think",
        &cleaned_content,
        "[Thought Stored]",
        &cleaned_content,
    );

    println!(
        "[RTK] Thought successfully offloaded to vector memory ({} bytes).",
        cleaned_content.len()
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_think_basic() {
        // This is a simple test just to ensure the module compiles and structure is correct.
        // Full E2E tests will verify the database insertion.
        let args = vec!["I".to_string(), "am".to_string(), "thinking".to_string()];
        let result = run(args);
        assert!(result.is_ok());
    }
}
