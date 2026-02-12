use crate::meta::RunnerMetadata;
use crate::perf;
use std::process::{Command, Output};

/// runner_args: &bench, &compiler, &compiler_args
pub fn runner_run(
    perf: bool,
    runner_args: (&String, &String, &Vec<String>),
    metadata: &RunnerMetadata,
) -> (bool, String) {
    let mut output: Output;
    let mut fellback: bool = false;

    match perf {
        false => {
            output = run_bench(runner_args.0);
        }
        true => {
            (fellback, output) = perf::run_perf(runner_args.0, &metadata.timestamp);
        }
    }

    if output.status.success() == false {
        let runner_stdrr = String::from_utf8_lossy(&output.stderr);

        if perf == false || fellback == true {
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
            output = run_bench(runner_args.0);
            if output.status.success() == false {
                panic!("perflab:\n{}", String::from_utf8_lossy(&output.stderr));
            }
            fellback = true;
        }
    }

    (
        fellback,
        String::from_utf8_lossy(&output.stdout).trim().to_string(),
    )
}

pub fn run_bench(bench: &String) -> Output {
    Command::new(format!("out/{}", bench))
        .output()
        .unwrap_or_else(|e| {
            panic!("perflab-Failed to execute command(out/{bench}), error:\n{e}");
        })
}
