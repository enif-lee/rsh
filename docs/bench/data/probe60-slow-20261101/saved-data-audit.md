# Study 28: independent actual saved-evidence audit

**PASS for saved measurement evidence and arithmetic; candidate not accepted.** No new workload, Docker, test, compiler or terminal emulator ran. The script independently read all source ledgers and recomputed complete-body statistics; it did not repeat full capture/oracle emulation.

Evidence: `probe2728-actual-audit.py`, `probe28-actual-audit.json`; 48 jobs / 12 blocks / 4 parallel × 2 evaluations, 7,431 complete bodies. Exact planned identities, prepared/harness/parent hashes, analysis snapshot, actual main/candidate SHA, server PID/starttime/listener, four-window fences and strict qdisc/UDP forcing passed. All FPS/low/age/gap and CSV resource fields match saved raw inputs.

Raw source audit: 19,781 records, 161,897,961 payload bytes, 222,066,054 JSON bytes. Every payload FRAME n/T/scene was read and joined to observed complete bodies. Actual window KST: 2026-11-01T01:55:23.022915+09:00 → 2026-11-01T01:59:12.443770+09:00.

CPU policy: startup admission 27.0%; every phase admitted on its first qualifying user+system<30 interval. Phase values: 25.0, 28.0, 25.0, 28.0, 24.0, 29.0, 23.0, 26.0, 26.0, 28.0, 27.0, 27.0%. Runtime maximum 51.0%; runtime CPU/process values remain context and were not rejection thresholds. No P-core affinity proof.

RAM schema2 source/PID nonce, ready=flag=ledger identities, same-process pidfd transaction, all four actual closures and original5s collection deadlines passed. Six pending writes retain null ends; none joins to an actually completed body. This proves retention of pending source attempts, not received pending bodies. Zero-body jobs0. SSH slow64k/random has5 bodies in each rep, so its two 1%/5% pairs remain null (four missing low values).

Positive: 1 Mbit/s random candidate23.0/22.6 FPS versus mains19.6–20.2; 256 kbit/s compress41.6/40.9 versus mains36.5–40.9. These are accompanied by higher work/rate and do not establish efficiency.

Adverse: 1 Mbit/s random p95 age176/131ms versus main114–116ms; RX922/910 versus788–807 kbit/s and server CPU2.97/2.96% versus2.54–2.68%. Candidate low1=12.8/12.7 is below the stronger main14.2/13.9. At64k compress rep1 candidate15.6/11.2/12.3 versus main A16.0/11.4/12.9 and B15.9/11.6/12.9. No all-metric improvement.

All rows below are **actual completed-body avg / 1% / 5% FPS**; no pooling across repetitions or source pipelines. Full age, RX, midpoint RSS, CPU and every deficit are in JSON.

| Net / scene / rep | SSH | Main A | Main B | Probe60 |
|---|---:|---:|---:|---:|
| 00-lowbw-compress-i0 | 23.6 / 6.3 / 7.1 | 56.0 / 43.0 / 46.3 | 57.1 / 28.9 / 35.2 | 56.1 / 43.3 / 46.2 |
| 01-lowbw-compress-i1 | 23.6 / 5.9 / 7.1 | 56.1 / 43.3 / 47.3 | 56.4 / 45.0 / 47.3 | 56.2 / 43.3 / 47.2 |
| 02-lowbw-bandwidth-i0 | 10.0 / 5.5 / 6.5 | 19.6 / 10.2 / 17.4 | 20.0 / 14.2 / 18.1 | 23.0 / 12.8 / 18.9 |
| 03-lowbw-bandwidth-i1 | 9.9 / 5.4 / 6.4 | 20.2 / 13.9 / 18.0 | 19.9 / 10.6 / 17.9 | 22.6 / 12.7 / 18.9 |
| 04-verylow-compress-i0 | 6.0 / 4.2 / 5.1 | 38.1 / 25.9 / 27.2 | 40.9 / 26.6 / 27.8 | 41.6 / 26.4 / 28.3 |
| 05-verylow-compress-i1 | 6.0 / 4.3 / 5.1 | 40.1 / 26.2 / 27.6 | 36.5 / 24.3 / 27.3 | 40.9 / 26.6 / 28.1 |
| 06-verylow-bandwidth-i0 | 2.5 / 1.5 / 1.5 | 5.9 / 5.4 / 5.5 | 5.9 / 5.3 / 5.5 | 5.9 / 5.3 / 5.5 |
| 07-verylow-bandwidth-i1 | 2.4 / 1.3 / 1.3 | 5.9 / 5.4 / 5.5 | 5.9 / 5.4 / 5.5 | 5.9 / 5.4 / 5.5 |
| 08-slow64k-compress-i0 | 1.4 / 1.0 / 1.0 | 16.0 / 8.5 / 12.4 | 15.6 / 8.8 / 12.5 | 15.9 / 11.3 / 12.1 |
| 09-slow64k-compress-i1 | 1.4 / 1.1 / 1.1 | 16.0 / 11.4 / 12.9 | 15.9 / 11.6 / 12.9 | 15.6 / 11.2 / 12.3 |
| 10-slow64k-bandwidth-i0 | 0.6 / — / — | 1.6 / 1.4 / 1.4 | 1.6 / 1.4 / 1.4 | 1.6 / 1.4 / 1.4 |
| 11-slow64k-bandwidth-i1 | 0.6 / — / — | 1.5 / 1.4 / 1.4 | 1.4 / 1.3 / 1.3 | 1.4 / 1.3 / 1.3 |

Resource verification checks raw batch→summary→CSV transfer and the frozen source formulas; raw RX/CPU counter endpoints and RSS readings are not separately saved, so underlying resource measurement arithmetic cannot be independently rebuilt from those counters. Resource interpretation: RX is whole client non-loopback received traffic in decimal kbit/s, not bidirectional bytes or payload throughput. RSS is a midpoint process-name sum in KiB, not peak/session memory. CPU is transport-process scheduler time as percent of one core, excluding source/observer Python. Body age is conditional on completed frames; it is not input, held-screen or GPU latency. Study27 uses legacy completed-write collection; study28 uses RAM prewrite/signal collection. Do not combine their absolute values or infer a causal capacity/probe mechanism without traced evidence.
