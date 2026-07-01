use anyhow::Result;
use std::path::Path;

pub fn symbols_find(query: &str) -> Result<()> {
    let list = rtk_index::query_symbols(query)?;
    if list.is_empty() {
        println!("No symbols found matching: '{}'", query);
        return Ok(());
    }

    println!(
        "{:<12} | {:<40} | {:<10} | Name",
        "Kind", "File Path", "Lines"
    );
    println!("{}", "-".repeat(90));
    for sym in list {
        let lines = format!("{}-{}", sym.line_start, sym.line_end);
        println!(
            "{:<12} | {:<40} | {:<10} | {}",
            sym.kind, sym.file_path, lines, sym.name
        );
    }
    Ok(())
}

pub fn deps_show(file: &str) -> Result<()> {
    let list = rtk_index::query_dependencies(file)?;
    if list.is_empty() {
        println!("No symbol dependencies tracked for file: '{}'", file);
        return Ok(());
    }

    println!("Dependencies for file: {}", file);
    println!("{}", "=".repeat(60));
    for (sym, callees) in list {
        if callees.is_empty() {
            println!("{} ({}) calls: None", sym.name, sym.kind);
        } else {
            println!("{} ({}) calls: {}", sym.name, sym.kind, callees.join(", "));
        }
    }
    Ok(())
}

pub fn refs_find(symbol: &str) -> Result<()> {
    let list = rtk_index::query_references(symbol)?;
    if list.is_empty() {
        println!("No references found calling symbol name: '{}'", symbol);
        return Ok(());
    }

    println!("References calling: {}", symbol);
    println!("{}", "-".repeat(60));
    for sym in list {
        println!(
            "- {} ({}) in {}:{}",
            sym.name, sym.kind, sym.file_path, sym.line_start
        );
    }
    Ok(())
}

pub fn impact_analyze(symbol: &str) -> Result<()> {
    let list = rtk_index::analyze_impact(symbol)?;
    if list.is_empty() {
        println!(
            "No upstream blast radius found for: '{}' (or symbol not found)",
            symbol
        );
        return Ok(());
    }

    let risk = if list.len() > 10 {
        "HIGH"
    } else if list.len() > 3 {
        "MEDIUM"
    } else {
        "LOW"
    };

    println!("Blast Radius Impact Analysis for: {}", symbol);
    println!("Risk Level: {}", risk);
    println!("Affected transitively upstream ({} symbols):", list.len());
    println!("{}", "-".repeat(60));
    for sym in list {
        println!(
            "- {} ({}) in {}:{}",
            sym.name, sym.kind, sym.file_path, sym.line_start
        );
    }
    Ok(())
}

pub fn flow(entry: &str, depth: usize) -> Result<()> {
    const MAX_NODES: usize = 200;
    let trace = rtk_index::trace_flow(entry, depth, MAX_NODES)?;
    let Some(trace) = trace else {
        println!("Symbol not indexed: '{}'", entry);
        return Ok(());
    };

    println!(
        "Flow: {} ({}:{})",
        trace.root.name, trace.root.file_path, trace.root.line_start
    );
    print_flow_children(&trace.root.children, "");
    let mut notes = format!(
        "[{} node(s) · max depth {}",
        trace.node_count, trace.max_depth_reached
    );
    if trace.revisits > 0 {
        notes.push_str(&format!(" · {} shared/cyclic ref(s)", trace.revisits));
    }
    if trace.ambiguous_hidden > 0 {
        notes.push_str(&format!(
            " · {} ambiguous callee(s) hidden",
            trace.ambiguous_hidden
        ));
    }
    if trace.capped {
        notes.push_str(" · node cap hit");
    }
    notes.push(']');
    println!("{}", notes);
    Ok(())
}

fn print_flow_children(children: &[rtk_index::graph::FlowNode], prefix: &str) {
    let last = children.len().saturating_sub(1);
    for (i, child) in children.iter().enumerate() {
        let is_last = i == last;
        let branch = if is_last { "└─ " } else { "├─ " };
        let mut label = format!(
            "{}{}{} ({}:{})",
            prefix, branch, child.name, child.file_path, child.line_start
        );
        if child.truncated && child.children.is_empty() {
            label.push_str(" …");
        }
        println!("{}", label);
        let child_prefix = format!("{}{}", prefix, if is_last { "   " } else { "│  " });
        print_flow_children(&child.children, &child_prefix);
    }
}

