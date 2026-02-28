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
