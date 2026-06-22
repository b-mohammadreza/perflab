# PerfLab comparison contract v0
This document defines the first comparison contract for two PerfLab result JSON files.

The goal of v0 comparison is intentionally narrow: compare two completed result files using their existing `summary` values. It does not attempt statistical confidence, outlier analysis, matrix comparison, or report generation.

## Inputs
A comparison takes two result JSON files:

* **baseline**: the reference/old result
* **candidate**: the new result being compared against the baseline

Comparison direction is always:

```text
delta = candidate - baseline
percent_delta = (candidate - baseline) / baseline * 100
```

For timing values, this means:

* positive delta = candidate is slower
* negative delta = candidate is faster

## Required hard-match fields
Before comparing, v0 comparison must require:

* `meta.schema_version` exists in both files
* `meta.schema_version` matches between files
* `meta.bench` exists in both files
* `meta.bench` matches between files
* `summary.phases_ns` exists in both files
* `summary.phases_ns.init` exists in both files
* `summary.phases_ns.compute` exists in both files
* `summary.phases_ns.teardown` exists in both files

If any of these requirements fail, comparison should stop with a clear error.

## Warning-only metadata differences
The following fields are useful for judging whether a comparison is fair, but v0 comparison should not fail on differences:

* `meta.compiler.path`
* `meta.compiler.version`
* `meta.compiler_args`
* `meta.cpu_pin`
* `meta.warmup`
* `meta.reps`
* `meta.perf_events_requested`
* `meta.workdir`
* `meta.git_sha`
* `meta.uname`

A future tool may print warnings when these differ.

## Phase comparison
v0 comparison compares median phase timings from:

* `summary.phases_ns.init`
* `summary.phases_ns.compute`
* `summary.phases_ns.teardown`

For each phase, report:

* baseline value
* candidate value
* absolute delta
* percent delta

Values remain in nanoseconds for v0.

## Perf comparison
Perf comparison is optional.

Perf comparison is available only when both files have:

* `summary.perf` as an object
* `summary.perf.events` as an object

When perf comparison is available, compare only event keys present in both files.

Do not treat missing event keys as zero.

If one or both files have `summary.perf: null`, phase comparison should still run, but perf comparison should be skipped with a message such as:

```text
perf comparison unavailable: one or both results have no summary.perf
```

If both files have perf data but no common event keys, perf comparison should be skipped with a message such as:

```text
perf comparison unavailable: no common perf event keys
```

For each common perf event, report:

* baseline value
* candidate value
* absolute delta
* percent delta

## Zero baseline policy
If the baseline value is zero, percent delta is not meaningful.

In that case:

* still report absolute delta
* mark percent delta as unavailable / not applicable

Do not divide by zero.

## Dynamic key policy
Perf event maps are dynamic.

Comparison should only use keys that are actually present in both files. Missing keys must not be synthesized with `0`, `"none"`, or other sentinel values.

This follows the existing PerfLab result schema policy for dynamic maps such as:

* `perf.events`
* `bench_output.params`
* `bench_output.check`

## Out of scope for v0
v0 comparison does not include:

* statistical confidence intervals
* min/max/MAD/stddev analysis
* comparison across directories
* build matrix comparison
* CSV or markdown report generation
* plots
* regression thresholds
* automatic pass/fail decisions

Those can be added after the basic two-file comparison path works.
