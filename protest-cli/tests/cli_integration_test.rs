#[allow(deprecated)]
use assert_cmd::{Command, cargo::cargo_bin};
use predicates::prelude::*;
use std::fs;
use std::process;
use tempfile::TempDir;

/// Helper to create a test failure file
fn create_test_failure(dir: &std::path::Path, test_name: &str, seed: u64, error: &str) {
    let test_dir = dir.join(test_name);
    fs::create_dir_all(&test_dir).unwrap();

    let failure_content = format!(
        r#"{{
  "seed": {},
  "input": "test_input_{}",
  "error_message": "{}",
  "timestamp": {{
    "secs_since_epoch": 1735833600,
    "nanos_since_epoch": 0
  }},
  "shrink_steps": 5,
  "metadata": {{}}
}}"#,
        seed, seed, error
    );

    let failure_file = test_dir.join(format!("failure_seed_{}.json", seed));
    fs::write(failure_file, failure_content).unwrap();
}

#[test]
fn test_list_no_failures() {
    let temp_dir = TempDir::new().unwrap();

    let mut cmd = Command::from_std(process::Command::new(cargo_bin!("protest")));
    cmd.arg("--dir").arg(temp_dir.path()).arg("list");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("No saved failures found"));
}

#[test]
fn test_list_with_failures() {
    let temp_dir = TempDir::new().unwrap();

    // Create some test failures
    create_test_failure(temp_dir.path(), "test_one", 12345, "Error one");
    create_test_failure(temp_dir.path(), "test_two", 67890, "Error two");

    let mut cmd = Command::from_std(process::Command::new(cargo_bin!("protest")));
    cmd.arg("--dir").arg(temp_dir.path()).arg("list");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Found 2 test(s) with failures"))
        .stdout(predicate::str::contains("test_one"))
        .stdout(predicate::str::contains("test_two"))
        .stdout(predicate::str::contains("1 failure"));
}

#[test]
fn test_list_verbose() {
    let temp_dir = TempDir::new().unwrap();

    create_test_failure(temp_dir.path(), "verbose_test", 99999, "Verbose error");

    let mut cmd = Command::from_std(process::Command::new(cargo_bin!("protest")));
    cmd.arg("--dir")
        .arg(temp_dir.path())
        .arg("list")
        .arg("--verbose");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("verbose_test"))
        .stdout(predicate::str::contains("seed=99999"))
        .stdout(predicate::str::contains("Error: Verbose error"))
        .stdout(predicate::str::contains("Shrink steps: 5"));
}

#[test]
fn test_show_command() {
    let temp_dir = TempDir::new().unwrap();

    create_test_failure(temp_dir.path(), "show_test", 11111, "Show error");

    let mut cmd = Command::from_std(process::Command::new(cargo_bin!("protest")));
    cmd.arg("--dir")
        .arg(temp_dir.path())
        .arg("show")
        .arg("show_test");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Failures for test 'show_test'"))
        .stdout(predicate::str::contains("Seed: 11111"))
        .stdout(predicate::str::contains("Input: test_input_11111"))
        .stdout(predicate::str::contains("Error: Show error"))
        .stdout(predicate::str::contains("Shrink steps: 5"))
        .stdout(predicate::str::contains("Reproduce:"));
}

#[test]
fn test_show_nonexistent_test() {
    let temp_dir = TempDir::new().unwrap();

    let mut cmd = Command::from_std(process::Command::new(cargo_bin!("protest")));
    cmd.arg("--dir")
        .arg(temp_dir.path())
        .arg("show")
        .arg("nonexistent");

    cmd.assert().success().stdout(predicate::str::contains(
        "No failures found for test 'nonexistent'",
    ));
}

#[test]
fn test_stats_command() {
    let temp_dir = TempDir::new().unwrap();

    create_test_failure(temp_dir.path(), "stats_test_1", 11111, "Error 1");
    create_test_failure(temp_dir.path(), "stats_test_1", 22222, "Error 2");
    create_test_failure(temp_dir.path(), "stats_test_2", 33333, "Error 3");

    let mut cmd = Command::from_std(process::Command::new(cargo_bin!("protest")));
    cmd.arg("--dir").arg(temp_dir.path()).arg("stats");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Failure Statistics"))
        .stdout(predicate::str::contains("Total tests with failures: 2"))
        .stdout(predicate::str::contains("Total failures: 3"))
        .stdout(predicate::str::contains("Average failures per test: 1.5"))
        .stdout(predicate::str::contains("Total shrink steps: 15"));
}

#[test]
fn test_stats_no_failures() {
    let temp_dir = TempDir::new().unwrap();

    let mut cmd = Command::from_std(process::Command::new(cargo_bin!("protest")));
    cmd.arg("--dir").arg(temp_dir.path()).arg("stats");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("No saved failures found"));
}

