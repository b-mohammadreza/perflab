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
            schema_version: 1,
            cpu_pin: runner_args.cpu,
            warmup: runner_args.warmup.unwrap_or(1u32),
            reps: runner_args.reps.unwrap_or(5u32),
            timestamp: timestamp,
            git_sha: &sys_env.git_sha,
            compiler: types::MetaCompiler {
                path: &runner_args.compiler.to_string_lossy().trim().to_string(),
                version: &sys_env.compiler_ver,
            },
            uname: &sys_env.uname,
            bench: &runner_args.bench,
            compiler_args: &runner_args.compiler_args,
            command: cmd_line,
            workdir: &sys_env.cur_dir,
            perf_events_requested: if runner_args.perf == true {
                Some(perf::get_perf_requested_events())
            } else {
                None
            },
            perf_stat_base_args: if runner_args.perf == true {
                Some(&perf::get_perf_stat_base_args())
            } else {
                None
            },
        },
        samples: run_samples,
        summary: summary,
    };

    serde_json::to_string_pretty(&runner_json).unwrap_or_else(|err| {
        panic!("perflab-Failed to make runner_json as pretty string, error:\n{err}");
    })
}
