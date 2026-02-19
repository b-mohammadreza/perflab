use crate::config::get_runner_args;
use chrono::prelude::*;
use std::process::Command;
use std::sync::OnceLock;

#[derive(Debug)]
pub struct RunnerSysEnvMetadata {
    pub git_sha: String,
    pub compiler_ver: String,
    pub uname: String,
}

static RUNNER_SYS_ENV: OnceLock<RunnerSysEnvMetadata> = OnceLock::new();

pub fn metadata_capture() {
    let runner_args = get_runner_args();

    let git_sha_output = Command::new("git")
        .arg("rev-parse")
        .arg("HEAD")
        .output()
        .unwrap_or_else(|e| {
            panic!("perflab-Failed to execute command(git), error:\n{e}");
        });
    if git_sha_output.status.success() == false {
        let err_str = String::from_utf8_lossy(&git_sha_output.stderr);
        panic!("perflab-Command(git) execution failed, error:\n{err_str}");
    }
    let git_sha = String::from_utf8_lossy(&git_sha_output.stdout).to_string();

    let compiler_ver_output = Command::new(format!("{}", runner_args.compiler))
        .arg("--version")
        .output()
        .unwrap_or_else(|e| {
            panic!(
                "perflab-Failed to execute command({}), error:\n{e}",
                runner_args.compiler
            );
        });
    if compiler_ver_output.status.success() == false {
        let err_str = String::from_utf8_lossy(&compiler_ver_output.stderr);
        panic!(
            "perflab-Command({}) execution failed, error:\n{err_str}",
            runner_args.compiler
        );
    }
    let compiler_ver = String::from_utf8_lossy(&compiler_ver_output.stdout).to_string();

    let uname_output = Command::new("uname")
        .arg("-a")
        .output()
        .unwrap_or_else(|e| {
            panic!("perflab-Failed to execute command(uname), error:\n{e}");
        });
    if uname_output.status.success() == false {
        let err_str = String::from_utf8_lossy(&uname_output.stderr);
        panic!("perflab-Failed to execute command(uname), error:\n{err_str}");
    }
    let uname = String::from_utf8_lossy(&uname_output.stdout).to_string();

    RUNNER_SYS_ENV
        .set(RunnerSysEnvMetadata {
            git_sha,
            compiler_ver,
            uname,
        })
        .expect("perflab-System environment meta-data already initialized!");
}

pub fn get_sys_env_meta() -> &'static RunnerSysEnvMetadata {
    RUNNER_SYS_ENV
        .get()
        .expect("perflab-System environment meta-data not initialized!")
}

pub fn get_timestamp() -> String {
    Local::now().format("%m-%d-%YT%H-%M-%S-%3f").to_string()
}
