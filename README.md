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
The results JSON contains:
- `meta`: includes `warmup` and `reps` (plus compiler info, git sha, uname, etc.)
- `samples`: array of per-rep samples (length == `reps`)
- `summary`: medians of `init/compute/teardown` and (when available) perf events

Perf failure policy:
- If `--perf` is enabled and perf fails on a given rep, that rep is still recorded, but its `perf` is `null`.
- `summary.perf` is computed as the median over reps where perf exists. If no reps have perf, `summary.perf` is `null`.
