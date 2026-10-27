# PerfLab Usage and Validation
This guide contains the operational details intentionally kept out of the main README.
## Build and run the CLI
From the repository root:
```bash
cargo build
cargo run -- --help
cargo run -- run --bench <name> --compiler <path> -- <compiler-flags...>
```
## Benchmark developer workflow
Benchmarks live under `bench/` and use Meson, Ninja, and Clang.
### One-time setup
```bash
meson setup bench/build --backend=ninja --native-file bench/clang.ini
```
To reconfigure from scratch:
```bash
meson setup --wipe bench/build --backend=ninja --native-file bench/clang.ini
```
### Build benchmarks
```bash
meson compile -C bench/build
```
### Run benchmark sanity checks
Run all registered benchmark tests:
```bash
meson test -C bench/build --verbose
```
Run one benchmark test:
```bash
meson test -C bench/build reduce --verbose
```
Run binaries directly:
```bash
./bench/build/bench/reduce
./bench/build/bench/matmul
```
## Linux perf counters
PerfLab can optionally run a benchmark under Linux `perf stat` and store parsed counters in the result JSON.
```bash
cargo run -- run --perf --bench reduce --compiler clang++ -- -O3
```
PerfLab requests user-space events with the `:u` suffix, such as `cycles:u` and `instructions:u`.
If performance-counter access is blocked by kernel settings or a security policy, PerfLab falls back to a normal benchmark run and records `"perf": null` instead of failing the full run.
## CPU pinning
Use `--cpu <cpu_id>` to pin compilation, benchmark execution, and performance-counter collection to one logical CPU.
```bash
perflab run --cpu <cpu_id> --bench matmul --compiler clang++ -- -O3
perflab run --cpu <cpu_id> --perf --bench matmul --compiler clang++ -- -O3
```
List logical CPUs with:
```bash
lscpu -e=CPU,CORE,SOCKET,MAXMHZ
```
On hybrid Intel systems, different logical CPUs can use different performance-monitoring blocks, such as `cpu_core/*` and `cpu_atom/*`. Reuse the same `--cpu` value across runs being compared.
When CPU pinning is enabled, the selected logical CPU is recorded in `meta.cpu_pin`.
## Warmups and repetitions
PerfLab discards warmup runs, stores each measured repetition, and derives summary metrics from those measured samples.
```bash
perflab run --warmup 1 --reps 5 --bench matmul --compiler clang++ -- -O3
perflab run --perf --warmup 1 --reps 5 --bench matmul --compiler clang++ -- -O3
```
Defaults:
- `--warmup 1`
- `--reps 5`
## Result files
PerfLab writes one JSON result file per run under `results/`.
Current result `schema_version` is `2`.
Top-level objects:
- `meta` — metadata required to understand and reproduce the run
- `samples` — one raw entry per measured repetition
- `summary` — derived summary metrics
### Metadata
`meta` includes:
- `schema_version` — current result schema version
- `command` — command-line arguments used to launch the run
- `workdir` — working directory
- `bench` — benchmark name
- `compiler` — compiler path and version
- `compiler_args` — compiler flags passed after `--`
- `git_sha` — repository commit
- `uname` — host kernel and system string
- `cpu_pin` — selected logical CPU or `null`
- `warmup` — number of warmup runs
- `reps` — number of measured repetitions
- `perf_events_requested` — requested events or `null`
### Samples and summary
`samples` contains one entry per measured repetition, and its length should equal `meta.reps`.
Each sample contains:
- `bench_output` — benchmark output for that repetition
- `perf` — performance-counter data or `null`
When performance-counter collection succeeds, `perf` contains:
- `csv_path` — path to the per-repetition CSV artifact under `out/`
- `perf_stat_args` — arguments passed to `perf stat`
- `perf_events.events` — map of parsed event counters
For schema v2, each runtime phase under `summary.phases_ns` contains:
- `median_ns`
- `min_ns`
- `max_ns`
- `spread_percent`
This applies to `init`, `compute`, and `teardown`.
Phase spread is:
```text
spread_percent = (max_ns - min_ns) / median_ns * 100
```
Zero-median behavior:
- if all measured values are zero, `spread_percent` is `0`
- if the median is zero but any measured value is nonzero, `spread_percent` is `null`
`spread_percent` is stored as a JSON number when available, not as a formatted percentage string. Human-facing compare output formats it as a percentage.
`summary.perf` contains median event values over samples with performance-counter data, or `null` when no sample has such data. Perf-counter noise metrics are not part of schema v2 yet.
### Null and dynamic-map policy
Stable known fields remain present and use explicit `null` when unavailable or not applicable. Examples include `meta.cpu_pin`, `meta.perf_events_requested`, per-sample `perf`, `summary.perf`, and an unavailable `spread_percent`.
Dynamic maps include only observed keys. PerfLab does not synthesize missing keys as zero or with another sentinel. This applies to:
- `perf.events`
- `bench_output.params`
- `bench_output.check`
## Comparing result files
Compare two completed result files with:
```bash
perflab compare <baseline.json> <candidate.json>
```
During development:
```bash
cargo run -- compare results/<baseline>.json results/<candidate>.json
```
Comparison direction is:
```text
delta = candidate - baseline
```
For timing values:
- positive delta means the candidate is slower
- negative delta means the candidate is faster
Compare v0 checks that both inputs have the same `meta.schema_version` and `meta.bench` and validates matching, internally consistent workloads before comparing summary values.
For runtime phases, delta calculations use `median_ns`. Compare also exposes baseline and candidate `spread_percent` values and derives a noise-aware effective threshold and verdict for each phase.
Compared fields:
- `summary.phases_ns.init.median_ns`
- `summary.phases_ns.compute.median_ns`
- `summary.phases_ns.teardown.median_ns`
- baseline and candidate phase `spread_percent`
- derived effective threshold and verdict for each runtime phase
- common keys in `summary.perf.events` when both inputs contain performance-counter data
- derived whole-process instructions per cycle (IPC), calculated from paired per-repetition counters, plus valid/total coverage
Min/max values remain in the result JSON but are not currently printed in the normal compare report.
### Derived metrics: IPC
Compare v0 reports **instructions per cycle (IPC)** as a diagnostic derived metric for the **whole benchmark process**, not just the `compute` phase. The comparison computes IPC from raw per-repetition counters in `samples[].perf.perf_events.events`; it does not add an IPC field to the result JSON schema and does not calculate IPC by dividing the `summary.perf` median counters.

