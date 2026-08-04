use crate::parser::ParsedSymbol;
use anyhow::{Context, Result};
use rusqlite::{params, Connection};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DbSymbol {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub file_path: String,
    pub line_start: usize,
    pub line_end: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DbDependency {
    pub caller_id: String,
    pub callee_name: String,
    pub callee_file_path: Option<String>,
    pub dependency_kind: String,
}

pub fn open_db() -> Result<Connection> {
    let path = if let Ok(p) = std::env::var("RTK_INDEX_DB_PATH") {
        std::path::PathBuf::from(p)
    } else {
        let rtk_dir = std::env::current_dir()?.join(".rtk");
        if !rtk_dir.exists() {
            std::fs::create_dir_all(&rtk_dir)
                .with_context(|| format!("create {}", rtk_dir.display()))?;
        }
        rtk_dir.join("rtk.db")
    };

    let conn = Connection::open(&path).with_context(|| format!("open db {}", path.display()))?;
    let _ = conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA synchronous = NORMAL;
         PRAGMA busy_timeout = 5000;",
    );

    conn.execute(
        "CREATE TABLE IF NOT EXISTS symbols (
            id         TEXT PRIMARY KEY,
            name       TEXT NOT NULL,
            kind       TEXT NOT NULL,
            file_path  TEXT NOT NULL,
            line_start INTEGER NOT NULL,
            line_end   INTEGER NOT NULL
        )",
        [],
    )
    .context("create symbols table")?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS dependencies (
            caller_id        TEXT NOT NULL,
            callee_name      TEXT NOT NULL,
            callee_file_path TEXT,
            dependency_kind  TEXT NOT NULL,
            PRIMARY KEY (caller_id, callee_name, dependency_kind)
        )",
        [],
    )
    .context("create dependencies table")?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS file_hashes (
            file_path    TEXT PRIMARY KEY,
            hash         TEXT NOT NULL,
            last_indexed INTEGER NOT NULL
        )",
        [],
    )
    .context("create file_hashes table")?;

    Ok(conn)
}

pub fn clear_index(conn: &Connection) -> Result<()> {
    conn.execute("DELETE FROM symbols", [])?;
    conn.execute("DELETE FROM dependencies", [])?;
    Ok(())
}

pub fn insert_symbols(conn: &Connection, symbols: &[ParsedSymbol]) -> Result<()> {
    let mut stmt_sym = conn.prepare(
        "INSERT OR REPLACE INTO symbols (id, name, kind, file_path, line_start, line_end) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
    )?;

    let mut stmt_dep = conn.prepare(
        "INSERT OR REPLACE INTO dependencies (caller_id, callee_name, callee_file_path, dependency_kind) \
         VALUES (?1, ?2, ?3, ?4)"
    )?;

    for sym in symbols {
        stmt_sym.execute(params![
            sym.id,
            sym.name,
            sym.kind,
            sym.file_path,
            sym.line_start as i64,
            sym.line_end as i64
        ])?;

        for call in &sym.calls {
            // Find if there is a known destination file for this callee name (heuristic)
            // For now, save caller_id and callee_name
            stmt_dep.execute(params![sym.id, call, None::<String>, "CALL"])?;
        }
    }

    Ok(())
}

pub fn find_symbols(conn: &Connection, name_query: &str) -> Result<Vec<DbSymbol>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, kind, file_path, line_start, line_end FROM symbols \
         WHERE name LIKE ?1",
    )?;

    let q = format!("%{}%", name_query);
    let rows = stmt.query_map(params![q], |r| {
        Ok(DbSymbol {
            id: r.get(0)?,
            name: r.get(1)?,
            kind: r.get(2)?,
            file_path: r.get(3)?,
            line_start: r.get::<_, i64>(4)? as usize,
            line_end: r.get::<_, i64>(5)? as usize,
        })
    })?;

    let mut list = Vec::new();
    for row in rows {
        list.push(row?);
    }
    Ok(list)
}

pub fn get_all_symbols(conn: &Connection) -> Result<Vec<DbSymbol>> {
    let mut stmt =
        conn.prepare("SELECT id, name, kind, file_path, line_start, line_end FROM symbols")?;
    let rows = stmt.query_map([], |r| {
        Ok(DbSymbol {
            id: r.get(0)?,
            name: r.get(1)?,
            kind: r.get(2)?,
            file_path: r.get(3)?,
            line_start: r.get::<_, i64>(4)? as usize,
            line_end: r.get::<_, i64>(5)? as usize,
        })
    })?;
    let mut list = Vec::new();
    for row in rows {
        list.push(row?);
    }
    Ok(list)
}

pub fn get_all_dependencies(conn: &Connection) -> Result<Vec<DbDependency>> {
    let mut stmt = conn.prepare(
        "SELECT caller_id, callee_name, callee_file_path, dependency_kind FROM dependencies",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok(DbDependency {
            caller_id: r.get(0)?,
            callee_name: r.get(1)?,
            callee_file_path: r.get(2)?,
            dependency_kind: r.get(3)?,
        })
    })?;
    let mut list = Vec::new();
    for row in rows {
        list.push(row?);
    }
    Ok(list)
}

pub fn get_symbol_references(conn: &Connection, symbol_name: &str) -> Result<Vec<DbSymbol>> {
    let mut stmt = conn.prepare(
        "SELECT s.id, s.name, s.kind, s.file_path, s.line_start, s.line_end \
         FROM dependencies d \
         JOIN symbols s ON d.caller_id = s.id \
         WHERE d.callee_name = ?1",
    )?;

    let rows = stmt.query_map(params![symbol_name], |r| {
        Ok(DbSymbol {
            id: r.get(0)?,
            name: r.get(1)?,
            kind: r.get(2)?,
            file_path: r.get(3)?,
            line_start: r.get::<_, i64>(4)? as usize,
            line_end: r.get::<_, i64>(5)? as usize,
        })
    })?;

    let mut list = Vec::new();
    for row in rows {
        list.push(row?);
    }
    Ok(list)
}

pub fn clear_file_index(conn: &Connection, file_path: &str) -> Result<()> {
    conn.execute(
        "DELETE FROM dependencies WHERE caller_id IN (SELECT id FROM symbols WHERE file_path = ?1)",
        [file_path],
    )?;
    conn.execute("DELETE FROM symbols WHERE file_path = ?1", [file_path])?;
    Ok(())
}

pub fn get_file_hashes(conn: &Connection) -> Result<std::collections::HashMap<String, String>> {
    let mut stmt = conn.prepare("SELECT file_path, hash FROM file_hashes")?;
    let rows = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?;
    let mut map = std::collections::HashMap::new();
    for row in rows {
        let (path, hash) = row?;
        map.insert(path, hash);
    }
    Ok(map)
}

pub fn insert_file_hash(conn: &Connection, file_path: &str, hash: &str) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO file_hashes (file_path, hash, last_indexed) VALUES (?1, ?2, strftime('%s','now'))",
        params![file_path, hash],
    )?;
    Ok(())
}

pub fn delete_file_hash(conn: &Connection, file_path: &str) -> Result<()> {
    conn.execute("DELETE FROM file_hashes WHERE file_path = ?1", [file_path])?;
    Ok(())
}
