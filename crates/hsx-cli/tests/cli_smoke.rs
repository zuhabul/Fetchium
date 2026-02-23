#![allow(deprecated)]
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn cli_version() {
    Command::cargo_bin("hsx")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("hsx"));
}

#[test]
fn cli_help() {
    Command::cargo_bin("hsx")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("AI-native search"))
        .stdout(predicate::str::contains("search"))
        .stdout(predicate::str::contains("fetch"))
        .stdout(predicate::str::contains("doctor"));
}

#[test]
fn cli_search_help() {
    Command::cargo_bin("hsx")
        .unwrap()
        .args(["search", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Search query"));
}

#[test]
fn cli_unknown_command() {
    Command::cargo_bin("hsx")
        .unwrap()
        .arg("nonexistent")
        .assert()
        .failure();
}

#[test]
fn cli_doctor_runs() {
    Command::cargo_bin("hsx")
        .unwrap()
        .arg("doctor")
        .assert()
        .success()
        .stdout(predicate::str::contains("HyperSearchX Doctor"));
}

#[test]
fn doctor_shows_system_info() {
    Command::cargo_bin("hsx")
        .unwrap()
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
