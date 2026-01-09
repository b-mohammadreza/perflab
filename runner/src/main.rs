use clap::{Parser, Subcommand};
use std::{io::Write, process::Command};
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
    /// Runs the benchmark. Options: --bench <name>, --compiler <name> -- <flags...>
    Run {
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
        Commands::Run {bench, compiler, compiler_args} => {
            metadata_capture(&compiler, &mut metadata);

            let result = runner_compile((&bench, &compiler, &compiler_args));
            if result == 0 {
                runner_run(&bench, &compiler, &compiler_args, &metadata);
            }
        }
    }
}

fn metadata_capture(compiler: &String, metadata: &mut RunnerMetadata) {
    let git_sha_output = Command::new("git")
        .arg("rev-parse")
        .arg("HEAD")
        .output().unwrap_or_else(|e| {
            panic!("Failed to execute metadata_capture-git sha: {}", e);
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
            panic!("Failed to execute metadata_capture-<compiler> --version: {}", e);
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
            panic!("Failed to execute metadata_capture-uname -a: {}", e);
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
        panic!("Failed to execute runner_compile: {}", e);
    });

    if output.status.success() == false {
        panic!("{}", String::from_utf8_lossy(&output.stderr));
    }

    println!("Compile ok!");

    0
}

fn runner_run (bench: &String, compiler: &String, compiler_args: &Vec<String>, metadata: &RunnerMetadata) {
    let output = Command::new(format!("out/{bench}"))
        .output().unwrap_or_else(|e| {
            panic!("Failed to execute runner_run-{bench}: {e}");
        });

    if output.status.success() == false {
        panic!("{}", String::from_utf8_lossy(&output.stderr));
    } else {
        let bench_json = String::from_utf8_lossy(&output.stdout).trim().to_string();

        let bench_json_val: Value = serde_json::from_str(&bench_json)
                                .unwrap_or_else(|err| {
                                    panic!("Failed to parse bench output - {bench_json}, Error: {err}");
                                });

        let result_schema = json!({
            "meta": {
                "timestamp": metadata.timestamp,
                "git_sha": metadata.git_sha,
                "compiler": {
                    "path": compiler,
                    "version": metadata.compiler_ver
                },
                "uname": metadata.uname,
                "bench": bench,
                "compiler_args": format!("{:?}", compiler_args)
            },
            "bench_output": bench_json_val
        });

        let file_path = format!("results/{}_{}.json", metadata.timestamp, bench);
        let mut json_file = fs::File::create(&file_path)
            .unwrap_or_else(|err| {
                panic!("Failed to create file {}, Error: {err}", &file_path);
            });

        let result_schem_pretty = serde_json::to_string_pretty(&result_schema)
            .unwrap_or_else(|err| {
                panic!("Failed to pretty string result_schema: {err}");
            }); 
        
        json_file.write_all(result_schem_pretty.as_bytes()).unwrap_or_else(|err| {
            panic!("Failed to write json file: {err}");
        });

    }
}
