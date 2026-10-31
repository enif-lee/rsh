# Far34 slow64k dashboard: saved-metric arithmetic review

**The saved complete-body result is 12 frames / 1.5 FPS, while the header result is 121 frames / 15.1 FPS. These are different criteria, not interchangeable results.** No new replay, benchmark, product or harness change was performed. This review independently recalculated the eight slow64k/compress jobs' body metrics from saved `complete_frames`; measurement's separate raw-capture/oracle investigation owns the coherence diagnosis.

For `34-transit-floor-slow-cpu30-screen/08-slow64k-compress-i0`, candidate slot3:

- Window: 8.000708818 s. All 12 complete frames are n=0 through11; first is at the window start and last at +0.656424761 s. There is then **7.344284058 s without a newly detected complete body**, 91.7954% of the window. This does not mean bytes or headers stopped arriving.
- Body per-second counts: `[12,0,0,0,0,0,0,0]`; average1.5 FPS; edge-inclusive maximum gap7344 ms. Header per-second counts remain `[14,13,16,16,16,15,16,15]`.
- Body1%/5% lows15.5/15.5 are correct for the existing definition: only the **11 observed adjacent complete-frame gaps**, p99/p95 both selecting the largest64.609766 ms gap. The trailing7.344 s is excluded from this percentile sample. These lows therefore do not express the prolonged absence of further complete bodies.
- Stored visible-state result: incomplete90.0708%, unknown0.1710%, complete9.7582%. It is time-weighted state evidence, distinct from the91.7954% interval without a new complete body. `218/242` checked states is not that time percentage. This review retained those percentages without independently regenerating them.
- Body age median388 ms and p95null refer only to the12 detected complete bodies; the p95 requires20 samples. They cannot establish freshness across the empty tail.

The same cohort's main-A/main-B retain127/126 complete bodies,15.9/15.7 FPS, maximum gaps134/143 ms. In iteration1 the candidate moves to slot1 and has122 complete bodies/15.2 FPS,6.1/12.1 lows,207 ms maximum gap and a38.8 ms empty tail. These per-evaluation outcomes must remain separate; neither averaging repetitions nor displaying lows alone preserves the observed dropout.

All stored body frame counts, average FPS,1%/5% lows, edge-inclusive maximum gaps, adjacent gaps and per-second counts matched independent arithmetic for all eight jobs. The JSON companion preserves exact values and the reviewed summary SHA. Header values are read from `uniform_header`; raw terminal coherence, source causality, physical presentation and a patch-caused failure are not established here.

Definition evidence: frozen `harness/agent.py:332–364` computes lows over adjacent observations and maximum gap over window edges; frozen `harness/replay.py:57–74` computes time-weighted visible-state percentages. A publication must show complete-body count/whole-window average and dropout/max gap beside the lows and label header FPS separately.
