# Probe60 slow screen · November 1, 2026

48 observations, four concurrent jobs and two evaluations per context/role:
1Mbit/s/256kbit/s/64kbit/s at nominalRTT100ms/no loss ×dashboard/random×
SSH/main-A/main-B/probe60. Roles cross slots for evaluation1; main controls
are byte-identical. All roles use exact24's RAM pre-write/schema2/nonce/pidfd
SIGUSR2 source, original5s collection and four-party closure fences. This
differs from27's legacy producer; absolute values are not pooled across them.

The candidate changes only the server-output lossless/TCP probe utilization
threshold0.8→0.6 plus four fixtures. Client input/UDP retain0.8. The corrected
scope, paired main9ab/source6729 versus candidate40a/source7b3, patch, fixture
source and manifests are bundled. Target4/library100pass+2ignored and exact
untouched-control3pass/1expectedfail demonstrate policy separation, not benefit
or safety. Original build/guard/control logs use deterministic gzip with
uncompressed hashes. Immutable earlier build/preparation/run status fields
precede the final core-results.json status.

Every per-run complete frame/body metric/source statistic and96 original
raw source/capture hashes are archived. Original payloads/captures stay local;
this is compact recalculation evidence, not an independent terminal-character
replay package. Original input manifests identify a wider local scope than
the files bundled here. Source pending attempts retain null write ends and
are distinct from actual body reception. Diagnostic22 excerpts are separate
functional-only evidence and are not performance evidence for this run.

Network is client RX decimal kbit/s; RSS is named-product-process midpoint
KiB (charts MiB), CPU percent of one core. Python observer/source costs are
excluded. Age is generation-to-complete-read, not RTT/input/GPU latency.
Resource verification checks saved batch/summary/CSV consistency and units;
underlying resource counters were not retained for independent remeasurement.
CPU30 is one user+system<30% admission interval; runtime load is context,
while timing/metadata/sampling failures still reject. Four workers are not
physical P-core affinity or automatic acceptance.

python3 verify-metrics.py recalculates each run. uv run --with matplotlib
python plot.py exports FPS/resources/age SVGs beside the report; optional
--preview writes local PNGs. Age charts use a labeled logarithmic scale.
package-source.py STUDY extracts saved local files without running/fetching
a benchmark. Null statistics stay blank/dashes/absent points: lows need at
least10 gaps, age p95 at least20 bodies. Repetitions are not pooled.
manifest.json hashes all archive files and three normalized SVGs; original
CSV/patch bytes are preserved.

Probe60 remains unselected. At1M/random, higher23.0/22.6FPS accompanies body
p95 age176/131ms versus main114–116ms, with mixed1% lows. Slow64k/dashboard
evaluation1 also retains FPS/low deficits. No universal improvement,
automatic A/A tolerance, SSH parity or regression-free product acceptance is
claimed; loss/moving/input/session/UDP/resource follow-ups remain necessary.
