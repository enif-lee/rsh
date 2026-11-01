# Sampled-floor transit: core development screen

Saved records, charts and verification sources are included; no new benchmark was run to assemble this archive.

This archive represents study 33 only: 48 jobs/12 blocks, four parallel roles,
two evaluations each, LAN/RTT 100/RTT 300 and dashboard/random scenes. Internal
main-A/B roles are identical actual-main artifacts; candidate is the
sampled-floor-gated transit allowance. Original 80% exploration remains.
The legacy completed-write source pipeline does not record pending attempts;
those are unknown rather than zero. Do not pool with the RAM-signal slow screen.

`core-results.json` will retain per-job complete-body frames and metrics,
header statistics, visible-state percentages, adjacent-gap sample count,
edge-inclusive maximum gap and the trailing interval without a new body.
The CSV preserves null lows/ages as empty fields and keeps evaluations
separate. A trailing interval is right-censored at window end and excluded
from the adjacent-gap low percentiles. Header FPS is not complete-body FPS.

FPS avg/1%/5% are primary, CPU/RSS secondary; age, network and recovery costs
remain explicit. Raw comparison flags are descriptive, not an automatic
selection policy. Some individual regressions can be acceptable with overall
improvement. This candidate is currently unselected; no global parity claim
or invented numeric weights are used.

Raw full captures/source bodies remain local. Their 96 hashes and a partial
source-header audit are retained; compact frames permit metric recalculation,
not a fresh body-oracle replay. The resource audit checks saved-derived
batch→summary→CSV values and units, not missing underlying counter endpoints.
CPU admission is one interval user+system<30%; runtime load is context.

`verify-metrics.py` will verify all archive/figure manifest hashes and
recalculate retained frame metrics/CSV values using the standard library.
`plot.py` uses Matplotlib and preserves both evaluations. `package-source.py`
needs the original local study directory and is provenance, not a portable
raw replay runner. Imported CSV/patch/log bytes and creation-stage metadata
are preserved; deterministic gzip keeps decompressed build-log bytes exact.

See the separate CAN control archive for parser interoperability evidence;
its result is not a replacement or correction to this dataset.
