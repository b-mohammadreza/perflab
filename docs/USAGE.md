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
PerfLab discards warmup runs, stores each measured repetition, and computes medians.

```bash
perflab run --warmup 1 --reps 5 --bench matmul --compiler clang++ -- -O3
perflab run --perf --warmup 1 --reps 5 --bench matmul --compiler clang++ -- -O3
```

Defaults:
- `--warmup 1`
- `--reps 5`

## Result files
PerfLab writes one JSON result file per run under `results/`.

Top-level objects:
- `meta` — metadata required to understand and reproduce the run
- `samples` — one raw entry per measured repetition
- `summary` — medians derived from the samples

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
- `events` — parsed event counters

`summary` contains:
- `summary.phases_ns` — median `init`, `compute`, and `teardown` timings
- `summary.perf` — median event values over samples with performance-counter data, or `null` when no sample has such data

### Null and dynamic-map policy
Stable known fields remain present and use explicit `null` when unavailable or not applicable. Examples include `meta.cpu_pin`, `meta.perf_events_requested`, per-sample `perf`, and `summary.perf`.

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

Compare v0 checks that both inputs have the same `meta.schema_version` and `meta.bench`, then compares median values from `summary`.

Compared fields:
- `summary.phases_ns.init`
- `summary.phases_ns.compute`
- `summary.phases_ns.teardown`
- common keys in `summary.perf.events` when both inputs contain performance-counter data

### Output formats
```bash
perflab compare <baseline.json> <candidate.json> --format text
perflab compare <baseline.json> <candidate.json> --format markdown
perflab compare <baseline.json> <candidate.json> --format csv
```

`text` is the default terminal report. `markdown` emits Markdown tables. `csv` emits rows suitable for spreadsheet import or later plotting.

### Compare behavior
- Performance-counter comparison is skipped if either input has `summary.perf: null`.
- Missing performance-counter data is nonfatal.
- Event keys are compared only when present in both inputs.
- Missing event keys are not treated as zero.
- Metadata differences such as compiler version, Git commit, host, CPU pin, repetitions, or requested events may produce warnings but do not stop comparison.
- Compare v0 uses existing medians only; it does not calculate confidence intervals, outlier analysis, regression thresholds, or pass/fail decisions.

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

`summary.perf: null` is valid. A non-null `summary.perf` must contain `summary.perf.events`.

Wrong field types are reported through path-aware typed deserialization, so an error identifies both the input side and a path such as `summary.phases_ns.compute`.

Fatal compare failures include:
- baseline or candidate file cannot be opened or read
- malformed JSON
- missing required field
- typed deserialization failure
- schema mismatch
- benchmark mismatch

## Output streams and exit status
- Successful comparison report: standard output
- Warning or unavailable-performance-counter diagnostic: standard error
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
The current automated test entry point is:

```bash
./scripts/smoke.py
```

`smoke.py` is currently the only automated project test suite. There is no Rust unit or integration test module yet.

The smoke suite should cover:
- successful text, Markdown, and CSV comparison
- missing baseline and candidate files
- malformed JSON
- missing required fields
- typed deserialization errors
- schema mismatch
- benchmark mismatch
- valid `summary.perf: null`
- correct standard output and standard error separation
- exit status `0` for success and `1` for fatal errors
- absence of `panicked at` and Rust backtraces

