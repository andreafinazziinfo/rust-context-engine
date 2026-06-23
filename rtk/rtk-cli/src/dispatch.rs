use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

use rtk_db::{config, session, status, think, tracking};
use rtk_filters::{
    cargo_build, cargo_test, docker_filter, git_branch, git_diff, git_log, git_show, git_status,
    go_test, gradle, ls_filter, npm_filter, pytest_filter,
};
use rtk_pack::pack;

use crate::cli::{
    AgentsCommands, ArtifactCommands, AuditCommands, BenchmarkCommands, BudgetCommands, Commands,
    ConfigCommands, ConfigDenyCommands, ConfigDlpCommands, ContextCommands, DepsCommands,
    FilterCommands, GraphCommands, ImpactCommands, IndexCommands, McpCommands, MemoryCommands,
    ModelCommands, RefsCommands, SessionStateCommands, SymbolsCommands, TelemetryCommands,
};
use crate::filter_pipeline::{
    execute_with_filter, run_distilled, run_filtered, run_filtered_combined, run_filtered_stderr,
    FilterMode,
};
use crate::{
    agents, artifact, benchmark, dashboard, distiller, doctor, dotnet, index_cli, plugins, rewrite,
    setup, sync_rules,
};

pub fn dispatch(command: Commands) -> Result<()> {
    match command {
        Commands::Rewrite { command } => rewrite::run(&command),
        Commands::Git { args } => {
            let subcmd = args.first().map(|s| s.as_str()).unwrap_or("");
            match subcmd {
                "diff" => run_filtered("git", &args, git_diff::filter),
                "status" if !has_flag(&args, &["--porcelain", "--short", "-s"]) => {
                    run_filtered("git", &args, git_status::filter)
                }
                "log" => run_filtered("git", &args, git_log::filter),
                "show" => run_filtered("git", &args, git_show::filter),
                "branch" if has_flag(&args, &["-v", "-vv", "--verbose"]) => {
                    run_filtered("git", &args, git_branch::filter)
                }
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
        Commands::Npm { args } => run_filtered_combined("npm", &args, npm_filter::filter),
        Commands::Yarn { args } => run_filtered_combined("yarn", &args, npm_filter::filter_yarn),
        Commands::Pnpm { args } => run_filtered_combined("pnpm", &args, npm_filter::filter),
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
                let tokens = rtk_db::tracking::count_tokens(&packed) as usize;
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
        Commands::Doctor => {
            match doctor::run_doctor() {
                doctor::DoctorOutcome::Ok => Ok(()),
                doctor::DoctorOutcome::Warnings => std::process::exit(2),
                doctor::DoctorOutcome::Critical => std::process::exit(1),
            }
        }
        Commands::Validate => crate::validate::run_validate(),
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
