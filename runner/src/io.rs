use crate::meta::RunnerMetadata;
use serde_json::{Value, json};
use std::fs;
use std::{
    collections::HashMap,
    io::{Read, Write},
};

/// runner_args: &bench, &compiler, &compiler_args, perf, cpu
pub fn finalize_and_write_result(
    fellback: bool,
    runner_args: (&String, &String, &Vec<String>, bool, Option<u16>),
    metadata: &RunnerMetadata,
    bench_json: &String,
) {
    if bench_json.is_empty() == true {
        panic!("perflab-Dead path!");
    }

    let mut perf_events: HashMap<String, u64> = HashMap::new();
    if runner_args.3 == true && fellback == false {
        perf_events = get_perf_events((&runner_args.0, &metadata.timestamp));
    }

    runner_write_json(fellback, &bench_json, &perf_events, runner_args, &metadata);
}

pub fn get_perf_events(csv_file_name: (&String, &String)) -> HashMap<String, u64> {
    let mut csv_file_text: String = String::new();
    let mut perf_events: HashMap<String, u64> = HashMap::new();

    fs::File::open(format!(
        "out/perf_{}_{}.csv",
        csv_file_name.1, csv_file_name.0
    ))
    .unwrap_or_else(|err| {
        panic!(
            "perflab-Cannot open file(out/perf_{}_{}.csv), error:\n{err}",
            csv_file_name.1, csv_file_name.0
        );
    })
    .read_to_string(&mut csv_file_text)
    .unwrap_or_else(|err| {
        panic!(
            "perflab-Failed reading file(out/perf_{}_{}.csv), error:\n{err}",
            csv_file_name.1, csv_file_name.0
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

/// runner_args: &bench, &compiler, &compiler_args, perf, cpu
pub fn runner_write_json(
    fellback: bool,
    bench_json: &String,
    perf_events: &HashMap<String, u64>,
    runner_args: (&String, &String, &Vec<String>, bool, Option<u16>),
    metadata: &RunnerMetadata,
) {
    let bench_json_val: Value = serde_json::from_str(&bench_json).unwrap_or_else(|err| {
        panic!("perflab-Failed to parse bench output, error:\n{err}");
    });

    let mut cpu_pin = json!(null);
    if let Some(cpu_id) = runner_args.4 {
        cpu_pin = json!(cpu_id);
    }

    let mut result_schema = json!({
        "meta": {
            "cpu_pin": cpu_pin,
            "timestamp": metadata.timestamp,
            "git_sha": metadata.git_sha.trim_end().replace(['\r', '\n'], ", "),
            "compiler": {
                "path": runner_args.1,
                "version": metadata.compiler_ver.trim_end().replace(['\r', '\n'], ", ")
            },
            "uname": metadata.uname.trim_end().replace(['\r', '\n'], ", "),
            "bench": runner_args.0,
            "compiler_args": runner_args.2
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

    let file_path = format!("results/{}_{}.json", metadata.timestamp, runner_args.0);
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
