use std::process::Command;

/// runner_args: &bench, &compiler, &compiler_args
pub fn runner_compile(runner_args: (&String, &String, &Vec<String>)) -> u32 {
    let mut cmd = Command::new(runner_args.1);
    for compiler_arg in runner_args.2.iter() {
        cmd.arg(compiler_arg);
    }
    cmd.arg(format!("bench/{}.cpp", runner_args.0));
    cmd.arg("-o");
    cmd.arg(format!("out/{}", runner_args.0));

    let output = cmd.output().unwrap_or_else(|e| {
        panic!(
            "perflab-Failed to execute command({}), error:\n{e}",
            runner_args.1
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
