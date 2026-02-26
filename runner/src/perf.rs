use crate::config::get_runner_args;
use std::process::{Command, Output};

pub fn run_perf(timestamp: &String) -> Result<Output, std::io::Error> {
    let runner_args = get_runner_args();

    Command::new("perf")
        .arg("stat")
        .arg("-x,")
        .arg("-e")
        .arg("cycles:u,instructions:u")
        .arg("-o")
        .arg(format!("out/perf_{}_{}.csv", timestamp, runner_args.bench))
        .arg("--")
        .arg(format!("out/{}", runner_args.bench))
        .output()
}
