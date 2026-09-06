use anyhow::{Context, Result};
use rtk_db::{config, dlp, tracking};
use std::io::IsTerminal;

use crate::{distiller, plugins};

pub enum FilterMode {
    Stdout(fn(&str) -> String),
    Stderr(fn(&str) -> String),
    Combined(fn(&str) -> String),
    Distilled,
    PluginFilter(plugins::Plugin),
}

/// Join stderr and stdout for `Combined` mode without inserting a spurious
/// blank line: a bare `format!("{stderr}\n{stdout}")` adds a separator even
/// when one side is empty or already newline-terminated, silently changing
/// the line count a downstream parser sees (the same #77 failure class,
/// found while sweeping every wrapped command for it in REL-2).
fn join_streams(stderr: &str, stdout: &str) -> String {
    if stderr.is_empty() {
        stdout.to_string()
    } else if stdout.is_empty() {
        stderr.to_string()
    } else if stderr.ends_with('\n') {
        format!("{stderr}{stdout}")
    } else {
        format!("{stderr}\n{stdout}")
    }
}

fn post_process_filter_output(text: &str, cmd: &str) -> String {
    let text = config::apply_regex_filters(text);
    let profile = config::get_config().get_profile_for_cmd(cmd);
    config::apply_profile_settings(&text, &profile)
}

/// Compress `raw` for display, unless the destination stream isn't an
/// interactive terminal (piped, redirected, or captured by a calling
/// process/agent). Every compression step here (per-command filters,
/// `post_process_filter_output`'s profile/regex rules, and `distiller`) can
/// drop or collapse lines to save tokens for a human/LLM reading a terminal
/// directly — but a downstream parser (`wc -l`, `grep -c`, JSON parsing, or
/// another command substituting this output into its own argv) has no way to
/// know that, and gets a plausible-but-wrong answer instead of an error
/// (#77). Once nothing is watching the terminal, correctness has to win over
/// compression, so we return `raw` untouched (still DLP-redacted).
///
/// Returns `(displayed, raw_redacted)`.
fn compress_for_display(
    raw: &str,
    cmd_label: &str,
    is_tty: bool,
    compress: impl FnOnce(&str) -> String,
) -> (String, String) {
    if !is_tty {
        // User-configured secret-stripping rules (`rtk config filter add`) are
        // a safety control, like DLP redaction below — not a token-savings
        // heuristic — so they must still run even though compression itself
        // is skipped here. Only `apply_profile_settings` (line caps, comment
        // stripping, json_only) and the per-command `compress` closure are
        // skipped: those are the human-readability steps that can silently
        // change a piped line count (#77).
        let safety_filtered = config::apply_regex_filters(raw);
        let raw_redacted = dlp::redact_with_source(&safety_filtered, cmd_label);
        return (raw_redacted.clone(), raw_redacted);
    }
    let raw_redacted = dlp::redact_with_source(raw, cmd_label);
    let compressed_redacted = dlp::redact_with_source(&compress(raw), cmd_label);
    (compressed_redacted, raw_redacted)
}

/// Append `marker` to whichever of `out_print`/`err_print` is the primary
/// non-empty stream — but only if *that* stream's destination is a real
/// terminal. Centralizes the tty guard so a future informational marker
/// can't be added without it (see the module doc for why the guard exists).
fn append_marker_if_tty(
    out_print: &mut String,
    err_print: &mut String,
    stdout_is_tty: bool,
    stderr_is_tty: bool,
    marker: &str,
) {
    if !out_print.trim().is_empty() && stdout_is_tty {
        out_print.push_str(marker);
    } else if !err_print.trim().is_empty() && stderr_is_tty {
        err_print.push_str(marker);
    }
}

