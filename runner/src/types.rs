use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize)]
pub struct RunnerJson<'lifetime> {
    pub meta: Meta<'lifetime>,
    pub samples: &'lifetime RunSampleVec,
    pub summary: &'lifetime Summary,
}

#[derive(Serialize)]
pub struct RunSample {
    pub bench_output: Bench,
    pub perf: Option<Perf>,
}

#[derive(Serialize)]
pub struct Summary {
    pub phases_ns: BenchPhasesNs,
    pub perf: Option<Perf>,
}

#[derive(Serialize)]
pub struct Meta<'lifetime> {
    pub schema_version: u32,
    pub cpu_pin: Option<u16>,
    pub warmup: u32,
    pub reps: u32,
    pub timestamp: &'lifetime String,
    pub git_sha: &'lifetime String,
    pub compiler: MetaCompiler<'lifetime>,
    pub uname: &'lifetime String,
    pub bench: &'lifetime String,
    pub compiler_args: &'lifetime Vec<String>,
    pub command: Vec<String>,
    pub workdir: &'lifetime String,
    pub perf_events_requested: Option<&'lifetime PerfRequestedEvents>,
    pub perf_stat_args: Option<&'lifetime PerfStatArgs>,
}

#[derive(Debug)]
pub struct RunnerSysEnvMetadata {
    pub git_sha: String,
    pub compiler_ver: String,
    pub uname: String,
    pub cur_dir: String,
}

#[derive(Serialize)]
pub struct MetaCompiler<'lifetime> {
    pub path: &'lifetime String,
    pub version: &'lifetime String,
}

#[derive(Serialize, Deserialize)]
pub struct Bench {
    pub bench: String,
    pub check: HashMap<String, u64>,
    pub params: BenchParams,
    pub phases_ns: BenchPhasesNs,
}

#[derive(Serialize, Deserialize)]
pub struct BenchParams {
    pub iters: u32,
    pub n: u64,
}

#[derive(Serialize, Deserialize)]
pub struct BenchPhasesNs {
    pub compute: u64,
    pub init: u64,
    pub teardown: u64,
}

#[derive(Serialize)]
pub struct Perf {
    pub events: HashMap<String, u64>,
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
