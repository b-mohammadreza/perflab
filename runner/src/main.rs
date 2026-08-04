use clap::Parser;
use perflab::compare;
use perflab::config;
use perflab::pipeline;
use perflab::types;

fn main() {
    let cl = config::CmdLine::parse();

    dispach_cmd(cl.command);
}

fn dispach_cmd(args: config::Commands) {
    match args {
        config::Commands::Run {
            warmup,
            reps,
            cpu,
            perf,
            bench,
            compiler,
            compiler_args,
        } => {
            config::set_runner_args(warmup, reps, cpu, perf, bench, compiler, compiler_args);

            pipeline::execute();
        }

        config::Commands::Compare {
            baseline,
            candidate,
            format,
        } => {
            config::set_cmp_args(baseline, candidate, format);

            match compare::execute() {
                Ok(()) => (),
                Err(err) => {
                    match err {
                        types::CompareError::ReadInput {
                            input,
                            path,
                            source,
                        } => {
                            let json_path_str: String = path.to_string_lossy().trim().to_string();
                            let input_side = compare::get_input_side_str(&input);
                            eprintln!(
                                "perflab-compare-error: Read json file failed! Side:{input_side}, Path:{json_path_str}, Error:\n\t{source}"
                            );
                        }
                        types::CompareError::MalformedJson {
                            input,
                            path,
                            source,
                        } => {
                            let json_path_str: String = path.to_string_lossy().trim().to_string();
                            let input_side = compare::get_input_side_str(&input);
                            eprintln!(
                                "perflab-compare-error: Create json object failed! Side:{input_side}, Path:{json_path_str}, Error:\n\t{source}"
                            );
                        }
                        types::CompareError::MissingRequiredField { input, field } => {
                            let input_side = compare::get_input_side_str(&input);
                            eprintln!(
                                "perflab-compare-error: Missing required field! Side:{input_side}, Field: `{field}`"
                            );
                        }
                        types::CompareError::Deserialize {
                            input,
                            path,
                            source,
                        } => {
                            let json_path_str: String = path.to_string_lossy().trim().to_string();
                            let input_side = compare::get_input_side_str(&input);
                            eprintln!(
                                "perflab-compare-error: Possible type mismatch! Side:{input_side}, Path:{json_path_str}, Error:\n\t{}-{}",
                                source.path().to_string(),
                                source.inner()
                            );
                        }
                        types::CompareError::SchemaMismatch {
                            baseline_ver,
                            candidate_ver,
                        } => {
                            let b_ver: String = match compare::format_warn_err_value(&baseline_ver)
                            {
                                Ok(ver) => ver,
                                Err(err) => {
                                    match err {
                                        types::CompareError::NotImplSerdeSerialize { source } => {
                                            eprintln!(
                                                "perflab-compare-error: Value({}) is not implementing serde::Serialize trait! Error:\n\t{}",
                                                baseline_ver, source
                                            );
                                        }
                                        _ => {}
                                    }
                                    std::process::exit(1);
                                }
                            };
                            let c_ver: String = match compare::format_warn_err_value(&candidate_ver)
                            {
                                Ok(ver) => ver,
                                Err(err) => {
                                    match err {
                                        types::CompareError::NotImplSerdeSerialize { source } => {
                                            eprintln!(
                                                "perflab-compare-error: Value({}) is not implementing serde::Serialize trait! Error:\n\t{}",
                                                candidate_ver, source
                                            );
                                        }
                                        _ => {}
                                    }
                                    std::process::exit(1);
                                }
                            };
                            eprintln!(
                                "perflab-compare-error: Schema mismatch, \n\tbaseline=\t{} \n\tcandidate=\t{}",
                                b_ver, c_ver
                            );
                        }
                        types::CompareError::BenchmarkMismatch {
                            baseline_bench,
                            candidate_bench,
                        } => {
                            let b_bench: String = match compare::format_warn_err_value(
                                &baseline_bench,
                            ) {
                                Ok(bench) => bench,
                                Err(err) => {
                                    match err {
                                        types::CompareError::NotImplSerdeSerialize { source } => {
                                            eprintln!(
                                                "perflab-compare-error: Value({}) is not implementing serde::Serialize trait! Error:\n\t{}",
                                                baseline_bench, source
                                            );
                                        }
                                        _ => {}
                                    }
                                    std::process::exit(1);
                                }
                            };
                            let c_bench: String = match compare::format_warn_err_value(
                                &candidate_bench,
                            ) {
                                Ok(bench) => bench,
                                Err(err) => {
                                    match err {
                                        types::CompareError::NotImplSerdeSerialize { source } => {
                                            eprintln!(
                                                "perflab-compare-error: Value({}) is not implementing serde::Serialize trait! Error:\n\t{}",
                                                candidate_bench, source
                                            );
                                        }
                                        _ => {}
                                    }
                                    std::process::exit(1);
                                }
                            };
                            eprintln!(
                                "perflab-compare-error: Benchmark mismatch, \n\tbaseline=\t{} \n\tcandidate=\t{}",
                                b_bench, c_bench
                            );
                        }
                        types::CompareError::NotImplSerdeSerialize { source } => {
                            eprintln!(
                                "perflab-compare-error: The value is not implementing serde::Serialize trait! Error:\n\t{}",
                                source
                            );
                        }
                    }
                    std::process::exit(1);
                }
            }
        }
    }
}
