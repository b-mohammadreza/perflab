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

    if verify_required(&baseline_obj, &candidate_obj) == true {
        verify_good_to_have(&baseline_obj, &candidate_obj);

        let bench = baseline_obj.meta.bench.clone();
        let schm_ver = baseline_obj.meta.schema_version.clone();
        let cmp_data: CmpGData = get_cmp_g_data(
            &baseline_path,
            &candidate_path,
            bench,
            schm_ver,
            &baseline_obj,
            &candidate_obj,
        );

        let cmp_renderer: Box<dyn CmpRenderer> = match cmp_args.format {
            types::Format::Text => Box::new(TextCmpRenderer {
                cmp_g_data: &cmp_data,
            }),
            types::Format::Markdown => Box::new(MarkdownCmpRenderer {
                cmp_g_data: &cmp_data,
            }),
            types::Format::Csv => Box::new(CsvCmpRenderer {
                cmp_g_data: &cmp_data,
            }),
        };

        cmp_renderer.render_cmp_result();
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
        eprintln!(
            "perflab-compare-error, schema mismatch: \n\tbaseline=\t{} \n\tcandidate=\t{}",
            format_warn_err_value(&baseline.meta.schema_version),
            format_warn_err_value(&candidate.meta.schema_version)
        );
        result = result && false;
    }
    if baseline.meta.bench != candidate.meta.bench {
        eprintln!(
            "perflab-compare-error, benchmark mismatch: \n\tbaseline=\t{} \n\tcandidate=\t{}",
            format_warn_err_value(&baseline.meta.bench),
            format_warn_err_value(&candidate.meta.bench)
        );
        result = result && false;
    }

    result
}

fn verify_good_to_have(baseline: &types::RunnerJson, candidate: &types::RunnerJson) {
    if baseline.meta.compiler.path != candidate.meta.compiler.path {
        eprintln!(
            "perflab-compare-warning, compiler.path differ: \n\tbaseline=\t{} \n\tcandidate=\t{}",
            format_warn_err_value(&baseline.meta.compiler.path),
            format_warn_err_value(&candidate.meta.compiler.path)
        );
    }
    if baseline.meta.compiler.version != candidate.meta.compiler.version {
        eprintln!(
            "perflab-compare-warning, compiler.version differ: \n\tbaseline=\t{} \n\tcandidate=\t{}",
            format_warn_err_value(&baseline.meta.compiler.version),
            format_warn_err_value(&candidate.meta.compiler.version)
        );
    }
    if baseline.meta.compiler_args != candidate.meta.compiler_args {
        eprintln!(
            "perflab-compare-warning, compiler_args differ: \n\tbaseline=\t{} \n\tcandidate=\t{}",
            format_warn_err_value(&baseline.meta.compiler_args),
            format_warn_err_value(&candidate.meta.compiler_args)
        );
    }
    if baseline.meta.cpu_pin != candidate.meta.cpu_pin {
        eprintln!(
            "perflab-compare-warning, cpu_pin differ: \n\tbaseline=\t{} \n\tcandidate=\t{}",
            format_warn_err_value(&baseline.meta.cpu_pin),
            format_warn_err_value(&candidate.meta.cpu_pin)
        );
    }
    if baseline.meta.warmup != candidate.meta.warmup {
        eprintln!(
            "perflab-compare-warning, warmup differ: \n\tbaseline=\t{} \n\tcandidate=\t{}",
            format_warn_err_value(&baseline.meta.warmup),
            format_warn_err_value(&candidate.meta.warmup)
        );
    }
    if baseline.meta.reps != candidate.meta.reps {
        eprintln!(
            "perflab-compare-warning, reps differ: \n\tbaseline=\t{} \n\tcandidate=\t{}",
            format_warn_err_value(&baseline.meta.reps),
            format_warn_err_value(&candidate.meta.reps)
        );
    }
    if baseline.meta.perf_events_requested != candidate.meta.perf_events_requested {
        eprintln!(
            "perflab-compare-warning, perf_events_requested differ: \n\tbaseline=\t{} \n\tcandidate=\t{}",
            format_warn_err_value(&baseline.meta.perf_events_requested),
            format_warn_err_value(&candidate.meta.perf_events_requested)
        );
    }
    if baseline.meta.workdir != candidate.meta.workdir {
        eprintln!(
            "perflab-compare-warning, workdir differ: \n\tbaseline=\t{} \n\tcandidate=\t{}",
            format_warn_err_value(&baseline.meta.workdir),
            format_warn_err_value(&candidate.meta.workdir)
        );
    }
    if baseline.meta.git_sha != candidate.meta.git_sha {
        eprintln!(
            "perflab-compare-warning, git_sha differ: \n\tbaseline=\t{} \n\tcandidate=\t{}",
            format_warn_err_value(&baseline.meta.git_sha),
            format_warn_err_value(&candidate.meta.git_sha)
        );
    }
    if baseline.meta.uname != candidate.meta.uname {
        eprintln!(
            "perflab-compare-warning, uname differ: \n\tbaseline=\t{} \n\tcandidate=\t{}",
            format_warn_err_value(&baseline.meta.uname),
            format_warn_err_value(&candidate.meta.uname)
        );
    }
}

