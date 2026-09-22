# PerfLab
PerfLab is a compiler-to-performance lab for repeatable C/C++ benchmarking. It builds benchmark variants, runs warmups and measured repetitions, optionally collects Linux `perf` counters, stores structured JSON results, and compares completed runs.
## Current capabilities
- C/C++ benchmarks built with Meson, Ninja, and Clang
- Configurable compiler and compiler flags
- CPU pinning for lower-noise measurements
- Warmup runs and repeated raw samples
- Phase summaries with median, min, max, and spread percentage
- Optional Linux `perf stat` collection with graceful fallback to `null`
- Result comparison in text, Markdown, and CSV formats
- Noise-aware per-phase effective thresholds and regression verdicts
- Explicit compare-input validation with non-panicking error handling
## Quick start
From the repository root:
```bash
cargo build
cargo run -- --help
cargo run -- run --bench <name> --compiler <path> -- <compiler-flags...>
cargo run -- compare results/<baseline>.json results/<candidate>.json
```
Run the current automated smoke suite with:
```bash
./scripts/smoke.py
```
## Repository layout
- `bench/` — C/C++ benchmarks, harnesses, and phase timing
- `runner/` — Rust command-line runner and comparison logic
- `schema/` — JSON schemas and examples
- `scripts/` — project validation scripts
- `docs/` — detailed usage and behavior documentation
- `report/` — future reporting and plotting work
## Documentation
Detailed setup, measurement, result-schema, comparison, validation, and testing instructions are in [`docs/USAGE.md`](docs/USAGE.md).
## Testing status
PerfLab uses layered automated testing:
- Rust unit tests cover internal comparison logic, threshold/verdict classification, input validation, deserialization, compatibility checks, and summary/noise-metric behavior.
- Rust integration tests exercise CLI behavior and representative fatal compare failures.
- `./scripts/smoke.py` provides end-to-end coverage with real benchmark runs, generated result files, perf/no-perf flows, and text, Markdown, and CSV comparison output.
