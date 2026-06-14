use crate::config::get_runner_args;

pub fn get_bench_src_path() -> String {
    let runner_args = get_runner_args();

    format!("bench/{}.cpp", runner_args.bench)
}

pub fn get_bench_bin_path() -> String {
    let runner_args = get_runner_args();

    format!("out/{}", runner_args.bench)
}

pub fn get_perf_stat_path(timestamp: &String, rep: u32) -> String {
    let runner_args = get_runner_args();

    format!(
        "out/perf_{}_{}_rep{:06}.csv",
        timestamp, runner_args.bench, rep
    )
}

pub fn get_result_json_path(timestamp: &String) -> String {
    let runner_args = get_runner_args();

    format!("results/{}_{}.json", timestamp, runner_args.bench)
}
