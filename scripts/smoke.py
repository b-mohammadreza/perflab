#!/usr/bin/python

import os
import sys
import subprocess
import json
from pathlib import Path

# config - hard coded for now
result_dir          = "results"
bench_bin_names     = ["matmul", "reduce"]
warmup_count        = 1
reps_count          = 3
cpu_affinity        = 2
compiler            = "clang++"
compiler_options    = ["-O3"]


def fail(msg):
    print(msg)
    sys.exit(1)


def run_cmd(cmd):
    return subprocess.run(cmd, shell=True, check=True)


def run_cmd_capture(cmd):
    completed = subprocess.run(cmd, shell=True, check=True, capture_output=True, text=True)

    if completed.stdout:
        print(completed.stdout, end="")
    if completed.stderr:
        print(completed.stderr, end="")

    return completed


def latest_result(perflab_root_dir, bench):
    res_path = perflab_root_dir / result_dir

    latest_json = max(
        res_path.glob(f'*{bench}.json'),
        key=lambda f:f.stat().st_mtime,
        default=None
    )

    if latest_json == None:
        fail(f'{res_path} has no result file for bench {bench}!')

    return latest_json


def rel_to_root(perflab_root_dir, path):
    return path.relative_to(perflab_root_dir)


def validate_result_json(perflab_root_dir, result_json, bench, expect_perf):
    try:
        with open(result_json, "r", encoding="utf-8") as jfile:
            jobject = json.load(jfile)

            ##
            # meta.schema_version
            ##
            schema_version = jobject.get("meta").get("schema_version")
            if schema_version == None:
                fail("meta.schema_version not found!")
            if schema_version != 1:
                fail("meta.schema_version NOK!")

            ##
            # meta.command
            ##
            meta_command = jobject.get("meta").get("command")
            if meta_command == None:
                fail("meta.command not found!")
            if type(meta_command) != list:
                fail("meta.command is not a list")
            for elem in meta_command:
                if type(elem) != str:
                    fail("Not all meta.command entries are strings!")

            ##
            # meta.workdir
            ##
            work_dir = jobject.get("meta").get("workdir")
            if work_dir == None:
                fail("meta.workdir not found!")
            if type(work_dir) != str:
                fail("meta.workdir is not a string!")

            ##
            # meta.perf_events_requested
            ##
            perf_events_req = jobject.get("meta").get("perf_events_requested", "MISSING")
            if perf_events_req == "MISSING":
                fail("meta.perf_events_requested not found!")
            if expect_perf and perf_events_req == None:
                fail("meta.perf_events_requested is null!")
            if not expect_perf and perf_events_req != None:
                fail("meta.perf_events_requested should be null when perf is off!")

            ##
            # meta.reps
            ##
            meta_reps = jobject.get("meta").get("reps")
            if meta_reps == None:
                fail("meta.reps is not found!")

            ##
            # samples
            ##
            obj_samples = jobject.get("samples")
            if obj_samples == None:
                fail("samples not found!")
            if meta_reps != len(obj_samples):
                fail("meta.reps != samples length!")

            for sample in obj_samples:
                ##
                # samples.sample.perf
                ##
                sample_perf = sample.get("perf", "MISSING")
                if sample_perf == "MISSING":
                    fail("samples.sample.perf not found!")

                if expect_perf:
                    if sample_perf == None:
                        fail("samples.sample.perf is null!")

                    ##
                    # samples.sample.perf.csv_path
                    ##
                    sample_perf_csv = sample_perf.get("csv_path")
                    if sample_perf_csv == None:
                        fail("samples.sample.perf.csv_path not found!")
                    if Path(perflab_root_dir / Path(sample_perf_csv)).is_file() == False:
                        fail("samples.sample.perf.csv_path not pointing to an existing file!")

                    ##
                    # samples.sample.perf.perf_stat_args
                    ##
                    sample_perf_args = sample_perf.get("perf_stat_args")
                    if sample_perf_args == None:
                        fail("samples.sample.perf.perf_stat_args not found!")

                    ##
                    # samples.sample.perf.events
                    ##
                    sample_perf_events = sample_perf.get("events")
                    if sample_perf_events == None:
                        fail("samples.sample.perf.events not found!")
                else:
                    if sample_perf != None:
                        fail("samples.sample.perf should be null when perf is off!")

            ##
            # summary.phases_ns.compute
            ##
            sum_ph_compute = jobject.get("summary").get("phases_ns").get("compute")
            if sum_ph_compute == None:
                fail("summary.phases_ns.compute not found!")
            if sum_ph_compute <= 0:
                fail("summary.phases_ns.compute is not positive!")

            if bench == "reduce":
                ##
                # summary.phases_ns.init
                ##
                sum_ph_init = jobject.get("summary").get("phases_ns").get("init")
                if sum_ph_init == None:
                    fail("summary.phases_ns.init not found!")
                if sum_ph_compute <= sum_ph_init:
                    fail("reduce: summary.phases_ns.compute not greater than summary.phases_ns.init!")

            ##
            # summary.perf
            ##
            sum_perf = jobject.get("summary").get("perf", "MISSING")
            if sum_perf == "MISSING":
                fail("summary.perf not found!")
            if type(sum_perf) == str:
                fail("summary.perf is string!")
            if expect_perf:
                if type(sum_perf) != dict:
                    fail("summary.perf is invalid!")
                if sum_perf.get("events") == None:
                    fail("summary.perf.events not found!")
            else:
                if sum_perf != None:
                    fail("summary.perf should be null when perf is off!")

    except json.JSONDecodeError:
        fail(f'{result_json} is not in json format!')