fn get_cmp_g_data(
    baseline_path: &String,
    candidate_path: &String,
    bench: String,
    schm_ver: u32,
    baseline: &types::RunnerJson,
    candidate: &types::RunnerJson,
) -> CmpGData {
    let mut cmp_g_data: CmpGData = CmpGData::new();

    cmp_g_data.baseline_path = baseline_path.to_string();
    cmp_g_data.candidate_path = candidate_path.to_string();
    cmp_g_data.bench = bench;
    cmp_g_data.schm_ver = schm_ver;

    cmp_g_data.init_phase = CmpItemData {
        item_name: String::from("init"),
        baseline: baseline.summary.phases_ns.init,
        candidate: candidate.summary.phases_ns.init,
        abs_delta: get_abs_delta(
            baseline.summary.phases_ns.init,
            candidate.summary.phases_ns.init,
        ),
        percent_delta: get_percent_delta(
            baseline.summary.phases_ns.init,
            candidate.summary.phases_ns.init,
        ),
    };

    cmp_g_data.compute_phase = CmpItemData {
        item_name: String::from("compute"),
        baseline: baseline.summary.phases_ns.compute,
        candidate: candidate.summary.phases_ns.compute,
        abs_delta: get_abs_delta(
            baseline.summary.phases_ns.compute,
            candidate.summary.phases_ns.compute,
        ),
        percent_delta: get_percent_delta(
            baseline.summary.phases_ns.compute,
            candidate.summary.phases_ns.compute,
        ),
    };

    cmp_g_data.tear_down_phase = CmpItemData {
        item_name: String::from("teardown"),
        baseline: baseline.summary.phases_ns.teardown,
        candidate: candidate.summary.phases_ns.teardown,
        abs_delta: get_abs_delta(
            baseline.summary.phases_ns.teardown,
            candidate.summary.phases_ns.teardown,
        ),
        percent_delta: get_percent_delta(
            baseline.summary.phases_ns.teardown,
            candidate.summary.phases_ns.teardown,
        ),
    };

    verify_summary_perf_avail(&mut cmp_g_data, baseline, candidate);

    if cmp_g_data.perf_unavail == false && cmp_g_data.perf_events_unavail == false {
        let perf_events = get_common_perf_events(baseline, candidate);

        for event in perf_events {
            cmp_g_data.perf_events.push(CmpItemData {
                item_name: event.event_name,
                baseline: event.baseline,
                candidate: event.candidate,
                abs_delta: get_abs_delta(event.baseline, event.candidate),
                percent_delta: get_percent_delta(event.baseline, event.candidate),
            });
        }
    }

    cmp_g_data
}

