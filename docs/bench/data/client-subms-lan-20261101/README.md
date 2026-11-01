# Study 37: focused LAN wait comparison

Sixteen records, four parallel jobs and two evaluations per role/scene.
The baseline is CAN-fixed source `93533574fffa` / SHA `c68e0f…84a136`,
not unfixed main. Candidate source `ca0abbd5fc40` / SHA `91e765…1d92cb`
changes only Linux client positive waits below 1 ms. Actual branch activation
and native wake precision are untraced. The inherited CAN tests are not
wait-specific tests.

All records have 480 body/header observations and average 60 FPS, totaling
7,680 source-matched bodies. No pending write, zero body or low value is
missing. Tails are mixed against CAN replicas and lower than SSH; the
candidate remains unselected. FPS avg/1%/5% are primary, CPU/RSS secondary;
there is no invented score or automatic single-metric veto.

RAM/nonce/pidfd/five-second collection and child SET/GET-before-exec method
match study 36. Data is eligible for qualitative focused comparison, not
full-matrix promotion. Final selection is null. Do not pool with earlier
unfixed-main or different source/geometry methods.

`records.json` preserves complete-frame sequences and resource metadata;
`metrics.csv` retains all 16 roles/evaluations and edge gaps. Resource
checks are saved-derived consistency and units, not reconstruction of missing
counter endpoints. Body age is generation-to-read, not input/GPU latency.

The independent audit's 64 endpoint-file and 32 live-server checks supplement
source clocks/headers/joins and capture identity. The 32 raw capture/source
hashes are retained; full raw glyph/payload files remain local. This archive
permits saved-statistic recalculation, not fresh terminal/body-oracle replay.
Local-path audit/extraction sources are provenance. Original build/preparation
status strings retain their creation stage rather than being silently updated.

Run `python3 verify-metrics.py` for manifest/import/metric checks. Plotting
alone requires Matplotlib: `uv run --with matplotlib python plot.py`.
Neither command starts a benchmark. Imported patch/log bytes are unchanged;
gzip preserves decompressed log bytes and SVG line whitespace is normalized.
