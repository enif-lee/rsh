# Study31: independent near-floor probe actual audit

**PASS for saved measurement evidence and arithmetic; candidate not accepted.** This audit independently reads raw source records, source/body header joins, clock projections, all frozen identities and numeric metrics; no Docker, build, test or terminal emulator replay.

48 jobs /12 blocks /4parallel ×2 evaluations; 21,617 complete bodies; 23,040 source records/192,593,640 payload bytes. Exact plan/role/SHA/slot/repetition coverage, before/after executable/listener/PID proofs, strict transport/qdisc snapshots, source fences and CPU30 single-interval admission passed. All avg/1%/5%/age/maxgap were recomputed from complete_frames; derived resource fields agree raw batch→summary→CSV. Original resource counter endpoints are not separately saved, so counter remeasurement is not claimed.

Actual windows KST: 2026-11-01T02:22:03.927057+09:00 → 2026-11-01T02:25:09.996303+09:00. Startup admitted27.0%; phase admitted18.0, 15.0, 23.0, 16.0, 16.0, 19.0, 20.0, 16.0, 24.0, 24.0, 20.0, 14.0%. Runtime maximum38.0% is context under current policy, not a rejection threshold. No P-core affinity is proved.

Legacy schema1 contains completed writes only; pending writes are unknown rather than zero. Zero-body jobs0; missing-low jobs0. RTT300/compress rep0 candidate50.4/27.6/30.3 trails mains54.9/55.0 average and stronger5% lows while SSH60/23.4/46.2. The1% value above SSH does not erase average/5% deficits. This is not universal improvement.

Actual-body avg /1% /5% FPS by repetition (null shown as—):

| Net/scene/rep | SSH | Main A | Main B | Near candidate |
|---|---:|---:|---:|---:|
| 00-lan-compress-i0 | 60.0 / 45.5 / 51.3 | 60.0 / 47.0 / 49.2 | 60.0 / 45.0 / 49.6 | 60.0 / 45.9 / 49.8 |
| 01-lan-compress-i1 | 60.0 / 43.3 / 49.7 | 59.7 / 41.9 / 48.4 | 59.9 / 41.6 / 48.0 | 59.9 / 40.5 / 47.4 |
| 02-lan-bandwidth-i0 | 60.0 / 47.9 / 51.8 | 60.0 / 44.9 / 47.6 | 60.0 / 44.6 / 48.1 | 60.0 / 44.2 / 48.5 |
| 03-lan-bandwidth-i1 | 60.0 / 41.9 / 50.1 | 59.9 / 42.0 / 47.7 | 59.9 / 41.4 / 47.3 | 59.9 / 44.4 / 47.7 |
| 04-wan-compress-i0 | 60.0 / 42.0 / 47.8 | 56.4 / 41.0 / 46.7 | 56.2 / 41.4 / 45.5 | 56.4 / 40.9 / 45.7 |
| 05-wan-compress-i1 | 60.0 / 38.8 / 48.3 | 56.5 / 43.9 / 47.1 | 55.9 / 42.8 / 46.5 | 56.0 / 41.8 / 46.9 |
| 06-wan-bandwidth-i0 | 60.0 / 41.3 / 48.7 | 54.6 / 43.7 / 46.7 | 54.7 / 30.6 / 46.6 | 55.1 / 42.5 / 46.7 |
| 07-wan-bandwidth-i1 | 60.0 / 38.0 / 47.6 | 54.1 / 22.5 / 45.5 | 54.6 / 23.7 / 47.1 | 54.6 / 29.5 / 46.6 |
| 08-highlat-compress-i0 | 60.0 / 23.4 / 46.2 | 54.9 / 41.3 / 45.9 | 55.0 / 37.9 / 46.2 | 50.4 / 27.6 / 30.3 |
| 09-highlat-compress-i1 | 60.0 / 24.9 / 47.7 | 55.0 / 41.3 / 46.1 | 51.0 / 28.3 / 30.2 | 55.1 / 44.0 / 47.4 |
| 10-highlat-bandwidth-i0 | 60.0 / 13.0 / 45.3 | 27.5 / 5.0 / 23.6 | 50.0 / 11.3 / 46.3 | 49.9 / 19.4 / 45.2 |
| 11-highlat-bandwidth-i1 | 60.0 / 12.6 / 46.5 | 49.6 / 30.7 / 47.0 | 49.5 / 24.8 / 46.0 | 49.9 / 25.3 / 45.8 |

Evidence: `probe3132-actual-audit.py`, `probe31-actual-audit.json`; JSON preserves all resource, age and raw deficit rows. Source31/schema1 and32/schema2 differ; do not absolutely pool. RX is client non-loopback decimal kbit/s, not bidirectional payload; RSS is midpoint process-name sum KiB, CPU percent of one core excluding source/observer Python. Age is conditional on completed bodies, not held-screen/input/GPU latency. No product promotion, all-path no-regression or SSH-parity claim.