#[test]
fn test_clean_specific_seed() {
    let temp_dir = TempDir::new().unwrap();

    create_test_failure(temp_dir.path(), "clean_test", 12345, "Error");
    create_test_failure(temp_dir.path(), "clean_test", 67890, "Error");

    let mut cmd = Command::from_std(process::Command::new(cargo_bin!("protest")));
    cmd.arg("--dir")
        .arg(temp_dir.path())
        .arg("clean")
        .arg("clean_test")
        .arg("--seed")
        .arg("12345")
        .arg("-y");

    cmd.assert().success().stdout(predicate::str::contains(
        "Deleted failure for 'clean_test' with seed 12345",
    ));

    // Verify only one failure remains
    let failures_dir = temp_dir.path().join("clean_test");
    let files: Vec<_> = fs::read_dir(&failures_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    assert_eq!(files.len(), 1);
}

#[test]
fn test_clean_all_for_test() {
    let temp_dir = TempDir::new().unwrap();

    create_test_failure(temp_dir.path(), "clean_all_test", 11111, "Error 1");
    create_test_failure(temp_dir.path(), "clean_all_test", 22222, "Error 2");

    let mut cmd = Command::from_std(process::Command::new(cargo_bin!("protest")));
    cmd.arg("--dir")
        .arg(temp_dir.path())
        .arg("clean")
        .arg("clean_all_test")
        .arg("-y");

    cmd.assert().success().stdout(predicate::str::contains(
        "Deleted 2 failure(s) for 'clean_all_test'",
    ));

    // Verify all failures are gone
    let failures_dir = temp_dir.path().join("clean_all_test");
    let files: Vec<_> = fs::read_dir(&failures_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    assert_eq!(files.len(), 0);
}

#[test]
fn test_clean_all_tests() {
    let temp_dir = TempDir::new().unwrap();

    create_test_failure(temp_dir.path(), "test_a", 11111, "Error");
    create_test_failure(temp_dir.path(), "test_b", 22222, "Error");
    create_test_failure(temp_dir.path(), "test_c", 33333, "Error");

    let mut cmd = Command::from_std(process::Command::new(cargo_bin!("protest")));
    cmd.arg("--dir").arg(temp_dir.path()).arg("clean").arg("-y");

    cmd.assert().success().stdout(predicate::str::contains(
        "Deleted 3 failure(s) across 3 test(s)",
    ));
}

#[test]
fn test_generate_command_single_test() {
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().join("regressions");

    create_test_failure(temp_dir.path(), "gen_test", 12345, "Test error");
    create_test_failure(temp_dir.path(), "gen_test", 67890, "Another error");

    let mut cmd = Command::from_std(process::Command::new(cargo_bin!("protest")));
    cmd.arg("--dir")
        .arg(temp_dir.path())
        .arg("generate")
        .arg("gen_test")
        .arg("-y")
        .arg("--output")
        .arg(&output_dir);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            "Generated regression tests for 'gen_test'",
        ))
        .stdout(predicate::str::contains("Test count: 2"));

    // Verify file was created
    let expected_file = output_dir.join("gen_test_regressions.rs");
    assert!(expected_file.exists());

    // Verify file content
    let content = fs::read_to_string(&expected_file).unwrap();
    assert!(content.contains("regression_gen_test_seed_12345"));
    assert!(content.contains("regression_gen_test_seed_67890"));
    assert!(content.contains("Test error"));
}

#[test]
fn test_generate_command_all_tests() {
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().join("regressions");

    create_test_failure(temp_dir.path(), "test_a", 11111, "Error A");
    create_test_failure(temp_dir.path(), "test_b", 22222, "Error B");

    let mut cmd = Command::from_std(process::Command::new(cargo_bin!("protest")));
    cmd.arg("--dir")
        .arg(temp_dir.path())
        .arg("generate")
        .arg("-y")
        .arg("--output")
        .arg(&output_dir);

    cmd.assert().success().stdout(predicate::str::contains(
        "Generated regression tests for 2 test(s)",
    ));

    // Verify files were created
    assert!(output_dir.join("test_a_regressions.rs").exists());
    assert!(output_dir.join("test_b_regressions.rs").exists());
}

#[test]
fn test_generate_command_no_failures() {
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().join("regressions");

    let mut cmd = Command::from_std(process::Command::new(cargo_bin!("protest")));
    cmd.arg("--dir")
        .arg(temp_dir.path())
        .arg("generate")
        .arg("-y")
        .arg("--output")
        .arg(&output_dir);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("No saved failures found"));
}

#[test]
fn test_help_command() {
    let mut cmd = Command::from_std(process::Command::new(cargo_bin!("protest")));
    cmd.arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            "Manage Protest property test failures",
        ))
        .stdout(predicate::str::contains("Commands:"))
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("show"))
        .stdout(predicate::str::contains("clean"))
        .stdout(predicate::str::contains("stats"));
}

#[test]
fn test_version_command() {
    let mut cmd = Command::from_std(process::Command::new(cargo_bin!("protest")));
    cmd.arg("--version");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("protest"));
}

#[test]
fn test_custom_dir_option() {
    let temp_dir = TempDir::new().unwrap();
    let custom_dir = temp_dir.path().join("custom_failures");
    fs::create_dir_all(&custom_dir).unwrap();

    create_test_failure(&custom_dir, "custom_test", 99999, "Custom error");

    let mut cmd = Command::from_std(process::Command::new(cargo_bin!("protest")));
    cmd.arg("--dir").arg(&custom_dir).arg("list");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("custom_test"));
}
