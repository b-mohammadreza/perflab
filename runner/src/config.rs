use clap::{Parser, Subcommand};
use std::sync::OnceLock;

#[derive(Parser)]
#[command(version, about, long_about=None)]
#[command(propagate_version = true)]
pub struct CmdLine {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Runs the benchmark. Options: [--cpu <id>] [--perf] --bench <name>, --compiler <name> -- <flags...>
    Run {
        /// logical Linux CPU ID
        #[arg(long, value_name = "id")]
        cpu: Option<u16>,

        /// Collect perf stat counters (best-effort)
        #[arg(short, long, action = clap::ArgAction::SetTrue)]
        perf: bool,

        #[arg(short, long, value_name = "name")]
        bench: String,

        #[arg(short, long, value_name = "path")]
        compiler: String,

        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        compiler_args: Vec<String>,
    },
}

#[derive(Debug)]
pub struct RunnerArgs {
    pub cpu: Option<u16>,
    pub perf: bool,
    pub bench: String,
    pub compiler: String,
    pub compiler_args: Vec<String>,
}

static RUNNER_CONFIG: OnceLock<RunnerArgs> = OnceLock::new();

pub fn init_runner_args(args: Commands) {
    match args {
        Commands::Run {
            cpu,
            perf,
            bench,
            compiler,
            compiler_args,
        } => {
            RUNNER_CONFIG
                .set(RunnerArgs {
                    cpu,
                    perf,
                    bench,
                    compiler,
                    compiler_args,
                })
                .expect("perflab-runner args already initialized!");
        }
    }
}

pub fn get_runner_args() -> &'static RunnerArgs {
    RUNNER_CONFIG
        .get()
        .expect("perflab-runner args not initialized!")
}
