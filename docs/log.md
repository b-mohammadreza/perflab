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
