mod common;
use std::fs;

/// Integration tests
fn rtk_bin() -> std::process::Command {
    std::process::Command::new(env!("CARGO_BIN_EXE_rtk"))
}

#[test]
fn rtk_binary_is_in_path() {
    assert!(rtk_bin()
        .arg("--version")
        .output()
        .expect("rtk not found")
        .status
        .success());
}

#[test]
fn rewrite_git_status_exit_0() {
    let out = rtk_bin()
        .args(["rewrite", "git status"])
        .output()
        .expect("rtk not found");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(
        String::from_utf8_lossy(&out.stdout).trim(),
        "rtk git status"
    );
}

#[test]
fn rewrite_unknown_cmd_exit_1() {
    let status = rtk_bin()
        .args(["rewrite", "python manage.py runserver"])
        .status()
        .expect("rtk not found");
    assert_eq!(status.code(), Some(1));
}

#[test]
fn rewrite_force_push_exit_2() {
    let status = rtk_bin()
        .args(["rewrite", "git push origin main --force"])
        .status()
        .expect("rtk not found");
    assert_eq!(status.code(), Some(2));
}

#[test]
fn rewrite_git_push_exit_3() {
    let out = rtk_bin()
        .args(["rewrite", "git push"])
        .output()
        .expect("rtk not found");
    assert_eq!(out.status.code(), Some(3));
    assert!(!out.stdout.is_empty());
}

#[test]
fn e2e_ide_pipeline_flow() {
    // 1. Simulate Claude sending a command that the hook catches
    let rewrite_out = rtk_bin().args(["rewrite", "git status"]).output().unwrap();

    assert_eq!(rewrite_out.status.code(), Some(0));
    let rewritten_cmd = String::from_utf8_lossy(&rewrite_out.stdout)
        .trim()
        .to_string();
    assert_eq!(rewritten_cmd, "rtk git status");

    // 2. Execute the proxied command
    let run_out = rtk_bin().args(["git", "status"]).output().unwrap();

    assert!(run_out.status.success() || run_out.status.code() == Some(128));
    let stdout_str = String::from_utf8_lossy(&run_out.stdout);

    // 3. Verify output contains standard RTK wrappers or git output
    assert!(
        stdout_str.contains("git") || stdout_str.contains("RTK") || stdout_str.contains("branch")
    );

    // We can also verify that a local SQLite DB was hit, but since
    // tests run concurrently, checking .rtk dir requires creating a temp dir.
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let temp_dir = std::env::temp_dir().join(format!("rtk_e2e_{timestamp}"));
    fs::create_dir_all(&temp_dir).unwrap();

    // Initialize a dummy git project so git status succeeds
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(&temp_dir)
        .output()
        .unwrap();

    let db_path = temp_dir.join("rtk.db");
    let proxied_run = rtk_bin()
        .current_dir(&temp_dir)
        .env("RTK_DB_PATH", &db_path)
        .args(["git", "status"])
        .output()
        .unwrap();

    assert!(proxied_run.status.success() || proxied_run.status.code() == Some(128));

    // Verify that the database was created
    assert!(db_path.exists());

    fs::remove_dir_all(temp_dir).unwrap();
}

/// Token savings on real fixtures
#[test]
#[ignore]
fn git_status_token_savings_gte_60pct() {
    let fixture = "tests/fixtures/git_status_raw.txt";
    if !std::path::Path::new(fixture).exists() {
        return;
    }
    let input = std::fs::read_to_string(fixture).unwrap();
    let out = rtk_bin()
        .args(["git", "status"])
        .output()
        .expect("rtk not found");
    let filtered = String::from_utf8_lossy(&out.stdout).to_string();
    let (savings, passes) = common::token_savings(&input, &filtered);
    assert!(passes, "git status savings {savings:.1}% < 60%");
}

#[test]
fn test_rtk_agents_lifecycle() {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let temp_dir = std::env::temp_dir().join(format!("rtk_agents_test_{timestamp}"));
    std::fs::create_dir_all(&temp_dir).unwrap();

    // 1. Run `rtk agents init --template solo-dev`
    let init_status = rtk_bin()
        .current_dir(&temp_dir)
        .args(["agents", "init", "--template", "solo-dev"])
        .status()
        .expect("rtk not found");
    assert!(init_status.success());

    // Check files are created
    let agents_md = temp_dir.join("AGENTS.md");
    let claude_md = temp_dir.join("CLAUDE.md");
    assert!(agents_md.exists());
    assert!(claude_md.exists());

    // 2. Run `rtk agents doctor`
    let doc_status = rtk_bin()
        .current_dir(&temp_dir)
        .args(["agents", "doctor"])
        .status()
        .expect("rtk not found");
    assert!(doc_status.success());

    // 3. Add some redundant spacing and comments to test compacting
    let bad_content = "<!-- comment -->\n\n\n# Header\n\n\nSome text\n\n\n";
    std::fs::write(&agents_md, bad_content).unwrap();

    // Run `rtk agents compact`
    let compact_status = rtk_bin()
        .current_dir(&temp_dir)
        .args(["agents", "compact"])
        .status()
        .expect("rtk not found");
    assert!(compact_status.success());

    // Read compacted file and verify it's smaller and contains compacted content
    let compacted = std::fs::read_to_string(&agents_md).unwrap();
    assert!(!compacted.contains("<!-- comment -->"));
    assert!(compacted.contains("# Header\n\nSome text"));

    std::fs::remove_dir_all(temp_dir).unwrap();
}

