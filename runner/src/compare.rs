use crate::config;
use crate::io;
use crate::types;
use serde_json;

pub fn execute() {
    let cmp_args = config::get_cmp_arg();

    let baseline_path = cmp_args.baseline.to_string_lossy().trim().to_string();
    let baseline_obj = get_runner_json(&baseline_path);

    let candidate_path = cmp_args.candidate.to_string_lossy().trim().to_string();
    let candidate_obj = get_runner_json(&candidate_path);

    if checks_required(&baseline_obj, &candidate_obj) == true {
        checks_good_to_have(&baseline_obj, &candidate_obj);

        let bench = baseline_obj.meta.bench.clone();
        let schm_ver = baseline_obj.meta.schema_version.clone();
        print_cmp_header(&baseline_path, &candidate_path, bench, schm_ver);

        checks_summary_phases(&baseline_obj, &candidate_obj);

        checks_summary_perf();
    }
}

fn get_runner_json(json_path: &String) -> types::RunnerJson {
    let json_data = io::read_txt_file(&json_path);
    let json_dser = &mut serde_json::Deserializer::from_str(&json_data);

    let json_res: Result<types::RunnerJson, _> = serde_path_to_error::deserialize(json_dser);
    match json_res {
        Ok(obj) => obj,
        Err(err) => {
            panic!(
                "perflab-compare-file({json_path}), error occured at {} - might be missing!",
                err.path().to_string()
            );
        }
    }
}

fn checks_required(baseline: &types::RunnerJson, candidate: &types::RunnerJson) -> bool {
    let mut result: bool = true;

    if baseline.meta.schema_version != candidate.meta.schema_version {
        println!(
            "perflab-compare-error, schema mismatch: \n\tbaseline={} \n\tcandidate={}",
            baseline.meta.schema_version, candidate.meta.schema_version
        );
        result = result && false;
    }
    if baseline.meta.bench != candidate.meta.bench {
        println!(
            "perflab-compare-error, benchmark mismatch: \n\tbaseline={} \n\tcandidate={}",
            baseline.meta.bench, candidate.meta.bench
        );
        result = result && false;
    }

    result
}

fn checks_good_to_have(baseline: &types::RunnerJson, candidate: &types::RunnerJson) {
    if baseline.meta.compiler.path != candidate.meta.compiler.path {
        println!(
            "perflab-compare-warning, compiler.path differ: \n\tbaseline={} \n\tcandidate={}",
            baseline.meta.compiler.path, candidate.meta.compiler.path
        );
    }
    if baseline.meta.compiler.version != candidate.meta.compiler.version {
        println!(
            "perflab-compare-warning, compiler.version differ: \n\tbaseline={} \n\tcandidate={}",
            baseline.meta.compiler.version, candidate.meta.compiler.version
        );
    }
    if baseline.meta.compiler_args != candidate.meta.compiler_args {
        println!(
            "perflab-compare-warning, compiler_args differ: \n\tbaseline={:?} \n\tcandidate={:?}",
            baseline.meta.compiler_args, candidate.meta.compiler_args
        );
    }
    if baseline.meta.cpu_pin != candidate.meta.cpu_pin {
        println!(
            "perflab-compare-warning, cpu_pin differ: \n\tbaseline={:?} \n\tcandidate={:?}",
            baseline.meta.cpu_pin, candidate.meta.cpu_pin
        );
    }
    if baseline.meta.warmup != candidate.meta.warmup {
        println!(
            "perflab-compare-warning, warmup differ: \n\tbaseline={} \n\tcandidate={}",
            baseline.meta.warmup, candidate.meta.warmup
        );
    }
    if baseline.meta.reps != candidate.meta.reps {
        println!(
            "perflab-compare-warning, reps differ: \n\tbaseline={} \n\tcandidate={}",
            baseline.meta.reps, candidate.meta.reps
        );
    }
    if baseline.meta.perf_events_requested != candidate.meta.perf_events_requested {
        println!(
            "perflab-compare-warning, perf_events_requested differ: \n\tbaseline={:?} \n\tcandidate={:?}",
            baseline.meta.perf_events_requested, candidate.meta.perf_events_requested
        );
    }
    if baseline.meta.workdir != candidate.meta.workdir {
        println!(
            "perflab-compare-warning, workdir differ: \n\tbaseline={} \n\tcandidate={}",
            baseline.meta.workdir, candidate.meta.workdir
        );
    }
    if baseline.meta.git_sha != candidate.meta.git_sha {
        println!(
            "perflab-compare-warning, git_sha differ: \n\tbaseline={} \n\tcandidate={}",
            baseline.meta.git_sha, candidate.meta.git_sha
        );
    }
    if baseline.meta.uname != candidate.meta.uname {
        println!(
            "perflab-compare-warning, uname differ: \n\tbaseline={} \n\tcandidate={}",
            baseline.meta.uname, candidate.meta.uname
        );
    }
}

fn print_cmp_header(baseline_path: &String, candidate_path: &String, bench: String, schm_ver: u32) {
    println!("PerfLab compare v0");
    println!("\tbaseline: {baseline_path}");
    println!("\tcandidate: {candidate_path}");
    println!("\tbench: {bench}");
    println!("\tschema: {schm_ver}");
    println!("");
}

fn checks_summary_phases(baseline: &types::RunnerJson, candidate: &types::RunnerJson) {
    println!("Phase comparison:");
    println!(
        "\t{:<20}{:<20}{:<20}{:<20}{:<20}",
        "phase", "baseline(ns)", "candidate(ns)", "delta(ns)", "delta(%)"
    );
    println!(
        "\t{:<20}{:<20}{:<20}{:<20}{:<20}",
        "init",
        baseline.summary.phases_ns.init,
        candidate.summary.phases_ns.init,
        get_abs_delta(
            baseline.summary.phases_ns.init,
            candidate.summary.phases_ns.init
        ),
        get_percent_delta(
            baseline.summary.phases_ns.init,
            candidate.summary.phases_ns.init
        )
    );
    println!(
        "\t{:<20}{:<20}{:<20}{:<20}{:<20}",
        "compute",
        baseline.summary.phases_ns.compute,
        candidate.summary.phases_ns.compute,
        get_abs_delta(
            baseline.summary.phases_ns.compute,
            candidate.summary.phases_ns.compute
        ),
        get_percent_delta(
            baseline.summary.phases_ns.compute,
            candidate.summary.phases_ns.compute
        )
    );
    println!(
        "\t{:<20}{:<20}{:<20}{:<20}{:<20}",
        "teardown",
        baseline.summary.phases_ns.teardown,
        candidate.summary.phases_ns.teardown,
        get_abs_delta(
            baseline.summary.phases_ns.teardown,
            candidate.summary.phases_ns.teardown
        ),
        get_percent_delta(
            baseline.summary.phases_ns.teardown,
            candidate.summary.phases_ns.teardown
        )
    );
}

fn get_abs_delta(baseline_phase: u64, candidate_phase: u64) -> String {
    format!("{:+}", candidate_phase as i64 - baseline_phase as i64)
}

fn get_percent_delta(baseline_phase: u64, candidate_phase: u64) -> String {
    match baseline_phase {
        0 => "N/A".to_string(),
        _ => format!(
            "{:+.2}%",
            ((candidate_phase as f64 - baseline_phase as f64) / baseline_phase as f64 * 100 as f64)
        ),
    }
}

fn checks_summary_perf() {
    println!("Perf comparison: TODO");
}