fn verify_summary_perf_avail(
    cmp_data: &mut CmpGData,
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

fn format_warn_err_value<T>(value: &T) -> String
where
    T: ?Sized + serde::Serialize,
{
    serde_json::to_string(value).unwrap_or_else(|err| {
        panic!("perflab-compare-value is not implementing serde::Serialize trait! error:\n{err}");
    })
}

/// To store global comparison data and pass its reference to indivisual renderers
struct CmpGData {
    pub baseline_path: String,
    pub candidate_path: String,
    pub bench: String,
    pub schm_ver: u32,
    pub init_phase: CmpItemData,
    pub compute_phase: CmpItemData,
    pub tear_down_phase: CmpItemData,
    pub perf_unavail: bool,
    pub perf_events_unavail: bool,
    pub perf_events: Vec<CmpItemData>,
}

impl CmpGData {
    fn new() -> CmpGData {
        CmpGData {
            baseline_path: String::from(""),
            candidate_path: String::from(""),
            bench: String::from(""),
            schm_ver: 0,
            init_phase: CmpItemData {
                item_name: String::from(""),
                baseline: 0u64,
                candidate: 0u64,
                abs_delta: String::from(""),
                percent_delta: String::from(""),
            },
            compute_phase: CmpItemData {
                item_name: String::from(""),
                baseline: 0u64,
                candidate: 0u64,
                abs_delta: String::from(""),
                percent_delta: String::from(""),
            },
            tear_down_phase: CmpItemData {
                item_name: String::from(""),
                baseline: 0u64,
                candidate: 0u64,
                abs_delta: String::from(""),
                percent_delta: String::from(""),
            },
            perf_unavail: false,
            perf_events_unavail: false,
            perf_events: Vec::new(),
        }
    }
}

struct CmpItemData {
    pub item_name: String,
    pub baseline: u64,
    pub candidate: u64,
    pub abs_delta: String,
    pub percent_delta: String,
}

trait CmpRenderer {
    fn render_cmp_result(&self) {
        self.render_cmp_header();
        self.render_summary_phases();
        self.render_summary_perf();
    }

    fn render_cmp_header(&self);
    fn render_summary_phases(&self);
    fn render_summary_perf(&self);
}

struct TextCmpRenderer<'cmp_g> {
    cmp_g_data: &'cmp_g CmpGData,
}

