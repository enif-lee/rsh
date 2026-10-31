# 52 low-bandwidth margin trace: mechanism audit

The 51 pattern is **not reproduced**, and the margin changes **no recorded branch-30 deadline result** in 52. There is no evidence here for a new RTT guard or another cadence adjustment. Keep 48 unselected; the negative activation finding is more informative than attributing ordinary differences to an unused policy expression.

This is a new functional experiment, not a retrospective explanation of the untraced 51 performance run. The saved summary has 16 functional-valid jobs, no functional-invalid reasons, performance eligibility false and one retained host-context rejection (random evaluation 1). All 24 CSV metadata files report zero drops, capacity 131072 and 80-byte records; their 260,587 rows are ordered and match metadata counts. Largest anchor bracket is 458 ns and largest absolute dump-clock discrepancy is 3958 ns. Metrics below are independently recalculated from saved complete-frame timestamps; no capture/emulator replay, raw source-ledger parsing, build, Docker or workload is performed. Total bodies/header observations are 4522/4533. The saved source summaries total 6887 records and zero pending writes; that source total is imported, not independently reparsed here.

## Observed metrics

Cells are avg / 1% / 5% low FPS. Lows use interior adjacent body gaps; max gap includes receive-window edges. Both repeated controls are traced COMBINED295f (`9621a9ce4e8a`), and margin is `e8a3f8def09b`, not the older CAN+PTY-only reference.

| Scene | Evaluation | Role | Body / header | FPS | Incomplete checks | Max gap ms |
|---|---:|---|---:|---:|---:|---:|
| dashboard | 0 | ssh | 190 / 191 | 23.7 / 6.5 / 7.1 | 780 | 179 |
| dashboard | 0 | main-trace-A | 480 / 480 | 60.0 / 42.6 / 45.6 | 0 | 28 |
| dashboard | 0 | main-trace-B | 480 / 480 | 60.0 / 41.0 / 45.9 | 0 | 27 |
| dashboard | 0 | margin-trace | 480 / 480 | 60.0 / 42.6 / 45.6 | 0 | 25 |
| dashboard | 1 | main-trace-B | 480 / 480 | 60.0 / 40.8 / 46.6 | 0 | 25 |
| dashboard | 1 | margin-trace | 480 / 480 | 60.0 / 40.3 / 45.7 | 0 | 26 |
| dashboard | 1 | ssh | 188 / 189 | 23.5 / 6.9 / 7.2 | 778 | 179 |
| dashboard | 1 | main-trace-A | 480 / 480 | 60.0 / 42.8 / 46.5 | 0 | 25 |
| random | 0 | ssh | 79 / 80 | 9.9 / 5.9 / 6.8 | 430 | 168 |
| random | 0 | main-trace-A | 186 / 188 | 23.2 / 12.5 / 18.9 | 4 | 83 |
| random | 0 | main-trace-B | 186 / 187 | 23.2 / 14.0 / 18.9 | 2 | 102 |
| random | 0 | margin-trace | 185 / 188 | 23.1 / 11.8 / 18.9 | 6 | 87 |
| random | 1 | main-trace-B | 189 / 190 | 23.6 / 12.6 / 19.1 | 2 | 80 |
| random | 1 | margin-trace | 183 / 183 | 22.9 / 17.3 / 18.7 | 0 | 72 |
| random | 1 | ssh | 80 / 81 | 10.0 / 4.6 / 6.4 | 414 | 219 |
| random | 1 | main-trace-A | 176 / 176 | 22.0 / 7.9 / 12.4 | 0 | 134 |

In random evaluation 0, margin has a modest lower 1% low (11.8 versus 12.5/14.0), with 6 incomplete checks and an 87.36 ms largest interior gap. Evaluation 1 reverses that relationship: margin 17.3 exceeds controls 12.6/7.9, with zero incomplete checks and a 71.53 ms largest interior gap. Margin average 23.1/22.9 does not establish an advantage. The two evaluations do not reproduce 51's pronounced margin-low deficit in both repetitions. Unknown/complete/incomplete percentages, headers, edge/tail gaps and all top-five gap endpoints remain in JSON/CSV rather than being discarded.

## Recorded policy activation

All six dashboard RSH jobs have no `screen_choice`/`screen_refresh` records: the screen/repaint context is unexercised, not proof of a zero-cost controller. All six random RSH jobs have zero branch-30 records during their eight-second receive windows. Their in-window scheduled records are branch 20 (known slow); branch 12 is its BDP-room rejection, branch 3 is not-all-sent, and branch 1 is repaint-false. No high-queue branch 40 appears.

