use clap::{Parser, Subcommand};
use std::{collections::HashMap, io::{Read, Write}, process::{Command, Output}};
use chrono::prelude::*;
use serde_json::{json, Value};
use std::fs;

#[derive(Parser)]
#[command(version, about, long_about=None)]
#[command(propagate_version=true)]
struct CmdLine {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Runs the benchmark. Options: [--perf] --bench <name>, --compiler <name> -- <flags...>
    Run {
        /// Collect perf stat counters (best-effort)
        #[arg(short, long, action = clap::ArgAction::SetTrue)]
        perf: bool,

        #[arg(short, long, value_name = "name")]
        bench: String,

        #[arg(short, long, value_name = "path")]
        compiler: String,

        #[arg(trailing_var_arg=true, allow_hyphen_values=true)]
        compiler_args: Vec<String>,
    }
}

struct RunnerMetadata {
    git_sha: String,
    compiler_ver: String,
    uname: String,
    timestamp: String,
}

fn main() {
    let cl = CmdLine::parse();

    let mut metadata: RunnerMetadata = RunnerMetadata { 
        git_sha: String::from(""), 
        compiler_ver: String::from(""), 
        uname: String::from(""), 
        timestamp: String::from("") 
    };

    match cl.command {
        Commands::Run {perf, bench, compiler, compiler_args} => {
            metadata_capture(&compiler, &mut metadata);

            let result = runner_compile((&bench, &compiler, &compiler_args));
            if result == 0 {
                runner_run(perf,(&bench, &compiler, &compiler_args), &metadata);
            }
        }
    }
}

fn metadata_capture(compiler: &String, metadata: &mut RunnerMetadata) {
    let git_sha_output = Command::new("git")
        .arg("rev-parse")
        .arg("HEAD")
        .output().unwrap_or_else(|e| {
            panic!("Failed to execute command(git), error: {e}");
        });
    if git_sha_output.status.success() == false {
        let err_str = String::from_utf8_lossy(&git_sha_output.stderr);
        println!("{err_str}");
    } else {
        metadata.git_sha = String::from_utf8_lossy(&git_sha_output.stdout).to_string();
    }

    let compiler_ver_output = Command::new(format!("{compiler}"))
        .arg("--version")
        .output().unwrap_or_else(|e| {
            panic!("Failed to execute command({compiler}), error: {e}");
        });
    if compiler_ver_output.status.success() == false {
        let err_str = String::from_utf8_lossy(&compiler_ver_output.stderr);
        println!("{err_str}");
    } else {
        metadata.compiler_ver = String::from_utf8_lossy(&compiler_ver_output.stdout).to_string();
    }

    let uname_output = Command::new("uname")
        .arg("-a")
        .output().unwrap_or_else(|e| {
            panic!("Failed to execute command(uname), error: {e}");
        });
    if uname_output.status.success() == false {
        let err_str = String::from_utf8_lossy(&uname_output.stderr);
        println!("{err_str}");
    } else {
        metadata.uname = String::from_utf8_lossy(&uname_output.stdout).to_string();
    }

    metadata.timestamp = Local::now().format("%m-%d-%YT%H-%M-%S-%3f").to_string();

} 

fn runner_compile (runner_args: (&String, &String, &Vec<String>)) -> u32 {
    let mut cmd = Command::new(runner_args.1);
    for compiler_arg in runner_args.2.iter() {
        cmd.arg(compiler_arg);
    }
    cmd.arg(format!("bench/{}.cpp", runner_args.0));
    cmd.arg("-o");
    cmd.arg(format!("out/{}", runner_args.0));

    let output = cmd.output().unwrap_or_else(|e| {
        panic!("Failed to execute command({}), error: {e}", runner_args.1);
    });

    if output.status.success() == false {
        panic!("Compile error: {}", String::from_utf8_lossy(&output.stderr));
    }

    println!("Compile ok!");

    0
}

