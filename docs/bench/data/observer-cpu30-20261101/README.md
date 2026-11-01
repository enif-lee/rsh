# Fixed-product observer-method screen · November 1, 2026

96 observations, four parallel jobs and two evaluations per observer mode,
network, scene and role. LAN/RTT100/RTT300 × dashboard/random × legacy/raw-only
uses SSH and three byte-identical main TCP rsh replicas. Whole-block order is
i0 legacy → i0 raw → i1 raw → i1 legacy, with crossed slots on evaluation1.
All rsh roles deploy SHA9ab79c22…ecb463/source6729d398d4f1. No candidate product
is present or selected.

Both modes retain their own existing first-header anchor. Raw-only disables
live pyte/feed/look work after anchoring; capture/regex/queue work remains.
Transition timestamps are sampled after flags change, not exact instruction
times. No startup or estimator-seeding exclusion follows. All comparisons
use offline complete-body results, not raw-mode live placeholders.

Actual receive windows are November1 KST. The legacy completed-post-write
source pipeline is unchanged from25; it is not pooled with24's RAM pre-write
and signal-source pipeline. All42,132 observed body headers match raw source
number/time/scene. All46,080 retained source headers pass recorded number/scene
and positive-time checks.
The producer does not record in-progress writes before write completion.

`observer-results.json` retains all96 complete-frame sequences, source and
product resource statistics, plus192 original raw capture/source hashes.
Original raw files stay local. This partial archive permits independent
metric recalculation, but terminal-character body-oracle replay requires the
raw captures and source payloads. The saved-data audit reports those original
checks; its original local audit is not rerun by the portable verifier.

Original prepared/harness/analysis manifests and plan identify frozen local
inputs; not all identified files or local paths are bundled. Preparation and
run-stage status fields remain immutable; final data status is in
`observer-results.json`. Exact product/image/pyte provenance is included.

CSV network is client RX decimal kbit/s, RSS named-process midpoint KiB and
CPU percent of one core for the product process. Charts convert RSS to MiB.
Age is generation-to-complete-read; not RTT/input/GPU latency. Python
observer/source costs are excluded. CPU30 is one admission interval below30%
user+system; runtime load remains context, not a rejection threshold.
`cpu30-summary.json` omits host process names.

`python3 verify-metrics.py` recalculates all96 body metrics and48 mode deltas
from archived frame sequences. `uv run --with matplotlib python plot.py`
exports three normalized SVGs beside the report (optional `--preview` writes local
PNGs). `package-source.py STUDY` extracts stored local artifacts; it does not
execute or fetch a benchmark. Evaluations are never pooled; missing values
remain null/blank/absent chart points. No observed zero or missing low occurred
in this actual study; general zero/null retention also has mock coverage.

`manifest.json` hashes all archive files and three SVGs. Imported CSV bytes
are preserved. The observer treatment has mixed effects; it establishes no
all-area SSH parity, automatic A/A tolerance or product-policy causal gain.
