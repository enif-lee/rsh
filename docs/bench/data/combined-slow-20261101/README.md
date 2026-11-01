# Combined policy: slow static screen, study 39

48 jobs / 12 blocks, four parallel roles and two evaluations per case.
Both baseline replicas are CAN-fixed source `93533574fffa` / SHA `c68e0f…`,
not unfixed main. Combined source `94f28a7b3b13` / SHA `7198d2…` adds the
sampled-floor transit allowance and broad 60% server-output probe condition;
CAN is shared. No sub-ms wait or near-floor predicate is combined.

Both the core and slow screens use RAM schema2 source, nonce/pidfd/original-five-second
collection, four-window fences and the study 37 child SET/GET-before-exec
method. Geometry is source-flow evidence, not a retained native GET sidecar.
No earlier pipeline or unfixed-main measurements are pooled here.

Saved totals: 7,867 complete bodies, 7,888 offline uniform headers,
19,765 source records and 7 pending writes; pending-body joins are zero.
The candidate is currently unselected. Data is eligible for qualitative
profile comparison, final selection is null, and individual costs are not
an automatic veto. FPS avg/1%/5% are primary; CPU/RSS are secondary, with
age/network/recovery costs kept separate and no invented weighting.

`core-results.json` and CSV preserve all evaluations, body/header differences,
visible-state percentages, adjacent-gap sample counts and boundary/trailing
gaps. Lows exclude window-edge intervals, and age covers detected bodies
only. Missing values stay null/empty. Do not substitute online/live placeholders
for offline body/header criteria or infer a GPU result from the body model.

Historical patch chain: transit → sampled floor → broad probe60 → CAN.
The reference already contains CAN. Proofs, manifests, build/control/guard
records retain original bytes and creation-stage wording. The 110 library passes
with two ignored, including 14 target fixtures, are not a separate 14-test
rerun or proof of actual branch activation/combined benefit. The oldfar+CAN
probe control has three passes and one expected growth-condition failure.

All 96 original capture/source hashes and 48 complete-frame sequences are
retained. Source payload/header/clock/joins were checked by the independent
audit; full glyph/capture data stays local. Resources are saved-derived
batch→summary→CSV consistency and units, not missing counter endpoints
reconstructed. CPU admission is one interval below 30%; runtime load is
context. P-core affinity and global SSH parity are unproved.

Run `python3 verify-metrics.py` for archive/figure manifest and saved-metric
checks. `uv run --with matplotlib python plot.py` rerenders the three charts.
Neither runs a benchmark. The local-input extractor and audit source paths
are provenance, not a portable raw terminal-replay package. Imported CSV,
patch and decompressed log bytes remain unchanged.