For each measured repetition, use the exact event keys:
- `cpu_core/instructions/u` — user-space retired instructions
- `cpu_core/cycles/u` — user-space CPU cycles

A repetition contributes an IPC measurement only if `perf` exists, both event keys are present, and cycles are greater than zero. **Zero instructions with positive cycles is valid** and yields IPC `0.0`. Skip samples with absent perf data, absent event keys, or zero cycles; do not substitute `cpu_atom/*` or other event names for the required keys.

```text
sample_ipc = instructions / cycles
run_ipc = median(valid sample_ipc values)
coverage = valid repetitions / all measured repetitions
```

The run IPC is the **median of per-repetition ratios**, not the ratio of counter medians, nor the ratio of counter totals. Coverage is tracked separately for baseline and candidate (for example, `3/5 valid`). If a run has no valid IPC measurements, its IPC is unavailable and displayed as `N/A`; comparison still succeeds.

Compare baseline and candidate using:
```text
ipc_delta = candidate_run_ipc - baseline_run_ipc
ipc_delta_percent = ipc_delta / baseline_run_ipc * 100
```

An absolute IPC delta is available only when both run IPC values exist. A percentage delta additionally requires a nonzero baseline IPC. If the baseline is zero and the candidate is available, the absolute delta is still reported while the percentage is `N/A`. IPC is diagnostic only: it does **not** receive a `REGRESSION` or `IMPROVEMENT` verdict, does not affect runtime-phase verdicts, and is not a compute-phase throughput measurement.

