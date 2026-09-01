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
            if schema_version != 2:
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
            # summary.phases_ns
            ##
            sum_phases = jobject.get("summary").get("phases_ns")
            if sum_phases == None:
                fail("summary.phases_ns not found!")

            for phase in ["init", "compute", "teardown"]:
                phase_summary = sum_phases.get(phase)
                if phase_summary == None:
                    fail(f"summary.phases_ns.{phase} not found!")
                if type(phase_summary) != dict:
                    fail(f"summary.phases_ns.{phase} is not an object!")

                median_ns = phase_summary.get("median_ns")
                min_ns = phase_summary.get("min_ns")
                max_ns = phase_summary.get("max_ns")
                spread_percent = phase_summary.get("spread_percent", "MISSING")

                if median_ns == None:
                    fail(f"summary.phases_ns.{phase}.median_ns not found!")
                if min_ns == None:
                    fail(f"summary.phases_ns.{phase}.min_ns not found!")
                if max_ns == None:
                    fail(f"summary.phases_ns.{phase}.max_ns not found!")
                if spread_percent == "MISSING":
                    fail(f"summary.phases_ns.{phase}.spread_percent not found!")

                if type(median_ns) != int:
                    fail(f"summary.phases_ns.{phase}.median_ns is not an integer!")
                if type(min_ns) != int:
                    fail(f"summary.phases_ns.{phase}.min_ns is not an integer!")
                if type(max_ns) != int:
                    fail(f"summary.phases_ns.{phase}.max_ns is not an integer!")

                if not (min_ns <= median_ns <= max_ns):
                    fail(f"summary.phases_ns.{phase}: min_ns <= median_ns <= max_ns failed!")

                if spread_percent is not None:
                    if type(spread_percent) not in (int, float):
                        fail(f"summary.phases_ns.{phase}.spread_percent is not numeric or null!")
                    if spread_percent < 0:
                        fail(f"summary.phases_ns.{phase}.spread_percent is negative!")

            sum_ph_compute = sum_phases.get("compute").get("median_ns")
            if sum_ph_compute <= 0:
                fail("summary.phases_ns.compute.median_ns is not positive!")

            if bench == "reduce":
                sum_ph_init = sum_phases.get("init").get("median_ns")
                if sum_ph_compute <= sum_ph_init:
                    fail("reduce: summary.phases_ns.compute.median_ns not greater than summary.phases_ns.init.median_ns!")

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


def validate_compare_output_format_text(compare_output, expect_perf):
    output = compare_output.stdout
    err_out = compare_output.stderr

    if "PerfLab compare v0" not in output:
        fail("compare output missing header!")
    if "Phase comparison:" not in output:
        fail("compare output missing phase comparison!")
    if "baseline spread" not in output:
        fail("compare output missing baseline spread!")
    if "candidate spread" not in output:
        fail("compare output missing candidate spread!")

    if expect_perf:
        if "Perf comparison:" not in output:
            fail("compare output missing perf comparison!")
        if "cpu_core/cycles/u" not in output:
            fail("compare output missing cpu_core/cycles/u!")
        if "cpu_core/instructions/u" not in output:
            fail("compare output missing cpu_core/instructions/u!")
    else:
        if "perf unavailable" not in err_out:
            fail("compare stderr did not report summary.perf unavailable!")
        if "perf unavailable" in output:
            fail("compare stdout contains summary.perf unavailable warning!")


def validate_compare_output_format_markdown(compare_output, expect_perf):
    output = compare_output.stdout
    err_out = compare_output.stderr

    if "# PerfLab compare v0" not in output:
        fail("Markdown compare output missing header!")
    if "## Phase comparison:" not in output:
        fail("Markdown compare output missing phase comparison!")
    if "baseline spread" not in output:
        fail("Markdown compare output missing baseline spread!")
    if "candidate spread" not in output:
        fail("Markdown compare output missing candidate spread!")

    if expect_perf:
        if "## Perf comparison:" not in output:
            fail("Markdown compare output missing perf comparison!")
        if "cpu_core/cycles/u" not in output:
            fail("Markdown compare output missing cpu_core/cycles/u!")
        if "cpu_core/instructions/u" not in output:
            fail("Markdown compare output missing cpu_core/instructions/u!")
    else:
        if "perf unavailable" not in err_out:
            fail("Markdown compare stderr did not report summary.perf unavailable!")
        if "perf unavailable" in output:
            fail("Markdown compare stdout contains summary.perf unavailable warning!")


def validate_compare_output_format_csv(compare_output, expect_perf):
    output = compare_output.stdout
    err_out = compare_output.stderr

    if "PerfLab compare v0" in output:
        fail("CSV compare output contains text renderer header!")
    if "# PerfLab compare v0" in output:
        fail("CSV compare output contains markdown renderer header!")
    if "baseline:" in output:
        fail("CSV compare output contains baseline path!")
    if "candidate:" in output:
        fail("CSV compare output contains candidate path!")

    if "kind,name,baseline,candidate,delta,delta_percent,baseline_spread_percent,candidate_spread_percent" not in output:
        fail("CSV compare output missing header/noise columns!")
    if "phase,init," not in output:
        fail("CSV compare output missing init phase comparison!")
    if "phase,compute," not in output:
        fail("CSV compare output missing compute phase comparison!")
    if "phase,teardown," not in output:
        fail("CSV compare output missing teardown phase comparison!")

    if expect_perf:
        if "perf,cpu_core/cycles/u," not in output:
            fail("CSV compare output missing cpu_core/cycles/u!")
        if "perf,cpu_core/instructions/u," not in output:
            fail("CSV compare output missing cpu_core/instructions/u!")
    else:
        if "perf,cpu_core/cycles/u," in output:
            fail("CSV compare output contains perf,cpu_core/cycles/u!")
        if "perf,cpu_core/instructions/u," in output:
            fail("CSV compare output contains perf,cpu_core/instructions/u!")
        if "perf unavailable" not in err_out:
            fail("CSV compare stderr did not report summary.perf unavailable!")
        if "perf unavailable" in output:
            fail("CSV compare stdout contains summary.perf unavailable warning!")


def compare_results(perflab_root_dir, baseline, candidate, expect_perf):
    baseline_rel = rel_to_root(perflab_root_dir, baseline)
    candidate_rel = rel_to_root(perflab_root_dir, candidate)

    print(f'Comparing text format...')
    cmd = f"cargo run -- compare {baseline_rel} {candidate_rel}"
    compare_output = run_cmd_capture(cmd)
    validate_compare_output_format_text(compare_output, expect_perf)

    print(f'Comparing markdown format...')
    cmd = f"cargo run -- compare {baseline_rel} {candidate_rel} --format markdown"
    compare_output = run_cmd_capture(cmd)
    validate_compare_output_format_markdown(compare_output, expect_perf)

    print(f'Comparing csv format...')
    cmd = f"cargo run -- compare {baseline_rel} {candidate_rel} --format csv"
    compare_output = run_cmd_capture(cmd)
    validate_compare_output_format_csv(compare_output, expect_perf)


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
