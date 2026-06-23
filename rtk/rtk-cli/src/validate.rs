use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Run project quality gate: `scripts/dev-gate.sh` if present, else `cargo fmt/clippy/test` on `rtk/`.
pub fn run_validate() -> Result<()> {
    let cwd = std::env::current_dir()?;
    if let Some(gate) = find_up(&cwd, "scripts/dev-gate.sh") {
        let root = gate
            .parent()
            .and_then(|p| p.parent())
            .context("invalid scripts/dev-gate.sh path")?;
        println!("rtk validate: {}", gate.display());
        return run_dev_gate(root, &gate);
    }

    let manifest = find_up(&cwd, "rtk/Cargo.toml")
        .context("no scripts/dev-gate.sh or rtk/Cargo.toml found — run from project root")?;
    let rtk_dir = manifest.parent().context("invalid rtk/Cargo.toml path")?;
    run_cargo_gate(rtk_dir)
}

fn find_up(start: &Path, rel: &str) -> Option<PathBuf> {
    let mut cur = start.to_path_buf();
    loop {
        let candidate = cur.join(rel);
        if candidate.is_file() {
            return Some(candidate);
        }
        if !cur.pop() {
            break;
        }
    }
    None
}

fn run_dev_gate(root: &Path, script: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        let status = Command::new("bash")
            .arg(script)
            .current_dir(root)
            .status()
            .context("failed to run dev-gate.sh")?;
        if !status.success() {
            std::process::exit(status.code().unwrap_or(1));
        }
        Ok(())
    }
    #[cfg(not(unix))]
    {
        let _ = (root, script);
        let rtk_dir = root.join("rtk");
        run_cargo_gate(&rtk_dir)
    }
}

fn run_cargo_gate(rtk_dir: &Path) -> Result<()> {
    let manifest = rtk_dir.join("Cargo.toml");
    let m = manifest
        .to_str()
        .context("non-UTF8 path to rtk/Cargo.toml")?;

    let steps: &[(&str, Vec<&str>)] = &[
        (
            "fmt --check",
            vec!["fmt", "--manifest-path", m, "--all", "--", "--check"],
        ),
        (
            "clippy",
            vec![
                "clippy",
                "--manifest-path",
                m,
                "--workspace",
                "--all-targets",
                "--",
                "-D",
                "warnings",
            ],
        ),
        ("test", vec!["test", "--manifest-path", m, "--workspace"]),
    ];

    for (label, args) in steps {
        println!("rtk validate: cargo {label}");
        let status = Command::new("cargo")
            .args(args)
            .status()
            .with_context(|| format!("failed to run cargo {label}"))?;
        if !status.success() {
            std::process::exit(status.code().unwrap_or(1));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_dev_gate_from_rtk_subdir() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .to_path_buf();
        assert!(find_up(&root.join("rtk"), "scripts/dev-gate.sh").is_some());
    }
}
