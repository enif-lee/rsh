# Study34: independent saved evidence audit

**PASS for recorded input integrity and arithmetic. Product/FPS selection remains a separate decision under the updated FPS-first tradeoff policy.** No new benchmark/product executable/build/test ran in this audit. Raw source/header/clock data and saved complete-frame arithmetic were read independently; no entire-study terminal replay was performed.

48 jobs/12 blocks/4parallel×2 evaluations; 7,616 frozen-oracle complete bodies; 19,774 source records/161,843,427 retained bytes. Exact48role-slot identities, SHA/live-server/PID proofs, strict qdisc/firewall, actual source4fences, driver/schedule/analysis/prepared hashes and CPU30 first qualifying admission passed. CPU maximum50.0% remains runtime context. All avg/1%/5%/age/maxgap values agree with complete_frames; resource fields agree raw batch→summary→CSV, not independent raw-counter remeasurement.

Window KST: 2026-11-01T03:08:52.161164+09:00 → 2026-11-01T03:18:54.440733+09:00. Source schema2. Pending attempts are8 with null ends and0 received pending-body joins. Zero-body jobs0. Missing low jobs: 2 (null retained).

**Coherence investigation:** `08-slow64k-compress-i0`, candidate slot3 retains12 body frames/1.5avg versus121 uniform headers/15.1avg. Last complete body is window+0.656425s and edge-inclusive maxgap7344ms. The1%/5%15.5 values consider only11 adjacent completed-frame gaps and omit the7.344s empty tail; they are not a recovery/tail advantage. Source has480completed writes, no pending, writep99.242543ms/max1.191091ms.

Targeted exact frozen replay reproduces the saved values. It first observes a mixed n13/n12 snapshot and later only3 differing characters `[0m` at zero-based row24/cols94–96 while all29 rowtags advance. Direct whole/read/bytewise pyte0.8 controls reproduce the same residue for minimal nested-ESC-in-CSI bytes; explicit CAN/ordinary reset controls remain blank. This suggests parser/model semantics rather than establishing a physical output blackout. Current Rust Terminal/actual-terminal comparison remains necessary; no arbitrary cell mask or original metric rewrite is applied. See `far34-body-mismatch-diagnostic.json`, `far34-pyte-stream-controls.json`, `far34-offline-replay-provenance.json` and receiver arithmetic review.

Selection priorities:1%low/5%low/avgFPS together first; baseline-control differences separate from SSH gap. CPU/RSS secondary; age/staleness/recovery/network explicit tradeoffs. Do not invent weights/tolerances, discard single-metric costs automatically, or treat inherited raw-gap labels as final selection. Different source pipelines are not absolutely pooled.

| Net/scene/rep | SSH avg/1%/5% | Main A | Main B | Floor candidate |
|---|---:|---:|---:|---:|
| 00-lowbw-compress-i0 | 23.5 / 6.7 / 7.0 | 56.0 / 41.3 / 46.4 | 57.4 / 28.2 / 40.3 | 60.0 / 41.6 / 47.8 |
| 01-lowbw-compress-i1 | 23.5 / 6.0 / 7.1 | 55.4 / 40.4 / 45.7 | 55.9 / 38.0 / 46.4 | 60.0 / 41.3 / 46.5 |
| 02-lowbw-bandwidth-i0 | 10.0 / 5.2 / 6.4 | 20.2 / 10.3 / 17.3 | 20.6 / 11.9 / 18.0 | 21.7 / 10.1 / 18.4 |
| 03-lowbw-bandwidth-i1 | 10.0 / 6.2 / 6.5 | 21.3 / 13.9 / 18.1 | 19.9 / 9.9 / 17.5 | 21.7 / 10.3 / 17.9 |
| 04-verylow-compress-i0 | 6.1 / 3.5 / 5.2 | 40.1 / 24.9 / 27.3 | 33.6 / 24.7 / 26.4 | 59.8 / 38.5 / 45.7 |
| 05-verylow-compress-i1 | 6.0 / 4.2 / 5.1 | 41.1 / 26.3 / 27.3 | 34.1 / 25.3 / 26.8 | 60.0 / 41.3 / 46.7 |
| 06-verylow-bandwidth-i0 | 2.5 / 1.5 / 1.5 | 5.9 / 5.4 / 5.5 | 6.0 / 5.3 / 5.5 | 5.9 / 5.4 / 5.4 |
| 07-verylow-bandwidth-i1 | 2.4 / 1.2 / 1.2 | 6.0 / 5.3 / 5.5 | 5.9 / 5.5 / 5.5 | 5.9 / 5.4 / 5.5 |
| 08-slow64k-compress-i0 | 1.5 / 1.1 / 1.1 | 15.9 / 11.4 / 12.7 | 15.7 / 11.3 / 13.1 | 1.5 / 15.5 / 15.5 |
| 09-slow64k-compress-i1 | 1.5 / 1.2 / 1.2 | 15.9 / 11.6 / 13.0 | 16.0 / 9.2 / 12.7 | 15.2 / 6.1 / 12.1 |
| 10-slow64k-bandwidth-i0 | 0.6 / — / — | 1.6 / 1.4 / 1.4 | 1.5 / 1.4 / 1.4 | 1.5 / 1.4 / 1.4 |
| 11-slow64k-bandwidth-i1 | 0.6 / — / — | 1.4 / 1.3 / 1.3 | 1.4 / 1.3 / 1.3 | 1.5 / 1.4 / 1.4 |

Full per-rep resource, age and deficit rows: `probe34-actual-audit.json`; audit script `transit3334-actual-audit.py`. RX is whole client non-loopback decimal kbit/s, RSS midpoint process-name sum KiB, CPU percent of one core excluding source/observer Python. Raw counter endpoints are not separately retained. Body age is conditional on completed bodies, not held-screen/input/GPU latency. No physical P-core affinity or unconditional promotion is established.
