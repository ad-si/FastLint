//! End-to-end tests against the built `fastlint` binary.

use std::fs;
use std::path::Path;
use std::process::Command;

use serde_json::Value;
use tempfile::TempDir;

/// Cargo provides the absolute path to the compiled binary at test time.
const BIN: &str = env!("CARGO_BIN_EXE_fastlint");

fn run(args: &[&str]) -> (i32, String, String) {
  let out = Command::new(BIN)
    .args(args)
    .output()
    .expect("running fastlint");
  (
    out.status.code().unwrap_or(-1),
    String::from_utf8_lossy(&out.stdout).into_owned(),
    String::from_utf8_lossy(&out.stderr).into_owned(),
  )
}

fn make_repo() -> TempDir {
  let dir = tempfile::tempdir().expect("tempdir");
  // Initialize as git repo so `.gitignore` rules apply.
  fs::create_dir(dir.path().join(".git")).unwrap();
  dir
}

fn write(dir: &Path, name: &str, body: &str) {
  fs::write(dir.join(name), body).unwrap();
}

#[test]
fn clean_directory_exits_zero() {
  let dir = make_repo();
  write(dir.path(), "main.rs", "fn main() {}\n");
  let (code, stdout, _) = run(&["--no-color", dir.path().to_str().unwrap()]);
  assert_eq!(code, 0, "stdout was: {stdout}");
  assert!(stdout.contains("No issues found"));
}

#[test]
fn merge_conflict_yields_error_exit() {
  let dir = make_repo();
  write(
    dir.path(),
    "x.txt",
    "<<<<<<< HEAD\nlhs\n=======\nrhs\n>>>>>>> branch\n",
  );
  let (code, stdout, _) = run(&["--no-color", dir.path().to_str().unwrap()]);
  assert_eq!(code, 1, "stdout was: {stdout}");
  assert!(stdout.contains("merge-conflict"));
  assert!(stdout.contains("error"));
}

#[test]
fn typos_emit_warnings_but_zero_exit() {
  let dir = make_repo();
  write(dir.path(), "note.md", "teh quick brown fox\n");
  let (code, stdout, _) = run(&["--no-color", dir.path().to_str().unwrap()]);
  assert_eq!(code, 0, "stdout was: {stdout}");
  assert!(stdout.contains("typos"));
  assert!(stdout.contains("the"));
}

#[test]
fn json_output_is_valid_array() {
  let dir = make_repo();
  write(
    dir.path(),
    "x.txt",
    "<<<<<<< HEAD\nlhs\n=======\nrhs\n>>>>>>> branch\ntrailing  \n",
  );
  let (_, stdout, _) = run(&["--json", dir.path().to_str().unwrap()]);
  let parsed: Value =
    serde_json::from_str(stdout.trim()).expect("stdout was not valid JSON");
  let arr = parsed.as_array().expect("expected JSON array");
  assert!(!arr.is_empty());
  let rules: Vec<&str> =
    arr.iter().map(|d| d["rule"].as_str().unwrap()).collect();
  assert!(rules.contains(&"merge-conflict"));
  assert!(rules.contains(&"trailing-whitespace"));
}

#[test]
fn json_for_clean_dir_is_empty_array() {
  let dir = make_repo();
  write(dir.path(), "main.rs", "fn main() {}\n");
  let (code, stdout, _) = run(&["--json", dir.path().to_str().unwrap()]);
  assert_eq!(code, 0);
  let parsed: Value = serde_json::from_str(stdout.trim()).unwrap();
  assert_eq!(parsed.as_array().unwrap().len(), 0);
}

#[test]
fn nonexistent_path_exits_with_usage_error() {
  let (code, _, stderr) = run(&["/definitely/does/not/exist"]);
  assert_eq!(code, 2);
  assert!(stderr.contains("does not exist"));
}

#[test]
fn respects_gitignore() {
  let dir = make_repo();
  write(dir.path(), ".gitignore", "ignored.txt\n");
  write(dir.path(), "ignored.txt", "teh typo here\n");
  write(dir.path(), "checked.txt", "all good\n");
  let (code, stdout, _) = run(&["--json", dir.path().to_str().unwrap()]);
  assert_eq!(code, 0);
  let parsed: Value = serde_json::from_str(stdout.trim()).unwrap();
  // The ignored file's typo must not appear.
  for d in parsed.as_array().unwrap() {
    let file = d["file"].as_str().unwrap();
    assert!(!file.ends_with("ignored.txt"), "leaked: {file}");
  }
}