#[test]
fn test_rtk_artifact_lifecycle() {
    let _lock = rtk_db::tracking::DB_TEST_LOCK.lock().unwrap();
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let temp_dir = std::env::temp_dir().join(format!("rtk_artifact_test_{timestamp}"));
    std::fs::create_dir_all(&temp_dir).unwrap();
    let db_path = temp_dir.join("artifacts.db");
    std::env::set_var("RTK_DB_PATH", &db_path);

    // Add artifact to DB directly
    rtk_db::artifact::artifact_add("art-cli-1", "reasoning", "cli test reasoning content", None)
        .unwrap();

    // Call CLI: rtk artifact list
    let list_out = rtk_bin()
        .env("RTK_DB_PATH", &db_path)
        .args(["artifact", "list"])
        .output()
        .expect("rtk not found");
    assert!(list_out.status.success());
    let list_str = String::from_utf8_lossy(&list_out.stdout);
    assert!(list_str.contains("art-cli-1"));
    assert!(list_str.contains("reasoning"));

    // Call CLI: rtk artifact get art-cli-1
    let get_out = rtk_bin()
        .env("RTK_DB_PATH", &db_path)
        .args(["artifact", "get", "art-cli-1"])
        .output()
        .expect("rtk not found");
    assert!(get_out.status.success());
    let get_str = String::from_utf8_lossy(&get_out.stdout);
    assert_eq!(get_str, "cli test reasoning content");

    // Call CLI: rtk artifact gc
    let gc_out = rtk_bin()
        .env("RTK_DB_PATH", &db_path)
        .args(["artifact", "gc"])
        .output()
        .expect("rtk not found");
    assert!(gc_out.status.success());

    std::env::remove_var("RTK_DB_PATH");
    std::fs::remove_dir_all(temp_dir).unwrap();
}

#[test]
fn test_rtk_index_cli_lifecycle() {
    let _lock = rtk_db::tracking::DB_TEST_LOCK.lock().unwrap();
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let temp_dir = std::env::temp_dir().join(format!("rtk_index_cli_test_{timestamp}"));
    std::fs::create_dir_all(&temp_dir).unwrap();

    let db_path = temp_dir.join("rtk_index.db");
    std::env::set_var("RTK_INDEX_DB_PATH", &db_path);

    // Create a simple rust file to index
    let code = r#"
        fn foo() {
            bar();
        }
        fn bar() {}
    "#;
    std::fs::write(temp_dir.join("lib.rs"), code).unwrap();

    // 1. Run index run
    let index_out = rtk_bin()
        .current_dir(&temp_dir)
        .env("RTK_INDEX_DB_PATH", &db_path)
        .args(["index", "run"])
        .output()
        .expect("rtk not found");
    assert!(index_out.status.success());
    let index_str = String::from_utf8_lossy(&index_out.stdout);
    assert!(index_str.contains("Indexed 2 symbols"));

    // 2. Run symbols find foo
    let find_out = rtk_bin()
        .current_dir(&temp_dir)
        .env("RTK_INDEX_DB_PATH", &db_path)
        .args(["symbols", "find", "foo"])
        .output()
        .expect("rtk not found");
    assert!(find_out.status.success());
    let find_str = String::from_utf8_lossy(&find_out.stdout);
    assert!(find_str.contains("foo"));
    assert!(find_str.contains("Function"));

    // 3. Run deps show lib.rs
    let deps_out = rtk_bin()
        .current_dir(&temp_dir)
        .env("RTK_INDEX_DB_PATH", &db_path)
        .args(["deps", "show", "lib.rs"])
        .output()
        .expect("rtk not found");
    assert!(deps_out.status.success());
    let deps_str = String::from_utf8_lossy(&deps_out.stdout);
    assert!(deps_str.contains("foo"));
    assert!(deps_str.contains("calls: bar"));

    // 4. Run refs find bar
    let refs_out = rtk_bin()
        .current_dir(&temp_dir)
        .env("RTK_INDEX_DB_PATH", &db_path)
        .args(["refs", "find", "bar"])
        .output()
        .expect("rtk not found");
    assert!(refs_out.status.success());
    let refs_str = String::from_utf8_lossy(&refs_out.stdout);
    assert!(refs_str.contains("foo"));

    // 5. Run impact analyze bar
    let impact_out = rtk_bin()
        .current_dir(&temp_dir)
        .env("RTK_INDEX_DB_PATH", &db_path)
        .args(["impact", "analyze", "bar"])
        .output()
        .expect("rtk not found");
    assert!(impact_out.status.success());
    let impact_str = String::from_utf8_lossy(&impact_out.stdout);
    assert!(impact_str.contains("foo"));
    assert!(impact_str.contains("Risk Level: LOW"));

    std::env::remove_var("RTK_INDEX_DB_PATH");
    std::fs::remove_dir_all(temp_dir).unwrap();
}

