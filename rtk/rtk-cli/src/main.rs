use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};

use rtk_db::{config, dlp, session, status, think, tracking};
use rtk_filters::{
    cargo_build, cargo_test, docker_filter, git_diff, git_log, git_status, go_test, gradle,
    ls_filter, pytest_filter,
};
use rtk_pack::pack;

mod agents;
mod artifact;
mod benchmark;
mod dashboard;
mod distiller;
mod doctor;
mod dotnet;
mod index_cli;
mod plugins;
mod rewrite;
mod setup;
mod sync_rules;

#[cfg(test)]
mod fuzz_tests;

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
    /// Run a yarn subcommand with filtered output
    Yarn {
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Run a pnpm subcommand with filtered output
    Pnpm {
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Run a composer subcommand with filtered output
    Composer {
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Run a terraform subcommand with filtered output
    Terraform {
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Run a dotnet subcommand with filtered output
    Dotnet {
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Store LLM Chain-of-Thought in the semantic memory instead of polluting the chat window
    Think {
        /// Optional text to store (if not provided, reads from stdin)
        #[arg(trailing_var_arg = true)]
        content: Vec<String>,
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
    /// Manage project artifacts
    Artifact {
        #[command(subcommand)]
        subcmd: ArtifactCommands,
    },
    /// Check budget status and alerts
    Budget {
        #[command(subcommand)]
        subcmd: BudgetCommands,
    },
    /// Model routing helpers and suggestions
    Model {
        #[command(subcommand)]
        subcmd: ModelCommands,
    },
    /// Model Context Protocol (MCP) server utilities
    Mcp {
        #[command(subcommand)]
        subcmd: McpCommands,
    },
    /// Query symbols in the codebase
    Symbols {
        #[command(subcommand)]
        subcmd: SymbolsCommands,
    },
    /// Query file dependencies
    Deps {
        #[command(subcommand)]
        subcmd: DepsCommands,
    },
    /// Find references to a symbol
    Refs {
        #[command(subcommand)]
        subcmd: RefsCommands,
    },
    /// Analyze upstream blast radius of a symbol
    Impact {
        #[command(subcommand)]
        subcmd: ImpactCommands,
    },
    /// Force full project indexing
    Index {
        #[command(subcommand)]
        subcmd: IndexCommands,
    },
    /// Export code graph data
    Graph {
        #[command(subcommand)]
        subcmd: GraphCommands,
    },
    /// Print token savings statistics
    Stats {
        /// Show text-based ASCII cost trend chart
        #[arg(short, long)]
        chart: bool,
    },
    /// Shorthand alias to print token savings stats (from upstream parity)
    Gain {
        /// Show text-based ASCII cost trend chart
        #[arg(short, long)]
        chart: bool,
    },
    /// Generate a detailed token savings audit report or audit codebase graph
    Audit {
        #[command(subcommand)]
        subcmd: Option<AuditCommands>,
        /// Output path for the Markdown report (defaults to rtk-audit.md)
        #[arg(short, long, default_value = "rtk-audit.md")]
        output: String,
    },
    /// Manage agent configurations and rule files (AGENTS.md / CLAUDE.md)
    Agents {
        #[command(subcommand)]
        subcmd: AgentsCommands,
    },
    /// Benchmark export and analysis utilities
    Benchmark {
        #[command(subcommand)]
        subcmd: BenchmarkCommands,
    },
    /// Check the health of the RTK installation and configuration
    Doctor,
    /// Estimate token counts and API costs for the active git diff
    #[command(alias = "est")]
    Estimate,
    /// Manage session-state variables for context handoff
    SessionState {
        #[command(subcommand)]
        subcmd: SessionStateCommands,
    },
    /// Launch the local savings dashboard in the default browser
    Dashboard {
        /// Start a real-time web server for live telemetry updates
        #[arg(short, long)]
        live: bool,
        /// Custom port for the live dashboard (defaults to 3000)
        #[arg(short, long)]
        port: Option<u16>,
    },
    /// Run a dynamic plugin command with declarative filtering
    Plugin {
        /// Name of the plugin defined in plugins.toml
        name: String,
        /// Arguments to pass to the plugin command
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
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
    /// Advanced context optimization and policy engine
    Context {
        #[command(subcommand)]
        subcmd: ContextCommands,
    },
    /// Manage and export telemetry data
    Telemetry {
        #[command(subcommand)]
        subcmd: TelemetryCommands,
    },
    /// Manage custom regex filtering rules
    Filter {
        #[command(subcommand)]
        subcmd: FilterCommands,
    },
}

#[derive(Subcommand, Debug)]
enum FilterCommands {
    /// Add a custom regex filtering rule to global config
    Add {
        /// The regex pattern to match
        #[arg(long)]
        pattern: String,
        /// The action to perform: strip or collapse
        #[arg(long)]
        action: String,
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
    /// Set the default active savings profile in global config
    Profile {
        /// The profile name to set (strict, balanced, developer, audit, json-only)
        name: String,
    },
    /// Export the global configuration file to stdout (JSON format)
    Export,
    /// Import and overwrite the global configuration file from a file path or stdin
    Import {
        /// Optional path to the JSON configuration file to import (omitted reads from stdin)
        #[arg(short, long)]
        path: Option<String>,
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
    /// Search memory values semantically (FTS5)
    Search { query: String },
    /// List all memory key-value pairs for the current project
    List,
    /// Overwrite an existing project memory value with alert logs
    Overwrite { key: String, value: String },
    /// Run the memory health check (duplicates, stale, contradictions)
    Doctor,
}

#[derive(Subcommand, Debug)]
enum BudgetCommands {
    /// Check budget status and total cost spent
    Check {
        /// Optional budget limit in USD
        #[arg(long)]
        limit: Option<f64>,
    },
}

#[derive(Subcommand, Debug)]
enum ModelCommands {
    /// Suggest model for a task type
    Suggest {
        /// The task type (e.g. simple, single-file-edit, complex, planning, audit)
        #[arg(long)]
        task: String,
    },
}

#[derive(Subcommand, Debug)]
enum GraphCommands {
    /// Export graph to a format (e.g. obsidian)
    Export {
        /// The export format (currently only 'obsidian' is supported)
        #[arg(long, default_value = "obsidian")]
        format: String,
        /// The output directory
        #[arg(short, long, default_value = "obsidian/")]
        output: String,
    },
}

#[derive(Subcommand, Debug)]
enum AuditCommands {
    /// Run graph audit (symbols count, edges count, query latency, graph coverage %)
    Graph,
}

#[derive(Subcommand, Debug)]
enum McpCommands {
    /// Start the stdio JSON-RPC MCP server
    Start,
    /// Install RTK MCP server config to client (claude, cursor, gemini)
    Install {
        /// The client application name
        #[arg(long)]
        client: String,
    },
    /// Ping the MCP server to check connectivity and diagnostic status
    Ping,
    /// Directly call a specific tool for testing/validation
    Call {
        /// Name of the tool to execute
        tool: String,
        /// JSON object of arguments passed to the tool
        #[arg(long)]
        args: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
enum ArtifactCommands {
    /// List all registered artifacts
    List,
    /// Get the content of a specific artifact
    Get {
        /// The artifact ID
        id: String,
    },
    /// Clean up artifacts older than 30 days
    Gc,
}

#[derive(Subcommand, Debug)]
enum SymbolsCommands {
    /// Find symbols by name
    Find {
        /// Name query to search
        query: String,
    },
}

#[derive(Subcommand, Debug)]
enum DepsCommands {
    /// Show dependencies of a file
    Show {
        /// File path to analyze
        file: String,
    },
}

#[derive(Subcommand, Debug)]
enum RefsCommands {
    /// Find references to a symbol
    Find {
        /// Symbol name to search
        symbol: String,
    },
}

#[derive(Subcommand, Debug)]
enum ImpactCommands {
    /// Analyze upstream blast radius
    Analyze {
        /// Symbol name to analyze
        symbol: String,
    },
}

#[derive(Subcommand, Debug)]
enum IndexCommands {
    /// Force complete project re-indexing
    Run,
    /// Show index freshness and coverage
    Status {
        /// Output JSON
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand, Debug)]
enum BenchmarkCommands {
    /// Export historical benchmark data to JSON or CSV
    Export {
        /// Output format: json or csv
        #[arg(short, long, default_value = "json")]
        format: String,
        /// Path to save the exported data
        #[arg(short, long)]
        output: String,
    },
}

#[derive(Subcommand, Debug)]
enum SessionStateCommands {
    /// Initialize default session-state variables for the current project
    Init,
    /// Retrieve current session-state formatted as JSON
    Get,
    /// Update a specific session-state key-value pair
    Update {
        /// Key to update (e.g. decisions, active_tasks, context_files, warnings)
        key: String,
        /// Value to assign (can be raw text or JSON array string)
        value: String,
    },
    /// Export the session-state to a markdown handoff document
    Export,
}

#[derive(Subcommand, Debug)]
enum AgentsCommands {
    /// Initialize agent rule files (AGENTS.md / CLAUDE.md)
    Init {
        /// The template type: solo-dev, team, OSS, mono-repo
        #[arg(long, default_value = "solo-dev")]
        template: String,
    },
    /// Run validation on agent rule files (syntax, duplicate or conflicting keys)
    Doctor,
    /// Compact agent rule files to save input tokens (e.g. into a denser or caveman-like format)
    Compact,
}

#[derive(Subcommand, Debug)]
enum ContextCommands {
    /// Compact standard input or context string under specified token limit using output profiles
    Compact {
        /// Maximum token budget (whitespace token count)
        #[arg(short, long)]
        max_tokens: usize,
        /// The profile preset to use: strict, balanced, developer
        #[arg(short, long, default_value = "balanced")]
        profile: String,
    },
}

#[derive(Subcommand, Debug)]
enum TelemetryCommands {
    /// Export recorded telemetry to external dashboards/formats
    Export {
        /// Output format: json or prometheus
        #[arg(short, long, default_value = "json")]
        format: String,
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
                "test" => run_filtered_combined("cargo", &args, cargo_test::filter),
                "build" | "check" => run_filtered_stderr("cargo", &args, cargo_build::filter),
                _ => passthrough("cargo", &args),
            }
        }
        Commands::Npm { args } => run_distilled("npm", &args),
        Commands::Yarn { args } => run_distilled("yarn", &args),
        Commands::Pnpm { args } => run_distilled("pnpm", &args),
        Commands::Composer { args } => run_distilled("composer", &args),
        Commands::Terraform { args } => run_distilled("terraform", &args),
        Commands::Dotnet { args } => {
            dotnet::execute_dotnet(&args);
            Ok(())
        },
        Commands::Think { content } => think::run(content),
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
            MemoryCommands::Search { query } => tracking::memory_search(&query).map(|results| {
                if results.is_empty() {
                    println!("No semantic matches found for: '{}'", query);
                } else {
                    println!("========================================");
                    println!("          SEMANTIC SEARCH RESULTS       ");
                    println!("========================================");
                    for (k, v) in results {
                        println!("- {k}:\n  {v}\n");
                    }
                    println!("========================================");
                }
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
            MemoryCommands::Overwrite { key, value } => tracking::memory_overwrite(&key, &value).map(|_| {
                println!("Memory overwritten successfully.");
            }),
            MemoryCommands::Doctor => tracking::memory_doctor().map(|report| {
                println!("🩺 RTK Memory Health Report");
                println!("========================================");
                if report.duplicates.is_empty() && report.stale.is_empty() && report.contradictory.is_empty() {
                    println!("✅ Memory is healthy! No duplicates, stale entries (>30 days), or contradictions found.");
                } else {
                    if !report.duplicates.is_empty() {
                        println!("⚠️  Duplicate keys found:");
                        for dup in report.duplicates {
                            println!("  - {}", dup);
                        }
                    }
                    if !report.stale.is_empty() {
                        println!("⚠️  Stale keys (>30 days since last access) found:");
                        for (key, last_access) in report.stale {
                            println!("  - {} (last accessed: {})", key, last_access);
                        }
                    }
                    if !report.contradictory.is_empty() {
                        println!("❌ Contradictory keys (case-insensitive variants with different values) found:");
                        for (k1, k2, diff) in report.contradictory {
                            println!("  - '{}' vs '{}': {}", k1, k2, diff);
                        }
                    }
                }
                println!("========================================");
            }),
        },
        Commands::Budget { subcmd } => match subcmd {
            BudgetCommands::Check { limit } => {
                let limit_usd = limit.unwrap_or(50.0);
                match rtk_db::pricing::check_budget(limit_usd) {
                    Ok(status) => {
                        println!("💰 Budget Status:");
                        println!("----------------------------------------");
                        println!("Limit:        ${:.2}", status.limit_usd);
                        println!("Spent:        ${:.6}", status.spent_usd);
                        println!("Percentage:   {:.2}%", status.percentage);
                        if status.exceeded {
                            println!("🚨 ALERT: Budget limit exceeded!");
                        } else {
                            println!("✅ Within budget limits.");
                        }
                        println!("----------------------------------------");
                        Ok(())
                    }
                    Err(e) => Err(anyhow::anyhow!("Failed to check budget: {e}")),
                }
            }
        },
        Commands::Model { subcmd } => match subcmd {
            ModelCommands::Suggest { task } => {
                let suggestion = rtk_db::pricing::suggest_model(&task);
                println!("Routing Suggestion: task '{}' -> use model '{}'", task, suggestion);
                Ok(())
            }
        },
        Commands::Mcp { subcmd } => match subcmd {
            McpCommands::Start => rtk_mcp::run_mcp_server(),
            McpCommands::Install { client } => rtk_mcp::install_mcp_client(&client),
            McpCommands::Ping => {
                let diagnostic = serde_json::json!({
                    "status": "ok",
                    "mcp_version": "2024-11-05",
                    "server_name": "rtk-mcp",
                    "version": env!("CARGO_PKG_VERSION"),
                    "tools_available": 8
                });
                match serde_json::to_string_pretty(&diagnostic) {
                    Ok(json_str) => println!("{}", json_str),
                    Err(e) => eprintln!("Failed to format JSON response: {e}"),
                }
                Ok(())
            }
            McpCommands::Call { tool, args } => {
                let parsed_args = if let Some(ref a_str) = args {
                    serde_json::from_str(a_str).unwrap_or(serde_json::Value::Null)
                } else {
                    serde_json::Value::Null
                };
                match rtk_mcp::execute_tool(&tool, parsed_args) {
                    Ok(res) => {
                        match serde_json::to_string_pretty(&res) {
                            Ok(json_str) => println!("{}", json_str),
                            Err(e) => eprintln!("Failed to format JSON response: {e}"),
                        }
                        Ok(())
                    }
                    Err(e) => Err(anyhow::anyhow!("Tool execution failed: {e}")),
                }
            }
        },
        Commands::Artifact { subcmd } => match subcmd {
            ArtifactCommands::List => artifact::list(),
            ArtifactCommands::Get { id } => artifact::get(&id),
            ArtifactCommands::Gc => artifact::gc(),
        },
        Commands::Symbols { subcmd } => match subcmd {
            SymbolsCommands::Find { query } => index_cli::symbols_find(&query),
        },
        Commands::Deps { subcmd } => match subcmd {
            DepsCommands::Show { file } => index_cli::deps_show(&file),
        },
        Commands::Refs { subcmd } => match subcmd {
            RefsCommands::Find { symbol } => index_cli::refs_find(&symbol),
        },
        Commands::Impact { subcmd } => match subcmd {
            ImpactCommands::Analyze { symbol } => index_cli::impact_analyze(&symbol),
        },
        Commands::Index { subcmd } => match subcmd {
            IndexCommands::Run => index_cli::index_run(),
            IndexCommands::Status { json } => index_cli::index_status(json),
        },
        Commands::Graph { subcmd } => match subcmd {
            GraphCommands::Export { format, output } => index_cli::graph_export(&format, &output),
        },
        Commands::Stats { chart } => tracking::print_stats_with_chart(chart),
        Commands::Gain { chart } => tracking::print_stats_with_chart(chart),
        Commands::Audit { subcmd, output } => match subcmd {
            Some(AuditCommands::Graph) => index_cli::audit_graph(),
            None => tracking::run_audit(&output),
        },
        Commands::Doctor => doctor::run_doctor(),
        Commands::Estimate => distiller::run_estimate(),
        Commands::SessionState { subcmd } => match subcmd {
            SessionStateCommands::Init => session::session_init().map(|_| {
                println!("✅ Session state initialized with default fields.");
            }),
            SessionStateCommands::Get => session::session_get().map(|json| {
                println!("{json}");
            }),
            SessionStateCommands::Update { key, value } => session::session_update(&key, &value).map(|_| {
                println!("✅ Updated session state key '{key}'.");
            }),
            SessionStateCommands::Export => session::session_export().map(|md| {
                println!("{md}");
            }),
        },
        Commands::Agents { subcmd } => match subcmd {
            AgentsCommands::Init { template } => agents::agents_init(&template),
            AgentsCommands::Doctor => agents::agents_doctor(),
            AgentsCommands::Compact => agents::agents_compact(),
        },
        Commands::Benchmark { subcmd } => match subcmd {
            BenchmarkCommands::Export { format, output } => {
                let res = if format.to_lowercase() == "csv" {
                    benchmark::export_csv(&output)
                } else {
                    benchmark::export_json(&output)
                };
                if let Err(e) = res {
                    eprintln!("Error exporting benchmark: {e}");
                    std::process::exit(1);
                }
                println!("Benchmark data exported to {} format successfully at: {}", format, output);
                Ok(())
            }
        },
        Commands::Dashboard { live, port } => dashboard::run_dashboard(live, port),
        Commands::Plugin { name, args } => {
            let plugins_cfg = plugins::load_plugins();
            if let Some(plugin) = plugins_cfg.plugins.into_iter().find(|p| p.name == name) {
                let bin = plugin.bin.clone();
                execute_with_filter(&bin, &args, FilterMode::PluginFilter(plugin))
            } else {
                Err(anyhow::anyhow!("Plugin '{}' not found in plugins.toml", name))
            }
        }
        Commands::SyncRules => sync_rules::run(Path::new(".")),
        Commands::ShowLog { id } => tracking::get_raw_log(id).map(|raw_log| {
            print!("{raw_log}");
        }),
        Commands::Filter { subcmd } => match subcmd {
            FilterCommands::Add { pattern, action } => {
                config::config_filter_add(&pattern, &action).map(|_| {
                    println!("✅ Added custom regex filter: pattern '{}', action '{}'", pattern, action);
                })
            }
        },
        Commands::Init { profile } => setup::run_init(&profile),
        Commands::Status => status::run_status(),
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
            ConfigCommands::Profile { name } => config::config_profile_set(&name).map(|_| {
                println!("⚙️  Default savings profile updated to: \"{}\"", name);
            }),
            ConfigCommands::Export => config::config_export(),
            ConfigCommands::Import { path } => {
                config::config_import(path.as_deref()).map(|_| {
                    println!("✅ Configuration successfully imported and updated.");
                })
            }
        },
        Commands::Context { subcmd } => {
            let res: Result<()> = (|| {
                match subcmd {
                    ContextCommands::Compact { max_tokens, profile } => {
                        use std::io::{self, Read};
                        let mut buffer = String::new();
                        io::stdin().read_to_string(&mut buffer)?;

                        let initial_tokens = buffer.split_whitespace().count();
                        if initial_tokens <= max_tokens {
                            print!("{buffer}");
                            return Ok(());
                        }

                        let compacted = if profile == "strict" {
                            buffer.lines()
                                .filter(|l| {
                                    let trimmed = l.trim();
                                    !trimmed.is_empty() && !trimmed.starts_with("//") && !trimmed.starts_with("#") && !trimmed.starts_with("/*") && !trimmed.starts_with("*")
                                })
                                .collect::<Vec<&str>>()
                                .join("\n")
                        } else if profile == "developer" {
                            buffer.lines()
                                .filter(|l| {
                                    let trimmed = l.trim().to_lowercase();
                                    trimmed.contains("error") || trimmed.contains("fail") || trimmed.contains("warn") || trimmed.contains("summary") || trimmed.contains("failed")
                                })
                                .collect::<Vec<&str>>()
                                .join("\n")
                        } else {
                            let mut lines = Vec::new();
                            let mut last_was_empty = false;
                            for line in buffer.lines() {
                                let is_empty = line.trim().is_empty();
                                if is_empty {
                                    if !last_was_empty {
                                        lines.push("");
                                        last_was_empty = true;
                                    }
                                } else {
                                    lines.push(line);
                                    last_was_empty = false;
                                }
                            }
                            lines.join("\n")
                        };

                        print!("{compacted}");
                        Ok(())
                    }
                }
            })();
            res
        },
        Commands::Telemetry { subcmd } => {
            let res: Result<()> = (|| {
                match subcmd {
                    TelemetryCommands::Export { format } => {
                        let records = rtk_db::tracking::get_all_telemetry()?;
                        if format.to_lowercase() == "prometheus" {
                            println!("# HELP rtk_total_commands Total number of commands run via RTK");
                            println!("# TYPE rtk_total_commands counter");
                            println!("rtk_total_commands {}", records.len());

                            let total_orig: i64 = records.iter().map(|r| r.original_tokens).sum();
                            let total_filt: i64 = records.iter().map(|r| r.filtered_tokens).sum();
                            let saved = total_orig.saturating_sub(total_filt);

                            println!("# HELP rtk_original_tokens_total Total original tokens parsed");
                            println!("# TYPE rtk_original_tokens_total counter");
                            println!("rtk_original_tokens_total {}", total_orig);

                            println!("# HELP rtk_filtered_tokens_total Total filtered tokens passed");
                            println!("# TYPE rtk_filtered_tokens_total counter");
                            println!("rtk_filtered_tokens_total {}", total_filt);

                            println!("# HELP rtk_saved_tokens_total Total tokens saved by filtering");
                            println!("# TYPE rtk_saved_tokens_total counter");
                            println!("rtk_saved_tokens_total {}", saved);
                        } else {
                            let json = serde_json::to_string_pretty(&records)?;
                            println!("{json}");
                        }
                        Ok(())
                    }
                }
            })();
            res
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
    PluginFilter(plugins::Plugin),
}

fn execute_with_filter(bin: &str, args: &[String], mode: FilterMode) -> Result<()> {
    let start = std::time::Instant::now();
    let output = std::process::Command::new(bin)
        .args(args)
        .output()
        .with_context(|| format!("failed to execute {bin}"))?;
    let duration_ms = start.elapsed().as_millis() as i64;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    let cmd_label = format!("{} {}", bin, args.first().map(|s| s.as_str()).unwrap_or(""));

    let (mut out_print, mut err_print, raw_db, filtered_db) = match mode {
        FilterMode::Stdout(filter) => {
            let filtered = filter(&stdout);
            let filtered = rtk_db::config::apply_regex_filters(&filtered);
            let r_filtered = dlp::redact_with_source(&filtered, &cmd_label);
            let r_stdout = dlp::redact_with_source(&stdout, &cmd_label);
            (
                r_filtered.clone(),
                dlp::redact_with_source(&stderr, &cmd_label),
                r_stdout.clone(),
                r_filtered,
            )
        }
        FilterMode::Stderr(filter) => {
            let filtered = filter(&stderr);
            let filtered = rtk_db::config::apply_regex_filters(&filtered);
            let r_filtered = dlp::redact_with_source(&filtered, &cmd_label);
            let r_stderr = dlp::redact_with_source(&stderr, &cmd_label);
            (
                dlp::redact_with_source(&stdout, &cmd_label),
                r_filtered.clone(),
                r_stderr.clone(),
                r_filtered,
            )
        }
        FilterMode::Combined(filter) => {
            let combined = format!("{stderr}\n{stdout}");
            let filtered = filter(&combined);
            let filtered = rtk_db::config::apply_regex_filters(&filtered);
            let r_filtered = dlp::redact_with_source(&filtered, &cmd_label);
            let r_combined = dlp::redact_with_source(&combined, &cmd_label);
            (
                r_filtered.clone(),
                String::new(),
                r_combined.clone(),
                r_filtered,
            )
        }
        FilterMode::Distilled => {
            let d_stdout = distiller::distill(&stdout, None);
            let d_stderr = distiller::distill(&stderr, None);
            let r_d_out = dlp::redact_with_source(&d_stdout, &cmd_label);
            let r_d_err = dlp::redact_with_source(&d_stderr, &cmd_label);
            let r_comb = dlp::redact_with_source(
                &format!("STDOUT:\n{stdout}\nSTDERR:\n{stderr}"),
                &cmd_label,
            );
            (
                r_d_out.clone(),
                r_d_err.clone(),
                r_comb,
                format!("{r_d_out}\n{r_d_err}"),
            )
        }
        FilterMode::PluginFilter(ref plugin) => {
            let capture_mode = plugin.filter_mode.as_deref().unwrap_or("stdout");
            match capture_mode {
                "stderr" => {
                    let filtered = plugins::filter_plugin(&stderr, plugin);
                    let filtered = rtk_db::config::apply_regex_filters(&filtered);
                    let r_filtered = dlp::redact_with_source(&filtered, &cmd_label);
                    let r_stderr = dlp::redact_with_source(&stderr, &cmd_label);
                    (
                        dlp::redact_with_source(&stdout, &cmd_label),
                        r_filtered.clone(),
                        r_stderr.clone(),
                        r_filtered,
                    )
                }
                "combined" => {
                    let combined = format!("{stderr}\n{stdout}");
                    let filtered = plugins::filter_plugin(&combined, plugin);
                    let filtered = rtk_db::config::apply_regex_filters(&filtered);
                    let r_filtered = dlp::redact_with_source(&filtered, &cmd_label);
                    let r_combined = dlp::redact_with_source(&combined, &cmd_label);
                    (
                        r_filtered.clone(),
                        String::new(),
                        r_combined.clone(),
                        r_filtered,
                    )
                }
                "distill" => {
                    let d_stdout = distiller::distill(&stdout, None);
                    let d_stderr = distiller::distill(&stderr, None);
                    let r_d_out = dlp::redact_with_source(&d_stdout, &cmd_label);
                    let r_d_err = dlp::redact_with_source(&d_stderr, &cmd_label);
                    let r_comb = dlp::redact_with_source(
                        &format!("STDOUT:\n{stdout}\nSTDERR:\n{stderr}"),
                        &cmd_label,
                    );
                    (
                        r_d_out.clone(),
                        r_d_err.clone(),
                        r_comb,
                        format!("{r_d_out}\n{r_d_err}"),
                    )
                }
                _ => {
                    // stdout default
                    let filtered = plugins::filter_plugin(&stdout, plugin);
                    let filtered = rtk_db::config::apply_regex_filters(&filtered);
                    let r_filtered = dlp::redact_with_source(&filtered, &cmd_label);
                    let r_stdout = dlp::redact_with_source(&stdout, &cmd_label);
                    (
                        r_filtered.clone(),
                        dlp::redact_with_source(&stderr, &cmd_label),
                        r_stdout.clone(),
                        r_filtered,
                    )
                }
            }
        }
    };

    match tracking::record(
        cmd_label.trim(),
        &raw_db,
        &filtered_db,
        &raw_db,
        Some(duration_ms),
    ) {
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

    if let Some(warning) = tracking::check_autonomy(&filtered_db) {
        if !out_print.trim().is_empty() {
            out_print.push_str(warning);
            out_print.push('\n');
        } else if !err_print.trim().is_empty() {
            err_print.push_str(warning);
            err_print.push('\n');
        }
    }

    if !out_print.is_empty() {
        print!("{out_print}");
    }
    if !err_print.is_empty() {
        eprint!("{err_print}");
    }

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
