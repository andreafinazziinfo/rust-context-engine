use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<serde_json::Value>,
    pub method: String,
    pub params: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

pub fn run_mcp_server() -> Result<()> {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut stdout_lock = stdout.lock();

    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        let req: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                let err_resp = JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: serde_json::Value::Null,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32700,
                        message: format!("Parse error: {}", e),
                        data: None,
                    }),
                };
                let _ = writeln!(stdout_lock, "{}", serde_json::to_string(&err_resp)?);
                let _ = stdout_lock.flush();
                continue;
            }
        };

        let response = handle_request(&req);
        if let Some(resp) = response {
            writeln!(stdout_lock, "{}", serde_json::to_string(&resp)?)?;
            stdout_lock.flush()?;
        }
    }

    Ok(())
}

fn handle_request(req: &JsonRpcRequest) -> Option<JsonRpcResponse> {
    let id = req.id.clone().unwrap_or(serde_json::Value::Null);

    match req.method.as_str() {
        "initialize" => {
            let res = json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {}
                },
                "serverInfo": {
                    "name": "rtk-mcp",
                    "version": "0.1.0"
                }
            });
            Some(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: Some(res),
                error: None,
            })
        }
        "notifications/initialized" => {
            // Notification: no response needed
            None
        }
        "ping" => Some(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(json!({})),
            error: None,
        }),
        "tools/list" => {
            let tools = get_tools_list();
            Some(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: Some(json!({ "tools": tools })),
                error: None,
            })
        }
        "tools/call" => {
            let params = req.params.as_ref().unwrap_or(&serde_json::Value::Null);
            let name = params.get("name").and_then(|n| n.as_str()).unwrap_or("");
            let arguments = params.get("arguments").cloned().unwrap_or(json!({}));

            let result = execute_tool(name, arguments);
            match result {
                Ok(content) => Some(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: Some(content),
                    error: None,
                }),
                Err(e) => Some(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32603,
                        message: e.to_string(),
                        data: None,
                    }),
                }),
            }
        }
        _ => Some(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code: -32601,
                message: format!("Method not found: {}", req.method),
                data: None,
            }),
        }),
    }
}

fn get_tools_list() -> serde_json::Value {
    json!([
        {
            "name": "search_code",
            "description": "Search code for a substring pattern across files in the current workspace.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "The search pattern or keyword"
                    }
                },
                "required": ["query"]
            }
        },
        {
            "name": "find_symbols",
            "description": "Find symbol definitions (struct, functions, classes, methods) in the workspace.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "The name query to search for"
                    }
                },
                "required": ["query"]
            }
        },
        {
            "name": "find_refs",
            "description": "Find all references to a given symbol name.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "symbol": {
                        "type": "string",
                        "description": "The symbol name to look up references for"
                    }
                },
                "required": ["symbol"]
            }
        },
        {
            "name": "project_memory",
            "description": "Perform get, set, overwrite, search, or list operations on project memory.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": ["get", "set", "overwrite", "list", "search"],
                        "description": "The memory operation to perform"
                    },
                    "key": {
                        "type": "string",
                        "description": "The key (required for get, set, overwrite)"
                    },
                    "value": {
                        "type": "string",
                        "description": "The value (required for set, overwrite)"
                    },
                    "query": {
                        "type": "string",
                        "description": "Search query (required for search)"
                    }
                },
                "required": ["action"]
            }
        },
        {
            "name": "artifact_get",
            "description": "Retrieve content and metadata of a registered RTK artifact.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": {
                        "type": "string",
                        "description": "The artifact ID"
                    }
                },
                "required": ["id"]
            }
        },
        {
            "name": "context_pack",
            "description": "Pack context of specified files/directories into a single structured output.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "paths": {
                        "type": "array",
                        "items": {
                            "type": "string"
                        },
                        "description": "List of absolute or relative paths to files/directories to pack"
                    },
                    "skeleton": {
                        "type": "boolean",
                        "description": "Export only code signatures/skeletons instead of full file content"
                    }
                },
                "required": ["paths"]
            }
        },
        {
            "name": "session_state",
            "description": "Query or update the session state variables for context handoff.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": ["get", "update", "export"],
                        "description": "Action: get, update, or export"
                    },
                    "key": {
                        "type": "string",
                        "description": "The key to update (required for update)"
                    },
                    "value": {
                        "type": "string",
                        "description": "The new value to set (required for update)"
                    }
                },
                "required": ["action"]
            }
        },
        {
            "name": "ping",
            "description": "Ping the RTK MCP server to check connectivity and diagnostic status.",
            "inputSchema": {
                "type": "object",
                "properties": {}
            }
        },
        {
            "name": "get_budget_status",
            "description": "Get the current LLM API budget spend and check if it exceeds a limit.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "limit": {
                        "type": "number",
                        "description": "Optional budget limit in USD to check against (default: 50.0)"
                    }
                }
            }
        }
    ])
}

