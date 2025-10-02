use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_help_command() {
    let mut cmd = Command::cargo_bin("loadster").unwrap();
    cmd.arg("--help");

    cmd.assert().success().stdout(predicate::str::contains(
        "A lightweight HTTP load testing tool",
    ));
}

#[test]
fn test_version_command() {
    let mut cmd = Command::cargo_bin("loadster").unwrap();
    cmd.arg("--version");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("loadster"));
}

#[test]
fn test_missing_url_argument() {
    let mut cmd = Command::cargo_bin("loadster").unwrap();

    cmd.assert().failure().stderr(predicate::str::contains(
        "required arguments were not provided",
    ));
}

#[test]
fn test_invalid_request_count() {
    let mut cmd = Command::cargo_bin("loadster").unwrap();
    cmd.args(&["http://example.com", "-n", "invalid"]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

#[test]
fn test_invalid_concurrency() {
    let mut cmd = Command::cargo_bin("loadster").unwrap();
    cmd.args(&["http://example.com", "-c", "invalid"]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

#[test]
#[ignore] // Ignored by default as it makes real HTTP requests
fn test_basic_load_test() {
    let mut cmd = Command::cargo_bin("loadster").unwrap();
    cmd.args(&["https://httpbin.org/get", "-n", "5", "-c", "2"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Results:"))
        .stdout(predicate::str::contains("Successful:"))
        .stdout(predicate::str::contains("Latency:"));
}

#[test]
#[ignore] // Ignored by default as it makes real HTTP requests
fn test_json_output() {
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("report.json");

    let mut cmd = Command::cargo_bin("loadster").unwrap();
    cmd.args(&[
        "https://httpbin.org/get",
        "-n",
        "5",
        "-c",
        "2",
        "-o",
        output_path.to_str().unwrap(),
    ]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Report saved to"));

    // Verify JSON file was created
    assert!(output_path.exists());

    // Verify JSON is valid
    let content = fs::read_to_string(&output_path).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();

    // Check required fields
    assert!(json.get("url").is_some());
    assert!(json.get("total_requests").is_some());
    assert!(json.get("successful").is_some());
    assert!(json.get("failed").is_some());
    assert!(json.get("latency").is_some());

    let latency = json.get("latency").unwrap();
    assert!(latency.get("avg_ms").is_some());
    assert!(latency.get("p50_ms").is_some());
    assert!(latency.get("p95_ms").is_some());
    assert!(latency.get("p99_ms").is_some());
}

#[test]
#[ignore] // Ignored by default as it makes real HTTP requests
fn test_concurrent_requests() {
    let mut cmd = Command::cargo_bin("loadster").unwrap();
    cmd.args(&["https://httpbin.org/delay/1", "-n", "10", "-c", "5"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Successful:"));
}

#[test]
fn test_custom_requests_flag() {
    let mut cmd = Command::cargo_bin("loadster").unwrap();
    cmd.args(&["http://example.com", "--requests", "50"]);

    // This will fail to connect but should parse args correctly
    cmd.assert().code(predicate::ne(2)); // Not an argument parsing error
}

#[test]
fn test_custom_concurrency_flag() {
    let mut cmd = Command::cargo_bin("loadster").unwrap();
    cmd.args(&["http://example.com", "--concurrency", "20"]);

    // This will fail to connect but should parse args correctly
    cmd.assert().code(predicate::ne(2)); // Not an argument parsing error
}

#[test]
fn test_short_flags() {
    let mut cmd = Command::cargo_bin("loadster").unwrap();
    cmd.args(&["http://example.com", "-n", "100", "-c", "10"]);

    // This will fail to connect but should parse args correctly
    cmd.assert().code(predicate::ne(2)); // Not an argument parsing error
}

#[test]
#[ignore] // Ignored by default as it makes real HTTP requests
fn test_failed_requests_handling() {
    let mut cmd = Command::cargo_bin("loadster").unwrap();
    cmd.args(&["https://httpbin.org/status/500", "-n", "5", "-c", "2"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Results:"));
}

#[test]
fn test_output_file_path_validation() {
    let mut cmd = Command::cargo_bin("loadster").unwrap();
    cmd.args(&[
        "http://example.com",
        "-o",
        "/invalid/path/that/does/not/exist/report.json",
    ]);

    // Should handle invalid path gracefully
    cmd.assert().code(predicate::ne(2)); // Not an argument parsing error
}
