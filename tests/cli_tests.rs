use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_version_command() {
    let mut cmd = Command::cargo_bin("rex").unwrap();
    cmd.arg("version")
        .assert()
        .success()
        .stdout(predicate::str::contains("0.1.0"));
}

#[test]
fn test_help_command() {
    let mut cmd = Command::cargo_bin("rex").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Redmine Extension"));
}

#[test]
fn test_init_command() {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("rex").unwrap();

    cmd.arg("init")
        .current_dir(&temp_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Created"));

    let config_path = temp_dir.path().join(".extensions.yml");
    assert!(config_path.exists());

    let content = fs::read_to_string(&config_path).unwrap();
    assert!(content.contains("plugins:"));
    assert!(content.contains("themes:"));
}

#[test]
fn test_state_command_no_lock() {
    let temp_dir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("rex").unwrap();
    cmd.arg("state")
        .current_dir(&temp_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("No extensions installed"));
}

#[test]
fn test_init_already_exists() {
    let temp_dir = TempDir::new().unwrap();

    // First init
    let mut cmd = Command::cargo_bin("rex").unwrap();
    cmd.arg("init").current_dir(&temp_dir).assert().success();

    // Second init should indicate file already exists
    let mut cmd = Command::cargo_bin("rex").unwrap();
    cmd.arg("init")
        .current_dir(&temp_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("already exists"));
}

#[test]
fn test_install_command_no_config() {
    let temp_dir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("rex").unwrap();
    cmd.arg("install")
        .current_dir(&temp_dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("ConfigNotFound"));
}
