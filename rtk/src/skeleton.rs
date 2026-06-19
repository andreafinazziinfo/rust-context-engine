pub fn skeletonize(content: &str, extension: &str) -> String {
    match extension.to_lowercase().as_str() {
        "rs" => skeletonize_braces(content),
        "py" => skeletonize_indentation(content),
        "js" | "ts" | "jsx" | "tsx" => skeletonize_braces(content),
        _ => content.to_string(), // Fallback: don't skeletonize unsupported files
    }
}

fn skeletonize_braces(content: &str) -> String {
    let mut out = String::with_capacity(content.len() / 2);
    let lines = content.lines();
    let mut skip_nesting = 0;

    for line in lines {
        let trimmed = line.trim();

        if skip_nesting > 0 {
            // Count braces in skipped lines
            for c in line.chars() {
                if c == '{' {
                    skip_nesting += 1;
                } else if c == '}' {
                    skip_nesting -= 1;
                }
            }
            if skip_nesting == 0 {
                // Suffix with closing brace
                out.push_str("}\n");
            }
            continue;
        }

        let is_container = trimmed.contains("impl ")
            || trimmed.contains("struct ")
            || trimmed.contains("enum ")
            || trimmed.contains("trait ")
            || trimmed.contains("class ")
            || trimmed.contains("interface ");

        let is_fn = !is_container
            && (trimmed.contains("fn ")
                || trimmed.contains("function ")
                || trimmed.contains("=>")
                || (trimmed.contains('(')
                    && trimmed.contains(')')
                    && trimmed.contains('{')
                    && !trimmed.contains("if ")
                    && !trimmed.contains("for ")
                    && !trimmed.contains("while ")
                    && !trimmed.contains("switch ")
                    && !trimmed.contains("catch ")
                    && !trimmed.contains("match ")));

        if is_fn {
            let open_braces = trimmed.chars().filter(|&c| c == '{').count();
            let close_braces = trimmed.chars().filter(|&c| c == '}').count();

            if open_braces > close_braces {
                // Output line, insert collapse comment, start skipping
                out.push_str(line);
                out.push_str(" /* collapsed */ ");
                skip_nesting = open_braces - close_braces;
            } else {
                out.push_str(line);
                out.push('\n');
            }
        } else {
            out.push_str(line);
            out.push('\n');
        }
    }

    out
}

fn skeletonize_indentation(content: &str) -> String {
    let mut out = String::with_capacity(content.len() / 2);
    let mut lines = content.lines().peekable();
    let mut skip_level: Option<usize> = None;
    let mut decorators = Vec::new();

    while let Some(line) = lines.next() {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            out.push('\n');
            continue;
        }

        // Check if we are skipping indentations
        let indent = get_indent_level(line);
        if let Some(level) = skip_level {
            if indent > level {
                continue; // Skip body
            } else {
                skip_level = None; // Out of skipped block
            }
        }

        // Capture python decorators
        if trimmed.starts_with('@') {
            decorators.push(line.to_string());
            continue;
        }

        let is_decl = trimmed.starts_with("def ") || trimmed.starts_with("class ");

        if is_decl {
            // Flush decorators
            for dec in decorators.drain(..) {
                out.push_str(&dec);
                out.push('\n');
            }

            out.push_str(line);
            out.push('\n');

            // Look ahead to check body indentation
            if let Some(next_line) = lines.peek() {
                let next_trimmed = next_line.trim();
                if !next_trimmed.is_empty() {
                    let next_indent = get_indent_level(next_line);
                    if next_indent > indent {
                        // Insert collapsed pass statement
                        let indent_str = " ".repeat(next_indent);
                        out.push_str(&format!("{indent_str}pass  # collapsed\n"));
                        skip_level = Some(indent);
                    }
                }
            }
        } else {
            // Discard any decorators that didn't precede a declaration
            decorators.clear();
            out.push_str(line);
            out.push('\n');
        }
    }

    out
}

fn get_indent_level(line: &str) -> usize {
    let mut count = 0;
    for c in line.chars() {
        if c == ' ' {
            count += 1;
        } else if c == '\t' {
            count += 4;
        } else {
            break;
        }
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skeleton_rs() {
        let raw = "\
use std::collections::HashMap;

pub struct App {
    pub name: String,
}

impl App {
    pub fn new(name: &str) -> Self {
        let x = 12;
        App { name: name.to_string() }
    }

    pub fn run(&self) {
        println!(\"Running {}\", self.name);
    }
}
";
        let skeleton = skeletonize(raw, "rs");
        assert!(skeleton.contains("pub struct App"));
        assert!(skeleton.contains("impl App"));
        assert!(skeleton.contains("pub fn new"));
        assert!(skeleton.contains("pub fn run"));
        assert!(!skeleton.contains("let x = 12"));
        assert!(!skeleton.contains("println!"));
        assert!(skeleton.contains("/* collapsed */"));
    }

    #[test]
    fn test_skeleton_py() {
        let raw = "\
import sys

@click.group()
class CLI:
    \"\"\"Docstring\"\"\"
    pass

@click.command()
def run_app(port: int):
    # Some setup
    print(f\"Running on {port}\")
    sys.exit(0)

def other_func():
    return 42
";
        let skeleton = skeletonize(raw, "py");
        assert!(skeleton.contains("class CLI:"));
        assert!(skeleton.contains("def run_app"));
        assert!(skeleton.contains("@click.command()"));
        assert!(!skeleton.contains("print(f\"Running"));
        assert!(skeleton.contains("pass  # collapsed"));
    }
}
