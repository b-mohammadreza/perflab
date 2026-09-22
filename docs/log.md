# PerfLab Log

## 2026-01-09
- Works now: runner can compile a selected benchmark, execute it, capture the single-line JSON (phase timings + checks), and write a combined `results/*.json` with toolchain + machine metadata.
- Known issue: `reduce` init dominates runtime (allocation/init cost dwarfs compute), so phase ratios aren’t ideal yet.
- Next step: integrate `perf stat` collection (cycles/instructions user-space) as optional attachment in results JSON (Task 5).

## 2026-01-12
- Added `perf stat` integration (best-effort): runner can collect user-space counters (cycles/instructions) and store them in results JSON.
- Hybrid Intel CPUs may report separate PMUs (e.g., `cpu_core/...` and `cpu_atom/...`); we store them as separate event keys (no aggregation).
- Graceful fallback: if perf fails/unavailable, PerfLab still produces results and sets `"perf": null`.

## 2026-02-28
- runner: refactor into lib/modules; keep `main.rs` minimal (CLI wiring).
- runner: add `--cpu <id>` CPU pinning (affinity set in runner so children inherit); record `meta.cpu_pin`.
- runner: add `--warmup <n>` (default 1) and `--reps <n>` (default 5); collect `samples[]` and compute `summary` (median).
- bench: increase `reduce` default iterations so compute dominates init; checksum unchanged.
- metadata: trim trailing newline from `uname`.

## 2026-06-14
- results: add per-rep perf CSV artifact linkage. Each successful perf sample now records its own `csv_path` and `perf_stat_args`, making raw perf artifacts traceable back to the exact repetition that produced them.
- test: add `scripts/smoke.py` as a v0.1 measurement-hygiene smoke check. It runs `matmul` and `reduce` with CPU pinning, perf, warmup, and repetitions, then validates the core results JSON contract.
- docs: document results schema versioning, `samples` vs `summary`, per-rep perf artifacts, and the explicit `null` policy for known optional fields.
- milestone: this completes the traceable stable-measurements layer on top of CPU pinning, warmup/reps, median summaries, and reduce compute-dominance cleanup.

## 2026-06-22
- compare: define v0 comparison contract in `schema/compare-v0.md`.
- v0 comparison compares two result JSON files: baseline vs candidate.
- hard requirements: matching `meta.schema_version` and `meta.bench`, plus required `summary.phases_ns` fields.
- phase deltas use `candidate - baseline`; positive timing delta means candidate is slower.
- perf comparison uses only common `summary.perf.events` keys and never treats missing events as zero.
- known issue: file I/O errors still panic through shared `io` helpers; tracked separately for error-handling cleanup.

## 2026-06-23
* compare: add `perflab compare <baseline> <candidate>` for v0 two-file result comparison.
* compare: validate matching `meta.schema_version` and `meta.bench`; warn on selected metadata differences.
* compare: print phase deltas for `init`, `compute`, and `teardown` using `summary.phases_ns`.
* compare: print perf event deltas for common `summary.perf.events` keys; skip perf comparison cleanly when either result has `summary.perf: null`.
* docs: document comparison usage, compared fields, skipped fields, and the current limitation that compare v0 uses medians only, not statistical confidence.

## 2026-06-29
* test: extend `scripts/smoke.py` to cover `perflab compare`.
* smoke: generate and validate two perf-enabled `matmul` runs, compare them, and check phase/perf comparison output.
* smoke: generate and validate two perf-enabled `reduce` runs, compare them, and check phase/perf comparison output.
* smoke: generate a no-perf `matmul` result and verify compare skips perf comparison cleanly.
* compare: polish v0 warning output so optional values print in JSON style (`null`, arrays, numbers) instead of Rust debug style (`Some(...)`, `None`).

