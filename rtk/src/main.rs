use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};

mod cargo_build;
mod cargo_test;
mod config;
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
mod status;
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
    Init {
        /// The savings profile to use: low, medium, high, max
        #[arg(long, default_value = "high")]
        profile: String,
    },
    /// Show the current RTK status and active profile
    Status,
    /// Force database garbage collection of logs older than 30 days
    Gc,
    /// Manage global personal configurations (guards, DLP patterns)
    Config {
        #[command(subcommand)]
        subcmd: ConfigCommands,
    },
}

#[derive(Subcommand)]
enum ConfigCommands {
    /// Show the current integrated configuration
    Show,
    /// Manage personal command guardrails
    Deny {
        #[command(subcommand)]
        subcmd: ConfigDenyCommands,
    },
    /// Manage custom Data Loss Prevention (DLP) secret patterns
    Dlp {
        #[command(subcommand)]
        subcmd: ConfigDlpCommands,
    },
}

#[derive(Subcommand)]
enum ConfigDenyCommands {
    /// Add a pattern to the list of denied command guards
    Add {
        /// The command substring or regex pattern to guard against
        pattern: String,
    },
}

#[derive(Subcommand)]
enum ConfigDlpCommands {
    /// Add a regex pattern to the list of custom DLP secret redactors
    Add {
        /// The regex pattern for custom secret detection
        pattern: String,
    },
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
                "test" => run_filtered_combined("cargo", &args, cargo_test::filter),
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
        Commands::Gradle { args } => run_filtered(&get_gradle_bin(), &args, gradle::filter),
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
        Commands::Init { profile } => setup::run_init(&profile),
        Commands::Status => crate::status::run_status(),
        Commands::Gc => tracking::gc().map(|purged| {
            println!("🗑️ Database garbage collection complete: removed {} log records older than 30 days.", purged);
        }),
        Commands::Config { subcmd } => match subcmd {
            ConfigCommands::Show => config::config_show(),
            ConfigCommands::Deny { subcmd } => match subcmd {
                ConfigDenyCommands::Add { pattern } => config::config_deny_add(&pattern).map(|_| {
                    println!("🔒 Added command guard pattern: \"{}\"", pattern);
                }),
            },
            ConfigCommands::Dlp { subcmd } => match subcmd {
                ConfigDlpCommands::Add { pattern } => config::config_dlp_add(&pattern).map(|_| {
                    println!("🛡️ Added custom DLP regex pattern: \"{}\"", pattern);
                }),
            },
        },
    };

    if let Err(e) = result {
        eprintln!("rtk: {e}");
        std::process::exit(1);
    }
}

fn get_gradle_bin() -> String {
    let mut current = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    loop {
        let wrapper_unix = current.join("gradlew");
        let wrapper_win = current.join("gradlew.bat");
        
        if wrapper_unix.exists() || wrapper_win.exists() {
            if cfg!(target_os = "windows") {
                return wrapper_win.to_string_lossy().to_string();
            } else {
                return wrapper_unix.to_string_lossy().to_string();
            }
        }
        
        if !current.pop() {
            break;
        }
    }
    "gradle".to_string()
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

enum FilterMode {
    Stdout(fn(&str) -> String),
    Stderr(fn(&str) -> String),
    Combined(fn(&str) -> String),
    Distilled,
}

fn execute_with_filter(bin: &str, args: &[String], mode: FilterMode) -> Result<()> {
    let output = std::process::Command::new(bin)
        .args(args)
        .output()
        .with_context(|| format!("failed to execute {bin}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    let (mut out_print, mut err_print, raw_db, filtered_db) = match mode {
        FilterMode::Stdout(filter) => {
            let filtered = filter(&stdout);
            let r_filtered = dlp::redact(&filtered);
            let r_stdout = dlp::redact(&stdout);
            (r_filtered.clone(), dlp::redact(&stderr), r_stdout.clone(), r_filtered)
        }
        FilterMode::Stderr(filter) => {
            let filtered = filter(&stderr);
            let r_filtered = dlp::redact(&filtered);
            let r_stderr = dlp::redact(&stderr);
            (dlp::redact(&stdout), r_filtered.clone(), r_stderr.clone(), r_filtered)
        }
        FilterMode::Combined(filter) => {
            let combined = format!("{stderr}\n{stdout}");
            let filtered = filter(&combined);
            let r_filtered = dlp::redact(&filtered);
            let r_combined = dlp::redact(&combined);
            (r_filtered.clone(), String::new(), r_combined.clone(), r_filtered)
        }
        FilterMode::Distilled => {
            let d_stdout = distiller::distill(&stdout, None);
            let d_stderr = distiller::distill(&stderr, None);
            let r_d_out = dlp::redact(&d_stdout);
            let r_d_err = dlp::redact(&d_stderr);
            let r_comb = dlp::redact(&format!("STDOUT:\n{stdout}\nSTDERR:\n{stderr}"));
            (r_d_out.clone(), r_d_err.clone(), r_comb, format!("{r_d_out}\n{r_d_err}"))
        }
    };

    let cmd_label = format!("{} {}", bin, args.first().map(|s| s.as_str()).unwrap_or(""));
    
    match tracking::record(cmd_label.trim(), &raw_db, &filtered_db, &raw_db) {
        Ok(log_id) => {
            if filtered_db.len() < raw_db.len() && !filtered_db.trim().is_empty() {
                let msg = format!("\n[Full output cached. Access with: rtk show-log {log_id}]\n");
                if !out_print.trim().is_empty() {
                    out_print.push_str(&msg);
                } else if !err_print.trim().is_empty() {
                    err_print.push_str(&msg);
                }
            }
        }
        Err(e) => eprintln!("rtk: tracking warning: {e}"),
    }

    if !out_print.is_empty() { print!("{out_print}"); }
    if !err_print.is_empty() { eprint!("{err_print}"); }

    if !output.status.success() {
        std::process::exit(output.status.code().unwrap_or(1));
    }
    Ok(())
}

fn run_filtered(bin: &str, args: &[String], filter: fn(&str) -> String) -> Result<()> {
    execute_with_filter(bin, args, FilterMode::Stdout(filter))
}

fn run_filtered_stderr(bin: &str, args: &[String], filter: fn(&str) -> String) -> Result<()> {
    execute_with_filter(bin, args, FilterMode::Stderr(filter))
}

fn run_filtered_combined(bin: &str, args: &[String], filter: fn(&str) -> String) -> Result<()> {
    execute_with_filter(bin, args, FilterMode::Combined(filter))
}

fn run_distilled(bin: &str, args: &[String]) -> Result<()> {
    execute_with_filter(bin, args, FilterMode::Distilled)
}
