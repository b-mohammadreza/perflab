use crate::config::get_runner_args;
use crate::meta::get_sys_env_meta;
use crate::perf;
use crate::types;
use serde_json;
use std::{
    collections::HashMap,
    env, fs,
    io::{Read, Write},
};

pub fn get_perf_events(timestamp: &String) -> types::Perf {
    let runner_args = get_runner_args();

    let mut csv_file_text: String = String::new();
    let mut perf_events: HashMap<String, u64> = HashMap::new();

    fs::File::open(format!("out/perf_{}_{}.csv", timestamp, runner_args.bench))
        .unwrap_or_else(|err| {
            panic!(
                "perflab-Cannot open file(out/perf_{}_{}.csv), error:\n{err}",
                timestamp, runner_args.bench
            );
        })
        .read_to_string(&mut csv_file_text)
        .unwrap_or_else(|err| {
            panic!(
                "perflab-Failed reading file(out/perf_{}_{}.csv), error:\n{err}",
                timestamp, runner_args.bench
            );
        });

    for line in csv_file_text.lines() {
        let fields: Vec<&str> = line.split(',').collect();

        let stat = match fields[0].trim().replace(',', "").to_string().parse() {
            Ok(val) => val,
            Err(_) => continue,
        };

        perf_events.insert(fields[2].trim().to_string(), stat);
    }

    types::Perf {
        events: perf_events,
    }
}

pub fn finalize_and_write_result(
    timestamp: &String,
    run_samples: &types::RunSampleVec,
    summary: &types::Summary,
) {
    let runner_args = get_runner_args();
    let sys_env = get_sys_env_meta();
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
                path: &runner_args.compiler,
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
            perf_stat_args: if runner_args.perf == true {
                Some(&perf::get_perf_stat_args(timestamp))
            } else {
                None
            },
        },
        samples: run_samples,
        summary: summary,
    };

    let file_path = format!("results/{}_{}.json", timestamp, runner_args.bench);
    let mut json_file = fs::File::create(&file_path).unwrap_or_else(|err| {
        panic!(
            "perflab-Failed to create file({}), error:\n{err}",
            &file_path
        );
    });

    let result_schem_pretty = serde_json::to_string_pretty(&runner_json).unwrap_or_else(|err| {
        panic!("perflab-Failed to make runner_json as pretty string, error:\n{err}");
    });

    json_file
        .write_all(result_schem_pretty.as_bytes())
        .unwrap_or_else(|err| {
            panic!("perflab-Failed to write json file, error:\n{err}");
        });
}
