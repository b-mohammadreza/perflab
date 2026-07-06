use crate::types;
use clap::{Parser, Subcommand};
use std::{path::PathBuf, sync::OnceLock};

#[derive(Parser)]
#[command(version, about, long_about=None)]
#[command(propagate_version = true)]
pub struct CmdLine {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Runs the benchmark. Options: [--warmup <num>] [--reps <num>] [--cpu <id>] [--perf] --bench <name>, --compiler <name> -- <flags...>
    Run {
        #[arg(short, long, value_name = "warmup")]
        warmup: Option<u32>,

        #[arg(short, long, value_parser = clap::value_parser!(u32).range(1..), value_name = "reps")]
        reps: Option<u32>,

        /// logical Linux CPU ID
        #[arg(short = 'u', long, value_name = "id")]
        cpu: Option<u16>,

        /// Collect perf stat counters (best-effort)
        #[arg(short, long, action = clap::ArgAction::SetTrue)]
        perf: bool,

        #[arg(short, long, value_name = "name")]
        bench: String,

        #[arg(short, long, value_name = "FILE")]
        compiler: PathBuf,

        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        compiler_args: Vec<String>,
    },

    /// Compare a baseline result JSON against a candidate result JSON.
    Compare {
        /// Baseline/reference result JSON file
        #[arg(value_name = "BASELINE")]
        baseline: PathBuf,

        /// Candidate/new result JSON file
        #[arg(value_name = "CANDIDATE")]
        candidate: PathBuf,

        #[arg(value_enum, default_value_t = types::Format::Text, short, long, value_name = "FORMAT")]
        format: types::Format,
    },
}

static RUNNER_CONFIG: OnceLock<types::RunnerArgs> = OnceLock::new();
static CMP_ARGS: OnceLock<types::CompareArgs> = OnceLock::new();

pub fn set_runner_args(
    warmup: Option<u32>,
    reps: Option<u32>,
    cpu: Option<u16>,
    perf: bool,
    bench: String,
    compiler: PathBuf,
    compiler_args: Vec<String>,
) {
    RUNNER_CONFIG
        .set(types::RunnerArgs {
            warmup,
            reps,
            cpu,
            perf,
            bench,
            compiler,
            compiler_args,
        })
        .expect("perflab-runner args already initialized!");
}

pub fn set_cmp_args(baseline: PathBuf, candidate: PathBuf, format: types::Format) {
    CMP_ARGS
        .set(types::CompareArgs {
            baseline,
            candidate,
            format,
        })
        .expect("perflab-compare args already initialized!")
}

pub fn get_runner_args() -> &'static types::RunnerArgs {
    RUNNER_CONFIG
        .get()
        .expect("perflab-runner args not initialized!")
}

pub fn get_cmp_arg() -> &'static types::CompareArgs {
    CMP_ARGS
        .get()
        .expect("perflab-compare args not initialized!")
}
