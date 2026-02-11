use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[allow(deprecated)]
fn cmd() -> Command {
    Command::cargo_bin("skill-issue").unwrap()
}

#[test]
fn test_clean_skill_exits_zero() {
    cmd()
        .arg("tests/fixtures/clean_skill")
        .arg("--no-color")
        .assert()
        .success()
        .stdout(predicate::str::contains("No issues found"));
}

#[test]
fn test_dangerous_skill_exits_two() {
    cmd()
        .arg("tests/fixtures/dangerous_skill")
        .arg("--no-color")
        .assert()
        .code(2)
        .stdout(predicate::str::contains("error(s)"));
}

#[test]
fn test_json_output_is_valid() {
    let output = cmd()
        .arg("tests/fixtures/dangerous_skill")
        .arg("--no-color")
        .arg("-f")
        .arg("json")
        .output()
        .unwrap();

    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("output should be valid JSON");
    assert!(json["findings"].is_array());
    assert!(json["summary"]["total"].as_u64().unwrap() > 0);
    assert_eq!(json["version"].as_str().unwrap(), env!("CARGO_PKG_VERSION"));
}

#[test]
fn test_sarif_output_is_valid() {
    let output = cmd()
        .arg("tests/fixtures/dangerous_skill")
        .arg("--no-color")
        .arg("-f")
        .arg("sarif")
        .output()
        .unwrap();

    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("output should be valid JSON");
    assert_eq!(json["version"].as_str().unwrap(), "2.1.0");
    assert!(json["runs"][0]["results"].is_array());
    assert!(json["runs"][0]["tool"]["driver"]["name"].as_str().unwrap() == "skill-issue");
}

#[test]
fn test_severity_filter() {
    // Only errors
    let output = cmd()
        .arg("tests/fixtures/dangerous_skill")
        .arg("--no-color")
        .arg("-s")
        .arg("error")
        .arg("-f")
        .arg("json")
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let findings = json["findings"].as_array().unwrap();
    for f in findings {
        assert_eq!(f["severity"].as_str().unwrap(), "error");
    }
}

#[test]
fn test_ignore_rule() {
    let output = cmd()
        .arg("tests/fixtures/dangerous_skill")
        .arg("--no-color")
        .arg("--ignore")
        .arg("SL-INJ-001")
        .arg("-f")
        .arg("json")
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let findings = json["findings"].as_array().unwrap();
    for f in findings {
        assert_ne!(f["rule_id"].as_str().unwrap(), "SL-INJ-001");
    }
}

#[test]
fn test_nonexistent_path() {
    cmd()
        .arg("/nonexistent/path")
        .arg("--no-color")
        .assert()
        .code(2)
        .stderr(predicate::str::contains("does not exist"));
}

#[test]
fn test_quiet_mode_clean() {
    let output = cmd()
        .arg("tests/fixtures/clean_skill")
        .arg("--no-color")
        .arg("-q")
        .output()
        .unwrap();

    assert!(output.stdout.is_empty() || output.stdout == b"\n");
}

#[test]
fn test_error_on_warning() {
    // With --error-on warning, warnings should cause exit code 2
    cmd()
        .arg("tests/fixtures/dangerous_skill")
        .arg("--no-color")
        .arg("--error-on")
        .arg("warning")
        .assert()
        .code(2);
}

#[test]
fn test_config_file() {
    let dir = TempDir::new().unwrap();
    let skill_dir = dir.path().join("skill");
    fs::create_dir(&skill_dir).unwrap();

    // Create a skill file with a finding
    fs::write(skill_dir.join("README.md"), "eval('dangerous code')\n").unwrap();

    // Create config that ignores the rule
    fs::write(
        skill_dir.join(".skill-issue.toml"),
        r#"
[settings]
ignore = ["SL-EXEC-002"]
"#,
    )
    .unwrap();

    let output = cmd()
        .arg(skill_dir.to_str().unwrap())
        .arg("--no-color")
        .arg("-f")
        .arg("json")
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let findings = json["findings"].as_array().unwrap();
    for f in findings {
        assert_ne!(f["rule_id"].as_str().unwrap(), "SL-EXEC-002");
    }
}

#[test]
fn test_version_flag() {
    cmd()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("skill-issue"));
}

#[test]
fn test_help_flag() {
    cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Static security analyzer"));
}

#[test]
fn test_scan_performance() {
    use std::time::Instant;

    let start = Instant::now();
    cmd()
        .arg("tests/fixtures/dangerous_skill")
        .arg("--no-color")
        .arg("-f")
        .arg("json")
        .output()
        .unwrap();
    let elapsed = start.elapsed();

    // Should complete in under 5 seconds (generous for CI)
    assert!(elapsed.as_secs() < 5, "Scan took too long: {:?}", elapsed);
}
