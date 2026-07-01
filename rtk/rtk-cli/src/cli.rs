use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "rtk",
    version,
    about = "Token-efficient CLI wrapper for Claude Code"
)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Commands,
}

#[derive(Subcommand)]
pub(crate) enum Commands {
    /// Rewrite a raw command to its RTK equivalent.
    /// Exit codes: 0=rewrite found, 1=no match, 2=deny, 3=ask
    Rewrite {
        /// The raw command string to rewrite
        command: String,
    },
    /// Run a git subcommand with filtered output
    Git {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Run a cargo subcommand with filtered output
    Cargo {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Run an npm subcommand with filtered output
    Npm {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Run a yarn subcommand with filtered output
    Yarn {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Run a pnpm subcommand with filtered output
    Pnpm {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Run a composer subcommand with filtered output
    Composer {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Run a terraform subcommand with filtered output
    Terraform {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Run a dotnet subcommand with filtered output
    Dotnet {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Store LLM Chain-of-Thought in the semantic memory instead of polluting the chat window
    Think {
        /// Optional text to store (if not provided, reads from stdin)
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        content: Vec<String>,
    },
    /// Run a pytest invocation with filtered output
    Pytest {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Run a ruff invocation with filtered output
    Ruff {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Run a mypy invocation with filtered output
    Mypy {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Run a pip invocation with filtered output
    Pip {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Run an eslint invocation with filtered output
    Eslint {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Run ls with filtered output
    Ls {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Run a gradle command with filtered output
    Gradle {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Run go test with filtered output
    GoTest {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Run a docker command with filtered output
    Docker {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
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
    /// Show which indexed symbols your uncommitted changes touch, with blast radius
    DetectChanges,
    /// Rename a symbol across linked files (AST-aware). Dry-run unless --apply.
    Rename {
        /// Current symbol name
        old_name: String,
        /// New symbol name
        new_name: String,
        /// Write the changes to disk (default: preview only)
        #[arg(long)]
        apply: bool,
    },
    /// Trace the downstream execution flow (call tree) from a symbol
    Flow {
        /// Entry symbol name
        symbol: String,
        /// Maximum call depth to trace
        #[arg(long, default_value_t = 6)]
        depth: usize,
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
    /// Run fmt + clippy + tests (dev-gate.sh or cargo workspace gate)
    Validate,
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
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
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
pub(crate) enum FilterCommands {
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
pub(crate) enum ConfigCommands {
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
pub(crate) enum ConfigDenyCommands {
    /// Add a pattern to the list of denied command guards
    Add {
        /// The command substring or regex pattern to guard against
        pattern: String,
    },
}

#[derive(Subcommand)]
pub(crate) enum ConfigDlpCommands {
    /// Add a regex pattern to the list of custom DLP secret redactors
    Add {
        /// The regex pattern for custom secret detection
        pattern: String,
    },
}

#[derive(Subcommand)]
pub(crate) enum MemoryCommands {
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
pub(crate) enum BudgetCommands {
    /// Check budget status and total cost spent
    Check {
        /// Optional budget limit in USD
        #[arg(long)]
        limit: Option<f64>,
    },
}

#[derive(Subcommand, Debug)]
pub(crate) enum ModelCommands {
    /// Suggest model for a task type
    Suggest {
        /// The task type (e.g. simple, single-file-edit, complex, planning, audit)
        #[arg(long)]
        task: String,
    },
}

#[derive(Subcommand, Debug)]
pub(crate) enum GraphCommands {
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
pub(crate) enum AuditCommands {
    /// Run graph audit (symbols count, edges count, query latency, graph coverage %)
    Graph,
}

#[derive(Subcommand, Debug)]
pub(crate) enum McpCommands {
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
pub(crate) enum ArtifactCommands {
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
pub(crate) enum SymbolsCommands {
    /// Find symbols by name
    Find {
        /// Name query to search
        query: String,
    },
}

#[derive(Subcommand, Debug)]
pub(crate) enum DepsCommands {
    /// Show dependencies of a file
    Show {
        /// File path to analyze
        file: String,
    },
}

#[derive(Subcommand, Debug)]
pub(crate) enum RefsCommands {
    /// Find references to a symbol
    Find {
        /// Symbol name to search
        symbol: String,
    },
}

#[derive(Subcommand, Debug)]
pub(crate) enum ImpactCommands {
    /// Analyze upstream blast radius
    Analyze {
        /// Symbol name to analyze
        symbol: String,
    },
}

#[derive(Subcommand, Debug)]
pub(crate) enum IndexCommands {
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
pub(crate) enum BenchmarkCommands {
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
pub(crate) enum SessionStateCommands {
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
pub(crate) enum AgentsCommands {
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
pub(crate) enum ContextCommands {
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
pub(crate) enum TelemetryCommands {
    /// Export recorded telemetry to external dashboards/formats
    Export {
        /// Output format: json or prometheus
        #[arg(short, long, default_value = "json")]
        format: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    /// Wrapped commands must forward leading `--flags` into `args` instead of
    /// letting clap reject them — otherwise the rewrite hook turns a valid
    /// `mypy --strict` into a broken `rtk mypy --strict`.
    #[test]
    fn leading_flags_pass_through() {
        let cli = Cli::try_parse_from(["rtk", "pytest", "--tb=short", "-v", "tests/"])
            .expect("leading flags should parse");
        match cli.command {
            Commands::Pytest { args } => {
                assert_eq!(args, vec!["--tb=short", "-v", "tests/"]);
            }
            _ => panic!("expected Pytest"),
        }

        let cli =
            Cli::try_parse_from(["rtk", "cargo", "--version"]).expect("leading flags should parse");
        match cli.command {
            Commands::Cargo { args } => assert_eq!(args, vec!["--version"]),
            _ => panic!("expected Cargo"),
        }
    }
}
