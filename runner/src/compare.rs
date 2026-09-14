use crate::config;
use crate::io;
use crate::types;
use serde_json;
use std::path::PathBuf;

pub fn execute() -> Result<(), types::CompareError> {
    let cmp_args = config::get_cmp_arg();

    let b_json_data: String =
        get_json_str(cmp_args.baseline.clone(), types::CmpInputSide::JsonBaseline)?;
    let b_generic_val: serde_json::Value = get_json_val_obj(
        &b_json_data,
        cmp_args.baseline.clone(),
        types::CmpInputSide::JsonBaseline,
    )?;
    verify_required_structure(&b_generic_val, types::CmpInputSide::JsonBaseline)?;
    let baseline_obj: types::RunnerJson = get_runner_json(
        &b_json_data,
        cmp_args.baseline.clone(),
        types::CmpInputSide::JsonBaseline,
    )?;

    let c_json_data: String = get_json_str(
        cmp_args.candidate.clone(),
        types::CmpInputSide::JsonCandidate,
    )?;
    let c_generic_val: serde_json::Value = get_json_val_obj(
        &c_json_data,
        cmp_args.candidate.clone(),
        types::CmpInputSide::JsonCandidate,
    )?;
    verify_required_structure(&c_generic_val, types::CmpInputSide::JsonCandidate)?;
    let candidate_obj: types::RunnerJson = get_runner_json(
        &c_json_data,
        cmp_args.candidate.clone(),
        types::CmpInputSide::JsonCandidate,
    )?;

    verify_required(&baseline_obj, &candidate_obj)?;
    verify_good_to_have(&baseline_obj, &candidate_obj)?;

    let baseline_path = cmp_args.baseline.to_string_lossy().trim().to_string();
    let candidate_path = cmp_args.candidate.to_string_lossy().trim().to_string();
    let bench = baseline_obj.meta.bench.clone();
    let schm_ver = baseline_obj.meta.schema_version.clone();
    let cmp_data: types::CmpGData = get_cmp_g_data(
        &baseline_path,
        &candidate_path,
        bench,
        schm_ver,
        &baseline_obj,
        &candidate_obj,
    );

    let cmp_renderer: Box<dyn types::CmpRenderer> = match cmp_args.format {
        types::Format::Text => Box::new(types::TextCmpRenderer {
            cmp_g_data: &cmp_data,
        }),
        types::Format::Markdown => Box::new(types::MarkdownCmpRenderer {
            cmp_g_data: &cmp_data,
        }),
        types::Format::Csv => Box::new(types::CsvCmpRenderer {
            cmp_g_data: &cmp_data,
        }),
    };

    cmp_renderer.render_cmp_result();

    Ok(())
}

pub fn get_input_side_str(cmp_json_type: &types::CmpInputSide) -> String {
    match cmp_json_type {
        types::CmpInputSide::JsonBaseline => String::from("Baseline"),
        types::CmpInputSide::JsonCandidate => String::from("Candidate"),
    }
}

fn get_json_str(
    json_path: PathBuf,
    cmp_json_type: types::CmpInputSide,
) -> Result<String, types::CompareError> {
    let json_path_str: String = json_path.to_string_lossy().trim().to_string();

    match io::read_txt_file(&json_path_str) {
        Ok(json_data) => Ok(json_data),
        Err(file_err) => Err(types::CompareError::ReadInput {
            input: cmp_json_type,
            path: json_path,
            source: file_err,
        }),
    }
}

fn get_json_val_obj(
    json_data: &String,
    json_path: PathBuf,
    cmp_json_type: types::CmpInputSide,
) -> Result<serde_json::Value, types::CompareError> {
    match serde_json::from_str(&json_data) {
        Ok(val) => Ok(val),
        Err(error) => Err(types::CompareError::MalformedJson {
            input: cmp_json_type,
            path: json_path,
            source: error,
        }),
    }
}

fn verify_required_structure(
    generic_val: &serde_json::Value,
    cmp_json_type: types::CmpInputSide,
) -> Result<(), types::CompareError> {
    if generic_val.pointer("/meta/schema_version").is_none() {
        Err(types::CompareError::MissingRequiredField {
            input: cmp_json_type,
            field: String::from("meta.schema_version"),
        })
    } else if generic_val.pointer("/meta/bench").is_none() {
        Err(types::CompareError::MissingRequiredField {
            input: cmp_json_type,
            field: String::from("meta.bench"),
        })
    } else if generic_val.pointer("/summary/phases_ns").is_none() {
        Err(types::CompareError::MissingRequiredField {
            input: cmp_json_type,
            field: String::from("summary.phases_ns"),
        })
    } else if generic_val.pointer("/summary/phases_ns/init").is_none() {
        Err(types::CompareError::MissingRequiredField {
            input: cmp_json_type,
            field: String::from("summary.phases_ns.init"),
        })
    } else if generic_val.pointer("/summary/phases_ns/compute").is_none() {
        Err(types::CompareError::MissingRequiredField {
            input: cmp_json_type,
            field: String::from("summary.phases_ns.compute"),
        })
    } else if generic_val.pointer("/summary/phases_ns/teardown").is_none() {
        Err(types::CompareError::MissingRequiredField {
            input: cmp_json_type,
            field: String::from("summary.phases_ns.teardown"),
        })
    } else if generic_val.pointer("/summary/perf").is_none() {
        Err(types::CompareError::MissingRequiredField {
            input: cmp_json_type,
            field: String::from("summary.perf"),
        })
    } else if generic_val
        .pointer("/summary/perf")
        .is_some_and(|perf| !perf.is_null())
    {
        if generic_val.pointer("/summary/perf/events").is_none() {
            Err(types::CompareError::MissingRequiredField {
                input: cmp_json_type,
                field: String::from("summary.perf.events"),
            })
        } else {
            Ok(())
        }
    } else {
        Ok(())
    }
}

