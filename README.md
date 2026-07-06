# PerfLab

## Build / run the CLI
From repo root:
- Build: `cargo build`
- Help: `cargo run -- --help`
- Run: `cargo run -- run --bench <name> --compiler <path> -- <compiler-flags...>`

## Repo layout
- `bench/` benchmarks (C/C++ kernels + harness + phase timing)
- `runner/` Rust CLI runner/orchestrator
- `schema/` JSON schemas/examples
- `report/` JSON → markdown/plots (later)

## Benchmarks (Meson + Ninja + clang++)
Benchmarks live under `bench/`. You can build and run them directly (developer sanity check) without the PerfLab runner.

### Setup (one-time)
From the repo root (assuming meson ninja-build clang are installed):
```bash
meson setup bench/build --backend=ninja --native-file bench/clang.ini
```

If you need to reconfigure from scratch:
```bash
meson setup --wipe bench/build --backend=ninja --native-file bench/clang.ini
```

### Build
```bash
meson compile -C bench/build
```

### Run (sanity check)
Run all registered benchmark tests:
```bash
meson test -C bench/build --verbose
```

Or run each benchmark individually:
```bash
meson test -C bench/build reduce --verbose
```

Or run binaries directly:
```bash
./bench/build/bench/reduce
./bench/build/bench/matmul
```

## Perf counters (Linux perf stat)
PerfLab can optionally collect CPU performance counters via Linux `perf stat`. When enabled, PerfLab runs the benchmark under `perf stat` and stores counters under `perf.events` in the results JSON.

Notes:
- PerfLab uses *user-space* events (the `:u` suffix), e.g. `cycles:u` and `instructions:u`.
- On some systems, `perf` access is restricted (kernel `perf_event_paranoid`, SELinux/AppArmor, etc.). In that case PerfLab falls back to a normal run and writes `"perf": null` in the results JSON.

Example:
```bash
cargo run -- run --perf --bench reduce --compiler clang++ -- -O3
```

## CPU pinning (determinism on hybrid CPUs)
PerfLab can pin the entire run (compile + perf collection + benchmark execution) to a single logical CPU to reduce noise and make results repeatable—especially on hybrid Intel systems (P-cores + E-cores).

### Usage
```bash
perflab run --cpu <cpu_id> --bench matmul --compiler clang++ -- -O3
perflab run --cpu <cpu_id> --perf --bench matmul --compiler clang++ -- -O3
```

`<cpu_id>` is a Linux logical CPU ID (the same numbering used by taskset -c).

### How to pick a CPU ID
List available logical CPUs:
```bash
lscpu -e=CPU,CORE,SOCKET,MAXMHZ
```

On hybrid systems, pinning to different CPU IDs may change which PMU block reports non-zero counters (e.g., `cpu_core/*` vs `cpu_atom/*`). For consistent comparisons, reuse the same `--cpu` value across runs.

### Output metadata
When `--cpu` is provided, results include the chosen CPU ID in metadata (e.g., `meta.cpu_pin`), so runs are self-describing.

## Warmup + repetitions (measurement hygiene)
Use warmups to discard cold-start effects, then collect multiple reps and summarize with medians.

### Usage
```bash
perflab run --warmup 1 --reps 5 --bench matmul --compiler clang++ -- -O3
perflab run --perf --warmup 1 --reps 5 --bench matmul --compiler clang++ -- -O3
```

### Defaults:
- `--warmup 1`
- `--reps 5`

## Output layout
PerfLab writes one results JSON file per run under `results/`.

The top-level layout is:

- `meta`: run metadata
- `samples`: raw per-repetition measurements
- `summary`: median summary derived from `samples`

### `meta`
`meta` contains information needed to understand and reproduce the run:

- `schema_version`: results schema version. Current version is `1`.
- `command`: argv used to launch the PerfLab run.
- `workdir`: working directory where the run was launched.
- `bench`: benchmark name.
- `compiler`: compiler path and version.
- `compiler_args`: compiler flags passed after `--`.
- `git_sha`: current repository commit.
- `uname`: host kernel/system string.
- `cpu_pin`: logical CPU ID when `--cpu` is used, otherwise `null`.
- `warmup`: number of warmup runs.
- `reps`: number of measured repetitions.
- `perf_events_requested`: requested perf events when `--perf` is used, otherwise `null`.

### `samples` vs `summary`
`samples` contains one entry per measured repetition. Its length should equal `meta.reps`.

Each sample contains:

- `bench_output`: benchmark JSON output for that repetition.
- `perf`: perf data for that repetition, or `null` if perf was unavailable.

When perf succeeds, `perf` contains:

- `csv_path`: path to the per-rep perf CSV artifact under `out/`.
- `perf_stat_args`: perf stat arguments used for that repetition.
- `events`: parsed perf event counters.

`summary` contains medians derived from `samples`:

- `summary.phases_ns`: median `init`, `compute`, and `teardown` phase timings.
- `summary.perf`: median perf events over samples where perf exists. If no samples have perf data, `summary.perf` is `null`.

### Null and dynamic-map policy
For stable known fields, PerfLab prefers explicit `null` over omitting keys when a value is not applicable. Examples: `meta.cpu_pin`, `meta.perf_events_requested`, per-sample `perf`, and `summary.perf`.

For dynamic key/value maps, keys are only present when observed. PerfLab does not synthesize missing keys with `0`, `"none"`, or other sentinel values. This applies to:

- `perf.events`
- `bench_output.params`
- `bench_output.check`

## Comparing result files
PerfLab can compare two completed result JSON files:

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

For timing values, positive delta means the candidate is slower and negative delta means the candidate is faster.

PerfLab compare v0 checks that both files have the same `meta.schema_version` and `meta.bench`. It then compares median values from `summary`.

Compared fields:

* `summary.phases_ns.init`
* `summary.phases_ns.compute`
* `summary.phases_ns.teardown`
* common keys in `summary.perf.events`, when perf data exists in both files

Output formats:

```bash
perflab compare <baseline.json> <candidate.json> --format text
perflab compare <baseline.json> <candidate.json> --format markdown
```

`text` is the default output format and preserves the normal terminal report.

`markdown` prints the comparison as Markdown tables, which is useful for GitHub issues, notes, and reports.

During development:

```bash
cargo run -- compare results/<baseline>.json results/<candidate>.json --format markdown
```

Skipped fields / behavior:

* perf comparison is skipped if either file has `summary.perf: null`
* perf event keys are compared only when present in both files
* missing perf event keys are not treated as zero
* metadata differences such as compiler version, git sha, uname, CPU pinning, reps, or requested perf events may produce warnings but do not stop comparison

Compare v0 uses existing median summaries only. It does not yet compute statistical confidence, outlier analysis, regression thresholds, or pass/fail decisions.

To validate the current measurement and compare path:

```bash
./scripts/smoke.py
```
