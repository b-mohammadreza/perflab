use crate::compile;
use crate::config::Commands;
use crate::meta;
use crate::run;

pub fn execute(config: Commands) {
    match config {
        Commands::Run {
            perf,
            bench,
            compiler,
            compiler_args,
        } => {
            let metadata = meta::metadata_capture(&compiler);

            let result = compile::runner_compile((&bench, &compiler, &compiler_args));
            if result == 0 {
                run::runner_run(perf, (&bench, &compiler, &compiler_args), &metadata);
            }
        }
    }
}