fn get_runner_json(
    json_data: &String,
    json_path: PathBuf,
    cmp_json_type: types::CmpInputSide,
) -> Result<types::RunnerJson, types::CompareError> {
    let json_dser = &mut serde_json::Deserializer::from_str(&json_data);
    let json_res: Result<types::RunnerJson, _> = serde_path_to_error::deserialize(json_dser);
    match json_res {
        Ok(obj) => Ok(obj),
        Err(err) => Err(types::CompareError::Deserialize {
            input: cmp_json_type,
            path: json_path,
            source: err,
        }),
    }
}

fn verify_required(
    baseline: &types::RunnerJson,
    candidate: &types::RunnerJson,
) -> Result<(), types::CompareError> {
    if baseline.meta.schema_version != candidate.meta.schema_version {
        Err(types::CompareError::SchemaMismatch {
            baseline_ver: baseline.meta.schema_version,
            candidate_ver: candidate.meta.schema_version,
        })
    } else if baseline.meta.bench != candidate.meta.bench {
        Err(types::CompareError::BenchmarkMismatch {
            baseline_bench: baseline.meta.bench.clone(),
            candidate_bench: candidate.meta.bench.clone(),
        })
    } else {
        Ok(())
    }
}

fn verify_good_to_have(
    baseline: &types::RunnerJson,
    candidate: &types::RunnerJson,
) -> Result<(), types::CompareError> {
    if baseline.meta.compiler.path != candidate.meta.compiler.path {
        eprintln!(
            "perflab-compare-warning: compiler.path differ, \n\tbaseline=\t{} \n\tcandidate=\t{}",
            format_warn_err_value(&baseline.meta.compiler.path)?,
            format_warn_err_value(&candidate.meta.compiler.path)?
        );
    }
    if baseline.meta.compiler.version != candidate.meta.compiler.version {
        eprintln!(
            "perflab-compare-warning: compiler.version differ, \n\tbaseline=\t{} \n\tcandidate=\t{}",
            format_warn_err_value(&baseline.meta.compiler.version)?,
            format_warn_err_value(&candidate.meta.compiler.version)?
        );
    }
    if baseline
        .meta
        .compiler_args
        .ne(&candidate.meta.compiler_args)
    {
        eprintln!(
            "perflab-compare-warning: compiler_args differ, \n\tbaseline=\t{} \n\tcandidate=\t{}",
            format_warn_err_value(&baseline.meta.compiler_args)?,
            format_warn_err_value(&candidate.meta.compiler_args)?
        );
    }
    if baseline.meta.cpu_pin.ne(&candidate.meta.cpu_pin) {
        eprintln!(
            "perflab-compare-warning: cpu_pin differ, \n\tbaseline=\t{} \n\tcandidate=\t{}",
            format_warn_err_value(&baseline.meta.cpu_pin)?,
            format_warn_err_value(&candidate.meta.cpu_pin)?
        );
    }
    if baseline.meta.warmup != candidate.meta.warmup {
        eprintln!(
            "perflab-compare-warning: warmup differ, \n\tbaseline=\t{} \n\tcandidate=\t{}",
            format_warn_err_value(&baseline.meta.warmup)?,
            format_warn_err_value(&candidate.meta.warmup)?
        );
    }
    if baseline.meta.reps != candidate.meta.reps {
        eprintln!(
            "perflab-compare-warning: reps differ, \n\tbaseline=\t{} \n\tcandidate=\t{}",
            format_warn_err_value(&baseline.meta.reps)?,
            format_warn_err_value(&candidate.meta.reps)?
        );
    }
    if baseline
        .meta
        .perf_events_requested
        .ne(&candidate.meta.perf_events_requested)
    {
        eprintln!(
            "perflab-compare-warning: perf_events_requested differ, \n\tbaseline=\t{} \n\tcandidate=\t{}",
            format_warn_err_value(&baseline.meta.perf_events_requested)?,
            format_warn_err_value(&candidate.meta.perf_events_requested)?
        );
    }
    if baseline.meta.workdir != candidate.meta.workdir {
        eprintln!(
            "perflab-compare-warning: workdir differ, \n\tbaseline=\t{} \n\tcandidate=\t{}",
            format_warn_err_value(&baseline.meta.workdir)?,
            format_warn_err_value(&candidate.meta.workdir)?
        );
    }
    if baseline.meta.git_sha != candidate.meta.git_sha {
        eprintln!(
            "perflab-compare-warning: git_sha differ, \n\tbaseline=\t{} \n\tcandidate=\t{}",
            format_warn_err_value(&baseline.meta.git_sha)?,
            format_warn_err_value(&candidate.meta.git_sha)?
        );
    }
    if baseline.meta.uname != candidate.meta.uname {
        eprintln!(
            "perflab-compare-warning: uname differ, \n\tbaseline=\t{} \n\tcandidate=\t{}",
            format_warn_err_value(&baseline.meta.uname)?,
            format_warn_err_value(&candidate.meta.uname)?
        );
    }
    Ok(())
}

