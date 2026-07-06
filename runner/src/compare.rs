use crate::config;
use crate::io;
use crate::types;
use serde_json;

const INDENT_LEN: usize = 30;

pub fn execute() {
    let cmp_args = config::get_cmp_arg();

    let format = cmp_args.format;
    match format {
        types::Format::Text => {
            println!("Text format!");
        }
        types::Format::Markdown => {
            println!("Markdown format!");
        }
    }

    let baseline_path = cmp_args.baseline.to_string_lossy().trim().to_string();
    let baseline_obj = get_runner_json(&baseline_path);

    let candidate_path = cmp_args.candidate.to_string_lossy().trim().to_string();
    let candidate_obj = get_runner_json(&candidate_path);

    if verify_required(&baseline_obj, &candidate_obj) == true {
        verify_good_to_have(&baseline_obj, &candidate_obj);

        let bench = baseline_obj.meta.bench.clone();
        let schm_ver = baseline_obj.meta.schema_version.clone();
        print_cmp_header(&baseline_path, &candidate_path, bench, schm_ver);

        verify_summary_phases(&baseline_obj, &candidate_obj);

        verify_summary_perf(&baseline_obj, &candidate_obj);
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

fn verify_required(baseline: &types::RunnerJson, candidate: &types::RunnerJson) -> bool {
    let mut result: bool = true;

    if baseline.meta.schema_version != candidate.meta.schema_version {
        println!(
            "perflab-compare-error, schema mismatch: \n\tbaseline=\t{} \n\tcandidate=\t{}",
            baseline.meta.schema_version, candidate.meta.schema_version
        );
        result = result && false;
    }
    if baseline.meta.bench != candidate.meta.bench {
        println!(
            "perflab-compare-error, benchmark mismatch: \n\tbaseline=\t{} \n\tcandidate=\t{}",
            baseline.meta.bench, candidate.meta.bench
        );
        result = result && false;
    }

    result
}

fn verify_good_to_have(baseline: &types::RunnerJson, candidate: &types::RunnerJson) {
    if baseline.meta.compiler.path != candidate.meta.compiler.path {
        println!(
            "perflab-compare-warning, compiler.path differ: \n\tbaseline=\t{} \n\tcandidate=\t{}",
            format_warning_value(&baseline.meta.compiler.path),
            format_warning_value(&candidate.meta.compiler.path)
        );
    }
    if baseline.meta.compiler.version != candidate.meta.compiler.version {
        println!(
            "perflab-compare-warning, compiler.version differ: \n\tbaseline=\t{} \n\tcandidate=\t{}",
            format_warning_value(&baseline.meta.compiler.version),
            format_warning_value(&candidate.meta.compiler.version)
        );
    }
    if baseline.meta.compiler_args != candidate.meta.compiler_args {
        println!(
            "perflab-compare-warning, compiler_args differ: \n\tbaseline=\t{} \n\tcandidate=\t{}",
            format_warning_value(&baseline.meta.compiler_args),
            format_warning_value(&candidate.meta.compiler_args)
        );
    }
    if baseline.meta.cpu_pin != candidate.meta.cpu_pin {
        println!(
            "perflab-compare-warning, cpu_pin differ: \n\tbaseline=\t{} \n\tcandidate=\t{}",
            format_warning_value(&baseline.meta.cpu_pin),
            format_warning_value(&candidate.meta.cpu_pin)
        );
    }
    if baseline.meta.warmup != candidate.meta.warmup {
        println!(
            "perflab-compare-warning, warmup differ: \n\tbaseline=\t{} \n\tcandidate=\t{}",
            format_warning_value(&baseline.meta.warmup),
            format_warning_value(&candidate.meta.warmup)
        );
    }
    if baseline.meta.reps != candidate.meta.reps {
        println!(
            "perflab-compare-warning, reps differ: \n\tbaseline=\t{} \n\tcandidate=\t{}",
            format_warning_value(&baseline.meta.reps),
            format_warning_value(&candidate.meta.reps)
        );
    }
    if baseline.meta.perf_events_requested != candidate.meta.perf_events_requested {
        println!(
            "perflab-compare-warning, perf_events_requested differ: \n\tbaseline=\t{} \n\tcandidate=\t{}",
            format_warning_value(&baseline.meta.perf_events_requested),
            format_warning_value(&candidate.meta.perf_events_requested)
        );
    }
    if baseline.meta.workdir != candidate.meta.workdir {
        println!(
            "perflab-compare-warning, workdir differ: \n\tbaseline=\t{} \n\tcandidate=\t{}",
            format_warning_value(&baseline.meta.workdir),
            format_warning_value(&candidate.meta.workdir)
        );
    }
    if baseline.meta.git_sha != candidate.meta.git_sha {
        println!(
            "perflab-compare-warning, git_sha differ: \n\tbaseline=\t{} \n\tcandidate=\t{}",
            format_warning_value(&baseline.meta.git_sha),
            format_warning_value(&candidate.meta.git_sha)
        );
    }
    if baseline.meta.uname != candidate.meta.uname {
        println!(
            "perflab-compare-warning, uname differ: \n\tbaseline=\t{} \n\tcandidate=\t{}",
            format_warning_value(&baseline.meta.uname),
            format_warning_value(&candidate.meta.uname)
        );
    }
}

fn print_cmp_header(baseline_path: &String, candidate_path: &String, bench: String, schm_ver: u32) {
    println!("");
    println!("PerfLab compare v0");
    println!("\tbaseline:\t{baseline_path}");
    println!("\tcandidate:\t{candidate_path}");
    println!("\tbench:\t{bench}");
    println!("\tschema:\t{schm_ver}");
    println!("");
}

fn verify_summary_phases(baseline: &types::RunnerJson, candidate: &types::RunnerJson) {
    println!("Phase comparison:");
    println!(
        "\t{:<w$}{:<w$}{:<w$}{:<w$}{:<w$}",
        "phase",
        "baseline(ns)",
        "candidate(ns)",
        "delta(ns)",
        "delta(%)",
        w = INDENT_LEN
    );
    println!(
        "\t{:<w$}{:<w$}{:<w$}{:<w$}{:<w$}",
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
        ),
        w = INDENT_LEN
    );
    println!(
        "\t{:<w$}{:<w$}{:<w$}{:<w$}{:<w$}",
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
        ),
        w = INDENT_LEN
    );
    println!(
        "\t{:<w$}{:<w$}{:<w$}{:<w$}{:<w$}",
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
        ),
        w = INDENT_LEN
    );
}

