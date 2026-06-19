use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::Path;

mod cargo_build;
mod cargo_test;
mod dashboard;
mod distiller;
mod dlp;
mod docker_filter;
mod git_diff;
mod git_log;
mod git_status;
mod go_test;
mod gradle;
mod ls_filter;
mod pack;
mod pytest_filter;
mod rewrite;
mod setup;
mod skeleton;
mod sync_rules;
mod tracking;

#[derive(Parser)]
#[command(
    name = "rtk",
    version,
    about = "Token-efficient CLI wrapper for Claude Code"
)]
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
    /// Run a gradle command with filtered output
    Gradle {
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Run go test with filtered output
    GoTest {
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Run a docker command with filtered output
    Docker {
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
        /// Generate skeletal structure of the code, exporting only signatures
        #[arg(short = 'k', long)]
        skeleton: bool,
        /// Maximum token budget (whitespace count). Errors if exceeded.
        #[arg(short, long)]
        limit: Option<usize>,
    },
    /// Manage long-term context memory for the current project
    Memory {
        #[command(subcommand)]
        subcmd: MemoryCommands,
    },
    /// Print token savings statistics
    Stats,
    /// Launch the local savings dashboard in the default browser
    Dashboard,
    /// Synchronize rule files from root workspace to subprojects
    SyncRules,
    /// Retrieve a cached raw log by its ID
    ShowLog {
        /// The log ID
        id: i64,
    },
    /// Bootstrap AI Efficiency rules in the current directory
    Init,
}

#[derive(Subcommand)]
enum MemoryCommands {
    /// Save a project memory key-value pair
    Set { key: String, value: String },
    /// Retrieve a project memory value by key
    Get { key: String },
    /// List all memory key-value pairs for the current project
    List,
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
        Commands::Gradle { args } => run_filtered(get_gradle_bin(), &args, gradle::filter),
        Commands::GoTest { args } => {
            let mut full_args = vec!["test".to_string()];
            full_args.extend(args);
            run_filtered("go", &full_args, go_test::filter)
        }
        Commands::Docker { args } => run_filtered("docker", &args, docker_filter::filter),
        Commands::Pack {
            path,
            strip,
            skeleton,
            limit,
        } => pack::pack_directory(Path::new(&path), strip, skeleton).and_then(|packed| {
            if let Some(lim) = limit {
                let tokens = packed.split_whitespace().count();
                if tokens > lim {
                    return Err(anyhow::anyhow!(
                        "Pack exceeded token limit! (Limit: {}, Total: {})",
                        lim,
                        tokens
                    ));
                }
            }
            print!("{packed}");
            Ok(())
        }),
        Commands::Memory { subcmd } => match subcmd {
            MemoryCommands::Set { key, value } => tracking::memory_set(&key, &value).map(|_| {
                println!("Memory saved: {} = {}", key, value);
            }),
            MemoryCommands::Get { key } => tracking::memory_get(&key).map(|val| {
                print!("{val}");
            }),
            MemoryCommands::List => tracking::memory_list().map(|list| {
                if list.is_empty() {
                    println!("No memory entries found for this project.");
                } else {
                    println!("========================================");
                    println!("          PROJECT CONTEXT MEMORY        ");
                    println!("========================================");
                    for (k, v) in list {
                        println!("{k}: {v}");
                    }
                    println!("========================================");
                }
            }),
        },
        Commands::Stats => tracking::print_stats(),
        Commands::Dashboard => dashboard::run_dashboard(),
        Commands::SyncRules => sync_rules::run(Path::new(".")),
        Commands::ShowLog { id } => tracking::get_raw_log(id).map(|raw_log| {
            print!("{raw_log}");
        }),
        Commands::Init => setup::run_init(),
    };

    if let Err(e) = result {
        eprintln!("rtk: {e}");
        std::process::exit(1);
    }
}

fn get_gradle_bin() -> &'static str {
    if Path::new("./gradlew").exists() || Path::new("gradlew.bat").exists() {
        if cfg!(target_os = "windows") {
            "gradlew.bat"
        } else {
            "./gradlew"
        }
    } else {
        "gradle"
    }
}