fn get_cmp_g_data(
    baseline_path: &String,
    candidate_path: &String,
    bench: String,
    schm_ver: u32,
    baseline: &types::RunnerJson,
    candidate: &types::RunnerJson,
) -> types::CmpGData {
    let mut cmp_g_data: types::CmpGData = types::CmpGData::new();

    cmp_g_data.baseline_path = baseline_path.to_string();
    cmp_g_data.candidate_path = candidate_path.to_string();
    cmp_g_data.bench = bench;
    cmp_g_data.schm_ver = schm_ver;

    cmp_g_data.init_phase = types::CmpItemData {
        item_name: String::from("init"),
        baseline: baseline.summary.phases_ns.init.median_ns,
        candidate: candidate.summary.phases_ns.init.median_ns,
        abs_delta: get_abs_delta(
            baseline.summary.phases_ns.init.median_ns,
            candidate.summary.phases_ns.init.median_ns,
        ),
        percent_delta: get_percent_delta(
            baseline.summary.phases_ns.init.median_ns,
            candidate.summary.phases_ns.init.median_ns,
        ),
        baseline_spread: get_spread_percent(baseline.summary.phases_ns.init.spread_percent),
        candidate_spread: get_spread_percent(candidate.summary.phases_ns.init.spread_percent),
    };

    cmp_g_data.compute_phase = types::CmpItemData {
        item_name: String::from("compute"),
        baseline: baseline.summary.phases_ns.compute.median_ns,
        candidate: candidate.summary.phases_ns.compute.median_ns,
        abs_delta: get_abs_delta(
            baseline.summary.phases_ns.compute.median_ns,
            candidate.summary.phases_ns.compute.median_ns,
        ),
        percent_delta: get_percent_delta(
            baseline.summary.phases_ns.compute.median_ns,
            candidate.summary.phases_ns.compute.median_ns,
        ),
        baseline_spread: get_spread_percent(baseline.summary.phases_ns.compute.spread_percent),
        candidate_spread: get_spread_percent(candidate.summary.phases_ns.compute.spread_percent),
    };

    cmp_g_data.tear_down_phase = types::CmpItemData {
        item_name: String::from("teardown"),
        baseline: baseline.summary.phases_ns.teardown.median_ns,
        candidate: candidate.summary.phases_ns.teardown.median_ns,
        abs_delta: get_abs_delta(
            baseline.summary.phases_ns.teardown.median_ns,
            candidate.summary.phases_ns.teardown.median_ns,
        ),
        percent_delta: get_percent_delta(
            baseline.summary.phases_ns.teardown.median_ns,
            candidate.summary.phases_ns.teardown.median_ns,
        ),
        baseline_spread: get_spread_percent(baseline.summary.phases_ns.teardown.spread_percent),
        candidate_spread: get_spread_percent(candidate.summary.phases_ns.teardown.spread_percent),
    };

    verify_summary_perf_avail(&mut cmp_g_data, baseline, candidate);

    if cmp_g_data.perf_unavail == false && cmp_g_data.perf_events_unavail == false {
        let perf_events = get_common_perf_events(baseline, candidate);

        for event in perf_events {
            cmp_g_data.perf_events.push(types::CmpItemData {
                item_name: event.event_name,
                baseline: event.baseline,
                candidate: event.candidate,
                abs_delta: get_abs_delta(event.baseline, event.candidate),
                percent_delta: get_percent_delta(event.baseline, event.candidate),
                baseline_spread: String::from(""),
                candidate_spread: String::from(""),
            });
        }
    }

    cmp_g_data
}

