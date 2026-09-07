use anyhow::{Context, Result};
use rtk_db::{config, dlp, tracking};

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

/// RTK exists to save an AI agent tokens: an agent reading a compressed
/// summary directly (e.g. "...and 19 more entries...") handles it fine — it's
/// prose, not data it's blindly counting. Compression only becomes unsafe
/// once that summary is *composed*: piped into `wc -l`/`grep -c`/a parser, or
/// reused as another command's argument. RTK can't reliably tell those two
/// situations apart from the OS side (an agent capturing our output and a
/// shell pipe both look like "not a terminal"), so instead of guessing we
/// keep compression on by default and let the caller ask for the untruncated
/// original with `RTK_RAW=1` for the specific invocation whose output will be
/// composed with something else (#77).
fn raw_requested() -> bool {
    std::env::var_os("RTK_RAW").is_some()
}

/// Compress `raw` for display, unless `RTK_RAW=1` asked for the untruncated
/// original (see `raw_requested`). Returns `(displayed, raw_redacted)`.
fn compress_for_display(
    raw: &str,
    cmd_label: &str,
    raw_requested: bool,
    compress: impl FnOnce(&str) -> String,
) -> (String, String) {
    if raw_requested {
        // User-configured secret-stripping rules (`rtk filter add`) are a
        // safety control, like DLP redaction below — not a token-savings
        // heuristic — so they still run even with compression skipped. Only
        // `apply_profile_settings` (line caps, comment stripping, json_only)
        // and the per-command `compress` closure are skipped: those are the
        // human-readability steps that can silently change a line count.
        let safety_filtered = config::apply_regex_filters(raw);
        let raw_redacted = dlp::redact_with_source(&safety_filtered, cmd_label);
        return (raw_redacted.clone(), raw_redacted);
    }
    let raw_redacted = dlp::redact_with_source(raw, cmd_label);
    let compressed_redacted = dlp::redact_with_source(&compress(raw), cmd_label);
    (compressed_redacted, raw_redacted)
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

    let raw = raw_requested();

    let (out_print, err_print, raw_db, filtered_db) = match mode {
        FilterMode::Stdout(filter) => {
            let (displayed, r_stdout) = compress_for_display(&stdout, &cmd_label, raw, |s| {
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
            let (displayed, r_stderr) = compress_for_display(&stderr, &cmd_label, raw, |s| {
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
            let (displayed, r_combined) = compress_for_display(&combined, &cmd_label, raw, |s| {
                post_process_filter_output(&filter(s), &cmd_label)
            });
            (displayed.clone(), String::new(), r_combined, displayed)
        }
        FilterMode::Distilled => {
            let r_comb = dlp::redact_with_source(
                &format!("STDOUT:\n{stdout}\nSTDERR:\n{stderr}"),
                &cmd_label,
            );
            let (out, _) =
                compress_for_display(&stdout, &cmd_label, raw, |s| distiller::distill(s, None));
            let (err, _) =
                compress_for_display(&stderr, &cmd_label, raw, |s| distiller::distill(s, None));
            (out.clone(), err.clone(), r_comb, format!("{out}\n{err}"))
        }
        FilterMode::PluginFilter(ref plugin) => {
            let capture_mode = plugin.filter_mode.as_deref().unwrap_or("stdout");
            match capture_mode {
                "stderr" => {
                    let (displayed, r_stderr) =
                        compress_for_display(&stderr, &cmd_label, raw, |s| {
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
                        compress_for_display(&combined, &cmd_label, raw, |s| {
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
                    let (out, _) = compress_for_display(&stdout, &cmd_label, raw, |s| {
                        distiller::distill(s, None)
                    });
                    let (err, _) = compress_for_display(&stderr, &cmd_label, raw, |s| {
                        distiller::distill(s, None)
                    });
                    (out.clone(), err.clone(), r_comb, format!("{out}\n{err}"))
                }
                _ => {
                    let (displayed, r_stdout) =
                        compress_for_display(&stdout, &cmd_label, raw, |s| {
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

    let log_id = match tracking::record(
        cmd_label.trim(),
        &raw_db,
        &filtered_db,
        &raw_db,
        Some(duration_ms),
    ) {
        Ok(id) => Some(id),
        Err(e) => {
            eprintln!("rtk: tracking warning: {e}");
            None
        }
    };

    if !out_print.is_empty() {
        print!("{out_print}");
    }
    if !err_print.is_empty() {
        eprint!("{err_print}");
    }

    // Informational markers always go to stderr, never mixed into out_print
    // (stdout). Shell command substitution (`$(...)`) only captures stdout,
    // so a marker here can never end up embedded verbatim in a later
    // command's argv if this output is captured and reused — the REL-3
    // cmd-corruption vector — regardless of RTK_RAW. They stay visible: to a
    // human at a terminal, and to an agent harness that reads both streams.
    if let Some(log_id) = log_id {
        if filtered_db.len() < raw_db.len() && !filtered_db.trim().is_empty() {
            eprint!("\n[Full output cached. Access with: rtk show-log {log_id}]\n");
        }
    }
    if let Some(warning) = tracking::check_autonomy(&filtered_db) {
        eprintln!("{warning}");
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
