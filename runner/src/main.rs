use clap::Parser;
use perflab::compare;
use perflab::config;
use perflab::pipeline;

fn main() {
    let cl = config::CmdLine::parse();

    dispach_cmd(cl.command);
}

fn dispach_cmd(args: config::Commands) {
    match args {
        config::Commands::Run {
            warmup,
            reps,
            cpu,
            perf,
            bench,
            compiler,
            compiler_args,
        } => {
            config::set_runner_args(warmup, reps, cpu, perf, bench, compiler, compiler_args);

            pipeline::execute();
        }

        config::Commands::Compare {
            baseline,
            candidate,
        } => {
            config::set_cmp_args(baseline, candidate);

            compare::execute();
        }
    }
}
