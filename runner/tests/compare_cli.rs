use std::path::PathBuf;
use std::process::Command;

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
