# PerfLab

## Build / run the CLI
From repo root:
- Build: `cargo build`
- Help: `cargo run -- --help`
- Run (WIP): `cargo run -- run --bench <name> --compiler <path> -- <compiler-flags...>`

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