fn run_filtered(bin: &str, args: &[String], filter: fn(&str) -> String) -> Result<()> {
    let output = std::process::Command::new(bin)
        .args(args)
        .output()
        .with_context(|| format!("failed to execute {bin}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let filtered = filter(&stdout);

    // DLP sensitive data scrubbing
    let redacted_filtered = dlp::redact(&filtered);
    let redacted_stdout = dlp::redact(&stdout);

    let cmd_label = format!("{} {}", bin, args.first().map(|s| s.as_str()).unwrap_or(""));
    let mut final_output = redacted_filtered.clone();

    match tracking::record(
        cmd_label.trim(),
        &redacted_stdout,
        &redacted_filtered,
        &redacted_stdout,
    ) {
        Ok(log_id) => {
            if redacted_filtered.len() < redacted_stdout.len()
                && !redacted_filtered.trim().is_empty()
            {
                final_output.push_str(&format!(
                    "\n[Full output cached. Access with: rtk show-log {}]\n",
                    log_id
                ));
            }
        }
        Err(e) => {
            eprintln!("rtk: tracking warning: {e}");
        }
    }

    print!("{final_output}");

    if !output.stderr.is_empty() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprint!("{}", dlp::redact(&stderr));
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

    // DLP sensitive data scrubbing
    let redacted_filtered = dlp::redact(&filtered);
    let redacted_stderr = dlp::redact(&stderr);

    let cmd_label = format!("{} {}", bin, args.first().map(|s| s.as_str()).unwrap_or(""));
    let mut final_filtered = redacted_filtered.clone();

    match tracking::record(
        cmd_label.trim(),
        &redacted_stderr,
        &redacted_filtered,
        &redacted_stderr,
    ) {
        Ok(log_id) => {
            if redacted_filtered.len() < redacted_stderr.len()
                && !redacted_filtered.trim().is_empty()
            {
                final_filtered.push_str(&format!(
                    "\n[Full output cached. Access with: rtk show-log {}]\n",
                    log_id
                ));
            }
        }
        Err(e) => {
            eprintln!("rtk: tracking warning: {e}");
        }
    }

    // stdout (usually empty for build/check) passes through unchanged (scrubbed for safety)
    if !output.stdout.is_empty() {
        print!("{}", dlp::redact(&String::from_utf8_lossy(&output.stdout)));
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
    let redacted_combined = dlp::redact(&combined_original);

    let distilled_stdout = distiller::distill(&stdout, None);
    let distilled_stderr = distiller::distill(&stderr, None);

    // DLP sensitive data scrubbing
    let redacted_dist_stdout = dlp::redact(&distilled_stdout);
    let redacted_dist_stderr = dlp::redact(&distilled_stderr);

    let mut final_stdout = redacted_dist_stdout.clone();
    let mut final_stderr = redacted_dist_stderr.clone();

    let cmd_label = format!("{} {}", bin, args.first().map(|s| s.as_str()).unwrap_or(""));

    // Calculate total original and filtered characters/tokens
    let total_orig_len = stdout.len() + stderr.len();
    let total_filt_len = distilled_stdout.len() + distilled_stderr.len();

    match tracking::record(
        cmd_label.trim(),
        &redacted_combined,
        &format!("{}\n{}", redacted_dist_stdout, redacted_dist_stderr),
        &redacted_combined,
    ) {
        Ok(log_id) => {
            if total_filt_len < total_orig_len {
                if !final_stdout.trim().is_empty() {
                    final_stdout.push_str(&format!(
                        "\n[Full output cached. Access with: rtk show-log {}]\n",
                        log_id
                    ));
                } else if !final_stderr.trim().is_empty() {
                    final_stderr.push_str(&format!(
                        "\n[Full output cached. Access with: rtk show-log {}]\n",
                        log_id
                    ));
                }
            }
        }
        Err(e) => {
            eprintln!("rtk: tracking warning: {e}");
        }
    }

    if !redacted_dist_stdout.is_empty() {
        print!("{final_stdout}");
    }
    if !redacted_dist_stderr.is_empty() {
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
