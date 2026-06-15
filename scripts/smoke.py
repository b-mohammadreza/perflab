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


def main():
    # run the bench
    script_file_path = Path(__file__).resolve()
    perflab_root_dir = script_file_path.parent.parent
    os.chdir(perflab_root_dir)
    print (f'perflab root: {perflab_root_dir}')

    cmd_format = (
        "cargo run -- run --perf --cpu {cpu} "
        "--warmup {warmup} --reps {reps} "
        "--bench {bench} --compiler {compiler} -- {c_ops}"
    )

    for bench in bench_bin_names:
        print(f'Running {bench} smoke...')

        subprocess.run(cmd_format.format(cpu=str(cpu_affinity)
                                , warmup=str(warmup_count)
                                , reps=str(reps_count)
                                , bench=bench
                                , compiler=compiler
                                , c_ops=" ".join(compiler_options)), shell=True, check=True)
        res_path = perflab_root_dir / result_dir

        latest_json = max(
            res_path.glob(f'*{bench}.json'),
            key=lambda f:f.stat().st_mtime,
            default=None
        )

        if latest_json:
            try:
                with open(latest_json, "r", encoding="utf-8") as jfile:
                    jobject = json.load(jfile)

                    #verification:
                    ##
                    # meta.schema_version
                    ##
                    schema_version = jobject.get("meta").get("schema_version")
                    if schema_version == None:
                        print("meta.schema_version not found!")
                        sys.exit(1)
                    if schema_version != 1:
                        print("meta.schema_version NOK!")
                        sys.exit(1)

                    ##
                    # meta.command
                    ##
                    meta_command = jobject.get("meta").get("command")
                    if meta_command == None:
                        print("meta.command not found!")
                        sys.exit(1)
                    if type(meta_command) != list:
                        print("meta.command is not a list")
                        sys.exit(1)
                    for elem in meta_command:
                        if type(elem) != str:
                            print("Not all meta.command entries are strings!")
                            sys.exit(1)

                    ##
                    # meta.workdir
                    ##
                    work_dir = jobject.get("meta").get("workdir")
                    if work_dir == None:
                        print("meta.workdir not found!")
                        sys.exit(1)
                    if type(work_dir) != str:
                        print("meta.workdir is not a string!")
                        sys.exit(1)

                    ##
                    # meta.perf_events_requested
                    ##
                    perf_events_req = jobject.get("meta").get("perf_events_requested", "MISSING")
                    if perf_events_req == "MISSING":
                        print("meta.perf_events_requested not found!")
                        sys.exit(1)
                    if perf_events_req == None:
                        print("meta.perf_events_requested is null!")
                        sys.exit(1)

                    ##
                    # meta.reps
                    ##
                    meta_reps = jobject.get("meta").get("reps")
                    if meta_reps == None:
                        print("meta.reps is not found!")
                        sys.exit(1)
                    ##
                    # samples
                    ##
                    obj_samples = jobject.get("samples")
                    if obj_samples == None:
                        print("samples not found!")
                        sys.exit(1)
                    if meta_reps != len(obj_samples):
                        print("meta.reps != samples length!")
                        sys.exit(1)

                    ##
                    # samples
                    ##
                    for sample in obj_samples:
                        ##
                        # samples.sample.perf
                        ##
                        sample_perf = sample.get("perf", "MISSING")
                        if sample_perf == "MISSING":
                            print("samples.sample.perf not found!")
                            sys.exit(1)
                        if sample_perf == None:
                            print("samples.sample.perf is null!")
                            sys.exit(1)

                        ##
                        # samples.sample.perf.csv_path
                        ##
                        sample_perf_cvs = sample_perf.get("csv_path")
                        if sample_perf_cvs == None:
                            print("samples.sample.perf.csv_path not found!")
                            sys.exit(1)
                        if Path(perflab_root_dir / Path(sample_perf_cvs)).is_file() == False:
                            print("samples.sample.perf.csv_path not pointing to an existing file!")
                            sys.exit(1)

                        ##
                        # samples.sample.perf.perf_stat_args
                        ##
                        sample_perf_args = sample_perf.get("perf_stat_args")
                        if sample_perf_args == None:
                            print("samples.sample.perf.perf_stat_args not found!")
                            sys.exit(1)
                        
                        ##
                        # samples.sample.perf.events
                        ##
                        sample_perf_events = sample_perf.get("events")
                        if sample_perf_events == None:
                            print("samples.sample.perf.events not found!")
                            sys.exit(1)

                    ##
                    # summary.phases_ns.compute
                    ##
                    sum_ph_compute = jobject.get("summary").get("phases_ns").get("compute")
                    if sum_ph_compute == None:
                        print("summary.phases_ns.compute not found!")
                        sys.exit(1)
                    if sum_ph_compute <= 0:
                        print("summary.phases_ns.compute is not positive!")
                        sys.exit(1)

                    if bench == "reduce":
                        ##
                        # summary.phases_ns.init
                        ##
                        sum_ph_init = jobject.get("summary").get("phases_ns").get("init")
                        if sum_ph_init == None:
                            print("summary.phases_ns.init not found!")
                            sys.exit(1)
                        if sum_ph_compute <= sum_ph_init:
                            print("reduce: summary.phases_ns.compute not greater than summary.phases_ns.init!")
                            sys.exit(1)

                    ##
                    # summary.perf
                    ##
                    sum_perf = jobject.get("summary").get("perf", "MISSING")
                    if sum_perf == "MISSING":
                        print("summary.perf not found!")
                        sys.exit(1)
                    if type(sum_perf) == str:
                        print("summary.perf is string!")
                        sys.exit(1)
                    if type(sum_perf) != dict and sum_perf != None:
                        print("summary.perf is invalid!")
                        sys.exit(1)

            except json.JSONDecodeError:
                print(f'{latest_json} is not in json format!')
                sys.exit(1)
        else:
            print(f'{res_path} is empty!')
            sys.exit(1)
        pass

        print(f'Checking {bench} result OK...')
    
    print("Smoke: OK")
    sys.exit(0)
        

if __name__ == '__main__':
    main()