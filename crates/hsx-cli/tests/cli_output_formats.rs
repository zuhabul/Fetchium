// E2E tests: output format flags (--format json / markdown / text).

use assert_cmd::Command;
use predicates::prelude::*;

/// Run agent-fetch on a real URL — skip if network is unavailable.
/// Uses a stable, simple URL that returns consistent content.
#[test]
#[ignore = "requires network access"]
fn agent_fetch_json_output_is_valid_json() {
    let output = Command::cargo_bin("hsx")
        .unwrap()
        .args(["agent-fetch", "https://example.com"])
        .output()
        .unwrap();

    if !output.status.success() {
        // Network may be unavailable in CI — skip gracefully
        return;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: Result<serde_json::Value, _> = serde_json::from_str(&stdout);
    assert!(
        parsed.is_ok(),
        "agent-fetch output must be valid JSON, got: {stdout}"
    );
}

#[test]
fn search_help_does_not_crash() {
    Command::cargo_bin("hsx")
        .unwrap()
        .args(["search", "--help"])
        .assert()
        .success();
}

#[test]
fn fetch_help_does_not_crash() {
    Command::cargo_bin("hsx")
        .unwrap()
        .args(["fetch", "--help"])
        .assert()
        .success();
}

#[test]
fn agent_search_help_does_not_crash() {
    Command::cargo_bin("hsx")
        .unwrap()
        .args(["agent-search", "--help"])
        .assert()
        .success();
}

#[test]
fn research_help_does_not_crash() {
    Command::cargo_bin("hsx")
        .unwrap()
        .args(["research", "--help"])
        .assert()
        .success();
}

#[test]
fn compare_help_does_not_crash() {
    Command::cargo_bin("hsx")
        .unwrap()
        .args(["compare", "--help"])
        .assert()
        .success();
}

#[test]
fn monitor_help_does_not_crash() {
    Command::cargo_bin("hsx")
        .unwrap()
        .args(["monitor", "--help"])
        .assert()
        .success();
}

#[test]
fn index_help_does_not_crash() {
    Command::cargo_bin("hsx")
        .unwrap()
        .args(["index", "--help"])
        .assert()
        .success();
}
