use anyhow::Result;
use std::path::Path;

pub mod db;
pub mod embeddings;
pub mod graph;
pub mod parser;

const STALE_SECS: i64 = 86400;

/// Re-index project when empty or older than 24h.
pub fn ensure_index_fresh(project_dir: &Path) -> Result<()> {
    let conn = db::open_db()?;
    let symbol_count: i64 = conn.query_row("SELECT COUNT(*) FROM symbols", [], |r| r.get(0))?;
    if symbol_count == 0 {
        let _ = index_project(project_dir)?;
        return Ok(());
    }
    let now: i64 = conn.query_row("SELECT CAST(strftime('%s','now') AS INTEGER)", [], |r| {
        r.get(0)
    })?;
    let max_indexed: i64 = conn.query_row(
        "SELECT COALESCE(MAX(last_indexed), 0) FROM file_hashes",
        [],
        |r| r.get(0),
    )?;
    if max_indexed == 0 || now - max_indexed > STALE_SECS {
        let _ = index_project(project_dir)?;
    }
    Ok(())
}

fn ensure_fresh_cwd() -> Result<()> {
    ensure_index_fresh(&std::env::current_dir()?)
}

pub fn index_project(project_dir: &Path) -> Result<usize> {
    let files = parser::scan_directory(project_dir)?;
    let conn = db::open_db()?;

    // Load cached file hashes
    let cached_hashes = db::get_file_hashes(&conn)?;

    let mut scanned_rel_paths = std::collections::HashSet::new();

    for file in &files {
        let rel_path = file
            .strip_prefix(project_dir)
            .unwrap_or(file)
            .to_string_lossy()
            .to_string()
            .replace('\\', "/");

        scanned_rel_paths.insert(rel_path.clone());

        // Read file and compute hash
        if let Ok(code) = std::fs::read_to_string(file) {
            use std::hash::{Hash, Hasher};
            let mut s = std::collections::hash_map::DefaultHasher::new();
            code.hash(&mut s);
            let current_hash = format!("{:x}", s.finish());

            let mut needs_indexing = true;
            if let Some(cached_hash) = cached_hashes.get(&rel_path) {
                if cached_hash == &current_hash {
                    needs_indexing = false;
                }
            }

            if needs_indexing {
                // Clear old symbols for this file
                db::clear_file_index(&conn, &rel_path)?;

                // Parse and insert new symbols
                if let Ok(syms) = parser::parse_file(file, project_dir) {
                    db::insert_symbols(&conn, &syms)?;
                }

                // Update hash in database
                db::insert_file_hash(&conn, &rel_path, &current_hash)?;
            }
        }
    }

    // Clean up deleted files from database
    for (cached_path, _) in cached_hashes {
        if !scanned_rel_paths.contains(&cached_path) {
            let _ = db::clear_file_index(&conn, &cached_path);
            let _ = db::delete_file_hash(&conn, &cached_path);
        }
    }

    // Return total number of symbols in database
    let all_symbols = db::get_all_symbols(&conn)?;
    Ok(all_symbols.len())
}

pub fn query_symbols(name_query: &str) -> Result<Vec<db::DbSymbol>> {
    ensure_fresh_cwd()?;
    let conn = db::open_db()?;
    db::find_symbols(&conn, name_query)
}

pub fn query_dependencies(file_path: &str) -> Result<Vec<(db::DbSymbol, Vec<String>)>> {
    ensure_fresh_cwd()?;
    let conn = db::open_db()?;
    let all_syms = db::get_all_symbols(&conn)?;
    let all_deps = db::get_all_dependencies(&conn)?;

    let mut file_symbols = Vec::new();
    for sym in all_syms {
        if sym.file_path == file_path {
            let mut callees = Vec::new();
            for dep in &all_deps {
                if dep.caller_id == sym.id {
                    callees.push(dep.callee_name.clone());
                }
            }
            file_symbols.push((sym, callees));
        }
    }
    Ok(file_symbols)
}

pub fn query_references(symbol_name: &str) -> Result<Vec<db::DbSymbol>> {
    ensure_fresh_cwd()?;
    let conn = db::open_db()?;
    db::get_symbol_references(&conn, symbol_name)
}

