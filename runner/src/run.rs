use crate::config::get_runner_args;
use crate::perf;
use std::process::{Command, Output};

pub fn runner_run(timestamp: &String) -> (bool, String) {
    let runner_args = get_runner_args();

    let mut output: Output;
    let mut fellback: bool = false;

    match runner_args.perf {
        false => {
            output = run_bench();
        }
        true => {
            (fellback, output) = perf::run_perf(timestamp);
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

pub fn run_bench() -> Output {
    let runner_args = get_runner_args();

    Command::new(format!("out/{}", runner_args.bench))
        .output()
        .unwrap_or_else(|e| {
            panic!(
                "perflab-Failed to execute command(out/{}), error:\n{e}",
                runner_args.bench
            );
        })
}
