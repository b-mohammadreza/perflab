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

// Sample-aware fixture for the new end-to-end workload checks. Legacy
// validation tests can continue to use the empty-sample fixture.
fn valid_workload_runner_json_value() -> serde_json::Value {
    let mut value = valid_runner_json_value();
    value["samples"] = serde_json::Value::Array(
        (0..3)
            .map(|_| {
                serde_json::json!({
                    "bench_output": {
                        "bench": "matmul",
                        "params": {"iters": 100, "n": 1000},
                        "check": {"checksum": 42},
                        "phases_ns": {"init": 100, "compute": 1000, "teardown": 50}
                    },
                    "perf": null
                })
            })
            .collect(),
    );
    value
}

// Each test writes its own pair of files so cargo's parallel test execution is safe.
fn run_compare_fixture(
    name: &str,
    baseline_json: &serde_json::Value,
    candidate_json: &serde_json::Value,
    format: &str,
) -> std::process::Output {
    let tmp_dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"));
    let baseline = tmp_dir.join(format!("perflab-{name}-baseline.json"));
    let candidate = tmp_dir.join(format!("perflab-{name}-candidate.json"));
    write_json(&baseline, baseline_json);
    write_json(&candidate, candidate_json);
    Command::new(env!("CARGO_BIN_EXE_perflab"))
        .arg("compare")
        .arg(&baseline)
        .arg(&candidate)
        .arg("--format")
        .arg(format)
        .output()
        .expect("failed to run perflab compare")
}

fn assert_fatal_compare_output(output: &std::process::Output, message: &str) {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(1), "stderr: {stderr}");
    assert!(stderr.contains("perflab-compare-error"), "stderr: {stderr}");
    assert!(stderr.contains(message), "stderr: {stderr}");
    assert!(!stdout.contains("PerfLab compare v0"), "stdout: {stdout}");
    assert!(!stdout.contains("kind,name,baseline"), "stdout: {stdout}");
    assert!(!stderr.contains("panicked at"), "stderr: {stderr}");
}

#[test]
fn compare_identical_workload_cli_test() {
    let baseline = valid_workload_runner_json_value();
    let candidate = valid_workload_runner_json_value();
    let output = run_compare_fixture("workload-valid", &baseline, &candidate, "csv");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "stderr: {stderr}");
    assert!(stdout.starts_with("kind,name,baseline,candidate,delta,delta_percent,"));
    assert!(stdout.contains("phase,compute,"));
    assert!(
        !stderr.contains("perflab-compare-error"),
        "stderr: {stderr}"
    );
}

#[test]
fn compare_empty_baseline_samples_cli_test() {
    let mut baseline = valid_workload_runner_json_value();
    let candidate = valid_workload_runner_json_value();
    baseline["samples"] = serde_json::json!([]);
    let output = run_compare_fixture("empty-baseline", &baseline, &candidate, "text");
    assert_fatal_compare_output(&output, "Empty sample!");
    assert!(String::from_utf8_lossy(&output.stderr).contains("Input side:\tBaseline"));
}

#[test]
fn compare_empty_candidate_samples_cli_test() {
    let baseline = valid_workload_runner_json_value();
    let mut candidate = valid_workload_runner_json_value();
    candidate["samples"] = serde_json::json!([]);
    let output = run_compare_fixture("empty-candidate", &baseline, &candidate, "markdown");
    assert_fatal_compare_output(&output, "Empty sample!");
    assert!(String::from_utf8_lossy(&output.stderr).contains("Input side:\tCandidate"));
}

#[test]
fn compare_sample_benchmark_mismatch_cli_test() {
    let baseline = valid_workload_runner_json_value();
    let mut candidate = valid_workload_runner_json_value();
    candidate["samples"][1]["bench_output"]["bench"] = serde_json::json!("reduce");
    let output = run_compare_fixture("sample-bench-mismatch", &baseline, &candidate, "csv");
    assert_fatal_compare_output(&output, "Sample benchmark mismatch!");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Input side:\tCandidate"));
    assert!(stderr.contains("Sample index:\t1"));
    assert!(stderr.contains("Expected bench:\tmatmul"));
    assert!(stderr.contains("Actual bench:\treduce"));
}

#[test]
fn compare_inconsistent_sample_params_cli_test() {
    let mut baseline = valid_workload_runner_json_value();
    let candidate = valid_workload_runner_json_value();
    baseline["samples"][2]["bench_output"]["params"]["iters"] = serde_json::json!(200);
    let output = run_compare_fixture("inconsistent-sample", &baseline, &candidate, "text");
    assert_fatal_compare_output(&output, "sample params!");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Input side:\tBaseline"));
    assert!(stderr.contains("Sample index:\t2"));
    assert!(stderr.contains("Expected params:"));
    assert!(stderr.contains("Actual params:"));
    assert!(stderr.contains("\"iters\":100"));
    assert!(stderr.contains("\"iters\":200"));
}

#[test]
fn compare_workload_params_mismatch_cli_test() {
    let baseline = valid_workload_runner_json_value();
    let mut candidate = valid_workload_runner_json_value();
    for sample in candidate["samples"]
        .as_array_mut()
        .expect("samples must be an array")
    {
        sample["bench_output"]["params"]["n"] = serde_json::json!(2000);
    }
    let output = run_compare_fixture("between-run-mismatch", &baseline, &candidate, "markdown");
    assert_fatal_compare_output(&output, "Workload params mismatch!");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Baseline params:"));
    assert!(stderr.contains("Candidate params:"));
    assert!(stderr.contains("\"n\":1000"));
    assert!(stderr.contains("\"n\":2000"));
}

#[test]
fn compare_compiler_args_mismatch_is_warning_cli_test() {
    let baseline = valid_workload_runner_json_value();
    let mut candidate = valid_workload_runner_json_value();
    candidate["meta"]["compiler_args"] = serde_json::json!(["-O0"]);
    let output = run_compare_fixture("compiler-flags-warning", &baseline, &candidate, "text");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "stderr: {stderr}");
    assert!(stdout.contains("PerfLab compare v0"), "stdout: {stdout}");
    assert!(stdout.contains("Phase comparison:"), "stdout: {stdout}");
    assert!(stderr.contains("compiler_args differ"), "stderr: {stderr}");
    assert!(
        !stderr.contains("perflab-compare-error"),
        "stderr: {stderr}"
    );
}
