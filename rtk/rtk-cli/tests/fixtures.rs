use std::fs;
use std::path::{Path, PathBuf};

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

fn read_fixture(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
        .replace("\r\n", "\n")
}

#[test]
fn test_golden_fixtures_git_status() {
    let fixtures_dir = fixtures_dir();
    let input = read_fixture(&fixtures_dir.join("git_status/input.txt"));
    let expected = read_fixture(&fixtures_dir.join("git_status/expected.txt"));

    let filtered = rtk_filters::git_status::filter(&input);
    assert_eq!(filtered.trim(), expected.trim());
}

#[test]
fn test_golden_fixtures_git_diff() {
    let fixtures_dir = fixtures_dir();
    let input = read_fixture(&fixtures_dir.join("git_diff/input.txt"));
    let expected = read_fixture(&fixtures_dir.join("git_diff/expected.txt"));

    let filtered = rtk_filters::git_diff::filter(&input);
    assert_eq!(filtered.trim(), expected.trim());
}

#[test]
fn test_golden_fixtures_cargo_build() {
    let fixtures_dir = fixtures_dir();
    let input = read_fixture(&fixtures_dir.join("cargo_build/input.txt"));
    let expected = read_fixture(&fixtures_dir.join("cargo_build/expected.txt"));

    let filtered = rtk_filters::cargo_build::filter(&input);
    assert_eq!(filtered.trim(), expected.trim());
}

#[test]
fn test_golden_fixtures_cargo_test() {
    let fixtures_dir = fixtures_dir();
    let input = read_fixture(&fixtures_dir.join("cargo_test/input.txt"));
    let expected = read_fixture(&fixtures_dir.join("cargo_test/expected.txt"));

    let filtered = rtk_filters::cargo_test::filter(&input);
    assert_eq!(filtered.trim(), expected.trim());
}