### Output formats
```bash
perflab compare <baseline.json> <candidate.json> --format text
perflab compare <baseline.json> <candidate.json> --format markdown
perflab compare <baseline.json> <candidate.json> --format csv
```
`text` is the default terminal report. `markdown` emits Markdown tables. `csv` emits rows suitable for spreadsheet import or later plotting.
All three formats expose baseline and candidate phase spread information, the effective threshold, and the phase verdict. Text and Markdown also show the fixed base threshold.
All three formats also report derived IPC, its absolute and percentage deltas, and baseline/candidate valid/total measurement coverage. Missing IPC values and unavailable deltas are displayed as `N/A`; numerical IPC values and absolute IPC deltas use three decimal places, and percentage deltas use two decimal places.
CSV uses these columns:
```text
kind,name,baseline,candidate,delta,delta_percent,baseline_spread_percent,candidate_spread_percent,effective_threshold_percent,verdict
```
The 10-column CSV layout is unchanged. Phase rows populate all columns. Perf rows leave the spread, effective-threshold, and verdict columns empty. IPC adds two kinds of rows:
- `derived` — `name=ipc`, baseline and candidate run IPCs, absolute delta, and percentage delta; spread, threshold, and verdict columns are empty.
- `derived_coverage` — `name=ipc`, `baseline` and `candidate` hold `valid/total` (such as `3/5`); all other data columns are empty.

