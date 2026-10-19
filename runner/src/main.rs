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
                            eprintln!(
                                "perflab-compare-error: Benchmark mismatch, \n\tbaseline=\t{} \n\tcandidate=\t{}",
                                baseline_bench, candidate_bench
                            );
                        }
                        types::CompareError::NotImplSerdeSerialize { source } => {
                            eprintln!(
                                "perflab-compare-error: The value is not implementing serde::Serialize trait! Error:\n\t{}",
                                source
                            );
                        }
                        types::CompareError::EmptySample { input } => {
                            let input_side = compare::get_input_side_str(&input);
                            eprintln!(
                                "perflab-compare-error: Empty sample!\n\tInput side:\t{}",
                                input_side
                            );
                        }
                        types::CompareError::SampleBenchmarkMismatch {
                            input,
                            sample_index,
                            expected_bench,
                            actual_bench,
                        } => {
                            let input_side = compare::get_input_side_str(&input);
                            eprintln!(
                                "perflab-compare-error: Sample benchmark mismatch!\n\tInput side:\t{}\n\tSample index:\t{}\n\tExpected bench:\t{}\n\tActual bench:\t{}",
                                input_side, sample_index, expected_bench, actual_bench
                            );
                        }
                        types::CompareError::InconsistentSampleParams {
                            input,
                            sample_index,
                            expected_params,
                            actual_params,
                        } => {
                            let input_side = compare::get_input_side_str(&input);
                            let e_params: String = match compare::format_warn_err_value(
                                &expected_params,
                            ) {
                                Ok(params) => params,
                                Err(err) => {
                                    match err {
                                        types::CompareError::NotImplSerdeSerialize { source } => {
                                            eprintln!(
                                                "perflab-compare-error: Value (expected_params) is not implementing serde::Serialize trait! Error:\n\t{}",
                                                source
                                            );
                                        }
                                        _ => {}
                                    }
                                    std::process::exit(1);
                                }
                            };
                            let a_params: String = match compare::format_warn_err_value(
                                &actual_params,
                            ) {
                                Ok(params) => params,
                                Err(err) => {
                                    match err {
                                        types::CompareError::NotImplSerdeSerialize { source } => {
                                            eprintln!(
                                                "perflab-compare-error: Value (actual_params) is not implementing serde::Serialize trait! Error:\n\t{}",
                                                source
                                            );
                                        }
                                        _ => {}
                                    }
                                    std::process::exit(1);
                                }
                            };
                            eprintln!(
                                "perflab-compare-error: Inconsistence sample params!\n\tInput side:\t{}\n\tSample index:\t{}\n\tExpected params:\t{}\n\tActual params:\t{}",
                                input_side, sample_index, e_params, a_params
                            );
                        }
                        types::CompareError::WorkloadParamsMismatch {
                            baseline_params,
                            candidate_params,
                        } => {
                            let b_params: String = match compare::format_warn_err_value(
                                &baseline_params,
                            ) {
                                Ok(params) => params,
                                Err(err) => {
                                    match err {
                                        types::CompareError::NotImplSerdeSerialize { source } => {
                                            eprintln!(
                                                "perflab-compare-error: Value (baseline_params) is not implementing serde::Serialize trait! Error:\n\t{}",
                                                source
                                            );
                                        }
                                        _ => {}
                                    }
                                    std::process::exit(1);
                                }
                            };
                            let c_params: String = match compare::format_warn_err_value(
                                &candidate_params,
                            ) {
                                Ok(params) => params,
                                Err(err) => {
                                    match err {
                                        types::CompareError::NotImplSerdeSerialize { source } => {
                                            eprintln!(
                                                "perflab-compare-error: Value (candidate_params) is not implementing serde::Serialize trait! Error:\n\t{}",
                                                source
                                            );
                                        }
                                        _ => {}
                                    }
                                    std::process::exit(1);
                                }
                            };
                            eprintln!(
                                "perflab-compare-error: Workload params mismatch!\n\tBaseline params:\t{}\n\tCandidate params:\t{}",
                                b_params, c_params
                            );
                        }
                    }
                    std::process::exit(1);
                }
            }
        }
    }
}