pub fn execute_with_filter(bin: &str, args: &[String], mode: FilterMode) -> Result<()> {
    let start = std::time::Instant::now();
    let output = std::process::Command::new(bin)
        .args(args)
        .output()
        .with_context(|| format!("failed to execute {bin}"))?;
    let duration_ms = start.elapsed().as_millis() as i64;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    let cmd_label = if args.is_empty() {
        bin.to_string()
    } else {
        format!("{bin} {}", args.join(" "))
    };

    let stdout_is_tty = std::io::stdout().is_terminal();
    let stderr_is_tty = std::io::stderr().is_terminal();

    let (mut out_print, mut err_print, raw_db, filtered_db) = match mode {
        FilterMode::Stdout(filter) => {
            let (displayed, r_stdout) =
                compress_for_display(&stdout, &cmd_label, stdout_is_tty, |s| {
                    post_process_filter_output(&filter(s), &cmd_label)
                });
            (
                displayed.clone(),
                dlp::redact_with_source(&stderr, &cmd_label),
                r_stdout,
                displayed,
            )
        }
        FilterMode::Stderr(filter) => {
            let (displayed, r_stderr) =
                compress_for_display(&stderr, &cmd_label, stderr_is_tty, |s| {
                    post_process_filter_output(&filter(s), &cmd_label)
                });
            (
                dlp::redact_with_source(&stdout, &cmd_label),
                displayed.clone(),
                r_stderr,
                displayed,
            )
        }
        FilterMode::Combined(filter) => {
            let combined = join_streams(&stderr, &stdout);
            let (displayed, r_combined) =
                compress_for_display(&combined, &cmd_label, stdout_is_tty, |s| {
                    post_process_filter_output(&filter(s), &cmd_label)
                });
            (displayed.clone(), String::new(), r_combined, displayed)
        }
        FilterMode::Distilled => {
            let r_comb = dlp::redact_with_source(
                &format!("STDOUT:\n{stdout}\nSTDERR:\n{stderr}"),
                &cmd_label,
            );
            let (out, _) = compress_for_display(&stdout, &cmd_label, stdout_is_tty, |s| {
                distiller::distill(s, None)
            });
            let (err, _) = compress_for_display(&stderr, &cmd_label, stderr_is_tty, |s| {
                distiller::distill(s, None)
            });
            (out.clone(), err.clone(), r_comb, format!("{out}\n{err}"))
        }
        FilterMode::PluginFilter(ref plugin) => {
            let capture_mode = plugin.filter_mode.as_deref().unwrap_or("stdout");
            match capture_mode {
                "stderr" => {
                    let (displayed, r_stderr) =
                        compress_for_display(&stderr, &cmd_label, stderr_is_tty, |s| {
                            post_process_filter_output(
                                &plugins::filter_plugin(s, plugin),
                                &cmd_label,
                            )
                        });
                    (
                        dlp::redact_with_source(&stdout, &cmd_label),
                        displayed.clone(),
                        r_stderr,
                        displayed,
                    )
                }
                "combined" => {
                    let combined = join_streams(&stderr, &stdout);
                    let (displayed, r_combined) =
                        compress_for_display(&combined, &cmd_label, stdout_is_tty, |s| {
                            post_process_filter_output(
                                &plugins::filter_plugin(s, plugin),
                                &cmd_label,
                            )
                        });
                    (displayed.clone(), String::new(), r_combined, displayed)
                }
                "distill" => {
                    let r_comb = dlp::redact_with_source(
                        &format!("STDOUT:\n{stdout}\nSTDERR:\n{stderr}"),
                        &cmd_label,
                    );
                    let (out, _) = compress_for_display(&stdout, &cmd_label, stdout_is_tty, |s| {
                        distiller::distill(s, None)
                    });
                    let (err, _) = compress_for_display(&stderr, &cmd_label, stderr_is_tty, |s| {
                        distiller::distill(s, None)
                    });
                    (out.clone(), err.clone(), r_comb, format!("{out}\n{err}"))
                }
                _ => {
                    let (displayed, r_stdout) =
                        compress_for_display(&stdout, &cmd_label, stdout_is_tty, |s| {
                            post_process_filter_output(
                                &plugins::filter_plugin(s, plugin),
                                &cmd_label,
                            )
                        });
                    (
                        displayed.clone(),
                        dlp::redact_with_source(&stderr, &cmd_label),
                        r_stdout,
                        displayed,
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
            // These informational markers are only meaningful for a human at
            // an interactive terminal. Appending them to a piped/captured
            // stream corrupts anything treating that stream as structured
            // data (the same #77 failure mode) — and if that captured text is
            // later reused verbatim as an argument to another rtk-wrapped
            // command (e.g. via shell `$(...)` substitution), the marker text
            // itself ends up embedded in the *next* command's argv, which is
            // the cmd-corruption anomaly tracked under REL-3.
            if filtered_db.len() < raw_db.len() && !filtered_db.trim().is_empty() {
                let msg = format!("\n[Full output cached. Access with: rtk show-log {log_id}]\n");
                append_marker_if_tty(
                    &mut out_print,
                    &mut err_print,
                    stdout_is_tty,
                    stderr_is_tty,
                    &msg,
                );
            }
        }
        Err(e) => eprintln!("rtk: tracking warning: {e}"),
    }

    if let Some(warning) = tracking::check_autonomy(&filtered_db) {
        let msg = format!("{warning}\n");
        append_marker_if_tty(
            &mut out_print,
            &mut err_print,
            stdout_is_tty,
            stderr_is_tty,
            &msg,
        );
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

pub fn run_filtered(bin: &str, args: &[String], filter: fn(&str) -> String) -> Result<()> {
    execute_with_filter(bin, args, FilterMode::Stdout(filter))
}

pub fn run_filtered_stderr(bin: &str, args: &[String], filter: fn(&str) -> String) -> Result<()> {
    execute_with_filter(bin, args, FilterMode::Stderr(filter))
}

pub fn run_filtered_combined(bin: &str, args: &[String], filter: fn(&str) -> String) -> Result<()> {
    execute_with_filter(bin, args, FilterMode::Combined(filter))
}

pub fn run_distilled(bin: &str, args: &[String]) -> Result<()> {
    execute_with_filter(bin, args, FilterMode::Distilled)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn join_streams_no_spurious_blank_line() {
        // stderr already newline-terminated: must not gain an extra blank
        // line at the seam (found sweeping `npm install`/`cargo test` for #77).
        assert_eq!(join_streams("warn\n", "\nok\n"), "warn\n\nok\n");
        assert_eq!(join_streams("warn", "ok\n"), "warn\nok\n");
        assert_eq!(join_streams("", "ok\n"), "ok\n");
        assert_eq!(join_streams("warn\n", ""), "warn\n");
    }
}
