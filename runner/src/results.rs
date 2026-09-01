use crate::config;
use crate::meta;
use crate::perf;
use crate::types;
use serde_json;
use std::env;

pub fn create_result_obj(
    timestamp: &String,
    run_samples: &types::RunSampleVec,
    summary: &types::Summary,
) -> String {
    let runner_args = config::get_runner_args();
    let sys_env = meta::get_sys_env_meta();
    let mut cmd_line: Vec<String> = Vec::new();

    for argument in env::args_os() {
        cmd_line.push(argument.to_string_lossy().trim().to_string());
    }

    let runner_json = types::RunnerJson {
        meta: types::Meta {
            schema_version: 2,
            cpu_pin: runner_args.cpu,
            warmup: runner_args.warmup.unwrap_or(1u32),
            reps: runner_args.reps.unwrap_or(5u32),
            timestamp: timestamp.clone(),     // creates a copy
            git_sha: sys_env.git_sha.clone(), // creates a copy
            compiler: types::MetaCompiler {
                path: runner_args.compiler.to_string_lossy().trim().to_string(), // creates a copy
                version: sys_env.compiler_ver.clone(),                           // creates a copy
            },
            uname: sys_env.uname.clone(),     // creates a copy
            bench: runner_args.bench.clone(), // creates a copy
            compiler_args: runner_args.compiler_args.clone(), // creates a copy
            command: cmd_line,
            workdir: sys_env.cur_dir.clone(), // creates a copy
            perf_events_requested: if runner_args.perf == true {
                Some(perf::get_perf_requested_events())
            } else {
                None
            },
            perf_stat_base_args: if runner_args.perf == true {
                Some(perf::get_perf_stat_base_args())
            } else {
                None
            },
        },
        samples: run_samples.clone(), // creates a copy
        summary: summary.clone(),     // creates a copy
    };

    serde_json::to_string_pretty(&runner_json).unwrap_or_else(|err| {
        panic!("perflab-Failed to make runner_json as pretty string, error:\n{err}");
    })
}
