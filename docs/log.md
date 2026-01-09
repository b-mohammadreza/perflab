# PerfLab Log

## 2026-01-09
- Works now: runner can compile a selected benchmark, execute it, capture the single-line JSON (phase timings + checks), and write a combined `results/*.json` with toolchain + machine metadata.
- Known issue: `reduce` init dominates runtime (allocation/init cost dwarfs compute), so phase ratios aren’t ideal yet.
- Next step: integrate `perf stat` collection (cycles/instructions user-space) as optional attachment in results JSON (Task 5).
