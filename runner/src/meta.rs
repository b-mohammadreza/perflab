use chrono::prelude::*;
use std::process::Command;

pub struct RunnerMetadata {
    pub git_sha: String,
    pub compiler_ver: String,
    pub uname: String,
    pub timestamp: String,
}

pub fn metadata_capture(compiler: &String) -> RunnerMetadata {
    let mut metadata: RunnerMetadata = RunnerMetadata {
        git_sha: String::from(""),
        compiler_ver: String::from(""),
        uname: String::from(""),
        timestamp: String::from(""),
    };

    let git_sha_output = Command::new("git")
        .arg("rev-parse")
        .arg("HEAD")
        .output()
        .unwrap_or_else(|e| {
            panic!("perflab-Failed to execute command(git), error:\n{e}");
        });
    if git_sha_output.status.success() == false {
        let err_str = String::from_utf8_lossy(&git_sha_output.stderr);
        println!("{err_str}");
    } else {
        metadata.git_sha = String::from_utf8_lossy(&git_sha_output.stdout).to_string();
    }

    let compiler_ver_output = Command::new(format!("{compiler}"))
        .arg("--version")
        .output()
        .unwrap_or_else(|e| {
            panic!("perflab-Failed to execute command({compiler}), error:\n{e}");
        });
    if compiler_ver_output.status.success() == false {
        let err_str = String::from_utf8_lossy(&compiler_ver_output.stderr);
        println!("{err_str}");
    } else {
        metadata.compiler_ver = String::from_utf8_lossy(&compiler_ver_output.stdout).to_string();
    }

    let uname_output = Command::new("uname")
        .arg("-a")
        .output()
        .unwrap_or_else(|e| {
            panic!("perflab-Failed to execute command(uname), error:\n{e}");
        });
    if uname_output.status.success() == false {
        let err_str = String::from_utf8_lossy(&uname_output.stderr);
        println!("{err_str}");
    } else {
        metadata.uname = String::from_utf8_lossy(&uname_output.stdout).to_string();
    }

    metadata.timestamp = Local::now().format("%m-%d-%YT%H-%M-%S-%3f").to_string();

    metadata
}
