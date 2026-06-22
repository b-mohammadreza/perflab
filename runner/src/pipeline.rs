use crate::affinity;
use crate::compile;
use crate::config;
use crate::io;
use crate::meta;
use crate::results;
use crate::run;
use crate::summary;

pub fn execute() {
    let runner_args = config::get_runner_args();

    if let Some(val) = runner_args.cpu {
        affinity::set_affinity(val);
    }

    let result = compile::runner_compile();

    if result == 0 {
        let warmup = runner_args.warmup.unwrap_or(1u32);
        run::runner_warmup(warmup);

        meta::metadata_capture();

        let reps = runner_args.reps.unwrap_or(5u32);
        let timestamp = meta::get_timestamp();
        let run_samples = run::runner_collect_samples(reps, &timestamp);

        let summary = summary::compute(&run_samples);

        let result_schem_pretty = results::create_result_obj(&timestamp, &run_samples, &summary);
        io::write_result(&timestamp, &result_schem_pretty);
    }
}