fn verify_summary_perf(baseline: &types::RunnerJson, candidate: &types::RunnerJson) {
    println!("Perf comparison:");

    verify_summary_perf_avail(baseline, candidate);

    let perf_events = get_common_perf_events(baseline, candidate);
    if perf_events.is_empty() == false {
        println!(
            "\t{:<w$}{:<w$}{:<w$}{:<w$}{:<w$}",
            "event",
            "baseline",
            "candidate",
            "delta",
            "delta(%)",
            w = INDENT_LEN
        );

        for event in perf_events {
            println!(
                "\t{:<w$}{:<w$}{:<w$}{:<w$}{:<w$}",
                event.event_name,
                event.baseline,
                event.candidate,
                get_abs_delta(event.baseline, event.candidate),
                get_percent_delta(event.baseline, event.candidate),
                w = INDENT_LEN
            );
        }
    }
}

fn verify_summary_perf_avail(baseline: &types::RunnerJson, candidate: &types::RunnerJson) {
    if let None = baseline.summary.perf {
        println!("perflab-compare-comparison unavailable: baseline has no summary.perf");
    } else if let None = candidate.summary.perf {
        println!("perflab-compare-comparison unavailable: candidate has no summary.perf");
    } else if let Some(perf_events) = baseline.summary.perf.as_ref() {
        if perf_events.events.is_empty() {
            println!("perflab-compare-comparison unavailable: baseline has no summary.perf.events");
        }
    } else if let Some(perf_events) = candidate.summary.perf.as_ref() {
        if perf_events.events.is_empty() {
            println!(
                "perflab-compare-comparison unavailable: candidate has no summary.perf.events"
            );
        }
    }
}

fn get_common_perf_events(
    baseline: &types::RunnerJson,
    candidate: &types::RunnerJson,
) -> types::CmpPerfEvents {
    let mut common_perf_events: types::CmpPerfEvents = Vec::new();

    if let Some(baseline_perf_events) = baseline.summary.perf.as_ref() {
        if let Some(candidate_perf_events) = candidate.summary.perf.as_ref() {
            for b_event in &baseline_perf_events.events {
                for c_event in &candidate_perf_events.events {
                    if b_event.0 == c_event.0 {
                        common_perf_events.push(types::CmpPerfEvent {
                            event_name: b_event.0.to_string(),
                            baseline: *b_event.1,
                            candidate: *c_event.1,
                        });
                    }
                }
            }
        }
    }

    common_perf_events.sort_by(|elem_1, elem_2| elem_1.event_name.cmp(&elem_2.event_name));
    common_perf_events
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

fn format_warning_value<T>(value: &T) -> String
where
    T: ?Sized + serde::Serialize,
{
    serde_json::to_string(value).unwrap_or_else(|err| {
        panic!("perflab-compare-value is not implementing serde::Serialize trait! error:\n{err}");
    })
}
