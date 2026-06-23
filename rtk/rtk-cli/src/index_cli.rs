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

    println!(
        "Graph export starting... format: {}, output: {}",
        format, output
    );
    let count = rtk_index::export_obsidian_graph(Path::new(output))?;
    println!(
        "✅ Obsidian graph exported successfully ({} symbol markdown files created in '{}')",
        count, output
    );
    Ok(())
}

pub fn audit_graph() -> Result<()> {
    let metrics = rtk_index::get_graph_metrics()?;
    println!("📊 RTK Code Intelligence Graph Audit Report");
    println!("==========================================");
    println!("Total Symbols:      {}", metrics.symbols_count);
    println!("Total Edges/Calls:  {}", metrics.edges_count);
    println!("Graph Coverage:     {:.2}%", metrics.graph_coverage);
    println!("Query Latency:      {:.4} ms", metrics.query_latency_ms);
    println!("==========================================");
    Ok(())
}
