use crate::config::get_runner_args;
use crate::run;
use std::process::{Command, Output};

pub fn run_perf(timestamp: &String) -> (bool, Output) {
    let runner_args = get_runner_args();

    let perf_output = Command::new("perf")
        .arg("stat")
        .arg("-x,")
        .arg("-e")
        .arg("cycles:u,instructions:u")
        .arg("-o")
        .arg(format!("out/perf_{}_{}.csv", timestamp, runner_args.bench))
        .arg("--")
        .arg(format!("out/{}", runner_args.bench))
        .output();

    match perf_output {
        Ok(val) => (false, val),
        Err(err) => {
            println!("perflab-Failed to execute command(perf), falling back..., error:\n{err}");
            (true, run::run_bench())
        }
    }
}
