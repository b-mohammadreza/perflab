use crate::io;
use crate::meta::RunnerMetadata;
use crate::perf;
use std::{
    collections::HashMap,
    process::{Command, Output},
};

pub fn runner_run(
    perf: bool,
    runner_args: (&String, &String, &Vec<String>),
    metadata: &RunnerMetadata,
) {
    let mut output: Output;
    let mut perf_events: HashMap<String, u64> = HashMap::new();
    let mut fallback: bool = false;

    match perf {
        false => {
            output = run_bench(runner_args.0);
        }
        true => {
            (fallback, output) = perf::run_perf(runner_args.0, &metadata.timestamp);
        }
    }

    if output.status.success() == false {
        let runner_stdrr = String::from_utf8_lossy(&output.stderr);

        if perf == false || fallback == true {
            /*
             *  If <perf == true> and also <fallback == true>:
             *      at this point, because <output.status> is failed, it means
             *      "perf" command execution was failed in perf::run_perf() and
             *      also the fallback command execution
             *      ,run::run_bench() in perf::run_perf(), was failed. So we should panic!
             */
            panic!("perflab:\n{runner_stdrr}");
        } else {
            // perf == true and fallback == false, falling back...
            println!("perflab-Unable to get perf stat, falling back..., error:\n{runner_stdrr}");
            output = run_bench(runner_args.0);
            if output.status.success() == false {
                panic!("perflab:\n{}", String::from_utf8_lossy(&output.stderr));
            }
            fallback = true;
        }
    }

    let bench_json = String::from_utf8_lossy(&output.stdout).trim().to_string();

    if bench_json.is_empty() == true {
        panic!("perflab-Dead path!");
    }

    if perf == true && fallback == false {
        io::get_perf_events((&runner_args.0, &metadata.timestamp), &mut perf_events);
    }

    io::runner_write_json(fallback, &bench_json, &perf_events, runner_args, &metadata);
}

pub fn run_bench(bench: &String) -> Output {
    Command::new(format!("out/{}", bench))
        .output()
        .unwrap_or_else(|e| {
            panic!("perflab-Failed to execute command(out/{bench}), error:\n{e}");
        })
}
