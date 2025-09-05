//! CLI tests focused on UX, exit codes, and messages.
use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;
use tempfile::TempDir;
use std::fs;

fn init_logging() {
    let _ = env_logger::builder().is_test(true).try_init();
}

/// Help output should be successful and contain usage text.
#[test]
fn help_shows_usage() {
    init_logging();
    let mut cmd = Command::cargo_bin("coreutils-switch").unwrap();
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("oxidizr-arch style coreutils switching"));
}

/// `check` should report compatible on scaffold (no-op System).
#[test]
fn check_reports_compatible_on_scaffold() {
    init_logging();
    let mut cmd = Command::cargo_bin("coreutils-switch").unwrap();
    cmd.arg("check");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Compatible: true"));
}

/// list-targets accepts a custom bin dir and runs successfully.
#[test]
fn list_targets_accepts_custom_bin_dir() {
    init_logging();
    let td = TempDir::new().unwrap();
    // create a fake replacement binary to make sure the code path runs (though default System returns empty list)
    let fake = td.path().join("date");
    fs::write(&fake, b"bin").unwrap();

    let mut cmd = Command::cargo_bin("coreutils-switch").unwrap();
    cmd.args(["--bin-dir", td.path().to_string_lossy().as_ref(), "list-targets"]);
    cmd.assert().success();
}

/// enable must fail without root and produce a clear error message.
#[test]
fn enable_fails_without_root_with_clear_message() {
    init_logging();
    let mut cmd = Command::cargo_bin("coreutils-switch").unwrap();
    cmd.args(["--assume-yes", "enable"]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("must be run as root"));
}
