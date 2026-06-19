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
    let rewrite_out = rtk_bin()
        .args(["rewrite", "cargo test"])
        .output()
        .unwrap();
    
    assert_eq!(rewrite_out.status.code(), Some(0));
    let rewritten_cmd = String::from_utf8_lossy(&rewrite_out.stdout).trim().to_string();
    assert_eq!(rewritten_cmd, "rtk cargo test");

    // 2. Execute the proxied command
    let run_out = rtk_bin()
        .args(["cargo", "test"])
        .output()
        .unwrap();

    assert!(run_out.status.success() || run_out.status.code() == Some(101));
    let stdout_str = String::from_utf8_lossy(&run_out.stdout);
    
    // 3. Verify output contains standard RTK wrappers or cargo output
    assert!(stdout_str.contains("cargo") || stdout_str.contains("RTK") || stdout_str.contains("test"));
    
    // We can also verify that a local SQLite DB was hit, but since
    // tests run concurrently, checking .rtk dir requires creating a temp dir.
    let timestamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis();
    let temp_dir = std::env::temp_dir().join(format!("rtk_e2e_{timestamp}"));
    fs::create_dir_all(&temp_dir).unwrap();

    let proxied_run = rtk_bin()
        .current_dir(&temp_dir)
        .args(["cargo", "test"])
        .output()
        .unwrap();
    
    assert!(proxied_run.status.success());
    
    // Verify that .rtk/logs.db or project_memory_fts.db was created
    // The DB logic usually writes to `.rtk/project_memory_fts.db` and `.rtk/logs.db`
    assert!(temp_dir.join(".rtk").exists());
    
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