#[test]
fn test_rtk_budget_and_mcp_cli_lifecycle() {
    let _lock = rtk_db::tracking::DB_TEST_LOCK.lock().unwrap();
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let temp_dir = std::env::temp_dir().join(format!("rtk_budget_mcp_test_{timestamp}"));
    std::fs::create_dir_all(&temp_dir).unwrap();

    let db_path = temp_dir.join("rtk.db");
    let index_db_path = temp_dir.join("rtk_index.db");

    // 1. Run budget check
    let budget_out = rtk_bin()
        .current_dir(&temp_dir)
        .env("RTK_DB_PATH", &db_path)
        .args(["budget", "check", "--limit", "10.0"])
        .output()
        .expect("rtk not found");
    assert!(budget_out.status.success());
    let budget_str = String::from_utf8_lossy(&budget_out.stdout);
    assert!(budget_str.contains("Budget Status"));
    assert!(budget_str.contains("Limit:        $10.00"));

    // 2. Run model routing suggest
    let model_out = rtk_bin()
        .current_dir(&temp_dir)
        .args(["model", "suggest", "--task", "complex-refactoring"])
        .output()
        .expect("rtk not found");
    assert!(model_out.status.success());
    let model_str = String::from_utf8_lossy(&model_out.stdout);
    assert!(model_str.contains(
        "Routing Suggestion: task 'complex-refactoring' -> use model 'claude-4.6-sonnet'"
    ));

    // 3. Setup temporary file to index & test MCP call
    let main_rs_path = temp_dir.join("main.rs");
    std::fs::write(&main_rs_path, "fn my_unique_mcp_test_func() {}").unwrap();

    // Force index run
    let index_out = rtk_bin()
        .current_dir(&temp_dir)
        .env("RTK_INDEX_DB_PATH", &index_db_path)
        .args(["index", "run"])
        .output()
        .expect("rtk not found");
    assert!(index_out.status.success());

    // Call MCP search_code tool
    let mcp_out = rtk_bin()
        .current_dir(&temp_dir)
        .env("RTK_INDEX_DB_PATH", &index_db_path)
        .args([
            "mcp",
            "call",
            "search_code",
            "--args",
            "{\"query\":\"my_unique_mcp_test_func\"}",
        ])
        .output()
        .expect("rtk not found");
    assert!(mcp_out.status.success());
    let mcp_str = String::from_utf8_lossy(&mcp_out.stdout);
    assert!(mcp_str.contains("my_unique_mcp_test_func"));

    // 4. Memory overwrite & doctor
    let overwrite_out = rtk_bin()
        .current_dir(&temp_dir)
        .env("RTK_DB_PATH", &db_path)
        .args(["memory", "overwrite", "some_key", "some_value"])
        .output()
        .expect("rtk not found");
    assert!(overwrite_out.status.success());

    let doctor_out = rtk_bin()
        .current_dir(&temp_dir)
        .env("RTK_DB_PATH", &db_path)
        .args(["memory", "doctor"])
        .output()
        .expect("rtk not found");
    assert!(doctor_out.status.success());
    let doctor_str = String::from_utf8_lossy(&doctor_out.stdout);
    assert!(doctor_str.contains("Memory is healthy"));

    // 5. Test graph export & audit graph commands
    let export_out = rtk_bin()
        .current_dir(&temp_dir)
        .env("RTK_INDEX_DB_PATH", &index_db_path)
        .args([
            "graph",
            "export",
            "--format",
            "obsidian",
            "--output",
            "obsidian_out/",
        ])
        .output()
        .expect("rtk not found");
    assert!(export_out.status.success());
    let export_str = String::from_utf8_lossy(&export_out.stdout);
    assert!(export_str.contains("Obsidian graph exported successfully"));

    let audit_out = rtk_bin()
        .current_dir(&temp_dir)
        .env("RTK_INDEX_DB_PATH", &index_db_path)
        .args(["audit", "graph"])
        .output()
        .expect("rtk not found");
    assert!(audit_out.status.success());
    let audit_str = String::from_utf8_lossy(&audit_out.stdout);
    assert!(audit_str.contains("RTK Code Intelligence Graph Audit Report"));
    assert!(audit_str.contains("Total Symbols:      1"));

    std::fs::remove_dir_all(&temp_dir).unwrap();
}
