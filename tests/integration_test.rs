#![allow(deprecated)]

use assert_cmd::Command;
use predicates::prelude::*;
use std::process::Command as StdCommand;
use tempfile::TempDir;

#[test]
fn test_help_command() {
    let mut cmd = Command::cargo_bin("git-trek").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Navigate git history"));
}

#[test]
fn test_version_command() {
    let mut cmd = Command::cargo_bin("git-trek").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("git-trek"));
}

#[test]
fn test_dry_run_flow() {
    // Setup temp git repo
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    // git init
    let status = StdCommand::new("git")
        .current_dir(path)
        .arg("init")
        .status()
        .unwrap();
    assert!(status.success());

    // git config (local)
    StdCommand::new("git")
        .current_dir(path)
        .args(["config", "user.email", "test@example.com"])
        .status().unwrap();
    StdCommand::new("git")
        .current_dir(path)
        .args(["config", "user.name", "Test User"])
        .status().unwrap();

    // git commit --allow-empty
    let status = StdCommand::new("git")
        .current_dir(path)
        .args(["commit", "--allow-empty", "-m", "Initial commit"])
        .status()
        .unwrap();
    assert!(status.success());

    // Run git-trek --dry-run
    let mut cmd = Command::cargo_bin("git-trek").unwrap();
    cmd.current_dir(path)
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("App initialized and rendered successfully"));
}
