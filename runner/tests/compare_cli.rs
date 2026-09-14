use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn valid_runner_json_value() -> serde_json::Value {
    serde_json::json!({
        "meta": {
            "schema_version": 2,
            "cpu_pin": 2,
            "warmup": 1,
            "reps": 3,
            "timestamp": "09-14-2026T12-00-00-000",
            "git_sha": "test-sha",
            "compiler": {
                "path": "clang++",
                "version": "test-version"
            },
            "uname": "test-uname",
            "bench": "matmul",
            "compiler_args": ["-O3"],
            "command": ["perflab", "run"],
            "workdir": "/test/workdir",
            "perf_events_requested": null,
            "perf_stat_base_args": null
        },
        "samples": [],
        "summary": {
            "phases_ns": {
                "init": {
                    "median_ns": 100,
                    "min_ns": 90,
                    "max_ns": 110,
                    "spread_percent": 20.0
                },
                "compute": {
                    "median_ns": 1000,
                    "min_ns": 950,
                    "max_ns": 1050,
                    "spread_percent": 10.0
                },
                "teardown": {
                    "median_ns": 50,
                    "min_ns": 45,
                    "max_ns": 55,
                    "spread_percent": 20.0
                }
            },
            "perf": null
        }
    })
}

fn write_json(path: &PathBuf, value: &serde_json::Value) {
    let json = serde_json::to_string(value).expect("failed to serialize integration-test JSON");
    fs::write(path, json).expect("failed to write integration-test JSON");
}

#[test]
fn perflab_cli_help_test() {
    let bin_path = env!("CARGO_BIN_EXE_perflab");
    let output = Command::new(bin_path)
        .arg("--help")
        .output()
        .expect("perflab_cli_help_test - Failed to run perflab binary!");

    assert!(output.status.success());
}

#[test]
fn compare_missing_baseline_file_test() {
    let bin_path = env!("CARGO_BIN_EXE_perflab");
    let missing_baseline =
        PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("perflab-missing-baseline.json");
    let candidate =
        PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("perflab-unused-candidate.json");

    let output = Command::new(bin_path)
        .arg("compare")
        .arg(&missing_baseline)
        .arg(&candidate)
        .output()
        .expect("compare_missing_baseline_file_test - Failed to run perflab binary!");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert_eq!(output.status.code(), Some(1));
    assert!(stderr.contains("perflab-compare-error"));
    assert!(stderr.contains("Baseline"));
    assert!(stderr.contains("Read json file failed"));
    assert!(!stdout.contains("PerfLab compare v0"));
    assert!(!stderr.contains("panicked at"));
}

#[test]
fn compare_malformed_json_cli_test() {
    let bin_path = env!("CARGO_BIN_EXE_perflab");
    let tmp_dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"));
    let baseline = tmp_dir.join("perflab-malformed-baseline.json");
    let candidate = tmp_dir.join("perflab-malformed-candidate-valid.json");

    fs::write(&baseline, r#"{"meta":"#).expect("failed to write malformed baseline JSON");
    write_json(&candidate, &valid_runner_json_value());

    let output = Command::new(bin_path)
        .arg("compare")
        .arg(&baseline)
        .arg(&candidate)
        .output()
        .expect("compare_malformed_json_cli_test - Failed to run perflab binary!");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert_eq!(output.status.code(), Some(1));
    assert!(stderr.contains("perflab-compare-error"));
    assert!(stderr.contains("Create json object failed"));
    assert!(stderr.contains("Side:Baseline"));
    assert!(!stdout.contains("PerfLab compare v0"));
    assert!(!stderr.contains("panicked at"));
}

#[test]
fn compare_schema_mismatch_cli_test() {
    let bin_path = env!("CARGO_BIN_EXE_perflab");
    let tmp_dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"));
    let baseline = tmp_dir.join("perflab-schema-baseline.json");
    let candidate = tmp_dir.join("perflab-schema-candidate.json");

    let baseline_json = valid_runner_json_value();
    let mut candidate_json = valid_runner_json_value();

    *candidate_json
        .pointer_mut("/meta/schema_version")
        .expect("schema_version must exist in valid integration-test JSON") = serde_json::json!(3);

    write_json(&baseline, &baseline_json);
    write_json(&candidate, &candidate_json);

    let output = Command::new(bin_path)
        .arg("compare")
        .arg(&baseline)
        .arg(&candidate)
        .output()
        .expect("compare_schema_mismatch_cli_test - Failed to run perflab binary!");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert_eq!(output.status.code(), Some(1));
    assert!(stderr.contains("perflab-compare-error"));
    assert!(stderr.contains("Schema mismatch"));
    assert!(stderr.contains("baseline="));
    assert!(stderr.contains("candidate="));
    assert!(stderr.contains('2'));
    assert!(stderr.contains('3'));
    assert!(!stdout.contains("PerfLab compare v0"));
    assert!(!stderr.contains("panicked at"));
}

#[test]
fn compare_benchmark_mismatch_cli_test() {
    let bin_path = env!("CARGO_BIN_EXE_perflab");
    let tmp_dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"));
    let baseline = tmp_dir.join("perflab-benchmark-baseline.json");
    let candidate = tmp_dir.join("perflab-benchmark-candidate.json");

    let baseline_json = valid_runner_json_value();
    let mut candidate_json = valid_runner_json_value();

    *candidate_json
        .pointer_mut("/meta/bench")
        .expect("bench must exist in valid integration-test JSON") = serde_json::json!("reduce");

    write_json(&baseline, &baseline_json);
    write_json(&candidate, &candidate_json);

    let output = Command::new(bin_path)
        .arg("compare")
        .arg(&baseline)
        .arg(&candidate)
        .output()
        .expect("compare_benchmark_mismatch_cli_test - Failed to run perflab binary!");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert_eq!(output.status.code(), Some(1));
    assert!(stderr.contains("perflab-compare-error"));
    assert!(stderr.contains("Benchmark mismatch"));
    assert!(stderr.contains("matmul"));
    assert!(stderr.contains("reduce"));
    assert!(!stdout.contains("PerfLab compare v0"));
    assert!(!stderr.contains("panicked at"));
}