pub fn execute_tool(name: &str, args: serde_json::Value) -> Result<serde_json::Value> {
    match name {
        "search_code" => {
            let query = args
                .get("query")
                .and_then(|q| q.as_str())
                .ok_or_else(|| anyhow!("Missing query"))?;
            let result_str = search_code_helper(query)?;
            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": result_str
                }]
            }))
        }
        "find_symbols" => {
            let query = args
                .get("query")
                .and_then(|q| q.as_str())
                .ok_or_else(|| anyhow!("Missing query"))?;
            let syms = rtk_index::query_symbols(query)?;
            let mut text = String::new();
            for s in syms {
                text.push_str(&format!(
                    "- {} ({}) in {}:{}-{}\n",
                    s.name, s.kind, s.file_path, s.line_start, s.line_end
                ));
            }
            if text.is_empty() {
                text = "No symbols found.".to_string();
            }
            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": text
                }]
            }))
        }
        "find_refs" => {
            let symbol = args
                .get("symbol")
                .and_then(|s| s.as_str())
                .ok_or_else(|| anyhow!("Missing symbol"))?;
            let refs = rtk_index::query_references(symbol)?;
            let mut text = String::new();
            for r in refs {
                text.push_str(&format!(
                    "- {} ({}) in {}:{}-{}\n",
                    r.name, r.kind, r.file_path, r.line_start, r.line_end
                ));
            }
            if text.is_empty() {
                text = "No references found.".to_string();
            }
            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": text
                }]
            }))
        }
        "project_memory" => {
            let action = args
                .get("action")
                .and_then(|a| a.as_str())
                .ok_or_else(|| anyhow!("Missing action"))?;
            let key = args.get("key").and_then(|k| k.as_str()).unwrap_or("");
            let value = args.get("value").and_then(|v| v.as_str()).unwrap_or("");
            let query = args.get("query").and_then(|q| q.as_str()).unwrap_or("");

            let text = match action {
                "get" => {
                    if key.is_empty() {
                        return Err(anyhow!("Missing key for get action"));
                    }
                    rtk_db::tracking::memory_get(key)?
                }
                "set" => {
                    if key.is_empty() || value.is_empty() {
                        return Err(anyhow!("Missing key or value for set action"));
                    }
                    rtk_db::tracking::memory_set(key, value)?;
                    format!("Memory set: {} = {}", key, value)
                }
                "overwrite" => {
                    if key.is_empty() || value.is_empty() {
                        return Err(anyhow!("Missing key or value for overwrite action"));
                    }
                    rtk_db::tracking::memory_overwrite(key, value)?;
                    format!("Memory overwritten: {} = {}", key, value)
                }
                "list" => {
                    let list = rtk_db::tracking::memory_list()?;
                    let mut out = String::new();
                    for (k, v) in list {
                        out.push_str(&format!("{k}: {v}\n"));
                    }
                    if out.is_empty() {
                        "No memory entries found.".to_string()
                    } else {
                        out
                    }
                }
                "search" => {
                    if query.is_empty() {
                        return Err(anyhow!("Missing query for search action"));
                    }
                    let res = rtk_db::tracking::memory_search(query)?;
                    let mut out = String::new();
                    for (k, v) in res {
                        out.push_str(&format!("- {k}:\n  {v}\n\n"));
                    }
                    if out.is_empty() {
                        "No search matches found.".to_string()
                    } else {
                        out
                    }
                }
                _ => return Err(anyhow!("Invalid memory action: {}", action)),
            };

            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": text
                }]
            }))
        }
        "artifact_get" => {
            let id = args
                .get("id")
                .and_then(|i| i.as_str())
                .ok_or_else(|| anyhow!("Missing id"))?;
            let art = rtk_db::artifact::artifact_get(id)?;
            let text = format!(
                "ID: {}\nType: {}\nCreated At: {}\nMetadata: {}\nContent:\n\n{}",
                art.id,
                art.r#type,
                art.created_at,
                art.metadata_json.unwrap_or_default(),
                art.content
            );
            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": text
                }]
            }))
        }
        "context_pack" => {
            let paths_val = args.get("paths").ok_or_else(|| anyhow!("Missing paths"))?;
            let paths: Vec<String> = serde_json::from_value(paths_val.clone())?;
            let skeleton = args
                .get("skeleton")
                .and_then(|s| s.as_bool())
                .unwrap_or(false);

            let mut text = String::new();
            text.push_str("<repository>\n");
            for p_str in paths {
                let path = Path::new(&p_str);
                if path.is_dir() {
                    let packed_dir = rtk_pack::pack::pack_directory(path, false, skeleton)?;
                    let clean = packed_dir
                        .replace("<repository>\n", "")
                        .replace("</repository>\n", "");
                    text.push_str(&clean);
                } else if path.is_file() {
                    use std::io::Read;
                    let mut content = String::new();
                    if skeleton {
                        let mut raw = String::new();
                        std::fs::File::open(path)?.read_to_string(&mut raw)?;
                        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                        let parsed = rtk_pack::skeleton::skeletonize(&raw, ext);
                        content.push_str(&parsed);
                    } else {
                        std::fs::File::open(path)?.read_to_string(&mut content)?;
                    }
                    text.push_str(&format!(
                        "<file path=\"{}\">\n{}\n</file>\n",
                        p_str, content
                    ));
                }
            }
            text.push_str("</repository>\n");

            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": text
                }]
            }))
        }
        "session_state" => {
            let action = args
                .get("action")
                .and_then(|a| a.as_str())
                .ok_or_else(|| anyhow!("Missing action"))?;
            let key = args.get("key").and_then(|k| k.as_str()).unwrap_or("");
            let value = args.get("value").and_then(|v| v.as_str()).unwrap_or("");

            let text = match action {
                "get" => rtk_db::session::session_get()?,
                "update" => {
                    if key.is_empty() || value.is_empty() {
                        return Err(anyhow!("Missing key or value for update action"));
                    }
                    rtk_db::session::session_update(key, value)?;
                    format!("Updated session state: {} = {}", key, value)
                }
                "export" => rtk_db::session::session_export()?,
                _ => return Err(anyhow!("Invalid session action: {}", action)),
            };

            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": text
                }]
            }))
        }
        "ping" => Ok(json!({
            "content": [{
                "type": "text",
                "text": "pong"
            }]
        })),
        "get_budget_status" => {
            let limit = args
                .get("limit")
                .and_then(|l| l.as_f64())
                .unwrap_or(50.0);
            let status = rtk_db::pricing::check_budget(limit)?;
            let text = format!(
                "Budget Limit: ${:.2} USD\nTotal Cost Spent: ${:.6} USD\nPercentage Used: {:.2}%\nExceeded: {}\nStatus: {}",
                status.limit_usd,
                status.spent_usd,
                status.percentage,
                status.exceeded,
                if status.exceeded { "🚨 ALERT: Budget limit exceeded!" } else { "✅ Within budget limits." }
            );
            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": text
                }]
            }))
        }
        _ => Err(anyhow!("Unknown tool: {}", name)),
    }
}

