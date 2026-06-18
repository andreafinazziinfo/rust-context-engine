use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::Path;

mod cargo_build;
mod cargo_test;
mod git_diff;
mod git_log;
mod git_status;
mod rewrite;
mod tracking;
mod ls_filter;
mod pytest_filter;
mod distiller;
mod pack;
mod sync_rules;

#[derive(Parser)]
#[command(name = "rtk", version, about = "Token-efficient CLI wrapper for Claude Code")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Rewrite a raw command to its RTK equivalent.
    /// Exit codes: 0=rewrite found, 1=no match, 2=deny, 3=ask
    Rewrite {
        /// The raw command string to rewrite
        command: String,
    },
    /// Run a git subcommand with filtered output
    Git {
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Run a cargo subcommand with filtered output
    Cargo {
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Run an npm subcommand with filtered output
    Npm {
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Run a pytest invocation with filtered output
    Pytest {
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Run ls with filtered output
    Ls {
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Pack a directory's text files into an XML context block
    Pack {
        /// Path to the directory to pack
        #[arg(default_value = ".")]
        path: String,
        /// Strip comments and collapse consecutive empty lines
        #[arg(short, long)]
        strip: bool,
    },
    /// Print token savings statistics
    Stats,
    /// Synchronize rule files from root workspace to subprojects
    SyncRules,
    /// Retrieve a cached raw log by its ID
    ShowLog {
        /// The log ID
        id: i64,
    },
}

fn main() {
    let cli = Cli::parse();

    let result: Result<()> = match cli.command {
        Commands::Rewrite { command } => rewrite::run(&command),
        Commands::Git { args } => {
            let subcmd = args.first().map(|s| s.as_str()).unwrap_or("");
            match subcmd {
                "diff" => run_filtered("git", &args, git_diff::filter),
                "status" if !has_flag(&args, &["--porcelain", "--short", "-s"]) => {
                    run_filtered("git", &args, git_status::filter)
                }
                "log" => run_filtered("git", &args, git_log::filter),
                _ => passthrough("git", &args),
            }
        }
        Commands::Cargo { args } => {
            let subcmd = args.first().map(|s| s.as_str()).unwrap_or("");
            match subcmd {
                "test" => run_filtered("cargo", &args, cargo_test::filter),
                "build" | "check" => run_filtered_stderr("cargo", &args, cargo_build::filter),
                _ => passthrough("cargo", &args),
            }
        }
        Commands::Npm { args } => {
            // NPM output is notoriously long, let's distill it by default
            run_distilled("npm", &args)
        }
        Commands::Pytest { args } => run_filtered("pytest", &args, pytest_filter::filter),
        Commands::Ls { args } => run_filtered("ls", &args, ls_filter::filter),
        Commands::Pack { path, strip } => {
            pack::pack_directory(Path::new(&path), strip).map(|packed| {
                print!("{packed}");
            })
        }
        Commands::Stats => tracking::print_stats(),
        Commands::SyncRules => sync_rules::run(Path::new(".")),
        Commands::ShowLog { id } => {
            tracking::get_raw_log(id).map(|raw_log| {
                print!("{raw_log}");
            })
        }
    };

    if let Err(e) = result {
        eprintln!("rtk: {e}");
        std::process::exit(1);
    }
}

fn run_filtered(bin: &str, args: &[String], filter: fn(&str) -> String) -> Result<()> {
    let output = std::process::Command::new(bin)
        .args(args)
        .output()
        .with_context(|| format!("failed to execute {bin}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let filtered = filter(&stdout);

    let cmd_label = format!("{} {}", bin, args.first().map(|s| s.as_str()).unwrap_or(""));
    let mut final_output = filtered.clone();
    
    match tracking::record(cmd_label.trim(), &stdout, &filtered, &stdout) {
        Ok(log_id) => {
            if filtered.len() < stdout.len() && !filtered.trim().is_empty() {
                final_output.push_str(&format!("\n[Full output cached. Access with: rtk show-log {}]\n", log_id));
            }
        }
        Err(e) => {
            eprintln!("rtk: tracking warning: {e}");
        }
    }

    print!("{final_output}");

    if !output.stderr.is_empty() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprint!("{stderr}");
    }

    if !output.status.success() {
        std::process::exit(output.status.code().unwrap_or(1));
    }
    Ok(())
}

/// Like `run_filtered` but filters **stderr** instead of stdout.
/// Used for commands whose meaningful output is entirely on stderr
/// (e.g. `cargo build`, `cargo check`).
fn run_filtered_stderr(bin: &str, args: &[String], filter: fn(&str) -> String) -> Result<()> {
    let output = std::process::Command::new(bin)
        .args(args)
        .output()
        .with_context(|| format!("failed to execute {bin}"))?;

    let stderr = String::from_utf8_lossy(&output.stderr);
    let filtered = filter(&stderr);

    let cmd_label = format!("{} {}", bin, args.first().map(|s| s.as_str()).unwrap_or(""));
    let mut final_filtered = filtered.clone();
    
    match tracking::record(cmd_label.trim(), &stderr, &filtered, &stderr) {
        Ok(log_id) => {
            if filtered.len() < stderr.len() && !filtered.trim().is_empty() {
                final_filtered.push_str(&format!("\n[Full output cached. Access with: rtk show-log {}]\n", log_id));
            }
        }
        Err(e) => {
            eprintln!("rtk: tracking warning: {e}");
        }
    }

    // stdout (usually empty for build/check) passes through unchanged
    if !output.stdout.is_empty() {
        print!("{}", String::from_utf8_lossy(&output.stdout));
    }
    // filtered diagnostics go back to stderr
    eprint!("{final_filtered}");

    if !output.status.success() {
        std::process::exit(output.status.code().unwrap_or(1));
    }
    Ok(())
}

/// Runs a command and applies the generic distiller on its output.
fn run_distilled(bin: &str, args: &[String]) -> Result<()> {
    let output = std::process::Command::new(bin)
        .args(args)
        .output()
        .with_context(|| format!("failed to execute {bin}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    let combined_original = format!("STDOUT:\n{stdout}\nSTDERR:\n{stderr}");
    
    let distilled_stdout = distiller::distill(&stdout, None);
    let distilled_stderr = distiller::distill(&stderr, None);
    
    let mut final_stdout = distilled_stdout.clone();
    let mut final_stderr = distilled_stderr.clone();

    let cmd_label = format!("{} {}", bin, args.first().map(|s| s.as_str()).unwrap_or(""));
    
    // Calculate total original and filtered characters/tokens
    let total_orig_len = stdout.len() + stderr.len();
    let total_filt_len = distilled_stdout.len() + distilled_stderr.len();

    match tracking::record(cmd_label.trim(), &combined_original, &format!("{}\n{}", distilled_stdout, distilled_stderr), &combined_original) {
        Ok(log_id) => {
            if total_filt_len < total_orig_len {
                if !final_stdout.trim().is_empty() {
                    final_stdout.push_str(&format!("\n[Full output cached. Access with: rtk show-log {}]\n", log_id));
                } else if !final_stderr.trim().is_empty() {
                    final_stderr.push_str(&format!("\n[Full output cached. Access with: rtk show-log {}]\n", log_id));
                }
            }
        }
        Err(e) => {
            eprintln!("rtk: tracking warning: {e}");
        }
    }

    if !distilled_stdout.is_empty() {
        print!("{final_stdout}");
    }
    if !distilled_stderr.is_empty() {
        eprint!("{final_stderr}");
    }

    if !output.status.success() {
        std::process::exit(output.status.code().unwrap_or(1));
    }
    Ok(())
}

fn has_flag(args: &[String], flags: &[&str]) -> bool {
    args.iter().any(|a| flags.contains(&a.as_str()))
}

fn passthrough(bin: &str, args: &[String]) -> Result<()> {
    let status = std::process::Command::new(bin)
        .args(args)
        .status()
        .with_context(|| format!("failed to execute {bin}"))?;

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }
    Ok(())
}
