use crate::affinity;
use crate::compile;
use crate::config;
use crate::io;
use crate::meta;
use crate::run;

pub fn execute(config: config::Commands) {
    config::init_runner_args(config);

    let runner_args = config::get_runner_args();

    if let Some(val) = runner_args.cpu {
        affinity::set_affinity(val);
    }

    meta::metadata_capture();

    let result = compile::runner_compile();

    if result == 0 {
        let timestamp = meta::get_timestamp();

        let (fellback, bench_jason) = run::runner_run(&timestamp);

        io::finalize_and_write_result(fellback, &timestamp, &bench_jason);
    }
}
