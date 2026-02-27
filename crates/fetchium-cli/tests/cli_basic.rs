// E2E tests: basic CLI behavior (version, help, doctor, unknown commands).

use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

fn fetchium() -> Command {
    Command::new(env!("CARGO_BIN_EXE_fetchium"))
}

#[test]
fn cli_version_prints_version_string() {
    fetchium()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("fetchium"));
}

#[test]
fn cli_help_shows_main_commands() {
    fetchium()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("search"))
        .stdout(predicate::str::contains("fetch"))
        .stdout(predicate::str::contains("doctor"));
}

#[test]
fn cli_no_args_shows_error_or_help() {
    let output = fetchium().output().unwrap();
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        combined.contains("Usage")
            || combined.contains("USAGE")
            || combined.contains("fetchium")
            || combined.contains("search"),
        "No-args invocation should show usage or help"
    );
}

#[test]
fn cli_doctor_runs_without_crash() {
    fetchium().arg("doctor").assert().success();
}

#[test]
fn cli_unknown_subcommand_shows_error() {
    fetchium()
        .arg("this-command-does-not-exist")
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("error")
                .or(predicate::str::contains("unrecognized"))
                .or(predicate::str::contains("unknown")),
        );
}

#[test]
fn cli_fetch_invalid_url_shows_error() {
    fetchium()
        .args(["fetch", "not-a-valid-url"])
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("Invalid")
                .or(predicate::str::contains("error"))
                .or(predicate::str::contains("URL")),
        );
}

#[test]
fn cli_search_help_shows_flags() {
    fetchium()
        .args(["search", "--help"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("format")
                .or(predicate::str::contains("output"))
                .or(predicate::str::contains("QUERY")),
        );
}
