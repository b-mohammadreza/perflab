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
    pub phases_ns: BenchPhasesNs,
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
}
