use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Basic CLI command tests
#[test]
fn test_version_command() {
    let mut cmd = Command::cargo_bin("rex").unwrap();
    cmd.arg("version")
        .assert()
        .success()
        .stdout(predicate::str::starts_with("rex "));
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
fn test_invalid_command() {
    let mut cmd = Command::cargo_bin("rex").unwrap();
    cmd.arg("invalid_command")
        .assert()
        .failure()
        .stderr(predicate::str::contains("unrecognized subcommand"));
}

/// Configuration and initialization tests
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
    assert!(content.contains("github:"));
    assert!(content.contains("git:"));
    assert!(content.contains("branch:"));
    assert!(content.contains("tag:"));
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

/// State command tests
#[test]
fn test_state_command_no_lock() {
    let temp_dir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("rex").unwrap();
    cmd.arg("state")
        .current_dir(&temp_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("No lock file found"));
}

/// Installation tests
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

#[test]
fn test_install_with_valid_config() {
    let temp_dir = TempDir::new().unwrap();

    // Create a test config with a simple GitHub repository
    let config_content = r#"plugins:
  - name: test_plugin
    github:
      repo: "octocat/Hello-World"
      branch: "master"

themes: []
"#;

    let config_path = temp_dir.path().join(".extensions.yml");
    fs::write(&config_path, config_content).unwrap();

    // Create necessary directories
    fs::create_dir_all(temp_dir.path().join("plugins")).unwrap();
    fs::create_dir_all(temp_dir.path().join("public").join("themes")).unwrap();

    let mut cmd = Command::cargo_bin("rex").unwrap();
    cmd.arg("install")
        .current_dir(&temp_dir)
        .timeout(std::time::Duration::from_secs(60))
        .assert()
        .success()
        .stdout(predicate::str::contains("Installing test_plugin"))
        .stdout(predicate::str::contains("Installed 1 extensions"));

    // Check that lock file was created
    let lock_path = temp_dir.path().join(".extensions.lock");
    assert!(lock_path.exists());

    // Check that plugin directory was created
    let plugin_path = temp_dir.path().join("plugins").join("test_plugin");
    assert!(plugin_path.exists());
}

#[test]
fn test_install_with_git_source() {
    let temp_dir = TempDir::new().unwrap();

    // Create a test config with a Git repository
    let config_content = r#"plugins: []

themes:
  - name: test_theme
    git:
      url: "https://github.com/octocat/Hello-World.git"
      branch: "master"
"#;

    let config_path = temp_dir.path().join(".extensions.yml");
    fs::write(&config_path, config_content).unwrap();

    // Create necessary directories
    fs::create_dir_all(temp_dir.path().join("plugins")).unwrap();
    fs::create_dir_all(temp_dir.path().join("public").join("themes")).unwrap();

    let mut cmd = Command::cargo_bin("rex").unwrap();
    cmd.arg("install")
        .current_dir(&temp_dir)
        .timeout(std::time::Duration::from_secs(60))
        .assert()
        .success()
        .stdout(predicate::str::contains("Installing test_theme"))
        .stdout(predicate::str::contains("Installed 1 extensions"));

    // Check that theme directory was created
    let theme_path = temp_dir.path().join("themes").join("test_theme");
    assert!(theme_path.exists());
}

#[test]
fn test_install_invalid_config_format() {
    let temp_dir = TempDir::new().unwrap();

    // Create an invalid YAML config
    let config_content = "invalid yaml content: [unclosed";
    let config_path = temp_dir.path().join(".extensions.yml");
    fs::write(&config_path, config_content).unwrap();

    let mut cmd = Command::cargo_bin("rex").unwrap();
    cmd.arg("install")
        .current_dir(&temp_dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error"));
}

/// State command with installed extensions
#[test]
fn test_state_with_installed_extensions() {
    let temp_dir = TempDir::new().unwrap();

    // First install some extensions
    let config_content = r#"plugins:
  - name: test_plugin
    github:
      repo: "octocat/Hello-World"
      branch: "master"

themes: []
"#;

    let config_path = temp_dir.path().join(".extensions.yml");
    fs::write(&config_path, config_content).unwrap();

    // Create necessary directories
    fs::create_dir_all(temp_dir.path().join("plugins")).unwrap();
    fs::create_dir_all(temp_dir.path().join("public").join("themes")).unwrap();

    // Install first
    let mut cmd = Command::cargo_bin("rex").unwrap();
    cmd.arg("install")
        .current_dir(&temp_dir)
        .timeout(std::time::Duration::from_secs(60))
        .assert()
        .success();

    // Then check state
    let mut cmd = Command::cargo_bin("rex").unwrap();
    cmd.arg("state")
        .current_dir(&temp_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Rexer: 0.1.0"))
        .stdout(predicate::str::contains("Plugins:"))
        .stdout(predicate::str::contains("test_plugin"));
}

/// Uninstall tests
#[test]
fn test_uninstall_no_lock_file() {
    let temp_dir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("rex").unwrap();
    cmd.arg("uninstall")
        .current_dir(&temp_dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("No lock file found"));
}

#[test]
fn test_uninstall_with_installed_extensions() {
    let temp_dir = TempDir::new().unwrap();

    // First install some extensions
    let config_content = r#"plugins:
  - name: test_plugin
    github:
      repo: "octocat/Hello-World"
      branch: "master"

themes: []
"#;

    let config_path = temp_dir.path().join(".extensions.yml");
    fs::write(&config_path, config_content).unwrap();

    // Create necessary directories
    fs::create_dir_all(temp_dir.path().join("plugins")).unwrap();
    fs::create_dir_all(temp_dir.path().join("public").join("themes")).unwrap();

    // Install first
    let mut cmd = Command::cargo_bin("rex").unwrap();
    cmd.arg("install")
        .current_dir(&temp_dir)
        .timeout(std::time::Duration::from_secs(60))
        .assert()
        .success();

    // Verify plugin exists
    let plugin_path = temp_dir.path().join("plugins").join("test_plugin");
    assert!(plugin_path.exists());

    // Then uninstall
    let mut cmd = Command::cargo_bin("rex").unwrap();
    cmd.arg("uninstall")
        .current_dir(&temp_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Uninstalling test_plugin"))
        .stdout(predicate::str::contains("Uninstalled all extensions"));

    // Verify plugin directory was removed
    assert!(!plugin_path.exists());

    // Verify lock file was removed
    let lock_path = temp_dir.path().join(".extensions.lock");
    assert!(!lock_path.exists());
}

/// Update tests
#[test]
fn test_update_no_lock_file() {
    let temp_dir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("rex").unwrap();
    cmd.arg("update")
        .current_dir(&temp_dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("No lock file found"));
}

#[test]
fn test_update_specific_extension() {
    let temp_dir = TempDir::new().unwrap();

    // First install some extensions
    let config_content = r#"plugins:
  - name: test_plugin
    github:
      repo: "octocat/Hello-World"
      branch: "master"

themes: []
"#;

    let config_path = temp_dir.path().join(".extensions.yml");
    fs::write(&config_path, config_content).unwrap();

    // Create necessary directories
    fs::create_dir_all(temp_dir.path().join("plugins")).unwrap();
    fs::create_dir_all(temp_dir.path().join("public").join("themes")).unwrap();

    // Install first
    let mut cmd = Command::cargo_bin("rex").unwrap();
    cmd.arg("install")
        .current_dir(&temp_dir)
        .timeout(std::time::Duration::from_secs(60))
        .assert()
        .success();

    // Then update specific extension
    let mut cmd = Command::cargo_bin("rex").unwrap();
    cmd.arg("update")
        .arg("test_plugin")
        .current_dir(&temp_dir)
        .timeout(std::time::Duration::from_secs(60))
        .assert()
        .success()
        .stdout(predicate::str::contains("Updating test_plugin"));
}

/// Reinstall tests
#[test]
fn test_reinstall_no_lock_file() {
    let temp_dir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("rex").unwrap();
    cmd.arg("reinstall")
        .arg("test_plugin")
        .current_dir(&temp_dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("No lock file found"));
}

#[test]
fn test_reinstall_nonexistent_extension() {
    let temp_dir = TempDir::new().unwrap();

    // Create empty lock file in correct JSON format
    let lock_content = r#"{"extensions": []}"#;
    let lock_path = temp_dir.path().join(".extensions.lock");
    fs::write(&lock_path, lock_content).unwrap();

    let mut cmd = Command::cargo_bin("rex").unwrap();
    cmd.arg("reinstall")
        .arg("nonexistent_plugin")
        .current_dir(&temp_dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("ExtensionNotFound"));
}

/// Default command behavior (should run install when no subcommand)
#[test]
fn test_default_command_no_config() {
    let temp_dir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("rex").unwrap();
    cmd.current_dir(&temp_dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("ConfigNotFound"));
}

/// Edit command test
#[test]
fn test_edit_command() {
    let temp_dir = TempDir::new().unwrap();

    // Create config file first
    let mut cmd = Command::cargo_bin("rex").unwrap();
    cmd.arg("init").current_dir(&temp_dir).assert().success();

    // Use 'true' command which always succeeds and does nothing
    let mut cmd = Command::cargo_bin("rex").unwrap();
    cmd.arg("edit")
        .env("EDITOR", "true") // true command always succeeds
        .current_dir(&temp_dir)
        .assert()
        .success();
}

/// Verbose and quiet mode tests
#[test]
fn test_verbose_mode() {
    let temp_dir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("rex").unwrap();
    cmd.arg("-v")
        .arg("state")
        .current_dir(&temp_dir)
        .assert()
        .success();
}

#[test]
fn test_quiet_mode() {
    let temp_dir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("rex").unwrap();
    cmd.arg("-q")
        .arg("state")
        .current_dir(&temp_dir)
        .assert()
        .success();
}