Across the six random lifetimes there are 40 branch-30 snapshots (6–8 each), all before the receive window. Their recorded raw P ranges from 58.661989 to 75.473317 ms. Animation is set, but both `clamp(P,8,20)` and `clamp(0.9P,8,20)` equal 20 ms for every one. Actual stored f is exactly 20 ms. Thus the changed expression does not change any recorded branch-30 result, including startup. This is not simply failure to see an improved frame: the candidate's changed helper result itself is absent.

The first recorded branch-20 outcomes are +8.678 to +17.429 ms relative to receive start. Last branch-30 observations precede receive start; intervening gates exist. These snapshots bracket the observed change of helper outcome but do not timestamp the exact slow-rate state transition. `screen_choice` has no caller identity, so snapshot counts are not counts of selected timer deadlines.

The trace source confirms branch 20 is evaluated before branch 30 (`baseline-source/src/server.rs:2020` onward; the margin-only change is at branch 30 near 2096). Branch 30's guard is already no known slow BDP plus low queue. A new 1M/RTT guard is not supported by this non-activation result.

## What limits the recorded random path

The PTY's output-gap medians are 16.538–16.570 ms across all six jobs, but actual refresh-entry interval medians are 38.65–47.96 ms. These are different measured objects; scalar PTY chunks do not identify complete producer-frame boundaries. Approximately 667–675 of 1430–1443 PTY-apply calls have retained tails/parser-not-ground, across controls and margin. Those are per-read counts, not incomplete-frame percentages or causality.

Every random refresh is a delta (kind 1) in generation 1; full-refresh/reset count is zero. There is no recorded within-window generation-reset explanation for probe interruption. Known-slow selected rates range roughly 95,765–220,410 B/s, with fixed sampled floors 105.983–107.590 ms; these are controller estimates, not measured physical link capacity. Actual body delivery remains around 22–24 FPS.

Both margin windows have eight `capacity_probe` records, and all sixteen satisfy the 0.6 predicate with a sufficient margin even after integer-truncation uncertainty. Their minimum recorded utilization is 0.6045 and 0.6757, respectively. Controls have 12–16 probes each and 1–4 sufficient failures. This does not show the margin is stuck below the utilization threshold in this experiment. It also does not prove continuous latent demand or that every capacity increase remains effective after other sampling. The event is emitted before the conditional update (`baseline-source/src/proto.rs:1662–1680`).

Server window-block markers are 0/0 for the margin windows versus 70–325 in controls; delivery-block counts are 0/1 versus 0–2. These are observed controller-state differences despite the margin expression never changing its recorded result, so attributing them to the policy would be unsupported. Identical controls also differ substantially. Screen-bound timer masks, sender-rate histories, per-second branch/probe bins and actual poll entry/return are retained in JSON. Typical poll overshoot is small (window p95 about 0.43–0.79 ms; maximum about 2.88–4.01 ms), and the 200 ms requested polls can be interrupted by PTY/network readiness; a requested duration is not a corresponding observed sleep.

The margin's largest body gaps overlap pending client polls whose latest gate reason is catch-up: 59.38 ms of the 87.36 ms evaluation-0 gap and 28.27 ms of the 71.53 ms evaluation-1 gap. The worse control-A evaluation-1 gap is 133.79 ms, with 125.42 ms under that gate. These overlaps are recalculated and match the saved late-gap examples. `Paced::catching_up` requires `behind` and a bounded catch-up interval (`baseline-source/src/client.rs:1306–1324`). This proves a recorded wait state while the receiver expects more stream bytes; it does not prove that removing the wait improves intact-frame lows or that the timer caused the missing bytes. No token/both-gate overlap occurs in these largest-gap examples.

## Limits and next decision

The saved summary retains complete-body timestamps and incomplete-state counts/time percentages, but not every incomplete state's timestamp or all uniform-header frame timestamps. This audit therefore does not invent an exact partial-state timeline. It retains top-five actual body gaps and surrounding server/client CSV event summaries. Their common epoch is useful for temporal comparison, but a nearby PTY read, generation scalar or screen-choice row is not a full source-frame lineage proof. No first unknown source frame is assigned from scalar trace columns.

No guard patch is justified by 52. A later discriminating check would first have to demonstrate a real changed branch-30 result under the affected input, then connect it to intact-body outcomes. If an exact partial-state sequence becomes necessary, a separately bounded replay retaining state timestamps would be a new offline diagnostic, not a replacement for the original metrics. Existing 51 observations remain valid as saved performance evidence, and their cause remains unresolved.

Evidence: `lowbw52-mechanism-audit.py`, `lowbw52-mechanism-audit.json`, `lowbw52-metrics.csv`; original study `52-lowbw-cadence-trace/probe-summary.json` and 24 endpoint CSV/metadata files. Input hashes and each endpoint's zero-drop/clock evidence are recorded in the JSON. No product or sealed study inputs were changed.