pub fn analyze_impact(symbol_name: &str) -> Result<Vec<db::DbSymbol>> {
    ensure_fresh_cwd()?;
    let conn = db::open_db()?;
    let all_syms = db::get_all_symbols(&conn)?;
    let all_deps = db::get_all_dependencies(&conn)?;

    let target_ids: Vec<String> = all_syms
        .iter()
        .filter(|s| s.name == symbol_name)
        .map(|s| s.id.clone())
        .collect();
    if target_ids.is_empty() {
        return Ok(Vec::new());
    }

    let impact_graph = graph::ImpactGraph::build(all_syms, all_deps);

    let mut affected_ids = std::collections::HashSet::new();
    for target_id in target_ids {
        let upstream = impact_graph.resolve_upstream(&target_id);
        for u in upstream {
            affected_ids.insert(u.id);
        }
    }

    let conn2 = db::open_db()?;
    let reloaded = db::get_all_symbols(&conn2)?;
    let result = reloaded
        .into_iter()
        .filter(|s| affected_ids.contains(&s.id))
        .collect();

    Ok(result)
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct GraphMetrics {
    pub symbols_count: usize,
    pub edges_count: usize,
    pub query_latency_ms: f64,
    pub graph_coverage: f64,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct IndexStatus {
    pub symbols_count: usize,
    pub edges_count: usize,
    pub last_indexed: Option<i64>,
    pub graph_coverage: f64,
    pub stale: bool,
}

pub fn get_index_status() -> Result<IndexStatus> {
    let conn = db::open_db()?;
    let symbols_count: usize = conn.query_row("SELECT COUNT(*) FROM symbols", [], |r| r.get(0))?;
    let edges_count: usize =
        conn.query_row("SELECT COUNT(*) FROM dependencies", [], |r| r.get(0))?;
    let last_indexed: Option<i64> = conn
        .query_row("SELECT MAX(last_indexed) FROM file_hashes", [], |r| {
            r.get(0)
        })
        .ok();
    let now: i64 = conn.query_row("SELECT CAST(strftime('%s','now') AS INTEGER)", [], |r| {
        r.get(0)
    })?;
    let stale = symbols_count == 0
        || match last_indexed {
            None => true,
            Some(0) => true,
            Some(ts) => now - ts > STALE_SECS,
        };
    let metrics = get_graph_metrics()?;
    Ok(IndexStatus {
        symbols_count,
        edges_count,
        last_indexed,
        graph_coverage: metrics.graph_coverage,
        stale,
    })
}

pub fn export_obsidian_graph(output_dir: &Path) -> Result<usize> {
    if !output_dir.exists() {
        std::fs::create_dir_all(output_dir)?;
    }

    let conn = db::open_db()?;
    let symbols = db::get_all_symbols(&conn)?;
    let dependencies = db::get_all_dependencies(&conn)?;

    let mut symbol_map = std::collections::HashMap::new();
    for sym in &symbols {
        symbol_map.insert(sym.id.clone(), sym.clone());
    }

    let mut outgoing: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();
    for dep in &dependencies {
        outgoing
            .entry(dep.caller_id.clone())
            .or_default()
            .push(dep.callee_name.clone());
    }

    let mut incoming: std::collections::HashMap<String, Vec<db::DbSymbol>> =
        std::collections::HashMap::new();
    for dep in &dependencies {
        if let Some(caller_sym) = symbol_map.get(&dep.caller_id) {
            incoming
                .entry(dep.callee_name.clone())
                .or_default()
                .push(caller_sym.clone());
        }
    }

    let mut files_written = 0;

    for sym in &symbols {
        let file_name = format!(
            "{} ({}).md",
            sym.name,
            Path::new(&sym.file_path)
                .file_name()
                .map(|f| f.to_string_lossy().into_owned())
                .unwrap_or_else(|| "unknown".to_string())
        );
        let file_path = output_dir.join(&file_name);

        let mut md = String::new();
        md.push_str(&format!("# Symbol: {}\n\n", sym.name));
        md.push_str(&format!("- **Kind:** {}\n", sym.kind));
        md.push_str(&format!(
            "- **Location:** `{}:{}-{}`\n\n",
            sym.file_path, sym.line_start, sym.line_end
        ));

        md.push_str("## Calls (Outgoing)\n");
        if let Some(callees) = outgoing.get(&sym.id) {
            let mut unique_callees = callees.clone();
            unique_callees.sort();
            unique_callees.dedup();
            for callee in unique_callees {
                let mut links = Vec::new();
                for other in &symbols {
                    if other.name == callee {
                        let other_file = Path::new(&other.file_path)
                            .file_name()
                            .map(|f| f.to_string_lossy().into_owned())
                            .unwrap_or_else(|| "unknown".to_string());
                        links.push(format!("[[{} ({})]]", other.name, other_file));
                    }
                }
                if links.is_empty() {
                    md.push_str(&format!("- [[{}]]\n", callee));
                } else {
                    for l in links {
                        md.push_str(&format!("- {}\n", l));
                    }
                }
            }
        } else {
            md.push_str("- None\n");
        }
        md.push('\n');

        md.push_str("## Referenced By (Incoming)\n");
        if let Some(callers) = incoming.get(&sym.name) {
            let mut unique_callers = callers.clone();
            unique_callers.sort_by(|a, b| a.id.cmp(&b.id));
            unique_callers.dedup_by(|a, b| a.id == b.id);
            for caller in unique_callers {
                let caller_file = Path::new(&caller.file_path)
                    .file_name()
                    .map(|f| f.to_string_lossy().into_owned())
                    .unwrap_or_else(|| "unknown".to_string());
                md.push_str(&format!("- [[{} ({})]]\n", caller.name, caller_file));
            }
        } else {
            md.push_str("- None\n");
        }

        std::fs::write(file_path, md)?;
        files_written += 1;
    }

    Ok(files_written)
}

pub fn get_graph_metrics() -> Result<GraphMetrics> {
    let conn = db::open_db()?;
    let symbols = db::get_all_symbols(&conn)?;
    let dependencies = db::get_all_dependencies(&conn)?;

    let symbols_count = symbols.len();
    let edges_count = dependencies.len();

    let mut connected_ids = std::collections::HashSet::new();
    for dep in &dependencies {
        connected_ids.insert(dep.caller_id.clone());
        for sym in &symbols {
            if sym.name == dep.callee_name {
                connected_ids.insert(sym.id.clone());
            }
        }
    }

    let graph_coverage = if symbols_count > 0 {
        (connected_ids.len() as f64 / symbols_count as f64) * 100.0
    } else {
        0.0
    };

    let start = std::time::Instant::now();
    let _ = db::find_symbols(&conn, "dummy_nonexistent_symbol")?;
    let query_latency_ms = start.elapsed().as_secs_f64() * 1000.0;

    Ok(GraphMetrics {
        symbols_count,
        edges_count,
        query_latency_ms,
        graph_coverage,
    })
}

#[allow(unused_variables)]
pub fn query_hybrid(
    query: &str,
    model_path: Option<&Path>,
    tokenizer_path: Option<&Path>,
    alpha: f32,
    limit: usize,
) -> Result<Vec<(db::DbSymbol, f64)>> {
    let conn = db::open_db()?;
    let db_symbols = db::find_symbols(&conn, query)?;

    #[cfg(feature = "embeddings")]
    {
        if let (Some(m_path), Some(t_path)) = (model_path, tokenizer_path) {
            if m_path.exists() && t_path.exists() {
                let embedder = embeddings::OnnxEmbedder::load_model(m_path, t_path)?;
                let query_embedding = embedder.embed_text(query)?;

                let all_symbols = db::get_all_symbols(&conn)?;
                let mut scored_symbols = Vec::new();

                for sym in all_symbols {
                    let sym_text = format!("{} {} {}", sym.kind, sym.name, sym.file_path);
                    if let Ok(sym_emb) = embedder.embed_text(&sym_text) {
                        let sem_score = embeddings::dot_product(&query_embedding, &sym_emb) as f64;
                        let lex_score = if sym.name.to_lowercase().contains(&query.to_lowercase()) {
                            1.0
                        } else {
                            0.0
                        };

                        let combined_score =
                            alpha as f64 * lex_score + (1.0 - alpha as f64) * sem_score;
                        scored_symbols.push((sym, combined_score));
                    }
                }

                scored_symbols
                    .sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
                scored_symbols.truncate(limit);
                return Ok(scored_symbols);
            }
        }
    }

    let mut results = Vec::new();
    for s in db_symbols {
        results.push((s, 1.0));
    }
    results.truncate(limit);
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;
    use std::sync::Mutex;

    static INDEX_TEST_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn test_index_and_query_lifecycle() {
        let _lock = INDEX_TEST_LOCK.lock().unwrap();
        let tmp_db = env::temp_dir().join(format!("rtk_index_test_{}.db", std::process::id()));
        env::set_var("RTK_INDEX_DB_PATH", &tmp_db);

        let temp_project = env::temp_dir().join(format!("rtk_index_proj_{}", std::process::id()));
        fs::create_dir_all(&temp_project).unwrap();

        let code_rs = r#"
            struct Config {
                port: u16,
            }
            fn main() {
                let cfg = Config { port: 80 };
                setup_logger();
            }
            fn setup_logger() {
                println!("logging");
            }
        "#;
        fs::write(temp_project.join("main.rs"), code_rs).unwrap();

        let count = index_project(&temp_project).unwrap();
        assert_eq!(count, 3);

        let syms = query_symbols("main").unwrap();
        assert_eq!(syms.len(), 1);
        assert_eq!(syms[0].name, "main");
        assert_eq!(syms[0].kind, "Function");

        let refs = query_references("setup_logger").unwrap();
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].name, "main");

        let impact = analyze_impact("setup_logger").unwrap();
        assert_eq!(impact.len(), 1);
        assert_eq!(impact[0].name, "main");

        // Test export_obsidian_graph
        let obsidian_dir = temp_project.join("obsidian");
        let exported_count = export_obsidian_graph(&obsidian_dir).unwrap();
        assert_eq!(exported_count, 3);
        let exported_file = obsidian_dir.join("main (main.rs).md");
        assert!(exported_file.exists());
        let md_content = fs::read_to_string(exported_file).unwrap();
        assert!(md_content.contains("# Symbol: main"));
        assert!(md_content.contains("[[setup_logger (main.rs)]]"));

        // Test get_graph_metrics
        let metrics = get_graph_metrics().unwrap();
        assert_eq!(metrics.symbols_count, 3);
        assert_eq!(metrics.edges_count, 1);
        assert!(metrics.graph_coverage > 0.0);

        env::remove_var("RTK_INDEX_DB_PATH");
        fs::remove_file(&tmp_db).ok();
        fs::remove_dir_all(&temp_project).ok();
    }

    #[test]
    fn test_incremental_indexing_cache() {
        let _lock = INDEX_TEST_LOCK.lock().unwrap();
        let tmp_db = env::temp_dir().join(format!("rtk_inc_test_{}.db", std::process::id()));
        env::set_var("RTK_INDEX_DB_PATH", &tmp_db);

        let temp_project = env::temp_dir().join(format!("rtk_inc_proj_{}", std::process::id()));
        fs::create_dir_all(&temp_project).unwrap();

        let file_path = temp_project.join("main.rs");

        // 1. Initial run with one function
        fs::write(&file_path, "fn hello() {}").unwrap();
        let count1 = index_project(&temp_project).unwrap();
        assert_eq!(count1, 1);

        let syms1 = query_symbols("hello").unwrap();
        assert_eq!(syms1.len(), 1);

        // 2. Run again without changing file - should use cache
        let count2 = index_project(&temp_project).unwrap();
        assert_eq!(count2, 1);

        // 3. Modify file - add another function
        fs::write(&file_path, "fn hello() {} \n fn world() {}").unwrap();
        let count3 = index_project(&temp_project).unwrap();
        assert_eq!(count3, 2);

        let syms2 = query_symbols("world").unwrap();
        assert_eq!(syms2.len(), 1);

        // 4. Delete file - index should clean up
        fs::remove_file(&file_path).unwrap();
        let count4 = index_project(&temp_project).unwrap();
        assert_eq!(count4, 0);

        env::remove_var("RTK_INDEX_DB_PATH");
        fs::remove_file(&tmp_db).ok();
        fs::remove_dir_all(&temp_project).ok();
    }
}
