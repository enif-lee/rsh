# Study 27: independent actual saved-evidence audit

**PASS for saved measurement evidence and arithmetic; candidate not accepted.** No new workload, Docker, test, compiler or terminal emulator ran. The script independently read all source ledgers and recomputed complete-body statistics; it did not repeat full capture/oracle emulation.

Evidence: `probe2728-actual-audit.py`, `probe27-actual-audit.json`; 48 jobs / 12 blocks / 4 parallel × 2 evaluations, 21,074 complete bodies. Exact planned identities, prepared/harness/parent hashes, analysis snapshot, actual main/candidate SHA, server PID/starttime/listener, four-window fences and strict qdisc/UDP forcing passed. All FPS/low/age/gap and CSV resource fields match saved raw inputs.

Raw source audit: 23,040 records, 192,593,640 payload bytes, 261,499,451 JSON bytes. Every payload FRAME n/T/scene was read and joined to observed complete bodies. Actual window KST: 2026-11-01T01:48:23.242320+09:00 → 2026-11-01T01:53:27.703737+09:00.

CPU policy: startup admission 23.0%; every phase admitted on its first qualifying user+system<30 interval. Phase values: 23.0, 29.0, 25.0, 18.0, 17.0, 27.0, 20.0, 27.0, 29.0, 27.0, 28.0, 24.0%. Runtime maximum 52.0%; runtime CPU/process values remain context and were not rejection thresholds. No P-core affinity proof.

Legacy schema1 contains completed source writes only; unrecorded pending writes cannot be counted as zero. All 23,040 source records had coherent clock brackets and raw header/body joins. Zero-body jobs0; missing low jobs0.

Positive: RTT100/random rep1 candidate53.9 FPS versus main A29.2/B29.0, but candidate low1=17.7 remains below SSH24.3; server CPU5.81% and RX2102 kbit/s exceed mains3.75/3.31% and1157/1147 kbit/s as useful work rises. RTT300/compress candidate55.0/55.2 beats slow main A46.0/37.6, while main B54.9/55.0 is already comparable. This does not isolate a universal threshold benefit.

Adverse/mixed: LAN/compress candidate59.7 in both reps versus mains59.9–60; RTT300/random rep0 candidate49.5 versus both mains50.2, with low5=44.9 below45.8/45.9. RTT100/random rep0 low1 improves41.9 versus27.8/31.6, but avg54.9 lies between mains55.1/54.5. No uniform no-regression or SSH parity.

All rows below are **actual completed-body avg / 1% / 5% FPS**; no pooling across repetitions or source pipelines. Full age, RX, midpoint RSS, CPU and every deficit are in JSON.

| Net / scene / rep | SSH | Main A | Main B | Probe60 |
|---|---:|---:|---:|---:|
| 00-lan-compress-i0 | 60.0 / 46.0 / 50.0 | 59.9 / 37.9 / 48.2 | 59.9 / 39.8 / 48.5 | 59.7 / 43.0 / 48.8 |
| 01-lan-compress-i1 | 60.0 / 45.8 / 50.5 | 60.0 / 42.6 / 48.1 | 60.0 / 42.7 / 46.8 | 59.7 / 42.3 / 47.8 |
| 02-lan-bandwidth-i0 | 60.0 / 47.6 / 50.3 | 60.0 / 44.2 / 48.5 | 60.0 / 44.2 / 47.6 | 60.0 / 44.6 / 47.6 |
| 03-lan-bandwidth-i1 | 60.0 / 45.0 / 49.2 | 60.0 / 43.0 / 48.5 | 60.0 / 42.1 / 46.9 | 59.9 / 44.0 / 47.4 |
| 04-wan-compress-i0 | 60.0 / 30.7 / 48.5 | 55.4 / 30.3 / 46.5 | 33.9 / 22.5 / 27.4 | 56.4 / 30.1 / 46.9 |
| 05-wan-compress-i1 | 60.0 / 46.5 / 49.3 | 56.5 / 44.8 / 47.2 | 56.7 / 44.8 / 47.4 | 56.2 / 45.0 / 47.8 |
| 06-wan-bandwidth-i0 | 60.0 / 25.8 / 45.8 | 55.1 / 27.8 / 46.3 | 54.5 / 31.6 / 46.4 | 54.9 / 41.9 / 46.9 |
| 07-wan-bandwidth-i1 | 60.0 / 24.3 / 45.0 | 29.2 / 11.5 / 24.0 | 29.0 / 11.3 / 23.1 | 53.9 / 17.7 / 43.4 |
| 08-highlat-compress-i0 | 60.0 / 23.0 / 45.6 | 46.0 / 25.6 / 28.8 | 54.9 / 41.7 / 45.3 | 55.0 / 39.2 / 45.4 |
| 09-highlat-compress-i1 | 60.0 / 22.6 / 46.7 | 37.6 / 26.5 / 28.0 | 55.0 / 44.1 / 47.3 | 55.2 / 41.6 / 46.9 |
| 10-highlat-bandwidth-i0 | 60.0 / 12.4 / 44.3 | 50.2 / 11.3 / 45.8 | 50.2 / 17.5 / 45.9 | 49.5 / 13.4 / 44.9 |
| 11-highlat-bandwidth-i1 | 60.0 / 12.0 / 46.3 | 50.0 / 14.4 / 45.0 | 49.5 / 10.7 / 45.6 | 50.1 / 22.4 / 45.3 |

Resource verification checks raw batch→summary→CSV transfer and the frozen source formulas; raw RX/CPU counter endpoints and RSS readings are not separately saved, so underlying resource measurement arithmetic cannot be independently rebuilt from those counters. Resource interpretation: RX is whole client non-loopback received traffic in decimal kbit/s, not bidirectional bytes or payload throughput. RSS is a midpoint process-name sum in KiB, not peak/session memory. CPU is transport-process scheduler time as percent of one core, excluding source/observer Python. Body age is conditional on completed frames; it is not input, held-screen or GPU latency. Study27 uses legacy completed-write collection; study28 uses RAM prewrite/signal collection. Do not combine their absolute values or infer a causal capacity/probe mechanism without traced evidence.