fn verify_summary_perf_avail(
    cmp_data: &mut types::CmpGData,
    baseline: &types::RunnerJson,
    candidate: &types::RunnerJson,
) {
    if let None = baseline.summary.perf {
        cmp_data.perf_unavail = true;
    } else if let None = candidate.summary.perf {
        cmp_data.perf_unavail = true;
    } else if let Some(perf_events) = baseline.summary.perf.as_ref() {
        if perf_events.events.is_empty() {
            cmp_data.perf_events_unavail = true;
        }
    } else if let Some(perf_events) = candidate.summary.perf.as_ref() {
        if perf_events.events.is_empty() {
            cmp_data.perf_events_unavail = true;
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
    format!("{:+}", candidate_phase as i64 - baseline_phase as i64).to_string()
}

fn get_percent_delta(baseline_phase: u64, candidate_phase: u64) -> String {
    match baseline_phase {
        0 => String::from("N/A"),
        _ => format!(
            "{:+.2}%",
            ((candidate_phase as f64 - baseline_phase as f64) / baseline_phase as f64 * 100.0)
        ),
    }
}

fn get_spread_percent(spread: Option<f64>) -> String {
    match spread {
        None => String::from("null"),
        Some(val) => format!("{:.2}%", val),
    }
}

/// To have all values formatted as json.
pub fn format_warn_err_value<T>(value: &T) -> Result<String, types::CompareError>
where
    T: ?Sized + serde::Serialize,
{
    match serde_json::to_string(value) {
        Ok(str_val) => Ok(str_val),
        Err(s_err) => Err(types::CompareError::NotImplSerdeSerialize { source: s_err }),
    }
}

impl types::CmpGData {
    fn new() -> types::CmpGData {
        types::CmpGData {
            baseline_path: String::from(""),
            candidate_path: String::from(""),
            bench: String::from(""),
            schm_ver: 0,
            init_phase: types::CmpItemData {
                item_name: String::from(""),
                baseline: 0u64,
                candidate: 0u64,
                abs_delta: String::from(""),
                percent_delta: String::from(""),
                baseline_spread: String::from(""),
                candidate_spread: String::from(""),
            },
            compute_phase: types::CmpItemData {
                item_name: String::from(""),
                baseline: 0u64,
                candidate: 0u64,
                abs_delta: String::from(""),
                percent_delta: String::from(""),
                baseline_spread: String::from(""),
                candidate_spread: String::from(""),
            },
            tear_down_phase: types::CmpItemData {
                item_name: String::from(""),
                baseline: 0u64,
                candidate: 0u64,
                abs_delta: String::from(""),
                percent_delta: String::from(""),
                baseline_spread: String::from(""),
                candidate_spread: String::from(""),
            },
            perf_unavail: false,
            perf_events_unavail: false,
            perf_events: Vec::new(),
        }
    }
}

impl<'cmp_g> types::CmpRenderer for types::TextCmpRenderer<'cmp_g> {
    fn render_cmp_header(&self) {
        println!("");
        println!("PerfLab compare v0");
        println!("\tbaseline:\t{}", self.cmp_g_data.baseline_path);
        println!("\tcandidate:\t{}", self.cmp_g_data.candidate_path);
        println!("\tbench:\t{}", self.cmp_g_data.bench);
        println!("\tschema:\t{}", self.cmp_g_data.schm_ver);
        println!("");
    }

    fn render_summary_phases(&self) {
        const INDENT_LEN: usize = 25;

        println!("Phase comparison:");
        println!(
            "\t{:<w$}{:<w$}{:<w$}{:<w$}{:<w$}{:<w$}{:<w$}",
            "phase",
            "baseline(ns)",
            "candidate(ns)",
            "delta(ns)",
            "delta(%)",
            "baseline spread",
            "candidate spread",
            w = INDENT_LEN
        );
        println!(
            "\t{:<w$}{:<w$}{:<w$}{:<w$}{:<w$}{:<w$}{:<w$}",
            self.cmp_g_data.init_phase.item_name,
            self.cmp_g_data.init_phase.baseline,
            self.cmp_g_data.init_phase.candidate,
            self.cmp_g_data.init_phase.abs_delta,
            self.cmp_g_data.init_phase.percent_delta,
            self.cmp_g_data.init_phase.baseline_spread,
            self.cmp_g_data.init_phase.candidate_spread,
            w = INDENT_LEN
        );
        println!(
            "\t{:<w$}{:<w$}{:<w$}{:<w$}{:<w$}{:<w$}{:<w$}",
            self.cmp_g_data.compute_phase.item_name,
            self.cmp_g_data.compute_phase.baseline,
            self.cmp_g_data.compute_phase.candidate,
            self.cmp_g_data.compute_phase.abs_delta,
            self.cmp_g_data.compute_phase.percent_delta,
            self.cmp_g_data.compute_phase.baseline_spread,
            self.cmp_g_data.compute_phase.candidate_spread,
            w = INDENT_LEN
        );
        println!(
            "\t{:<w$}{:<w$}{:<w$}{:<w$}{:<w$}{:<w$}{:<w$}",
            self.cmp_g_data.tear_down_phase.item_name,
            self.cmp_g_data.tear_down_phase.baseline,
            self.cmp_g_data.tear_down_phase.candidate,
            self.cmp_g_data.tear_down_phase.abs_delta,
            self.cmp_g_data.tear_down_phase.percent_delta,
            self.cmp_g_data.tear_down_phase.baseline_spread,
            self.cmp_g_data.tear_down_phase.candidate_spread,
            w = INDENT_LEN
        );
        println!("");
    }

    fn render_summary_perf(&self) {
        const INDENT_LEN: usize = 25;

        if self.cmp_g_data.perf_unavail {
            eprintln!(
                "perflab-compare-warning: perf unavailable, perf data is unavailable in one or both inputs"
            );
        } else if self.cmp_g_data.perf_events_unavail {
            eprintln!(
                "perflab-compare-error: perf unavailable, perf events are unavailable in one or both inputs"
            );
        } else {
            println!("Perf comparison:");
            println!(
                "\t{:<w$}{:<w$}{:<w$}{:<w$}{:<w$}",
                "event",
                "baseline",
                "candidate",
                "delta",
                "delta(%)",
                w = INDENT_LEN
            );

            for item in &self.cmp_g_data.perf_events {
                println!(
                    "\t{:<w$}{:<w$}{:<w$}{:<w$}{:<w$}",
                    item.item_name,
                    item.baseline,
                    item.candidate,
                    item.abs_delta,
                    item.percent_delta,
                    w = INDENT_LEN
                );
            }
        }
    }
}

impl<'cmp_g> types::CmpRenderer for types::MarkdownCmpRenderer<'cmp_g> {
    fn render_cmp_header(&self) {
        println!("");
        println!("# PerfLab compare v0");
        println!("- baseline: `{}`", self.cmp_g_data.baseline_path);
        println!("- candidate: `{}`", self.cmp_g_data.candidate_path);
        println!("- bench: `{}`", self.cmp_g_data.bench);
        println!("- schema: `{}`", self.cmp_g_data.schm_ver);
        println!("");
    }

    fn render_summary_phases(&self) {
        println!("## Phase comparison:");
        println!(
            "| {} | {} | {} | {} | {} | {} | {} |\n|---|---:|---:|---:|---:|---:|---:|",
            "phase",
            "baseline(ns)",
            "candidate(ns)",
            "delta(ns)",
            "delta(%)",
            "baseline spread",
            "candidate spread"
        );
        println!(
            "| {} | {} | {} | {} | {} | {} | {} |",
            self.cmp_g_data.init_phase.item_name,
            self.cmp_g_data.init_phase.baseline,
            self.cmp_g_data.init_phase.candidate,
            self.cmp_g_data.init_phase.abs_delta,
            self.cmp_g_data.init_phase.percent_delta,
            self.cmp_g_data.init_phase.baseline_spread,
            self.cmp_g_data.init_phase.candidate_spread,
        );
        println!(
            "| {} | {} | {} | {} | {} | {} | {} |",
            self.cmp_g_data.compute_phase.item_name,
            self.cmp_g_data.compute_phase.baseline,
            self.cmp_g_data.compute_phase.candidate,
            self.cmp_g_data.compute_phase.abs_delta,
            self.cmp_g_data.compute_phase.percent_delta,
            self.cmp_g_data.compute_phase.baseline_spread,
            self.cmp_g_data.compute_phase.candidate_spread,
        );
        println!(
            "| {} | {} | {} | {} | {} | {} | {} |",
            self.cmp_g_data.tear_down_phase.item_name,
            self.cmp_g_data.tear_down_phase.baseline,
            self.cmp_g_data.tear_down_phase.candidate,
            self.cmp_g_data.tear_down_phase.abs_delta,
            self.cmp_g_data.tear_down_phase.percent_delta,
            self.cmp_g_data.tear_down_phase.baseline_spread,
            self.cmp_g_data.tear_down_phase.candidate_spread,
        );
        println!("");
    }

    fn render_summary_perf(&self) {
        if self.cmp_g_data.perf_unavail {
            eprintln!(
                "perflab-compare-warning: perf unavailable, perf data is unavailable in one or both inputs"
            );
        } else if self.cmp_g_data.perf_events_unavail {
            eprintln!(
                "perflab-compare-error: perf unavailable, perf events are unavailable in one or both inputs"
            );
        } else {
            println!("## Perf comparison:");
            println!(
                "| {} | {} | {} | {} | {} |\n|---|---:|---:|---:|---:|",
                "event", "baseline", "candidate", "delta", "delta(%)"
            );

            for item in &self.cmp_g_data.perf_events {
                println!(
                    "| {} | {} | {} | {} | {} |",
                    item.item_name,
                    item.baseline,
                    item.candidate,
                    item.abs_delta,
                    item.percent_delta
                );
            }
        }
    }
}

impl<'cmp_g> types::CmpRenderer for types::CsvCmpRenderer<'cmp_g> {
    fn render_cmp_header(&self) {
        println!(
            "kind,name,baseline,candidate,delta,delta_percent,baseline_spread_percent,candidate_spread_percent"
        );
    }

    fn render_summary_phases(&self) {
        println!(
            "{},{},{},{},{},{},{},{}",
            "phase",
            self.cmp_g_data.init_phase.item_name,
            self.cmp_g_data.init_phase.baseline,
            self.cmp_g_data.init_phase.candidate,
            self.cmp_g_data.init_phase.abs_delta,
            self.cmp_g_data.init_phase.percent_delta,
            self.cmp_g_data.init_phase.baseline_spread,
            self.cmp_g_data.init_phase.candidate_spread,
        );
        println!(
            "{},{},{},{},{},{},{},{}",
            "phase",
            self.cmp_g_data.compute_phase.item_name,
            self.cmp_g_data.compute_phase.baseline,
            self.cmp_g_data.compute_phase.candidate,
            self.cmp_g_data.compute_phase.abs_delta,
            self.cmp_g_data.compute_phase.percent_delta,
            self.cmp_g_data.compute_phase.baseline_spread,
            self.cmp_g_data.compute_phase.candidate_spread,
        );
        println!(
            "{},{},{},{},{},{},{},{}",
            "phase",
            self.cmp_g_data.tear_down_phase.item_name,
            self.cmp_g_data.tear_down_phase.baseline,
            self.cmp_g_data.tear_down_phase.candidate,
            self.cmp_g_data.tear_down_phase.abs_delta,
            self.cmp_g_data.tear_down_phase.percent_delta,
            self.cmp_g_data.tear_down_phase.baseline_spread,
            self.cmp_g_data.tear_down_phase.candidate_spread,
        );
    }

    fn render_summary_perf(&self) {
        if self.cmp_g_data.perf_unavail {
            eprintln!(
                "perflab-compare-warning: perf unavailable, perf data is unavailable in one or both inputs"
            );
        } else if self.cmp_g_data.perf_events_unavail {
            eprintln!(
                "perflab-compare-error: perf unavailable, perf events are unavailable in one or both inputs"
            );
        } else {
            for item in &self.cmp_g_data.perf_events {
                println!(
                    "{},{},{},{},{},{}",
                    "perf",
                    item.item_name,
                    item.baseline,
                    item.candidate,
                    item.abs_delta,
                    item.percent_delta,
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::assert_eq;

    use super::*;

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

    #[test]
    fn get_abs_delta_test() {
        assert_eq!(String::from("+20"), get_abs_delta(100, 120));
        assert_eq!(String::from("-20"), get_abs_delta(120, 100));
        assert_eq!(String::from("+0"), get_abs_delta(100, 100));
    }

    #[test]
    fn get_percent_delta_test() {
        assert_eq!(String::from("+20.00%"), get_percent_delta(100, 120));
        assert_eq!(String::from("-20.00%"), get_percent_delta(100, 80));
        assert_eq!(String::from("+0.00%"), get_percent_delta(100, 100));
        assert_eq!(String::from("N/A"), get_percent_delta(0, 100));
    }

    #[test]
    fn get_spread_percent_test() {
        assert_eq!(String::from("1.23%"), get_spread_percent(Some(1.234)));
        assert_eq!(String::from("0.00%"), get_spread_percent(Some(0.0)));
        assert_eq!(String::from("null"), get_spread_percent(None));
    }

    #[test]
    fn verify_required_structure_valid_test() {
        let json_val = serde_json::json!({
            "meta": {
                "schema_version": 2,
                "bench": "matmul"
            },
            "summary": {
                "phases_ns": {
                    "init": {},
                    "compute": {},
                    "teardown": {}
                },
                "perf": null
            }
        });

        let result = verify_required_structure(&json_val, types::CmpInputSide::JsonBaseline);

        assert!(result.is_ok());
    }

    #[test]
    fn verify_required_structure_missing_fields_test() {
        let cases = [
            (
                serde_json::json!({
                    "meta": {
                        "bench": "matmul"
                    },
                    "summary": {
                        "phases_ns": {
                            "init": {},
                            "compute": {},
                            "teardown": {}
                        },
                        "perf": null
                    }
                }),
                "meta.schema_version",
            ),
            (
                serde_json::json!({
                    "meta": {
                        "schema_version": 2
                    },
                    "summary": {
                        "phases_ns": {
                            "init": {},
                            "compute": {},
                            "teardown": {}
                        },
                        "perf": null
                    }
                }),
                "meta.bench",
            ),
            (
                serde_json::json!({
                    "meta": {
                        "schema_version": 2,
                        "bench": "matmul"
                    },
                    "summary": {
                        "perf": null
                    }
                }),
                "summary.phases_ns",
            ),
            (
                serde_json::json!({
                    "meta": {
                        "schema_version": 2,
                        "bench": "matmul"
                    },
                    "summary": {
                        "phases_ns": {
                            "compute": {},
                            "teardown": {}
                        },
                        "perf": null
                    }
                }),
                "summary.phases_ns.init",
            ),
            (
                serde_json::json!({
                    "meta": {
                        "schema_version": 2,
                        "bench": "matmul"
                    },
                    "summary": {
                        "phases_ns": {
                            "init": {},
                            "teardown": {}
                        },
                        "perf": null
                    }
                }),
                "summary.phases_ns.compute",
            ),
            (
                serde_json::json!({
                    "meta": {
                        "schema_version": 2,
                        "bench": "matmul"
                    },
                    "summary": {
                        "phases_ns": {
                            "init": {},
                            "compute": {}
                        },
                        "perf": null
                    }
                }),
                "summary.phases_ns.teardown",
            ),
            (
                serde_json::json!({
                    "meta": {
                        "schema_version": 2,
                        "bench": "matmul"
                    },
                    "summary": {
                        "phases_ns": {
                            "init": {},
                            "compute": {},
                            "teardown": {}
                        }
                    }
                }),
                "summary.perf",
            ),
        ];

        for (json_val, expected_field) in cases {
            let result = verify_required_structure(&json_val, types::CmpInputSide::JsonBaseline);

            match result {
                Err(types::CompareError::MissingRequiredField { input, field }) => {
                    assert!(matches!(input, types::CmpInputSide::JsonBaseline));
                    assert_eq!(field, expected_field);
                }
                Ok(()) => panic!("expected missing required field error"),
                Err(other) => panic!("unexpected compare error: {other:?}"),
            }
        }
    }

    #[test]
    fn verify_required_structure_missing_perf_events_test() {
        let json_val = serde_json::json!({
            "meta": {
                "schema_version": 2,
                "bench": "matmul"
            },
            "summary": {
                "phases_ns": {
                    "init": {},
                    "compute": {},
                    "teardown": {}
                },
                "perf": {}
            }
        });

        let result = verify_required_structure(&json_val, types::CmpInputSide::JsonCandidate);

        match result {
            Err(types::CompareError::MissingRequiredField { input, field }) => {
                assert!(matches!(input, types::CmpInputSide::JsonCandidate));
                assert_eq!(field, "summary.perf.events");
            }
            Ok(()) => panic!("expected missing perf events error"),
            Err(other) => panic!("unexpected compare error: {other:?}"),
        }
    }

    #[test]
    fn verify_required_structure_with_perf_events_test() {
        let json_val = serde_json::json!({
            "meta": {
                "schema_version": 2,
                "bench": "matmul"
            },
            "summary": {
                "phases_ns": {
                    "init": {},
                    "compute": {},
                    "teardown": {}
                },
                "perf": {
                    "events": {}
                }
            }
        });

        let result = verify_required_structure(&json_val, types::CmpInputSide::JsonBaseline);

        assert!(result.is_ok());
    }

    #[test]
    fn get_runner_json_wrong_schema_version_type_test() {
        let mut json_val = valid_runner_json_value();
        *json_val
            .pointer_mut("/meta/schema_version")
            .expect("schema_version must exist in valid test JSON") = serde_json::json!("2");

        let json_data = serde_json::to_string(&json_val).expect("failed to serialize test JSON");
        let json_path = PathBuf::from("baseline.json");

        let result = get_runner_json(
            &json_data,
            json_path.clone(),
            types::CmpInputSide::JsonBaseline,
        );

        match result {
            Err(types::CompareError::Deserialize {
                input,
                path,
                source,
            }) => {
                assert!(matches!(input, types::CmpInputSide::JsonBaseline));
                assert_eq!(path, json_path);
                assert_eq!(source.path().to_string(), "meta.schema_version");
            }
            Ok(_) => panic!("expected typed deserialization error"),
            Err(other) => panic!("unexpected compare error: {other:?}"),
        }
    }

    #[test]
    fn get_runner_json_wrong_compute_median_type_test() {
        let mut json_val = valid_runner_json_value();
        *json_val
            .pointer_mut("/summary/phases_ns/compute/median_ns")
            .expect("compute median_ns must exist in valid test JSON") = serde_json::json!("1000");

        let json_data = serde_json::to_string(&json_val).expect("failed to serialize test JSON");
        let json_path = PathBuf::from("candidate.json");

        let result = get_runner_json(
            &json_data,
            json_path.clone(),
            types::CmpInputSide::JsonCandidate,
        );

        match result {
            Err(types::CompareError::Deserialize {
                input,
                path,
                source,
            }) => {
                assert!(matches!(input, types::CmpInputSide::JsonCandidate));
                assert_eq!(path, json_path);
                assert_eq!(
                    source.path().to_string(),
                    "summary.phases_ns.compute.median_ns"
                );
            }
            Ok(_) => panic!("expected typed deserialization error"),
            Err(other) => panic!("unexpected compare error: {other:?}"),
        }
    }

    #[test]
    fn get_runner_json_wrong_compute_spread_type_test() {
        let mut json_val = valid_runner_json_value();
        *json_val
            .pointer_mut("/summary/phases_ns/compute/spread_percent")
            .expect("compute spread_percent must exist in valid test JSON") =
            serde_json::json!("10.0");

        let json_data = serde_json::to_string(&json_val).expect("failed to serialize test JSON");
        let json_path = PathBuf::from("candidate.json");

        let result = get_runner_json(
            &json_data,
            json_path.clone(),
            types::CmpInputSide::JsonCandidate,
        );

        match result {
            Err(types::CompareError::Deserialize {
                input,
                path,
                source,
            }) => {
                assert!(matches!(input, types::CmpInputSide::JsonCandidate));
                assert_eq!(path, json_path);
                assert_eq!(
                    source.path().to_string(),
                    "summary.phases_ns.compute.spread_percent"
                );
            }
            Ok(_) => panic!("expected typed deserialization error"),
            Err(other) => panic!("unexpected compare error: {other:?}"),
        }
    }

    #[test]
    fn verify_required_schema_mismatch_test() {
        let baseline_json = valid_runner_json_value();
        let mut candidate_json = valid_runner_json_value();

        *candidate_json
            .pointer_mut("/meta/schema_version")
            .expect("schema_version must exist in valid test JSON") = serde_json::json!(3);

        let baseline_data =
            serde_json::to_string(&baseline_json).expect("failed to serialize baseline test JSON");
        let candidate_data = serde_json::to_string(&candidate_json)
            .expect("failed to serialize candidate test JSON");

        let baseline = get_runner_json(
            &baseline_data,
            PathBuf::from("baseline.json"),
            types::CmpInputSide::JsonBaseline,
        )
        .expect("baseline test JSON must deserialize");

        let candidate = get_runner_json(
            &candidate_data,
            PathBuf::from("candidate.json"),
            types::CmpInputSide::JsonCandidate,
        )
        .expect("candidate test JSON must deserialize");

        let result = verify_required(&baseline, &candidate);

        match result {
            Err(types::CompareError::SchemaMismatch {
                baseline_ver,
                candidate_ver,
            }) => {
                assert_eq!(baseline_ver, 2);
                assert_eq!(candidate_ver, 3);
            }
            Ok(()) => panic!("expected schema mismatch error"),
            Err(other) => panic!("unexpected compare error: {other:?}"),
        }
    }

    #[test]
    fn verify_required_benchmark_mismatch_test() {
        let baseline_json = valid_runner_json_value();
        let mut candidate_json = valid_runner_json_value();

        *candidate_json
            .pointer_mut("/meta/bench")
            .expect("bench must exist in valid test JSON") = serde_json::json!("reduce");

        let baseline_data =
            serde_json::to_string(&baseline_json).expect("failed to serialize baseline test JSON");
        let candidate_data = serde_json::to_string(&candidate_json)
            .expect("failed to serialize candidate test JSON");

        let baseline = get_runner_json(
            &baseline_data,
            PathBuf::from("baseline.json"),
            types::CmpInputSide::JsonBaseline,
        )
        .expect("baseline test JSON must deserialize");

        let candidate = get_runner_json(
            &candidate_data,
            PathBuf::from("candidate.json"),
            types::CmpInputSide::JsonCandidate,
        )
        .expect("candidate test JSON must deserialize");

        let result = verify_required(&baseline, &candidate);

        match result {
            Err(types::CompareError::BenchmarkMismatch {
                baseline_bench,
                candidate_bench,
            }) => {
                assert_eq!(baseline_bench, "matmul");
                assert_eq!(candidate_bench, "reduce");
            }
            Ok(()) => panic!("expected benchmark mismatch error"),
            Err(other) => panic!("unexpected compare error: {other:?}"),
        }
    }

    #[test]
    fn verify_required_matching_inputs_test() {
        let baseline_json = valid_runner_json_value();
        let candidate_json = valid_runner_json_value();

        let baseline_data =
            serde_json::to_string(&baseline_json).expect("failed to serialize baseline test JSON");
        let candidate_data = serde_json::to_string(&candidate_json)
            .expect("failed to serialize candidate test JSON");

        let baseline = get_runner_json(
            &baseline_data,
            PathBuf::from("baseline.json"),
            types::CmpInputSide::JsonBaseline,
        )
        .expect("baseline test JSON must deserialize");

        let candidate = get_runner_json(
            &candidate_data,
            PathBuf::from("candidate.json"),
            types::CmpInputSide::JsonCandidate,
        )
        .expect("candidate test JSON must deserialize");

        let result = verify_required(&baseline, &candidate);

        assert!(result.is_ok());
    }

    #[test]
    fn get_json_str_missing_inputs_test() {
        let cases = [
            (
                PathBuf::from("/definitely/missing/perflab-baseline.json"),
                true,
            ),
            (
                PathBuf::from("/definitely/missing/perflab-candidate.json"),
                false,
            ),
        ];

        for (json_path, is_baseline) in cases {
            let input_side = if is_baseline {
                types::CmpInputSide::JsonBaseline
            } else {
                types::CmpInputSide::JsonCandidate
            };

            let result = get_json_str(json_path.clone(), input_side);

            match result {
                Err(types::CompareError::ReadInput { input, path, .. }) => {
                    assert_eq!(path, json_path);

                    if is_baseline {
                        assert!(matches!(input, types::CmpInputSide::JsonBaseline));
                    } else {
                        assert!(matches!(input, types::CmpInputSide::JsonCandidate));
                    }
                }
                Ok(_) => panic!("expected input read error"),
                Err(other) => panic!("unexpected compare error: {other:?}"),
            }
        }
    }

    #[test]
    fn get_json_str_directory_path_test() {
        let dir_path = PathBuf::from("runner");
        let result = get_json_str(dir_path.clone(), types::CmpInputSide::JsonBaseline);

        match result {
            Err(types::CompareError::ReadInput { input, path, .. }) => {
                assert!(matches!(input, types::CmpInputSide::JsonBaseline));
                assert_eq!(path, dir_path);
            }
            Ok(_) => panic!("expected input read error for directory path"),
            Err(other) => panic!("unexpected compare error: {other:?}"),
        }
    }
}
