use crate::config::get_runner_args;
use std::process::Command;

pub fn runner_compile() -> u32 {
    let runner_args = get_runner_args();

    let mut cmd = Command::new(&runner_args.compiler);
    for compiler_arg in runner_args.compiler_args.iter() {
        cmd.arg(compiler_arg);
    }
    cmd.arg(format!("bench/{}.cpp", runner_args.bench));
    cmd.arg("-o");
    cmd.arg(format!("out/{}", runner_args.bench));

    let output = cmd.output().unwrap_or_else(|e| {
        panic!(
            "perflab-Failed to execute command({}), error:\n{e}",
            runner_args.compiler
        );
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