fn runner_run (perf: bool, runner_args: (&String, &String, &Vec<String>), metadata: &RunnerMetadata) {
    let mut output: Output;
    let mut perf_events: HashMap<String, u64> = HashMap::new();
    let mut fallback: bool = false;

    match perf {
        false => {
                output = run_bench(runner_args.0);
            },
        true => {
            let perf_output = Command::new("perf")
                .arg("stat")
                .arg("-x,")
                .arg("-e")
                .arg("cycles:u,instructions:u")
                .arg("-o")
                .arg(format!("out/perf_{}_{}.csv", metadata.timestamp, runner_args.0))
                .arg("--")
                .arg(format!("out/{}", runner_args.0))
                .output();
                
            match perf_output {
                Ok(val) => output = val,
                Err(err) => {
                    println!("Failed to execute command(perf), error: {err}");
                    output = perf_fallback(runner_args.0);
                    fallback = true;
                }
            }
        }
    }

    if output.status.success() == false {
        let runner_stdrr = String::from_utf8_lossy(&output.stderr);
        if perf == false || fallback == true {
            panic!("{runner_stdrr}");
        } else {
            println!("Unable to get perf stat, error: {runner_stdrr}");
            output = perf_fallback(runner_args.0);
            if output.status.success() == false {
                panic!("{}", String::from_utf8_lossy(&output.stderr));
            }
            fallback = true;
        }
    } 

    let bench_json = String::from_utf8_lossy(&output.stdout).trim().to_string();

    if bench_json.is_empty() == true {
        return;
    }

    if perf == true && fallback == false {
        get_perf_events((&runner_args.0, &metadata.timestamp), &mut perf_events);
    }
    
    runner_write_json(fallback,
        &bench_json,
        &perf_events,
        runner_args, &metadata);
}

fn run_bench(bench: &String) -> Output {
    Command::new(format!("out/{}", bench))
        .output().unwrap_or_else(|e| {
            panic!("Failed to execute command(out/{bench}), error: {e}");
        })
}

fn perf_fallback(bench: &String) -> Output {
    run_bench(bench)
}

fn get_perf_events(csv_file_name: (&String, &String), events: &mut HashMap<String, u64>) {
    let mut csv_file_text: String = String::new(); 

    fs::File::open(format!("out/perf_{}_{}.csv", csv_file_name.1, csv_file_name.0))
        .unwrap_or_else(|err| {
            panic!("Cannot open file(out/perf_{}_{}.csv), error: {err}", csv_file_name.1, csv_file_name.0);
        }).read_to_string(&mut csv_file_text).unwrap_or_else(|err| {
            panic!("Failed reading file(out/perf_{}_{}.csv), error: {err}", csv_file_name.1, csv_file_name.0);
        });

    for line in csv_file_text.lines() {
        let fields: Vec<&str> = line.split(',').collect();

        let stat = match fields[0].trim().replace(',', "").to_string().parse() {
            Ok(val) => val,
            Err(_) => continue
        };

        events.insert(fields[2].trim().to_string(), stat);
    }
}

fn runner_write_json(fallback: bool,
    bench_json: &String,
    perf_events: &HashMap<String, u64>,
    runner_args: (&String, &String, &Vec<String>),
    metadata: &RunnerMetadata) {

    let bench_json_val: Value = serde_json::from_str(&bench_json)
                            .unwrap_or_else(|err| {
                                panic!("Failed to parse bench output, error: {err}");
                            });

    let mut result_schema = json!({
        "meta": {
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
        result_schema.as_object_mut().unwrap_or_else(|| {
            panic!("Failed to get underlying object map (result_schema)!");
        }).insert("perf".to_string(), json!({
            "events" : perf_events
        }));
    } else if fallback == true {
        result_schema.as_object_mut().unwrap_or_else(|| {
            panic!("Failed to get underlying object map (result_schema, fallback)!");
        }).insert("perf".to_string(), json!(null));
    }

    let file_path = format!("results/{}_{}.json", metadata.timestamp, runner_args.0);
    let mut json_file = fs::File::create(&file_path)
        .unwrap_or_else(|err| {
            panic!("Failed to create file({}), error: {err}", &file_path);
        });

    let result_schem_pretty = serde_json::to_string_pretty(&result_schema)
        .unwrap_or_else(|err| {
            panic!("Failed to make result_schema as pretty string, error: {err}");
        }); 
    
    json_file.write_all(result_schem_pretty.as_bytes()).unwrap_or_else(|err| {
        panic!("Failed to write json file, error: {err}");
    });
}