impl<'cmp_g> CmpRenderer for TextCmpRenderer<'cmp_g> {
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
        const INDENT_LEN: usize = 30;

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
            self.cmp_g_data.init_phase.item_name,
            self.cmp_g_data.init_phase.baseline,
            self.cmp_g_data.init_phase.candidate,
            self.cmp_g_data.init_phase.abs_delta,
            self.cmp_g_data.init_phase.percent_delta,
            w = INDENT_LEN
        );
        println!(
            "\t{:<w$}{:<w$}{:<w$}{:<w$}{:<w$}",
            self.cmp_g_data.compute_phase.item_name,
            self.cmp_g_data.compute_phase.baseline,
            self.cmp_g_data.compute_phase.candidate,
            self.cmp_g_data.compute_phase.abs_delta,
            self.cmp_g_data.compute_phase.percent_delta,
            w = INDENT_LEN
        );
        println!(
            "\t{:<w$}{:<w$}{:<w$}{:<w$}{:<w$}",
            self.cmp_g_data.tear_down_phase.item_name,
            self.cmp_g_data.tear_down_phase.baseline,
            self.cmp_g_data.tear_down_phase.candidate,
            self.cmp_g_data.tear_down_phase.abs_delta,
            self.cmp_g_data.tear_down_phase.percent_delta,
            w = INDENT_LEN
        );
        println!("");
    }

    fn render_summary_perf(&self) {
        const INDENT_LEN: usize = 30;

        println!("Perf comparison:");

        if self.cmp_g_data.perf_unavail {
            println!(
                "perflab-compare-perf unavailable: perf data is unavailable in one or both inputs"
            );
        } else if self.cmp_g_data.perf_events_unavail {
            println!(
                "perflab-compare-perf unavailable: perf events are unavailable in one or both inputs"
            );
        } else {
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

struct MarkdownCmpRenderer<'cmp_g> {
    cmp_g_data: &'cmp_g CmpGData,
}

impl<'cmp_g> CmpRenderer for MarkdownCmpRenderer<'cmp_g> {
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
            "| {} | {} | {} | {} | {} |\n|---|---:|---:|---:|---:|",
            "phase", "baseline(ns)", "candidate(ns)", "delta(ns)", "delta(%)"
        );
        println!(
            "| {} | {} | {} | {} | {} |",
            self.cmp_g_data.init_phase.item_name,
            self.cmp_g_data.init_phase.baseline,
            self.cmp_g_data.init_phase.candidate,
            self.cmp_g_data.init_phase.abs_delta,
            self.cmp_g_data.init_phase.percent_delta,
        );
        println!(
            "| {} | {} | {} | {} | {} |",
            self.cmp_g_data.compute_phase.item_name,
            self.cmp_g_data.compute_phase.baseline,
            self.cmp_g_data.compute_phase.candidate,
            self.cmp_g_data.compute_phase.abs_delta,
            self.cmp_g_data.compute_phase.percent_delta,
        );
        println!(
            "| {} | {} | {} | {} | {} |",
            self.cmp_g_data.tear_down_phase.item_name,
            self.cmp_g_data.tear_down_phase.baseline,
            self.cmp_g_data.tear_down_phase.candidate,
            self.cmp_g_data.tear_down_phase.abs_delta,
            self.cmp_g_data.tear_down_phase.percent_delta,
        );
        println!("");
    }

    fn render_summary_perf(&self) {
        println!("## Perf comparison:");

        if self.cmp_g_data.perf_unavail {
            println!(
                "perflab-compare-perf unavailable: perf data is unavailable in one or both inputs"
            );
        } else if self.cmp_g_data.perf_events_unavail {
            println!(
                "perflab-compare-perf unavailable: perf events are unavailable in one or both inputs"
            );
        } else {
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

struct CsvCmpRenderer<'cmp_g> {
    cmp_g_data: &'cmp_g CmpGData,
}

impl<'cmp_g> CmpRenderer for CsvCmpRenderer<'cmp_g> {
    fn render_cmp_header(&self) {
        println!("kind,name,baseline,candidate,delta,delta_percent");
    }

    fn render_summary_phases(&self) {
        println!(
            "{},{},{},{},{},{}",
            "phase",
            self.cmp_g_data.init_phase.item_name,
            self.cmp_g_data.init_phase.baseline,
            self.cmp_g_data.init_phase.candidate,
            self.cmp_g_data.init_phase.abs_delta,
            self.cmp_g_data.init_phase.percent_delta,
        );
        println!(
            "{},{},{},{},{},{}",
            "phase",
            self.cmp_g_data.compute_phase.item_name,
            self.cmp_g_data.compute_phase.baseline,
            self.cmp_g_data.compute_phase.candidate,
            self.cmp_g_data.compute_phase.abs_delta,
            self.cmp_g_data.compute_phase.percent_delta,
        );
        println!(
            "{},{},{},{},{},{}",
            "phase",
            self.cmp_g_data.tear_down_phase.item_name,
            self.cmp_g_data.tear_down_phase.baseline,
            self.cmp_g_data.tear_down_phase.candidate,
            self.cmp_g_data.tear_down_phase.abs_delta,
            self.cmp_g_data.tear_down_phase.percent_delta,
        );
    }

    fn render_summary_perf(&self) {
        if self.cmp_g_data.perf_unavail {
            eprintln!(
                "perflab-compare-perf unavailable: perf data is unavailable in one or both inputs"
            );
        } else if self.cmp_g_data.perf_events_unavail {
            eprintln!(
                "perflab-compare-perf unavailable: perf events are unavailable in one or both inputs"
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
