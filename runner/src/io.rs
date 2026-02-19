use crate::config::get_runner_args;
use crate::meta::get_sys_env_meta;
use serde_json::{Value, json};
use std::fs;
use std::{
    collections::HashMap,
    io::{Read, Write},
};

pub fn finalize_and_write_result(fellback: bool, timestamp: &String, bench_json: &String) {
    let runner_args = get_runner_args();

    if bench_json.is_empty() == true {
        panic!("perflab-Dead path!");
    }

    let mut perf_events: HashMap<String, u64> = HashMap::new();
    if runner_args.perf == true && fellback == false {
        perf_events = get_perf_events(timestamp);
    }

    runner_write_json(fellback, &bench_json, &perf_events, timestamp);
}

pub fn get_perf_events(timestamp: &String) -> HashMap<String, u64> {
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

    perf_events
}

pub fn runner_write_json(
    fellback: bool,
    bench_json: &String,
    perf_events: &HashMap<String, u64>,
    timestamp: &String,
) {
    let runner_args = get_runner_args();
    let metadata = get_sys_env_meta();

    let bench_json_val: Value = serde_json::from_str(&bench_json).unwrap_or_else(|err| {
        panic!("perflab-Failed to parse bench output, error:\n{err}");
    });

    let mut cpu_pin = json!(null);
    if let Some(cpu_id) = runner_args.cpu {
        cpu_pin = json!(cpu_id);
    }

    let mut result_schema = json!({
        "meta": {
            "cpu_pin": cpu_pin,
            "timestamp": timestamp,
            "git_sha": metadata.git_sha.trim_end().replace(['\r', '\n'], ", "),
            "compiler": {
                "path": runner_args.compiler,
                "version": metadata.compiler_ver.trim_end().replace(['\r', '\n'], ", ")
            },
            "uname": metadata.uname.trim_end().replace(['\r', '\n'], ", "),
            "bench": runner_args.bench,
            "compiler_args": runner_args.compiler_args
        },
        "bench_output": bench_json_val
    });

    if perf_events.is_empty() == false {
        result_schema
            .as_object_mut()
            .unwrap_or_else(|| {
                panic!("perflab-Failed to get underlying object map (result_schema)!");
            })
            .insert(
                "perf".to_string(),
                json!({
                    "events" : perf_events
                }),
            );
    } else if fellback == true {
        result_schema
            .as_object_mut()
            .unwrap_or_else(|| {
                panic!("perflab-Failed to get underlying object map (result_schema, fellback)!");
            })
            .insert("perf".to_string(), json!(null));
    }

    let file_path = format!("results/{}_{}.json", timestamp, runner_args.bench);
    let mut json_file = fs::File::create(&file_path).unwrap_or_else(|err| {
        panic!(
            "perflab-Failed to create file({}), error:\n{err}",
            &file_path
        );
    });

    let result_schem_pretty = serde_json::to_string_pretty(&result_schema).unwrap_or_else(|err| {
        panic!("perflab-Failed to make result_schema as pretty string, error:\n{err}");
    });

    json_file
        .write_all(result_schem_pretty.as_bytes())
        .unwrap_or_else(|err| {
            panic!("perflab-Failed to write json file, error:\n{err}");
        });
}
