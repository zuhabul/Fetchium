use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

fn fetchium() -> Command {
    Command::new(env!("CARGO_BIN_EXE_fetchium"))
}

#[test]
fn cli_version() {
    fetchium()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("fetchium"));
}

#[test]
fn cli_help() {
    fetchium()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("universal retrieval layer"))
        .stdout(predicate::str::contains("search"))
        .stdout(predicate::str::contains("fetch"))
        .stdout(predicate::str::contains("doctor"));
}

#[test]
fn cli_search_help() {
    fetchium()
        .args(["search", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Search query"));
}

#[test]
fn cli_unknown_command() {
    fetchium().arg("nonexistent").assert().failure();
}

#[test]
fn cli_doctor_runs() {
    fetchium()
        .arg("doctor")
        .assert()
        .success()
        .stdout(predicate::str::contains("Fetchium Doctor"));
}

#[test]
fn doctor_shows_system_info() {
    fetchium()
        .arg("doctor")
        .assert()
        .success()
        .stdout(predicate::str::contains("System Information"))
        .stdout(predicate::str::contains("CPU"))
        .stdout(predicate::str::contains("RAM"))
        .stdout(predicate::str::contains("Resource Tier"))
        .stdout(predicate::str::contains("Configuration"))
        .stdout(predicate::str::contains("External Tools"));
}
