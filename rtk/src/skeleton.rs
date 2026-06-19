pub fn skeletonize(content: &str, extension: &str) -> String {
    // Try AST-based skeletonization first
    if let Some(skel) = skeletonize_ast(content, extension) {
        return skel;
    }

    // Fallbacks
    match extension.to_lowercase().as_str() {
        "rs" | "go" | "java" | "c" | "cpp" | "cc" | "cxx" | "h" | "hpp" | "kt" => {
            skeletonize_braces(content)
        }
        "py" => skeletonize_indentation(content),
        "js" | "ts" | "jsx" | "tsx" => skeletonize_braces(content),
        _ => content.to_string(), // Fallback: don't skeletonize unsupported files
    }
}

fn skeletonize_ast(content: &str, extension: &str) -> Option<String> {
    use tree_sitter::Parser;

    let mut parser = Parser::new();
    let language = match extension.to_lowercase().as_str() {
        "rs" => tree_sitter_rust::language(),
        "py" => tree_sitter_python::language(),
        "js" | "jsx" => tree_sitter_javascript::language(),
        "ts" => tree_sitter_typescript::language_typescript(),
        "tsx" => tree_sitter_typescript::language_tsx(),
        "go" => tree_sitter_go::language(),
        "java" => tree_sitter_java::language(),
        _ => return None,
    };

    parser.set_language(&language).ok()?;
    let tree = parser.parse(content, None)?;
    let root = tree.root_node();

    // To avoid complex AST queries, we do a simple regex-like approach on the AST:
    // We collect ranges of all `block` or `statement_block` nodes that are children of function-like nodes.
    let mut ranges_to_collapse = Vec::new();

    let mut cursor = root.walk();
    let mut reached_root = false;

    while !reached_root {
        let node = cursor.node();
        let kind = node.kind();

        // Typical function block nodes across languages
        if kind == "block" || kind == "statement_block" {
            if let Some(parent) = node.parent() {
                let p_kind = parent.kind();
                if p_kind.contains("function")
                    || p_kind.contains("method")
                    || p_kind.contains("arrow")
                    || p_kind == "func_literal"
                {
                    // Python uses `block` for everything. Let's just drop it if it's inside a function.
                    // Keep the start and end tokens (e.g. `{` and `}`)
                    ranges_to_collapse.push((node.start_byte(), node.end_byte(), kind));
                }
            }
        }

        if cursor.goto_first_child() {
            continue;
        }

        if cursor.goto_next_sibling() {
            continue;
        }

        loop {
            if !cursor.goto_parent() {
                reached_root = true;
                break;
            }
            if cursor.goto_next_sibling() {
                break;
            }
        }
    }

    if ranges_to_collapse.is_empty() {
        return None; // Fallback to manual if AST found nothing
    }

    // Sort by start byte, filter overlapping
    ranges_to_collapse.sort_by_key(|r| r.0);

    let mut out = String::with_capacity(content.len());
    let mut last_idx = 0;

    for (start, end, kind) in ranges_to_collapse {
        if start < last_idx {
            continue;
        }

        out.push_str(&content[last_idx..start]);

        if kind == "block" {
            // Check if it's brace-based or indentation-based (Python)
            let is_python = extension == "py";
            if is_python {
                out.push_str("    pass  # collapsed");
            } else {
                out.push_str("{ /* collapsed */ }");
            }
        } else {
            out.push_str("{ /* collapsed */ }");
        }

        last_idx = end;
    }

    out.push_str(&content[last_idx..]);
    Some(out)
}

fn skeletonize_braces(content: &str) -> String {
    let mut out = String::with_capacity(content.len() / 2);
    let lines = content.lines();
    let mut skip_nesting = 0;

    for line in lines {
        let trimmed = line.trim();

        if skip_nesting > 0 {
            let (open, close) = count_braces(line);
            skip_nesting += open;
            if skip_nesting > close {
                skip_nesting -= close;
            } else {
                skip_nesting = 0;
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
            || trimmed.contains("interface ")
            || trimmed.contains("type ");

        let is_fn = !is_container
            && (trimmed.contains("fn ")
                || trimmed.contains("func ")
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
            let (open_braces, close_braces) = count_braces(line);

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

fn count_braces(line: &str) -> (usize, usize) {
    let mut open = 0;
    let mut close = 0;
    let mut in_string = false;
    let mut in_char = false;
    let mut chars = line.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\' {
            chars.next(); // skip escaped char
            continue;
        }
        if c == '"' && !in_char {
            in_string = !in_string;
            continue;
        }
        if c == '\'' && !in_string {
            in_char = !in_char;
            continue;
        }
        if !in_string && !in_char {
            if c == '/' && chars.peek() == Some(&'/') {
                break; // line comment
            }
            if c == '{' {
                open += 1;
            } else if c == '}' {
                close += 1;
            }
        }
    }
    (open, close)
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