pub fn rename(old_name: &str, new_name: &str, apply: bool) -> Result<()> {
    let plan = rtk_index::rename_symbol(old_name, new_name, apply)?;
    if plan.total_sites == 0 {
        println!(
            "No identifier occurrences of '{}' found in indexed files.",
            old_name
        );
        return Ok(());
    }

    let verb = if plan.applied {
        "Renamed"
    } else {
        "Would rename"
    };
    println!(
        "{} '{}' → '{}' — {} occurrence(s) across {} file(s):",
        verb,
        plan.old_name,
        plan.new_name,
        plan.total_sites,
        plan.files.len()
    );
    println!("{}", "-".repeat(60));
    for f in &plan.files {
        println!("- {} ({} occurrence(s))", f.file_path, f.sites);
    }
    if !plan.applied {
        println!("\nDry run — re-run with --apply to write these changes.");
    }
    Ok(())
}

pub fn detect_changes() -> Result<()> {
    let changed = rtk_index::detect_changes()?;
    if changed.is_empty() {
        println!("No indexed symbols touched by the current changes (working tree vs HEAD).");
        return Ok(());
    }

    let highest = if changed.iter().any(|c| c.risk == "HIGH") {
        "HIGH"
    } else if changed.iter().any(|c| c.risk == "MEDIUM") {
        "MEDIUM"
    } else {
        "LOW"
    };

    println!(
        "Detected changes touching {} symbol(s) — highest risk: {}",
        changed.len(),
        highest
    );
    println!("{}", "-".repeat(60));
    for c in &changed {
        println!(
            "- {} ({}) in {}:{}-{} → risk {} ({} affected upstream)",
            c.name, c.kind, c.file_path, c.line_start, c.line_end, c.risk, c.impact_count
        );
    }
    Ok(())
}

pub fn index_run() -> Result<()> {
    println!("🔍 Indexing codebase AST...");
    let count = rtk_index::index_project(Path::new("."))?;
    println!("✅ Indexed {} symbols successfully.", count);
    Ok(())
}

pub fn index_status(json: bool) -> Result<()> {
    let status = rtk_index::get_index_status()?;
    if json {
        println!("{}", serde_json::to_string_pretty(&status)?);
        return Ok(());
    }
    println!("RTK Index Status");
    println!("================");
    println!("Symbols:        {}", status.symbols_count);
    println!("Edges:          {}", status.edges_count);
    println!(
        "Last indexed:   {}",
        status
            .last_indexed
            .map(|ts| ts.to_string())
            .unwrap_or_else(|| "never".into())
    );
    println!("Graph coverage: {:.2}%", status.graph_coverage);
    println!(
        "Stale:          {}",
        if status.stale { "yes" } else { "no" }
    );
    Ok(())
}

pub fn graph_export(format: &str, output: &str) -> Result<()> {
    if format.to_lowercase() != "obsidian" {
        return Err(anyhow::anyhow!(
            "Unsupported format: '{}'. Currently supported formats: obsidian",
            format
        ));
    }

    let out = Path::new(output);
    println!(
        "Graph export starting... format: {}, output: {}",
        format, output
    );
    let count = rtk_index::export_obsidian_graph(out)?;
    let status = rtk_index::get_index_status()?;
    let index_md = out.join("index.md");
    let summary = format!(
        "# RTK Code Graph\n\n\
         - Symbols: {}\n\
         - Edges: {}\n\
         - Graph coverage: {:.1}%\n\
         - Notes exported: {}\n\
         - Index stale: {}\n\n\
         Open any `Symbol (file).md` note for backlinks.\n",
        status.symbols_count,
        status.edges_count,
        status.graph_coverage,
        count,
        if status.stale {
            "yes — run `rtk index run`"
        } else {
            "no"
        }
    );
    std::fs::write(&index_md, summary)?;
    println!(
        "✅ Obsidian graph exported successfully ({} symbol markdown files + index.md in '{}')",
        count, output
    );
    Ok(())
}

pub fn audit_graph() -> Result<()> {
    let status = rtk_index::get_index_status()?;
    let metrics = rtk_index::get_graph_metrics()?;
    println!("📊 RTK Code Intelligence Graph Audit Report");
    println!("==========================================");
    println!("Total Symbols:      {}", metrics.symbols_count);
    println!("Total Edges/Calls:  {}", metrics.edges_count);
    println!("Graph Coverage:     {:.2}%", metrics.graph_coverage);
    println!("Query Latency:      {:.4} ms", metrics.query_latency_ms);
    println!(
        "Last Indexed:       {}",
        status
            .last_indexed
            .map(|ts| ts.to_string())
            .unwrap_or_else(|| "never".into())
    );
    println!(
        "Index Stale:        {}",
        if status.stale { "yes" } else { "no" }
    );
    println!("==========================================");
    if metrics.symbols_count == 0 {
        println!("💡 Empty graph — run: rtk index run");
    } else if status.stale {
        println!("💡 Stale index — run: rtk index run");
    }
    if metrics.graph_coverage < 50.0 && metrics.symbols_count > 0 {
        println!(
            "💡 Low coverage ({:.0}%) — many orphan symbols",
            metrics.graph_coverage
        );
    }
    println!("💡 Export: rtk graph export --output ./graph-notes");
    Ok(())
}
