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

    let mut comparison_warnings: Vec<types::ComparisonWarning> =
        verify_good_to_have(&baseline_obj, &candidate_obj);

    let mut perf_warning: Option<types::ComparisonWarning> = None;
    if let Some(warning) = verify_summary_perf_avail(&baseline_obj, &candidate_obj) {
        perf_warning = Some(warning.clone());
        comparison_warnings.push(warning);
    }

    let mut comparison_result: types::ComparisonResult = types::ComparisonResult::default();
    comparison_result.meta.baseline_path = cmp_args.baseline.to_string_lossy().trim().to_string();
    comparison_result.meta.candidate_path = cmp_args.candidate.to_string_lossy().trim().to_string();
    comparison_result.meta.bench = baseline_obj.meta.bench.clone();
    comparison_result.meta.schm_ver = baseline_obj.meta.schema_version.clone();

    comparison_result.warnings = comparison_warnings;

    comparison_result.phase_comparisons = get_phase_comparisons(&baseline_obj, &candidate_obj);
    comparison_result.perf_comparisons =
        get_perf_comparisons(&baseline_obj, &candidate_obj, perf_warning);

    render_warnings(&comparison_result.warnings)?;

    let cmp_renderer: Box<dyn types::CmpRenderer> = match cmp_args.format {
        types::Format::Text => Box::new(types::TextCmpRenderer {
            cmp_g_data: &comparison_result,
        }),
        types::Format::Markdown => Box::new(types::MarkdownCmpRenderer {
            cmp_g_data: &comparison_result,
        }),
        types::Format::Csv => Box::new(types::CsvCmpRenderer {
            cmp_g_data: &comparison_result,
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

fn render_warnings(warnings: &Vec<types::ComparisonWarning>) -> Result<(), types::CompareError> {
    for warning in warnings {
        match warning {
            types::ComparisonWarning::CompilerPathMismatch {
                baseline,
                candidate,
            } => {
                eprintln!(
                    "perflab-compare-warning: compiler.path differ, \n\tbaseline=\t{} \n\tcandidate=\t{}",
                    format_warn_err_value(&baseline)?,
                    format_warn_err_value(&candidate)?
                );
            }
            types::ComparisonWarning::CompilerVersionMismatch {
                baseline,
                candidate,
            } => {
                eprintln!(
                    "perflab-compare-warning: compiler.version differ, \n\tbaseline=\t{} \n\tcandidate=\t{}",
                    format_warn_err_value(&baseline)?,
                    format_warn_err_value(&candidate)?
                );
            }
            types::ComparisonWarning::CompilerArgsMismatch {
                baseline,
                candidate,
            } => {
                eprintln!(
                    "perflab-compare-warning: compiler_args differ, \n\tbaseline=\t{} \n\tcandidate=\t{}",
                    format_warn_err_value(&baseline)?,
                    format_warn_err_value(&candidate)?
                );
            }
            types::ComparisonWarning::CpuPinMismatch {
                baseline,
                candidate,
            } => {
                eprintln!(
                    "perflab-compare-warning: cpu_pin differ, \n\tbaseline=\t{} \n\tcandidate=\t{}",
                    format_warn_err_value(&baseline)?,
                    format_warn_err_value(&candidate)?
                );
            }
            types::ComparisonWarning::WarmupMismatch {
                baseline,
                candidate,
            } => {
                eprintln!(
                    "perflab-compare-warning: warmup differ, \n\tbaseline=\t{} \n\tcandidate=\t{}",
                    format_warn_err_value(&baseline)?,
                    format_warn_err_value(&candidate)?
                );
            }
            types::ComparisonWarning::RepsMismatch {
                baseline,
                candidate,
            } => {
                eprintln!(
                    "perflab-compare-warning: reps differ, \n\tbaseline=\t{} \n\tcandidate=\t{}",
                    format_warn_err_value(&baseline)?,
                    format_warn_err_value(&candidate)?
                );
            }
            types::ComparisonWarning::PerfEventsRequestedMismatch {
                baseline,
                candidate,
            } => {
                eprintln!(
                    "perflab-compare-warning: perf_events_requested differ, \n\tbaseline=\t{} \n\tcandidate=\t{}",
                    format_warn_err_value(&baseline)?,
                    format_warn_err_value(&candidate)?
                );
            }
            types::ComparisonWarning::WorkdirMismatch {
                baseline,
                candidate,
            } => {
                eprintln!(
                    "perflab-compare-warning: workdir differ, \n\tbaseline=\t{} \n\tcandidate=\t{}",
                    format_warn_err_value(&baseline)?,
                    format_warn_err_value(&candidate)?
                );
            }
            types::ComparisonWarning::GitShaMismatch {
                baseline,
                candidate,
            } => {
                eprintln!(
                    "perflab-compare-warning: git_sha differ, \n\tbaseline=\t{} \n\tcandidate=\t{}",
                    format_warn_err_value(&baseline)?,
                    format_warn_err_value(&candidate)?
                );
            }
            types::ComparisonWarning::UnameMismatch {
                baseline,
                candidate,
            } => {
                eprintln!(
                    "perflab-compare-warning: uname differ, \n\tbaseline=\t{} \n\tcandidate=\t{}",
                    format_warn_err_value(&baseline)?,
                    format_warn_err_value(&candidate)?
                );
            }
            _ => {}
        }
    }

    Ok(())
}

fn verify_good_to_have(
    baseline: &types::RunnerJson,
    candidate: &types::RunnerJson,
) -> Vec<types::ComparisonWarning> {
    let mut comparison_warnings: Vec<types::ComparisonWarning> = Vec::new();

    if baseline.meta.compiler.path != candidate.meta.compiler.path {
        comparison_warnings.push(types::ComparisonWarning::CompilerPathMismatch {
            baseline: baseline.meta.compiler.path.clone(),
            candidate: candidate.meta.compiler.path.clone(),
        });
    }
    if baseline.meta.compiler.version != candidate.meta.compiler.version {
        comparison_warnings.push(types::ComparisonWarning::CompilerVersionMismatch {
            baseline: baseline.meta.compiler.version.clone(),
            candidate: candidate.meta.compiler.version.clone(),
        });
    }
    if baseline
        .meta
        .compiler_args
        .ne(&candidate.meta.compiler_args)
    {
        comparison_warnings.push(types::ComparisonWarning::CompilerArgsMismatch {
            baseline: baseline.meta.compiler_args.clone(),
            candidate: candidate.meta.compiler_args.clone(),
        });
    }
    if baseline.meta.cpu_pin.ne(&candidate.meta.cpu_pin) {
        comparison_warnings.push(types::ComparisonWarning::CpuPinMismatch {
            baseline: baseline.meta.cpu_pin,
            candidate: candidate.meta.cpu_pin,
        });
    }
    if baseline.meta.warmup != candidate.meta.warmup {
        comparison_warnings.push(types::ComparisonWarning::WarmupMismatch {
            baseline: baseline.meta.warmup,
            candidate: candidate.meta.warmup,
        });
    }
    if baseline.meta.reps != candidate.meta.reps {
        comparison_warnings.push(types::ComparisonWarning::RepsMismatch {
            baseline: baseline.meta.reps,
            candidate: candidate.meta.reps,
        });
    }
    if baseline
        .meta
        .perf_events_requested
        .ne(&candidate.meta.perf_events_requested)
    {
        comparison_warnings.push(types::ComparisonWarning::PerfEventsRequestedMismatch {
            baseline: baseline.meta.perf_events_requested.clone(),
            candidate: candidate.meta.perf_events_requested.clone(),
        });
    }
    if baseline.meta.workdir != candidate.meta.workdir {
        comparison_warnings.push(types::ComparisonWarning::WorkdirMismatch {
            baseline: baseline.meta.workdir.clone(),
            candidate: candidate.meta.workdir.clone(),
        });
    }
    if baseline.meta.git_sha != candidate.meta.git_sha {
        comparison_warnings.push(types::ComparisonWarning::GitShaMismatch {
            baseline: baseline.meta.git_sha.clone(),
            candidate: candidate.meta.git_sha.clone(),
        });
    }
    if baseline.meta.uname != candidate.meta.uname {
        comparison_warnings.push(types::ComparisonWarning::UnameMismatch {
            baseline: baseline.meta.uname.clone(),
            candidate: candidate.meta.uname.clone(),
        });
    }

    comparison_warnings
}

fn get_phase_comparisons(
    baseline: &types::RunnerJson,
    candidate: &types::RunnerJson,
) -> Vec<types::PhaseComparison> {
    let mut phase_comparisons: Vec<types::PhaseComparison> = Vec::new();

    phase_comparisons.push(types::PhaseComparison {
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
        baseline_spread: baseline.summary.phases_ns.init.spread_percent,
        candidate_spread: candidate.summary.phases_ns.init.spread_percent,
    });

    phase_comparisons.push(types::PhaseComparison {
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
        baseline_spread: baseline.summary.phases_ns.compute.spread_percent,
        candidate_spread: candidate.summary.phases_ns.compute.spread_percent,
    });

    phase_comparisons.push(types::PhaseComparison {
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
        baseline_spread: baseline.summary.phases_ns.teardown.spread_percent,
        candidate_spread: candidate.summary.phases_ns.teardown.spread_percent,
    });

    phase_comparisons
}

fn get_perf_comparisons(
    baseline: &types::RunnerJson,
    candidate: &types::RunnerJson,
    perf_warning: Option<types::ComparisonWarning>,
) -> Vec<types::PerfComparison> {
    let mut perf_comparisons: Vec<types::PerfComparison> = Vec::new();

    match perf_warning {
        Some(types::ComparisonWarning::PerfUnavailable) => perf_comparisons,
        Some(types::ComparisonWarning::PerfEventsUnavailable) => perf_comparisons,
        _ => {
            if let Some(baseline_perf_events) = baseline.summary.perf.as_ref() {
                if let Some(candidate_perf_events) = candidate.summary.perf.as_ref() {
                    for b_event in &baseline_perf_events.events {
                        for c_event in &candidate_perf_events.events {
                            if b_event.0 == c_event.0 {
                                perf_comparisons.push(types::PerfComparison {
                                    event_name: b_event.0.to_string(),
                                    baseline: *b_event.1,
                                    candidate: *c_event.1,
                                    abs_delta: get_abs_delta(*b_event.1, *c_event.1),
                                    percent_delta: get_percent_delta(*b_event.1, *c_event.1),
                                });
                            }
                        }
                    }
                }
            }

            perf_comparisons.sort_by(|elem_1, elem_2| elem_1.event_name.cmp(&elem_2.event_name));

            perf_comparisons
        }
    }
}

fn verify_summary_perf_avail(
    baseline: &types::RunnerJson,
    candidate: &types::RunnerJson,
) -> Option<types::ComparisonWarning> {
    if let None = baseline.summary.perf {
        Some(types::ComparisonWarning::PerfUnavailable)
    } else if let None = candidate.summary.perf {
        Some(types::ComparisonWarning::PerfUnavailable)
    } else if let Some(perf_events) = baseline.summary.perf.as_ref() {
        if perf_events.events.is_empty() {
            Some(types::ComparisonWarning::PerfEventsUnavailable)
        } else {
            None
        }
    } else if let Some(perf_events) = candidate.summary.perf.as_ref() {
        if perf_events.events.is_empty() {
            Some(types::ComparisonWarning::PerfEventsUnavailable)
        } else {
            None
        }
    } else {
        None
    }
}

fn get_abs_delta(baseline_phase: u64, candidate_phase: u64) -> i64 {
    candidate_phase as i64 - baseline_phase as i64
}

fn get_abs_delta_str(val: i64) -> String {
    format!("{:+}", val).to_string()
}

fn get_percent_delta(baseline_phase: u64, candidate_phase: u64) -> Option<f64> {
    match baseline_phase {
        0 => None,
        _ => Some((candidate_phase as f64 - baseline_phase as f64) / baseline_phase as f64 * 100.0),
    }
}

fn get_percent_delta_str(val: Option<f64>) -> String {
    match val {
        None => String::from("N/A"),
        Some(val) => format!("{:+.2}%", val).to_string(),
    }
}

fn get_spread_percent_str(spread: Option<f64>) -> String {
    match spread {
        None => String::from("null"),
        Some(val) => format!("{:.2}%", val).to_string(),
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

impl<'cmp_g> types::CmpRenderer for types::TextCmpRenderer<'cmp_g> {
    fn render_cmp_header(&self) {
        println!("");
        println!("PerfLab compare v0");
        println!("\tbaseline:\t{}", self.cmp_g_data.meta.baseline_path);
        println!("\tcandidate:\t{}", self.cmp_g_data.meta.candidate_path);
        println!("\tbench:\t{}", self.cmp_g_data.meta.bench);
        println!("\tschema:\t{}", self.cmp_g_data.meta.schm_ver);
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
        for item in &self.cmp_g_data.phase_comparisons {
            println!(
                "\t{:<w$}{:<w$}{:<w$}{:<w$}{:<w$}{:<w$}{:<w$}",
                item.item_name,
                item.baseline,
                item.candidate,
                get_abs_delta_str(item.abs_delta),
                get_percent_delta_str(item.percent_delta),
                get_spread_percent_str(item.baseline_spread),
                get_spread_percent_str(item.candidate_spread),
                w = INDENT_LEN
            );
        }
        println!("");
    }

    fn render_summary_perf(&self) {
        const INDENT_LEN: usize = 25;

        if self
            .cmp_g_data
            .warnings
            .contains(&types::ComparisonWarning::PerfUnavailable)
        {
            eprintln!(
                "perflab-compare-warning: perf unavailable, perf data is unavailable in one or both inputs"
            );
        } else if self
            .cmp_g_data
            .warnings
            .contains(&types::ComparisonWarning::PerfEventsUnavailable)
        {
            eprintln!(
                "perflab-compare-warning: perf unavailable, perf events are unavailable in one or both inputs"
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

            for item in &self.cmp_g_data.perf_comparisons {
                println!(
                    "\t{:<w$}{:<w$}{:<w$}{:<w$}{:<w$}",
                    item.event_name,
                    item.baseline,
                    item.candidate,
                    get_abs_delta_str(item.abs_delta),
                    get_percent_delta_str(item.percent_delta),
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
        println!("- baseline: `{}`", self.cmp_g_data.meta.baseline_path);
        println!("- candidate: `{}`", self.cmp_g_data.meta.candidate_path);
        println!("- bench: `{}`", self.cmp_g_data.meta.bench);
        println!("- schema: `{}`", self.cmp_g_data.meta.schm_ver);
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
        for item in &self.cmp_g_data.phase_comparisons {
            println!(
                "| {} | {} | {} | {} | {} | {} | {} |",
                item.item_name,
                item.baseline,
                item.candidate,
                get_abs_delta_str(item.abs_delta),
                get_percent_delta_str(item.percent_delta),
                get_spread_percent_str(item.baseline_spread),
                get_spread_percent_str(item.candidate_spread),
            );
        }
        println!("");
    }

    fn render_summary_perf(&self) {
        if self
            .cmp_g_data
            .warnings
            .contains(&types::ComparisonWarning::PerfUnavailable)
        {
            eprintln!(
                "perflab-compare-warning: perf unavailable, perf data is unavailable in one or both inputs"
            );
        } else if self
            .cmp_g_data
            .warnings
            .contains(&types::ComparisonWarning::PerfEventsUnavailable)
        {
            eprintln!(
                "perflab-compare-error: perf unavailable, perf events are unavailable in one or both inputs"
            );
        } else {
            println!("## Perf comparison:");
            println!(
                "| {} | {} | {} | {} | {} |\n|---|---:|---:|---:|---:|",
                "event", "baseline", "candidate", "delta", "delta(%)"
            );

            for item in &self.cmp_g_data.perf_comparisons {
                println!(
                    "| {} | {} | {} | {} | {} |",
                    item.event_name,
                    item.baseline,
                    item.candidate,
                    get_abs_delta_str(item.abs_delta),
                    get_percent_delta_str(item.percent_delta)
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
        for item in &self.cmp_g_data.phase_comparisons {
            println!(
                "{},{},{},{},{},{},{},{}",
                "phase",
                item.item_name,
                item.baseline,
                item.candidate,
                get_abs_delta_str(item.abs_delta),
                get_percent_delta_str(item.percent_delta),
                get_spread_percent_str(item.baseline_spread),
                get_spread_percent_str(item.candidate_spread),
            );
        }
    }

    fn render_summary_perf(&self) {
        if self
            .cmp_g_data
            .warnings
            .contains(&types::ComparisonWarning::PerfUnavailable)
        {
            eprintln!(
                "perflab-compare-warning: perf unavailable, perf data is unavailable in one or both inputs"
            );
        } else if self
            .cmp_g_data
            .warnings
            .contains(&&types::ComparisonWarning::PerfEventsUnavailable)
        {
            eprintln!(
                "perflab-compare-error: perf unavailable, perf events are unavailable in one or both inputs"
            );
        } else {
            for item in &self.cmp_g_data.perf_comparisons {
                println!(
                    "{},{},{},{},{},{}",
                    "perf",
                    item.event_name,
                    item.baseline,
                    item.candidate,
                    get_abs_delta_str(item.abs_delta),
                    get_percent_delta_str(item.percent_delta),
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
        assert_eq!(20, get_abs_delta(100, 120));
        assert_eq!(-20, get_abs_delta(120, 100));
        assert_eq!(0, get_abs_delta(100, 100));
    }

    #[test]
    fn get_percent_delta_test() {}

    #[test]
    fn get_spread_percent_test() {}

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
