use crate::io;
use crate::paths;
use crate::types;
use std::{
    collections::HashMap,
    process::{Command, Output},
    sync::OnceLock,
};

static PERF_EVENTS_REQUESTED: OnceLock<types::PerfRequestedEvents> = OnceLock::new();

pub fn run_perf(timestamp: &String, rep: u32) -> Result<Output, std::io::Error> {
    Command::new("perf")
        .args(get_perf_stat_args(timestamp, rep))
        .output()
}

pub fn get_perf_requested_events() -> &'static types::PerfRequestedEvents {
    PERF_EVENTS_REQUESTED.get_or_init(|| vec!["cycles:u".to_string(), "instructions:u".to_string()])
}

pub fn get_perf_stat_base_args() -> types::PerfStatArgs {
    let mut perf_stat_base_args: types::PerfStatArgs = types::PerfStatArgs::new();

    perf_stat_base_args.push("stat".to_string());
    perf_stat_base_args.push("-x,".to_string());
    perf_stat_base_args.push("-e".to_string());
    perf_stat_base_args.push(get_perf_requested_events().join(","));

    perf_stat_base_args
}

pub fn get_perf_events(timestamp: &String, rep: u32) -> types::Perf {
    let csv_file_text = io::read_perf_file(timestamp, rep);
    let mut perf_events: HashMap<String, u64> = HashMap::new();

    for line in csv_file_text.lines() {
        let fields: Vec<&str> = line.split(',').collect();

        let stat = match fields[0].trim().replace(',', "").parse() {
            Ok(val) => val,
            Err(_) => continue,
        };

        perf_events.insert(fields[2].trim().to_string(), stat);
    }

    types::Perf {
        csv_path: paths::get_perf_stat_path(timestamp, rep),
        perf_stat_args: get_perf_stat_args(timestamp, rep),
        perf_events: types::PerfEvents {
            events: perf_events,
        },
    }
}

fn get_perf_stat_args(timestamp: &String, rep: u32) -> types::PerfStatArgs {
    let mut perf_stat_args: types::PerfStatArgs = get_perf_stat_base_args();

    perf_stat_args.push("-o".to_string());
    perf_stat_args.push(paths::get_perf_stat_path(timestamp, rep));
    perf_stat_args.push("--".to_string());
    perf_stat_args.push(paths::get_bench_bin_path());

    perf_stat_args
}
