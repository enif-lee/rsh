# Study32: independent near-floor probe actual audit

**PASS for saved measurement evidence and arithmetic; candidate not accepted.** This audit independently reads raw source records, source/body header joins, clock projections, all frozen identities and numeric metrics; no Docker, build, test or terminal emulator replay.

48 jobs /12 blocks /4parallel ×2 evaluations; 7,302 complete bodies; 19,780 source records/161,907,017 payload bytes. Exact plan/role/SHA/slot/repetition coverage, before/after executable/listener/PID proofs, strict transport/qdisc snapshots, source fences and CPU30 single-interval admission passed. All avg/1%/5%/age/maxgap were recomputed from complete_frames; derived resource fields agree raw batch→summary→CSV. Original resource counter endpoints are not separately saved, so counter remeasurement is not claimed.

Actual windows KST: 2026-11-01T02:28:40.188218+09:00 → 2026-11-01T02:31:54.222164+09:00. Startup admitted22.0%; phase admitted16.0, 20.0, 22.0, 21.0, 19.0, 14.0, 14.0, 22.0, 26.0, 16.0, 11.0, 27.0%. Runtime maximum43.0% is context under current policy, not a rejection threshold. No P-core affinity is proved.

RAM schema2 nonce/ready/flag/ledger identity, actual4closure→pidfd signal→original5s dump checks pass. Seven source attempts retain pending null ends and zero pending attempts join received completed bodies. Zero-body jobs0. Missing lows remain null: SSH64k/random5bodies in both reps, plus main A/B rep0 with10bodies (9gaps), four jobs/eight missing1%/5% values.

1M/random candidate21.2/20.4 average and p95age117/118ms avoid the prior broad candidate’s repeated high age in these observations, but1M/compress rep1 candidate52.4/28.5/30.5 trails both mains56.2/~44/~47. Resource and remaining per-rep adverse outcomes prevent acceptance. This is a screen, not proof every previous regression is resolved.

Actual-body avg /1% /5% FPS by repetition (null shown as—):

| Net/scene/rep | SSH | Main A | Main B | Near candidate |
|---|---:|---:|---:|---:|
| 00-lowbw-compress-i0 | 23.6 / 6.5 / 7.0 | 56.2 / 41.1 / 46.4 | 53.7 / 29.4 / 33.4 | 56.2 / 41.2 / 46.5 |
| 01-lowbw-compress-i1 | 23.7 / 6.5 / 7.0 | 56.2 / 44.2 / 47.2 | 56.2 / 44.3 / 47.4 | 52.4 / 28.5 / 30.5 |
| 02-lowbw-bandwidth-i0 | 10.0 / 5.9 / 6.7 | 20.1 / 9.3 / 15.2 | 21.4 / 12.6 / 18.3 | 21.2 / 14.0 / 18.3 |
| 03-lowbw-bandwidth-i1 | 10.0 / 4.9 / 6.2 | 20.0 / 11.3 / 18.1 | 19.9 / 12.5 / 17.7 | 20.4 / 10.0 / 17.7 |
| 04-verylow-compress-i0 | 6.0 / 5.1 / 5.2 | 40.7 / 26.3 / 27.6 | 33.9 / 25.3 / 26.8 | 36.2 / 25.8 / 27.3 |
| 05-verylow-compress-i1 | 6.1 / 5.1 / 5.1 | 40.9 / 26.6 / 28.0 | 40.4 / 25.3 / 27.3 | 38.7 / 24.7 / 26.8 |
| 06-verylow-bandwidth-i0 | 2.4 / 1.8 / 1.8 | 5.9 / 5.3 / 5.5 | 5.7 / 2.9 / 5.4 | 5.9 / 5.4 / 5.5 |
| 07-verylow-bandwidth-i1 | 2.4 / 1.2 / 1.2 | 5.9 / 5.3 / 5.5 | 6.0 / 5.5 / 5.6 | 5.9 / 5.1 / 5.4 |
| 08-slow64k-compress-i0 | 1.5 / 1.2 / 1.2 | 15.5 / 11.6 / 12.9 | 16.2 / 11.6 / 13.3 | 16.0 / 11.4 / 12.7 |
| 09-slow64k-compress-i1 | 1.4 / 1.0 / 1.0 | 16.0 / 11.1 / 12.9 | 16.1 / 11.0 / 12.4 | 15.6 / 11.1 / 12.3 |
| 10-slow64k-bandwidth-i0 | 0.6 / — / — | 1.2 / — / — | 1.2 / — / — | 1.5 / 0.8 / 0.8 |
| 11-slow64k-bandwidth-i1 | 0.6 / — / — | 1.6 / 1.4 / 1.4 | 1.6 / 1.3 / 1.3 | 1.4 / 1.3 / 1.3 |

Evidence: `probe3132-actual-audit.py`, `probe32-actual-audit.json`; JSON preserves all resource, age and raw deficit rows. Source31/schema1 and32/schema2 differ; do not absolutely pool. RX is client non-loopback decimal kbit/s, not bidirectional payload; RSS is midpoint process-name sum KiB, CPU percent of one core excluding source/observer Python. Age is conditional on completed bodies, not held-screen/input/GPU latency. No product promotion, all-path no-regression or SSH-parity claim.
