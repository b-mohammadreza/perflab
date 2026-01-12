# PerfLab — Vision (v0)

Mission: ... TBD

Near-term MVP (target: next week): end-to-end flow works:
C/C++ benchmark → clang build → run → benchmark emits phase timing (init/compute/teardown) → runner writes one JSON result file with metadata.

Scope boundaries:
- PerfLab is an owned repo; it consumes LLVM/Clang/MLIR via supported interfaces (no upstream changes required).
- Linux-first; focus on stable measurements over feature breadth.

Decisions (v0):
- Single-thread-first (reduce noise; add OpenMP later).
- Phase timing is inside benchmarks; runner wraps/records results.
- Perf counters are best-effort via `perf stat` user-space events (`:u`); if unavailable, still produce JSON.

Milestone (Q1): end-to-end v0 CPU runner achieved (compile bench → run → capture phase JSON → write results/*.json with metadata).
Status: done (Jan 2026).

