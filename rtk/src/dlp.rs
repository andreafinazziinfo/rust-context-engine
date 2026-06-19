use lazy_static::lazy_static;
use regex::Regex;

pub fn redact(text: &str) -> String {
    lazy_static! {
        // Match PEM Private Keys
        static ref PRIVATE_KEY: Regex = Regex::new(
            r"(?s)-----BEGIN [A-Z ]+-----.*?-----END [A-Z ]+-----"
        ).unwrap();

        // Match JWT Tokens
        static ref JWT: Regex = Regex::new(
            r"\beyJh[A-Za-z0-9-_=]+\.[A-Za-z0-9-_=]+\.[A-Za-z0-9-_.+/=]*\b"
        ).unwrap();

        // Match typical API keys:
        // OpenAI: sk-proj-...
        // Stripe: sk_live_... / sk_test_...
        // AWS client id/secret: AKIA...
        static ref API_KEYS: Regex = Regex::new(
            r"(?i)\b(sk_(live|test)_[a-zA-Z0-9]{24}|sk-proj-[a-zA-Z0-9]{20,}|AKIA[0-9A-Z]{16})\b"
        ).unwrap();

        // Database credentials in URI: e.g. postgres://user:password@host
        static ref DB_URI: Regex = Regex::new(
            r"\b[a-zA-Z0-9\+]+://[a-zA-Z0-9_\-\.]+:[^@\s]+@[a-zA-Z0-9_\-\.]+"
        ).unwrap();
    }

    // 1. First redact specific large patterns (private keys)
    let mut redacted = PRIVATE_KEY
        .replace_all(text, "[REDACTED_PRIVATE_KEY]")
        .into_owned();

    // 2. Redact database credentials in URIs
    redacted = DB_URI
        .replace_all(&redacted, |caps: &regex::Captures| {
            let matched = caps.get(0).unwrap().as_str();
            if let Some(at_idx) = matched.find('@') {
                if let Some(slash_idx) = matched.find("://") {
                    let scheme = &matched[..slash_idx + 3];
                    let host = &matched[at_idx..];
                    return format!("{scheme}[REDACTED_CREDENTIALS]{host}");
                }
            }
            "[REDACTED_DB_URI]".to_string()
        })
        .into_owned();

    // 3. Redact JWTs and common API keys
    redacted = JWT.replace_all(&redacted, "[REDACTED_JWT]").into_owned();
    redacted = API_KEYS
        .replace_all(&redacted, "[REDACTED_API_KEY]")
        .into_owned();

    // 4. Entropy-based scanner for other random secrets
    let mut final_text = String::with_capacity(redacted.len());
    let mut current_word = String::new();

    for c in redacted.chars() {
        if c.is_alphanumeric() || c == '_' || c == '-' || c == '/' || c == '+' || c == '=' {
            current_word.push(c);
        } else {
            if !current_word.is_empty() {
                final_text.push_str(&check_and_redact_word(&current_word));
                current_word.clear();
            }
            final_text.push(c);
        }
    }
    if !current_word.is_empty() {
        final_text.push_str(&check_and_redact_word(&current_word));
    }

    final_text
}

fn check_and_redact_word(word: &str) -> String {
    if word.len() >= 24 && word.len() <= 128 {
        let is_git_hash = word.len() == 40 && word.chars().all(|c| c.is_ascii_hexdigit());

        if !is_git_hash {
            let entropy = shannon_entropy(word);
            // High entropy threshold: 4.5 bits/symbol is very high for random strings
            if entropy > 4.5 {
                return "[REDACTED_SECRET]".to_string();
            }
        }
    }
    word.to_string()
}

fn shannon_entropy(s: &str) -> f64 {
    if s.is_empty() {
        return 0.0;
    }
    let mut counts = [0usize; 256];
    let len = s.len() as f64;
    for &b in s.as_bytes() {
        counts[b as usize] += 1;
    }
    let mut entropy = 0.0;
    for &count in counts.iter() {
        if count > 0 {
            let p = count as f64 / len;
            entropy -= p * p.log2();
        }
    }
    entropy
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redact_private_key() {
        let input = "hello\n-----BEGIN PRIVATE KEY-----\nMIIEvgIBADANBgkqhkiG9w0BAQEFAASCBKgwggSkAgEAAoIBAQC6...\n-----END PRIVATE KEY-----\nworld";
        let output = redact(input);
        assert!(output.contains("[REDACTED_PRIVATE_KEY]"));
        assert!(!output.contains("MIIEvgI"));
    }

    #[test]
    fn test_redact_jwt() {
        let input = "token: eyJhR2VuZGEiOiJ1c2VyIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c.signature";
        let output = redact(input);
        assert!(output.contains("[REDACTED_JWT]"));
        assert!(!output.contains("eyJhR2VuZGEi"));
    }

    #[test]
    fn test_redact_api_keys() {
        let input = "openai: sk-proj-1234567890abcdef1234567890abcdef12345678, stripe: sk_live_51234567890abcdef123456";
        let output = redact(input);
        assert!(output.contains("[REDACTED_API_KEY]"));
        assert!(!output.contains("sk-proj-"));
    }

    #[test]
    fn test_redact_db_uri() {
        let input = "postgres://admin:verysecurepassword123@localhost:5432/mydb";
        let output = redact(input);
        assert!(output.contains("postgres://[REDACTED_CREDENTIALS]@localhost:5432/mydb"));
        assert!(!output.contains("verysecurepassword123"));
    }

    #[test]
    fn test_redact_high_entropy_secret() {
        let input = "secret: 8f7B2zK9wP3qR6vT1yX4mN7bV0cZ3aL9xJ2fH5dG8";
        let output = redact(input);
        assert!(output.contains("[REDACTED_SECRET]"));
        assert!(!output.contains("8f7B2zK9wP3q"));
    }

    #[test]
    fn test_preserve_safe_git_hash() {
        let input = "commit: 4f20c9d8e7a6b5c4d3e2f1a0b9c8d7e6f5a4b3c2";
        let output = redact(input);
        assert!(output.contains("4f20c9d8e7a6b5c4d3e2f1a0b9c8d7e6f5a4b3c2"));
        assert!(!output.contains("[REDACTED_SECRET]"));
    }
}