fn search_code_helper(query: &str) -> Result<String> {
    let files = rtk_index::parser::scan_directory(Path::new("."))?;
    let mut matches = Vec::new();
    let lower_query = query.to_lowercase();
    for file in files {
        if let Ok(content) = std::fs::read_to_string(&file) {
            for (line_idx, line) in content.lines().enumerate() {
                if line.to_lowercase().contains(&lower_query) {
                    matches.push(json!({
                        "file": file.to_string_lossy().replace('\\', "/"),
                        "line": line_idx + 1,
                        "content": line.trim()
                    }));
                    if matches.len() >= 100 {
                        break;
                    }
                }
            }
        }
        if matches.len() >= 100 {
            break;
        }
    }
    Ok(serde_json::to_string_pretty(&matches)?)
}

pub fn install_mcp_client(client: &str) -> Result<()> {
    let exe_path = std::env::current_exe()?
        .to_string_lossy()
        .replace('\\', "/");

    match client.to_lowercase().as_str() {
        "claude" => {
            let app_data = std::env::var("APPDATA").map(PathBuf::from).or_else(|_| {
                dirs::home_dir()
                    .map(|h| h.join("AppData").join("Roaming"))
                    .ok_or_else(|| anyhow!("Could not resolve AppData directory"))
            })?;

            let claude_dir = app_data.join("Claude");
            if !claude_dir.exists() {
                std::fs::create_dir_all(&claude_dir)?;
            }
            let config_path = claude_dir.join("claude_desktop_config.json");

            let mut config_json: serde_json::Value = if config_path.exists() {
                let content = std::fs::read_to_string(&config_path)?;
                serde_json::from_str(&content).unwrap_or(json!({}))
            } else {
                json!({})
            };

            if !config_json.is_object() {
                config_json = json!({});
            }
            let mcp_servers = config_json
                .as_object_mut()
                .context("config is not an object")?
                .entry("mcpServers".to_string())
                .or_insert(json!({}));

            mcp_servers
                .as_object_mut()
                .context("mcpServers is not an object")?
                .insert(
                    "rtk".to_string(),
                    json!({
                        "command": exe_path,
                        "args": ["mcp", "start"]
                    }),
                );

            let pretty = serde_json::to_string_pretty(&config_json)?;
            std::fs::write(&config_path, pretty)?;
            println!(
                "✅ Successfully installed RTK MCP server config for Claude Desktop at: {}",
                config_path.display()
            );
            Ok(())
        }
        "cursor" => {
            let mut updated_any = false;

            // 1. Update ~/.cursor/mcp.json
            if let Some(cursor_dir) = dirs::home_dir().map(|h| h.join(".cursor")) {
                if !cursor_dir.exists() {
                    let _ = std::fs::create_dir_all(&cursor_dir);
                }
                let mcp_path = cursor_dir.join("mcp.json");
                let mut config_json = if mcp_path.exists() {
                    let content = std::fs::read_to_string(&mcp_path)?;
                    serde_json::from_str(&content).unwrap_or(json!({}))
                } else {
                    json!({})
                };

                if !config_json.is_object() {
                    config_json = json!({});
                }

                let mcp_servers = config_json
                    .as_object_mut()
                    .context("config is not an object")?
                    .entry("mcpServers".to_string())
                    .or_insert(json!({}));

                mcp_servers
                    .as_object_mut()
                    .context("mcpServers is not an object")?
                    .insert(
                        "rtk".to_string(),
                        json!({
                            "command": exe_path.clone(),
                            "args": ["mcp", "start"]
                        }),
                    );

                let pretty = serde_json::to_string_pretty(&config_json)?;
                std::fs::write(&mcp_path, pretty)?;
                println!(
                    "✅ Successfully installed RTK MCP server config for Cursor at: {}",
                    mcp_path.display()
                );
                updated_any = true;
            }

            // 2. Update storage.json
            let cursor_user_dir = if cfg!(windows) {
                std::env::var("APPDATA")
                    .ok()
                    .map(|p| PathBuf::from(p).join("Cursor").join("User"))
            } else if cfg!(target_os = "macos") {
                dirs::home_dir().map(|h| {
                    h.join("Library")
                        .join("Application Support")
                        .join("Cursor")
                        .join("User")
                })
            } else {
                dirs::home_dir().map(|h| h.join(".config").join("Cursor").join("User"))
            };

            if let Some(user_dir) = cursor_user_dir {
                let storage_path = user_dir.join("globalStorage").join("storage.json");
                if storage_path.exists() {
                    let content = std::fs::read_to_string(&storage_path)?;
                    let mut storage_json: serde_json::Value =
                        serde_json::from_str(&content).unwrap_or(json!({}));
                    if storage_json.is_object() {
                        let mcp_servers = storage_json
                            .as_object_mut()
                            .context("storage is not an object")?
                            .entry("mcpServers".to_string())
                            .or_insert(json!({}));
                        mcp_servers
                            .as_object_mut()
                            .context("mcpServers is not an object")?
                            .insert(
                                "rtk".to_string(),
                                json!({
                                    "command": exe_path.clone(),
                                    "args": ["mcp", "start"]
                                }),
                            );
                        let pretty = serde_json::to_string_pretty(&storage_json)?;
                        std::fs::write(&storage_path, pretty)?;
                        println!(
                            "✅ Successfully installed RTK MCP server config for Cursor in storage.json at: {}",
                            storage_path.display()
                        );
                        updated_any = true;
                    }
                }
            }

            if !updated_any {
                return Err(anyhow!(
                    "Could not find any Cursor configuration directory to update"
                ));
            }

            Ok(())
        }
        "gemini" => {
            println!("💡 To install on Gemini / Vertex AI client:");
            println!("  Configure the stdio runner to execute:");
            println!("  \"{}\" mcp start", exe_path);
            Ok(())
        }
        _ => Err(anyhow!(
            "Unknown client: {}. Supported: claude, cursor, gemini",
            client
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_execute_get_budget_status() {
        let args = json!({
            "limit": 100.0
        });
        let result = execute_tool("get_budget_status", args).unwrap();
        let content = result.get("content").unwrap().as_array().unwrap();
        let text = content[0].get("text").unwrap().as_str().unwrap();
        assert!(text.contains("Budget Limit: $100.00 USD"));
        assert!(text.contains("Total Cost Spent:"));
    }
}
