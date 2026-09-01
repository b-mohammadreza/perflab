use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};

#[derive(Serialize, Deserialize)]
pub struct RunnerJson {
    pub meta: Meta,
    pub samples: RunSampleVec,
    pub summary: Summary,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RunSample {
    pub bench_output: Bench,
    pub perf: Option<Perf>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Summary {
    pub phases_ns: SummaryPhasesNs,
    pub perf: Option<PerfEvents>,
}

#[derive(Serialize, Deserialize)]
pub struct Meta {
    pub schema_version: u32,
    pub cpu_pin: Option<u16>,
    pub warmup: u32,
    pub reps: u32,
    pub timestamp: String,
    pub git_sha: String,
    pub compiler: MetaCompiler,
    pub uname: String,
    pub bench: String,
    pub compiler_args: Vec<String>,
    pub command: Vec<String>,
    pub workdir: String,
    pub perf_events_requested: Option<PerfRequestedEvents>,
    pub perf_stat_base_args: Option<PerfStatArgs>,
}

#[derive(Debug)]
pub struct RunnerSysEnvMetadata {
    pub git_sha: String,
    pub compiler_ver: String,
    pub uname: String,
    pub cur_dir: String,
}

#[derive(Serialize, Deserialize)]
pub struct MetaCompiler {
    pub path: String,
    pub version: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Bench {
    pub bench: String,
    pub check: HashMap<String, u64>,
    pub params: BenchParams,
    pub phases_ns: BenchPhasesNs,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct BenchParams {
    pub iters: u32,
    pub n: u64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct BenchPhasesNs {
    pub compute: u64,
    pub init: u64,
    pub teardown: u64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SummaryPhasesNs {
    pub compute: SummaryAttributes,
    pub init: SummaryAttributes,
    pub teardown: SummaryAttributes,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SummaryAttributes {
    pub median_ns: u64,
    pub min_ns: u64,
    pub max_ns: u64,
    pub spread_percent: Option<f64>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PerfEvents {
    pub events: HashMap<String, u64>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Perf {
    pub csv_path: String,
    pub perf_stat_args: PerfStatArgs,

    #[serde(flatten)]
    pub perf_events: PerfEvents,
}

pub type RunSampleVec = Vec<RunSample>;

pub struct BenchPhasesArrs {
    pub compute_arr: Vec<u64>,
    pub init_arr: Vec<u64>,
    pub teardown_arr: Vec<u64>,
}

pub type PerfArrs = HashMap<String, Vec<u64>>;

pub type PerfRequestedEvents = Vec<String>;

pub type PerfStatArgs = Vec<String>;

#[derive(Debug)]
pub struct RunnerArgs {
    pub warmup: Option<u32>,
    pub reps: Option<u32>,
    pub cpu: Option<u16>,
    pub perf: bool,
    pub bench: String,
    pub compiler: PathBuf,
    pub compiler_args: Vec<String>,
}

#[derive(Debug)]
pub struct CompareArgs {
    pub baseline: PathBuf,
    pub candidate: PathBuf,
    pub format: Format,
}

#[derive(Debug)]
pub struct CmpPerfEvent {
    pub event_name: String,
    pub baseline: u64,
    pub candidate: u64,
}

pub type CmpPerfEvents = Vec<CmpPerfEvent>;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Format {
    /// Compare result in text format
    Text,
    /// Compare result in markdown format
    Markdown,
    /// Compare result in csv format
    Csv,
}

pub enum CmpInputSide {
    JsonBaseline,
    JsonCandidate,
}

pub struct CmpItemData {
    pub item_name: String,
    pub baseline: u64,
    pub candidate: u64,
    pub abs_delta: String,
    pub percent_delta: String,
    pub baseline_spread: String,
    pub candidate_spread: String,
}

/// To store global comparison data and pass its reference to indivisual renderers
pub struct CmpGData {
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

pub trait CmpRenderer {
    fn render_cmp_result(&self) {
        self.render_cmp_header();
        self.render_summary_phases();
        self.render_summary_perf();
    }

    fn render_cmp_header(&self);
    fn render_summary_phases(&self);
    fn render_summary_perf(&self);
}

pub struct TextCmpRenderer<'cmp_g> {
    pub cmp_g_data: &'cmp_g CmpGData,
}

pub struct MarkdownCmpRenderer<'cmp_g> {
    pub cmp_g_data: &'cmp_g CmpGData,
}

pub struct CsvCmpRenderer<'cmp_g> {
    pub cmp_g_data: &'cmp_g CmpGData,
}

pub enum CompareError {
    ReadInput {
        input: CmpInputSide,
        path: PathBuf,
        source: std::io::Error,
    },
    MalformedJson {
        input: CmpInputSide,
        path: PathBuf,
        source: serde_json::Error,
    },
    MissingRequiredField {
        input: CmpInputSide,
        field: String,
    },
    Deserialize {
        input: CmpInputSide,
        path: PathBuf,
        source: serde_path_to_error::Error<serde_json::Error>,
    },
    SchemaMismatch {
        baseline_ver: u32,
        candidate_ver: u32,
    },
    BenchmarkMismatch {
        baseline_bench: String,
        candidate_bench: String,
    },
    NotImplSerdeSerialize {
        source: serde_json::Error,
    },
}