## 2026-07-06
* compare: add `--format text` and `--format markdown` options.
* compare: keep `text` as the default output format.
* compare: add Markdown rendering for phase and perf comparison tables.
* compare: preserve perf-null behavior in Markdown mode by printing phase comparison and reporting perf comparison as unavailable.
* validation: manually checked default text, explicit text, Markdown, and perf-vs-no-perf Markdown comparison.
* validation: confirmed `cargo fmt --check`, `cargo build`, and `./scripts/smoke.py` pass.

## 2026-07-13
* compare: refactor comparison output behind a shared renderer interface.
* compare: keep format selection separate from rendering and invoke the selected renderer through a common interface.
* compare: preserve existing text and Markdown output exactly after the refactor.
* compare: route warnings, errors, and comparison-unavailable diagnostics to standard error.
* validation: confirm pre-refactor and post-refactor text and Markdown outputs have no differences.
* validation: confirm `cargo fmt --check`, `cargo build`, and `./scripts/smoke.py` pass.

## 2026-07-29
- compare: add CSV output format for phase and perf comparison rows.
- compare: keep CSV standard output clean by excluding report headers and diagnostics.
- compare: reuse the shared comparison rendering path for text, Markdown, and CSV output.
- validation: check CSV output, redirected CSV output, existing text/Markdown output, and smoke.
- test: extend smoke validation to cover `perflab compare --format csv`.
- test: verify CSV output includes phase and perf rows for perf-enabled comparisons.
- test: verify perf-unavailable CSV output keeps report rows on stdout and diagnostics outside CSV output.

## 2026-08-03
- compare: validate required result JSON paths before typed deserialization.
- compare: report the exact path of missing required fields.
- compare: handle input file and JSON parsing failures without panicking.
- test: add negative compare-input validation coverage.
- validation: confirm valid text, Markdown, and CSV output remains unchanged.

## 2026-09-14
- results/schema v2: phase summaries now carry `median_ns`, `min_ns`, `max_ns`, and `spread_percent` for `init`, `compute`, and `teardown`.
- summary: define zero-median spread behavior: all-zero samples produce `0`, while a zero median with any nonzero sample produces `null`.
- test: expand Rust unit coverage for median/min/max/spread calculations and schema-v2 numeric/null spread serialization/deserialization.
- test: add focused compare unit tests plus CLI integration tests for input validation, typed-deserialization paths, schema/benchmark compatibility, and fatal CLI behavior.
- smoke: keep end-to-end coverage broad while moving detailed metric correctness and error-path checks into Rust tests.
- docs: document layered testing responsibilities across Rust unit tests, CLI integration tests, and `scripts/smoke.py`.

## 2026-09-21
- compare: refactor generated comparison data around `ComparisonResult`, with separate phase comparisons, perf comparisons, comparison metadata, and structured warnings.
- compare: preserve warnings as nonfatal diagnostics on standard error while keeping text, Markdown, and CSV reports on standard output.
- compare: add a fixed `3.0%` base regression threshold.
- compare: derive each runtime phase effective threshold as `max(base threshold, baseline spread, candidate spread)`.
- compare: add per-phase verdicts: `REGRESSION`, `IMPROVEMENT`, `NO_MEANINGFUL_CHANGE`, and `UNAVAILABLE`; threshold boundaries are inclusive.
- compare: mark verdict unavailable when the percentage delta or effective threshold cannot be derived.
- compare: keep perf counters numeric-only in v0; perf rows do not receive thresholds or verdicts.
- render: add base threshold, effective threshold, and verdict to text and Markdown phase reports.
- render: extend CSV output with `effective_threshold_percent` and `verdict`; keep a consistent 10-column shape for both phase and perf rows.
- test: add focused threshold/verdict, formatter, phase-comparison, warning, and perf-availability unit coverage.
- smoke: validate threshold/verdict renderer integration without depending on a particular verdict from noisy real runs.
- validation: `cargo fmt --check`, `cargo build`, 37 Rust unit tests, 5 CLI integration tests, and `./scripts/smoke.py` all pass.
- next: environment/comparability validation is the next Priority 0 PerfLab item.
