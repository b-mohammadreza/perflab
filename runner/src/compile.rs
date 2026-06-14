use crate::config::get_runner_args;
use crate::paths;
use std::process::Command;

pub fn runner_compile() -> u32 {
    let runner_args = get_runner_args();
    let compiler = runner_args.compiler.as_str();

    let mut cmd = Command::new(compiler);
    for compiler_arg in runner_args.compiler_args.iter() {
        cmd.arg(compiler_arg);
    }
    cmd.arg(paths::get_bench_src_path());
    cmd.arg("-o");
    cmd.arg(paths::get_bench_bin_path());

    let output = cmd.output().unwrap_or_else(|e| {
        panic!("perflab-compile-Failed to execute command({compiler}), error:\n{e}");
    });

    if output.status.success() == false {
        panic!(
            "perflab-Compile error:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    println!("perflab-Compile ok!");

    0
}
