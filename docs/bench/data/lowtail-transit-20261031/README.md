# TCP transit core-screen evidence

This is the separate 2026-10-31 transit-allowance screen: 48 cases, 4 parallel
jobs, two evaluations per context, 21,925 observed complete bodies. Data checks
passed; no all-path performance acceptance is claimed. See the
[report](../../lowtail-20261031.md#follow-up-tcp-transit-allowance).

`core-results.json` retains per-repetition complete frames, metrics and original
capture/source hashes. `core-metrics.csv` uses KiB for RSS, decimal kbit/s for
received interface traffic, and one-core percent for CPU. `acceptance-per-rep`
files retain deficits and missing observations without an automatic pass.
`runtime-platform.json` records the actual eight image IDs/software versions
before the first receive window. The header audit checked all observed bodies.

Raw captures/source bodies remain local; the public archive supports metric
recalculation but not body-oracle replay. `run-status.json` was written before
analysis, and the paired-build transcript before measurement; their historical
`analysis_pending`/`not_measured` fields are not final analysis verdicts.
`package-source.py` is the original-machine extraction record, not a portable
entry point. Run `python3 verify-metrics.py` here to check the compact metrics.
`plot.py` reads these results and exports the two transit SVGs beside the report
with Matplotlib; its export version is in `plot-environment.json`.

This source producer is the same as the quarter core screen. Neither screen is
pooled with trace-on diagnostics or the later signal-collector pipeline.
