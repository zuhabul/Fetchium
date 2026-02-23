// E2E tests: basic CLI behavior (version, help, doctor, unknown commands).

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn cli_version_prints_version_string() {
    Command::cargo_bin("hsx")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("hsx"));
}

#[test]
fn cli_help_shows_main_commands() {
    Command::cargo_bin("hsx")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("search"))
        .stdout(predicate::str::contains("fetch"))
        .stdout(predicate::str::contains("doctor"));
}

#[test]
fn cli_no_args_shows_error_or_help() {
    let output = Command::cargo_bin("hsx").unwrap().output().unwrap();
    // Should either fail with a helpful message, or succeed with usage info
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        combined.contains("Usage") || combined.contains("USAGE") || combined.contains("hsx")
            || combined.contains("search"),
        "No-args invocation should show usage or help"
    );
}

#[test]
fn cli_doctor_runs_without_crash() {
    Command::cargo_bin("hsx")
        .unwrap()
        .arg("doctor")
        .assert()
        .success();
}

#[test]
fn cli_unknown_subcommand_shows_error() {
    Command::cargo_bin("hsx")
        .unwrap()
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
    Command::cargo_bin("hsx")
        .unwrap()
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
    Command::cargo_bin("hsx")
        .unwrap()
        .args(["search", "--help"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("format")
                .or(predicate::str::contains("output"))
                .or(predicate::str::contains("QUERY")),
        );
}
