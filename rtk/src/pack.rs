use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Packs directory contents into a single XML representation.
/// Respects ignore patterns (.gitignore, .rtkignore, and standard defaults)
/// and optional content minification/stripping.
pub fn pack_directory(dir_path: &Path, strip: bool, skeleton: bool) -> Result<String> {
    let mut out = String::new();
    out.push_str("<repository>\n");

    let canonical_root = dir_path
        .canonicalize()
        .with_context(|| format!("failed to canonicalize root path: {}", dir_path.display()))?;

    let ignore_patterns = load_ignore_patterns(&canonical_root);

    pack_recursive(
        &canonical_root,
        &canonical_root,
        &ignore_patterns,
        strip,
        skeleton,
        &mut out,
    )?;

    out.push_str("</repository>\n");
    Ok(out)
}

fn load_ignore_patterns(dir: &Path) -> Vec<String> {
    let mut patterns = vec![
        ".git".to_string(),
        "node_modules".to_string(),
        "target".to_string(),
        "dist".to_string(),
        ".venv".to_string(),
        ".pytest_cache".to_string(),
        "__pycache__".to_string(),
        ".idea".to_string(),
        ".vscode".to_string(),
        "build".to_string(),
        "Cargo.lock".to_string(),
        "package-lock.json".to_string(),
    ];

    for ignore_file in &[".gitignore", ".rtkignore"] {
        let path = dir.join(ignore_file);
        if path.exists() {
            if let Ok(content) = fs::read_to_string(path) {
                for line in content.lines() {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() && !trimmed.starts_with('#') {
                        let normalized = trimmed.replace('\\', "/");
                        patterns.push(normalized);
                    }
                }
            }
        }
    }
    patterns
}

fn should_ignore(relative_path: &str, patterns: &[String]) -> bool {
    let path_norm = relative_path.replace('\\', "/");
    let path_parts: Vec<&str> = path_norm.split('/').collect();

    for pattern in patterns {
        let pat_norm = pattern.trim_end_matches('/');

        // Exact directory or file name match in parts
        if path_parts.contains(&pat_norm) {
            return true;
        }

        // Relative path prefix or substring match
        if path_norm.starts_with(pat_norm) || path_norm.contains(&format!("/{pat_norm}")) {
            return true;
        }
    }
    false
}

/// Collapses empty lines and strips full-line comments based on file type.
pub fn strip_content(content: &str, extension: &str) -> String {
    let mut result = String::with_capacity(content.len());
    let mut last_was_empty = false;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            if !last_was_empty {
                result.push('\n');
                last_was_empty = true;
            }
            continue;
        }

        // Detect full-line comments for common languages
        let is_comment = match extension {
            "rs" | "js" | "ts" | "go" | "java" | "cpp" | "c" | "css" | "swift" | "scala" => {
                trimmed.starts_with("//") || (trimmed.starts_with("/*") && trimmed.ends_with("*/"))
            }
            "py" | "sh" | "yml" | "yaml" | "ini" | "toml" | "dockerfile" | "rb" | "pl" => {
                trimmed.starts_with('#')
            }
            "html" | "xml" | "md" => trimmed.starts_with("<!--") && trimmed.ends_with("-->"),
            _ => false,
        };

        if is_comment {
            continue;
        }

        result.push_str(line);
        result.push('\n');
        last_was_empty = false;
    }
    result
}

fn pack_recursive(
    root: &Path,
    current: &Path,
    ignore_patterns: &[String],
    strip: bool,
    skeleton: bool,
    out: &mut String,
) -> Result<()> {
    let relative_path = current
        .strip_prefix(root)
        .unwrap_or(current)
        .to_string_lossy()
        .replace('\\', "/");

    if !relative_path.is_empty() && should_ignore(&relative_path, ignore_patterns) {
        return Ok(());
    }

    if current.is_dir() {
        for entry in fs::read_dir(current)? {
            let entry = entry?;
            pack_recursive(root, &entry.path(), ignore_patterns, strip, skeleton, out)?;
        }
    } else if current.is_file() {
        if is_binary_file(current) {
            return Ok(());
        }

        if let Ok(content) = fs::read_to_string(current) {
            let ext = current
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or_default();

            let mut processed = if strip {
                strip_content(&content, ext)
            } else {
                content
            };

            if skeleton {
                processed = crate::skeleton::skeletonize(&processed, ext);
            }

            // Always run DLP sensitive data scrubbing for safety
            let redacted = crate::dlp::redact(&processed);

            out.push_str(&format!("  <file path=\"{relative_path}\">\n"));
            out.push_str("    <![CDATA[\n");
            out.push_str(&redacted);
            if !redacted.ends_with('\n') {
                out.push('\n');
            }
            out.push_str("    ]]>\n");
            out.push_str("  </file>\n");
        }
    }
    Ok(())
}