def run_bench(perflab_root_dir, bench, expect_perf):
    perf_arg = "--perf " if expect_perf else ""
    cmd_format = (
        "cargo run -- run {perf_arg}--cpu {cpu} "
        "--warmup {warmup} --reps {reps} "
        "--bench {bench} --compiler {compiler} -- {c_ops}"
    )

    run_cmd(cmd_format.format(perf_arg=perf_arg
                            , cpu=str(cpu_affinity)
                            , warmup=str(warmup_count)
                            , reps=str(reps_count)
                            , bench=bench
                            , compiler=compiler
                            , c_ops=" ".join(compiler_options)))

    result_json = latest_result(perflab_root_dir, bench)
    validate_result_json(perflab_root_dir, result_json, bench, expect_perf)
    return result_json


def validate_compare_output(compare_output, expect_perf):
    output = compare_output.stdout

    if "PerfLab compare v0" not in output:
        fail("compare output missing header!")
    if "Phase comparison:" not in output:
        fail("compare output missing phase comparison!")
    if "Perf comparison:" not in output:
        fail("compare output missing perf comparison!")

    if expect_perf:
        if "cpu_core/cycles/u" not in output:
            fail("compare output missing cpu_core/cycles/u!")
        if "cpu_core/instructions/u" not in output:
            fail("compare output missing cpu_core/instructions/u!")
    else:
        if "summary.perf" not in output:
            fail("compare output did not report summary.perf unavailable!")


def compare_results(perflab_root_dir, baseline, candidate, expect_perf):
    baseline_rel = rel_to_root(perflab_root_dir, baseline)
    candidate_rel = rel_to_root(perflab_root_dir, candidate)

    cmd = f"cargo run -- compare {baseline_rel} {candidate_rel}"
    compare_output = run_cmd_capture(cmd)
    validate_compare_output(compare_output, expect_perf)


def main():
    script_file_path = Path(__file__).resolve()
    perflab_root_dir = script_file_path.parent.parent
    os.chdir(perflab_root_dir)
    print (f'perflab root: {perflab_root_dir}')

    perf_results = {}

    for bench in bench_bin_names:
        print(f'Running {bench} smoke A...')
        bench_a = run_bench(perflab_root_dir, bench, True)
        print(f'Checking {bench} result A OK...')

        print(f'Running {bench} smoke B...')
        bench_b = run_bench(perflab_root_dir, bench, True)
        print(f'Checking {bench} result B OK...')

        print(f'Comparing {bench} smoke results...')
        compare_results(perflab_root_dir, bench_a, bench_b, True)
        print(f'Checking {bench} compare OK...')

        perf_results[bench] = bench_a

    print('Running matmul no-perf smoke...')
    matmul_no_perf = run_bench(perflab_root_dir, "matmul", False)
    print('Checking matmul no-perf result OK...')

    print('Comparing matmul perf vs no-perf smoke results...')
    compare_results(perflab_root_dir, perf_results["matmul"], matmul_no_perf, False)
    print('Checking matmul perf vs no-perf compare OK...')

    print("Smoke: OK")
    sys.exit(0)


if __name__ == '__main__':
    main()
