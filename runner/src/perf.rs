use crate::run;
use std::process::{Command, Output};

pub fn run_perf(bench: &String, timestamp: &String) -> (bool, Output) {
    let perf_output = Command::new("perf")
        .arg("stat")
        .arg("-x,")
        .arg("-e")
        .arg("cycles:u,instructions:u,nonesence")
        .arg("-o")
        .arg(format!("out/perf_{}_{}.csv", timestamp, bench))
        .arg("--")
        .arg(format!("out/{}", bench))
        .output();

    match perf_output {
        Ok(val) => (false, val),
        Err(err) => {
            println!("perflab-Failed to execute command(perf), falling back..., error:\n{err}");
            (true, run::run_bench(bench))
        }
    }
}
