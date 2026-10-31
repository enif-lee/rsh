# November 1 low-bandwidth transit screen

48 observations, four parallel jobs, two evaluations per role/context. All
links have nominal RTT100ms and no configured loss; rates are1Mbit/s,
256kbit/s and64kbit/s. Actual receive windows were November1 KST. The
environment was created October31 at23:59:52; that preparation timestamp and
the investigation folder date do not date the later receive windows.

All four roles use schema2 source RAM registration before write and a separate
pidfd/SIGUSR2 snapshot after all four windows close. This method differs from
the legacy producer in19/20/25; absolute results are not pooled. The raw source
audit covers19,764 records,19,759 completed writes and5 pending/null ends;
none of those pending records joins a received complete body. All7,699 actual
body headers match source number/time/scene. This is not a new terminal replay.

`core-results.json` retains all48 complete-frame sequences, body metrics,
source statistics/quality and96 original source/capture hashes. Raw bodies and
captures remain local. The compact archive supports independent metric
recalculation, not independent body-oracle replay. Some slow SSH tail/age
statistics are null: blank CSV cells, dashes in tables and no chart point
preserve missing values rather than treating them as zero.

`core-metrics.csv` uses decimal received-interface kbit/s, midpoint client/server
RSS in KiB and CPU percent of one core. Charts divide KiB by1024 into MiB.
Age is source-generation to complete-body-read, not RTT/GPU/input latency.
`acceptance-per-rep` files retain every role/resource and do not pool repeats,
set an A/A tolerance or automatically pass the candidate.

`cpu30-summary.json` preserves the user-authorized admission-only rule: one
interval user+system CPU<30%; runtime CPU/process values are context only.
Timing/metadata failures still reject. No host process names are archived.
`runtime-platform.json` records all eight actual image IDs/software versions.
Sampler acceptance is not P-core affinity or policy causality.

`run-status.json` was written before analysis, and the paired-build transcript
before measurement. Their analysis-pending/not-measured stage fields are
immutable historical records; final data/acceptance are in core-results.
Prepared/harness manifests refer to original local inputs, not a claim that
all raw/code inputs are bundled. This is a partial public archive.

The original build log is stored as deterministic gzip; decompression preserves
every byte, including original whitespace. `raw-build-log-sha256.json` records
its uncompressed SHA. No original log is normalized.

Run `python3 verify-metrics.py` here. With Matplotlib available, `python3 plot.py`
writes the two SVGs beside the report and records its export version. Optional
`--preview` also writes local PNG previews. `package-source.py STUDY` is the
extraction procedure and requires the original local raw study directory; it
does not fetch missing raw inputs or launch a workload. Imported CSV/patch
bytes are preserved. `manifest.json` hashes archived files and SVGs.

The transit candidate remains unselected. Better dashboard throughput and SSH
comparisons do not erase identical-main tail deficits or provide all-path
non-regression/SSH parity. See the [report](../../lowtail-20261031.md).
