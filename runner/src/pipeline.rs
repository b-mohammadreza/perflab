use crate::affinity;
use crate::compile;
use crate::config::Commands;
use crate::io;
use crate::meta;
use crate::run;

pub fn execute(config: Commands) {
    match config {
        Commands::Run {
            cpu,
            perf,
            bench,
            compiler,
            compiler_args,
        } => {
            if let Some(val) = cpu {
                affinity::set_affinity(val);
            }

            let metadata = meta::metadata_capture(&compiler);

            let result = compile::runner_compile((&bench, &compiler, &compiler_args));

            if result == 0 {
                let (fellback, bench_jason) =
                    run::runner_run(perf, (&bench, &compiler, &compiler_args), &metadata);

                io::finalize_and_write_result(
                    fellback,
                    (&bench, &compiler, &compiler_args, perf, cpu),
                    &metadata,
                    &bench_jason,
                );
            }
        }
    }
}
