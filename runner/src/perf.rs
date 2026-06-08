use crate::config::get_runner_args;
use crate::types;
use std::{
    process::{Command, Output},
    sync::OnceLock,
};

static PERF_EVENTS_REQUESTED: OnceLock<types::PerfRequestedEvents> = OnceLock::new();

pub fn run_perf(timestamp: &String) -> Result<Output, std::io::Error> {
    Command::new("perf")
        .args(get_perf_stat_args(timestamp))
        .output()
}

pub fn get_perf_requested_events() -> &'static types::PerfRequestedEvents {
    PERF_EVENTS_REQUESTED.get_or_init(|| vec!["cycles:u".to_string(), "instructions:u".to_string()])
}

pub fn get_perf_stat_args(timestamp: &String) -> types::PerfStatArgs {
    let runner_args = get_runner_args();
    let mut perf_stat_args: types::PerfStatArgs = types::PerfStatArgs::new();

    perf_stat_args.push("stat".to_string());
    perf_stat_args.push("-x,".to_string());
    perf_stat_args.push("-e".to_string());
    perf_stat_args.push(get_perf_requested_events().join(","));
    perf_stat_args.push("-o".to_string());
    perf_stat_args.push(format!("out/perf_{}_{}.csv", timestamp, runner_args.bench));
    perf_stat_args.push("--".to_string());
    perf_stat_args.push(format!("out/{}", runner_args.bench));

    perf_stat_args
}
