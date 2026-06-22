use crate::config;
use crate::types;
use chrono::prelude::*;
use std::env;
use std::process::Command;
use std::sync::OnceLock;

static RUNNER_SYS_ENV: OnceLock<types::RunnerSysEnvMetadata> = OnceLock::new();

pub fn metadata_capture() {
    let git_sha = get_git_sha();
    let compiler_ver = get_compiler_version();
    let uname = get_uname();
    let cur_dir = get_cur_dir();

    RUNNER_SYS_ENV
        .set(types::RunnerSysEnvMetadata {
            git_sha: git_sha,
            compiler_ver: compiler_ver,
            uname: uname,
            cur_dir: cur_dir,
        })
        .expect("perflab-System environment meta-data already initialized!");
}

pub fn get_sys_env_meta() -> &'static types::RunnerSysEnvMetadata {
    RUNNER_SYS_ENV
        .get()
        .expect("perflab-System environment meta-data not initialized!")
}

pub fn get_timestamp() -> String {
    Local::now().format("%m-%d-%YT%H-%M-%S-%3f").to_string()
}

fn get_git_sha() -> String {
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

    String::from_utf8_lossy(&git_sha_output.stdout)
        .trim_end()
        .to_string()
}

fn get_compiler_version() -> String {
    let runner_args = config::get_runner_args();

    let compiler = &runner_args.compiler.to_string_lossy().trim().to_string();
    let compiler_ver_output = Command::new(compiler)
        .arg("--version")
        .output()
        .unwrap_or_else(|e| {
            panic!("perflab-Failed to execute command({compiler}), error:\n{e}");
        });
    if compiler_ver_output.status.success() == false {
        let err_str = String::from_utf8_lossy(&compiler_ver_output.stderr);
        panic!("perflab-Command({compiler}) execution failed, error:\n{err_str}");
    }

    String::from_utf8_lossy(&compiler_ver_output.stdout)
        .trim_end()
        .replace(['\r', '\n'], ", ")
}

fn get_uname() -> String {
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

    String::from_utf8_lossy(&uname_output.stdout)
        .trim_end()
        .to_string()
}

fn get_cur_dir() -> String {
    env::current_dir()
        .unwrap_or_else(|err| panic!("perflab-Os env, unable to get current directory! Err: {err}"))
        .into_os_string()
        .to_string_lossy()
        .trim()
        .to_string()
}
