use crate::config::get_runner_args;
use crate::io;
use crate::perf;
use crate::types;
use std::process::{Command, Output};

pub fn runner_warmup(warmup: u32) {
    for count in 1..=warmup {
        let output = run_bench();

        if output.status.success() == false {
            let runner_stdrr = String::from_utf8_lossy(&output.stderr);
            panic!("perflab-warmup:\n{runner_stdrr}");
        }

        println!("warmup - {count}/{warmup}...OK");
    }
}

pub fn runner_collect_samples(reps: u32, timestamp: &String) -> types::RunSampleVec {
    let mut run_samples: types::RunSampleVec = types::RunSampleVec::new();

    for count in 1..=reps {
        run_samples.push(runner_run(timestamp));

        println!("run - {count}/{reps}...OK");
    }

    run_samples
}

fn runner_run(timestamp: &String) -> types::RunSample {
    let runner_args = get_runner_args();

    let mut output: Output;
    let mut fellback: bool = false;

    match runner_args.perf {
        false => {
            output = run_bench();
        }
        true => {
            output = perf::run_perf(&timestamp).unwrap_or_else(|err| {
                println!("perflab-Failed to execute command(perf), falling back..., error:\n{err}");
                fellback = true;
                run_bench()
            });
        }
    }

    if output.status.success() == false {
        let runner_stdrr = String::from_utf8_lossy(&output.stderr);

        if runner_args.perf == false || fellback == true {
            /*
             *  If <perf == true> and also <fellback == true> (already fell back):
             *      at this point, because <output.status> is failed, it means
             *      "perf" command execution was failed in perf::run_perf() and
             *      also the fallback command execution
             *      ,run::run_bench() in perf::run_perf(), was failed. So we should panic!
             */
            panic!("perflab:\n{runner_stdrr}");
        } else {
            // <perf == true> and <fellback == false> (not yet fell back), falling back...
            println!("perflab-Unable to get perf stat, falling back..., error:\n{runner_stdrr}");
            output = run_bench();
            if output.status.success() == false {
                panic!("perflab-run:\n{}", String::from_utf8_lossy(&output.stderr));
            }
            fellback = true;
        }
    }

    let mut perf_json: Option<types::Perf> = None;
    if runner_args.perf == true && fellback == false {
        perf_json = Some(io::get_perf_events(&timestamp));
    }

    types::RunSample {
        bench_output: serde_json::from_str(String::from_utf8_lossy(&output.stdout).trim())
            .unwrap_or_else(|err| {
                panic!("perflab-Failed to parse bench output, error:\n{err}");
            }),
        perf: perf_json,
    }
}

pub fn run_bench() -> Output {
    let runner_args = get_runner_args();

    Command::new(format!("out/{}", runner_args.bench))
        .output()
        .unwrap_or_else(|e| {
            panic!(
                "perflab-run-Failed to execute command(out/{}), error:\n{e}",
                runner_args.bench
            );
        })
}