fn is_binary_file(path: &Path) -> bool {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_default();

    matches!(
        ext.as_str(),
        "png"
            | "jpg"
            | "jpeg"
            | "gif"
            | "ico"
            | "exe"
            | "dll"
            | "so"
            | "dylib"
            | "zip"
            | "tar"
            | "gz"
            | "pdf"
            | "db"
            | "sqlite"
            | "bin"
            | "woff"
            | "woff2"
            | "ttf"
            | "eot"
            | "mp3"
            | "mp4"
            | "wav"
            | "avi"
            | "mov"
            | "dmg"
            | "iso"
            | "lock"
            | "pyc"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pack_directory() {
        let temp_dir = std::env::temp_dir().join(format!("rtk_pack_test_{}", std::process::id()));
        fs::create_dir_all(&temp_dir).unwrap();

        // Create main.rs
        let file1_path = temp_dir.join("main.rs");
        fs::write(&file1_path, "fn main() {}\n").unwrap();

        // Create target directory (ignored by default)
        let ignored_dir = temp_dir.join("target");
        fs::create_dir_all(&ignored_dir).unwrap();
        let file2_path = ignored_dir.join("output.bin");
        fs::write(&file2_path, &[0, 1, 2, 3]).unwrap();

        // Pack directory without strip
        let packed = pack_directory(&temp_dir, false, false).unwrap();
        assert!(packed.contains("<repository>"));
        assert!(packed.contains("<file path=\"main.rs\">"));
        assert!(packed.contains("fn main() {}"));
        assert!(
            !packed.contains("output.bin"),
            "should ignore target/ files"
        );

        fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_strip_content() {
        let source = concat!(
            "// This is a comment\n",
            "fn main() {\n",
            "    let x = 1;\n",
            "\n",
            "\n",
            "    // another comment\n",
            "    println!(\"x = {}\", x);\n",
            "}\n"
        );
        let stripped = strip_content(source, "rs");
        let expected = concat!(
            "fn main() {\n",
            "    let x = 1;\n",
            "\n",
            "    println!(\"x = {}\", x);\n",
            "}\n"
        );
        assert_eq!(stripped, expected);
    }

    #[test]
    fn test_custom_ignore() {
        let temp_dir = std::env::temp_dir().join(format!("rtk_ignore_test_{}", std::process::id()));
        fs::create_dir_all(&temp_dir).unwrap();

        // Create custom ignored file
        let file_path = temp_dir.join("secrets.txt");
        fs::write(&file_path, "top_secret_token\n").unwrap();

        // Create .rtkignore
        let ignore_path = temp_dir.join(".rtkignore");
        fs::write(&ignore_path, "secrets.txt\n").unwrap();

        let packed = pack_directory(&temp_dir, false, false).unwrap();
        assert!(
            !packed.contains("<file path=\"secrets.txt\">"),
            "should respect custom .rtkignore patterns"
        );

        fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_pack_dlp_and_skeleton() {
        let temp_dir =
            std::env::temp_dir().join(format!("rtk_pack_dlp_test_{}", std::process::id()));
        fs::create_dir_all(&temp_dir).unwrap();

        let file_path = temp_dir.join("main.rs");
        let content = "\
// A comment
const API_KEY: &str = \"sk-proj-1234567890abcdef1234567890abcdef12345678\";
fn test() {
    let x = 10;
}
";
        fs::write(&file_path, content).unwrap();

        let packed = pack_directory(&temp_dir, false, true).unwrap();
        assert!(packed.contains("[REDACTED_API_KEY]"));
        assert!(!packed.contains("sk-proj-"));
        assert!(!packed.contains("let x = 10"));

        fs::remove_dir_all(&temp_dir).ok();
    }
}