For example:
```csv
kind,name,baseline,candidate,delta,delta_percent,baseline_spread_percent,candidate_spread_percent,effective_threshold_percent,verdict
derived,ipc,2.000,1.800,-0.200,-10.00%,,,,
derived_coverage,ipc,5/5,3/5,,,,,,
```
### Regression verdict policy
Compare v0 applies verdicts only to runtime phases: `init`, `compute`, and `teardown`. Performance-counter and derived IPC rows are compared numerically but do not receive verdicts.
The fixed base threshold is:
```text
3.0%
```
For each runtime phase:
```text
effective_threshold = max(3.0%, baseline_spread, candidate_spread)
```
The percentage delta remains:
```text
delta_percent = (candidate_median - baseline_median) / baseline_median * 100
```
Classification is:
- `delta_percent >= +effective_threshold` -> `REGRESSION`
- `delta_percent <= -effective_threshold` -> `IMPROVEMENT`
- otherwise -> `NO_MEANINGFUL_CHANGE`
The threshold boundary is inclusive.
A phase verdict is `UNAVAILABLE` when either:
- percentage delta is unavailable, including when the baseline median is zero
- baseline or candidate `spread_percent` is unavailable, so no effective threshold can be derived
This policy is intentionally a conservative noise heuristic. It is not a confidence interval, a statistical-significance test, or a repeated-confirmation policy.
### Compare behavior
- Performance-counter comparison is skipped if either input has `summary.perf: null`.
- Missing performance-counter data is nonfatal and produces a warning.
- Event keys are compared only when present in both inputs.
- Missing event keys are not treated as zero.
- Metadata differences such as compiler version, Git commit, host, CPU pin, repetitions, or requested events may produce warnings but do not stop comparison.
- Runtime-phase verdicts use the fixed base threshold plus both runs' observed phase spreads.
- Perf counters and derived IPC do not receive verdicts in v0.
- Compare does not yet calculate confidence intervals, Median Absolute Deviation, statistical significance, or an overall benchmark pass/fail verdict.
### Workload comparability validation
Before producing phase comparisons or regression verdicts, Compare v0 validates the measured workload in both input files:
- Each input must contain at least one measured sample in `samples[]`. An empty array is a fatal `EmptySample` error because workload identity cannot be established.
- Every sample's `bench_output.bench` must match its run's `meta.bench`. A mismatch is a fatal `SampleBenchmarkMismatch` error.
- All samples within a single run must have identical `bench_output.params.iters` and `bench_output.params.n`. A mismatch is a fatal `InconsistentSampleParams` error.
- Once both runs are internally consistent, baseline and candidate must have identical `iters` and `n`. A mismatch is a fatal `WorkloadParamsMismatch` error.
The validator reports the input side and zero-based sample index for sample-specific errors. Workload mismatches stop comparison before phase deltas, thresholds, or verdicts are generated.
Compiler path/version/flags, CPU pinning, host, warmup/repetition configuration, Git SHA, and requested perf-event differences may produce warnings but do not, by themselves, block comparison: these differences can be intentional or are not treated as workload-identity failures in v1.
This v1 check does not independently validate that `samples[]` length matches `meta.reps`, verify benchmark checksums, or assess statistical confidence.
## Compare input validation
Before typed deserialization, compare explicitly requires these JSON paths:
- `meta.schema_version`
- `meta.bench`
- `summary.phases_ns`
- `summary.phases_ns.init`
- `summary.phases_ns.compute`
- `summary.phases_ns.teardown`
- `summary.perf`
- `summary.perf.events` when `summary.perf` is non-null
For schema v2, typed deserialization validates the nested phase-summary fields, including `median_ns`, `min_ns`, `max_ns`, and `spread_percent`.
`summary.perf: null` is valid. A non-null `summary.perf` must contain `summary.perf.events`.
Wrong field types are reported through path-aware typed deserialization, so an error identifies both the input side and the failing path.
Fatal compare failures include:
- baseline or candidate file cannot be opened or read
- malformed JSON
- missing required field
- typed deserialization failure
- schema mismatch
- benchmark mismatch
- empty measured samples, sample benchmark mismatch, inconsistent sample workload parameters, or baseline/candidate workload-parameter mismatch
## Output streams and exit status
- Successful comparison report: standard output
- Warnings, including unavailable performance-counter data: standard error
- Fatal comparison error: standard error
- Successful comparison: exit status `0`
- Successful comparison with unavailable performance-counter data: exit status `0`
- Fatal comparison failure: exit status `1`
Compare input failures return structured errors instead of exposing a Rust panic or backtrace.
## Validation workspace
For manual investigations, use a repository-local ignored workspace rather than `/tmp`, so before-and-after outputs survive reboots and cleanup.
Suggested layout:
```text
.work/compare-validation/
```
Add `.work/` to `.gitignore` before using it.
## Automated testing
PerfLab uses three levels of automated testing.
### Rust unit tests
Unit tests cover internal logic including:
- compare delta, spread, threshold, and verdict formatting
- effective-threshold calculation
- regression, improvement, no-meaningful-change, unavailable, and threshold-boundary verdict cases
- required-field validation
- path-aware typed deserialization failures
- schema and benchmark compatibility checks
- within-run workload consistency and between-run workload-parameter checks
- input read failures
- median, minimum, maximum, and spread calculations
- zero-median spread behavior
- schema-v2 numeric/null spread representation
- IPC per-repetition pairing, median of valid ratios, even-number medians, missing perf/events, zero cycles, zero instructions, exact event-key matching, and valid/total coverage
- derived IPC comparison with positive/negative deltas, zero baselines, unavailable values, and generic integer/float delta calculations
Run them with:
```bash
cargo test -p perflab
```
### Rust CLI integration tests
Integration tests under `runner/tests/` exercise the built `perflab` command-line interface, including representative fatal compare failures, successful workload comparisons, metadata-warning behavior, and exit/error behavior. They also verify IPC reporting in text, Markdown, and CSV, including unavailable or partially missing perf measurements and the fixed 10-column CSV layout.
Run the compare integration tests with:
```bash
cargo test -p perflab --test compare_cli
```
### End-to-end smoke testing
Run the end-to-end suite with:
```bash
./scripts/smoke.py
```
The smoke suite intentionally stays broader and smaller than the Rust tests. It exercises real benchmark runs, generated JSON results, perf and no-perf flows, and successful comparison output in text, Markdown, and CSV formats, including derived IPC and measurement coverage. It also checks that threshold/verdict fields are present in the rendered comparison formats without asserting a specific noise-dependent verdict from real benchmark runs. Detailed validation and error-path cases belong primarily in the Rust unit and integration tests rather than being duplicated in the smoke suite.

As of 2026-10-26, validation completed successfully with **63 Rust unit tests**, **17 CLI integration tests**, `cargo build`, `git diff --check`, and `python3 scripts/smoke.py` (matmul/reduce perf runs plus matmul no-perf comparison).
