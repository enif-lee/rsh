# November 1 ThinLTO core screen

48 observations, four parallel jobs, two evaluations per role/context: LAN,
RTT100 and RTT300 × dashboard/random × SSH/main-A/main-B/ThinLTO. Actual
environment and receive windows are November1 KST; the investigation folder
name retains its October31 start date.

This uses the legacy post-write source ledger from19/20/23, with the current
CPU30 admission policy. It is not pooled with24's pre-write/signal-source
pipeline or earlier repeats. All20,308 observed body headers match raw source
number/time/scene; the extractor does not rerun terminal-character emulation.

The current paired sources differ only in Cargo.toml's release profile:
`lto="thin"`, `codegen-units=1`. Original patch/build log/source manifests and
the source-guarded matched build transcript are included. The candidate's
3,114,232 bytes versus3,695,752 bytes is15.7% smaller, not a runtime win.
The release-library check passed96 tests with2 ignored. Build/not-measured
and run/analysis-pending fields describe those earlier immutable stages;
final data status is in `core-results.json`.

The original build log is stored as deterministic gzip to preserve its bytes
and whitespace. `raw-build-log-sha256.json` records the uncompressed SHA;
decompression recovers the original log exactly.

`core-results.json` retains all48 complete-frame sequences/metrics/source
statistics and96 original raw source/capture hashes. Raw files remain local;
this partial public archive supports metric recalculation, not independent
body-oracle replay. Prepared/harness manifests identify original inputs but
do not imply every input is bundled.

CSV RSS is midpoint KiB, CPU percent of one core and received-interface traffic
decimal kbit/s. Charts convert RSS to MiB. Age is generation-to-complete-read,
not RTT/input/GPU latency. `cpu30-summary.json` records admission intervals
below30% total user+system and runtime load as context without host process
names. Runtime levels≥30 do not invalidate data under the user's policy.
Image/version evidence is in `runtime-platform.json`; no P-core isolation or
automatic candidate acceptance is implied.

Run `python3 verify-metrics.py` here; `python3 plot.py` with Matplotlib exports
two SVGs beside the report (optional `--preview` writes local PNGs).
`package-source.py STUDY` requires the original local study/raw directory and
records extraction only; it does not run or fetch a benchmark. Repetitions
are never pooled. Null statistics remain blank/dashes/absent points.
`manifest.json` hashes the archived files and normalized SVGs; imported
CSV/patch bytes remain unchanged.

ThinLTO remains unselected. Candidate WAN/random29.4/29.5FPS versus main54–55
is retained, and the separate final RTT300/random collapse affects all three
rsh roles. Neither observation establishes the compiler as the cause. These
experimental files are separate from PR#3's product TCP batching change.
