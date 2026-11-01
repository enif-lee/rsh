# 65 bounded TCP margin quiet — unbuilt scratch hypothesis

65 is a scope/cap variant of an existing quiet mechanism, not a new algorithm
or selected fix. Parent is actual source60 fa934e198646. Only its no-slow-BDP,
low-queue branch30, TCP and animating case changes: original margin due is
maxed with min(output_at+existing FRAME_GATHER_GAP1ms,last+existing SCREEN_GAP20ms).
There is no new state, constant, option, protocol/API, global screen_gap,
rate estimator/window/credit/RTO, UDP, known-slow branch20 or high-queue change.
Unknown BDP is not a claim that slow_rate is necessarily None.

Mechanism hypothesis: last+0.9P can already be eligible when a fresh PTY
piece arrives. Terminal parser ground does not mean a complete producer
frame. A brief pre-render quiet window may coalesce adjacent pieces before
refresh constructs a screen delta. The existing gathered_at quiet is later
zraw/compressor flushing; Session::output returns early for repaint_due, so
it is not the same protection. Large zraw can also bypass quiet at16KiB.
No actual frame atomicity or FPS gain is inferred.

The unbounded output_at+1ms proposal is rejected: Session::output gaps<5ms
keep the old animation classification and advance output_at. Endless tiny
writes could postpone forever. This candidate caps the logical raw branch30
deadline at last+20ms, while never preceding the original8..20ms margin
deadline. This is not an actual poll/service-time bound; TAIL/credit can still
defer effective output. At normal P16.6, maximum extra raw deadline is about
5.1ms, not universally1ms; at the8ms clamp it can be12ms. After a long idle,
last+20ms is already past, so the first new partial frame can still refresh
immediately. Pauses>1ms within a producer frame and cap expiry can still
expose partial frames. Continuous output can tend toward50 logical updates/s
and alter age/network/CPU and frame skipping; no no-regression claim follows.

Prior counterevidence is retained. benchmark/lab/optimize.py34-36,79-83 added
eligible.max((output+1ms).min(last+screen_gap+10ms)) in next_screen_at for all
TCP animation, including known-slow/high-queue cases. Its room+fresh+quiet
combination had RTT300 random18.0 complete-body FPS vs50.5/48.2 controls;
the separate traced48.2 does not cancel it (docs/bench/tcp-batch-20261030.md
prior-policy selection audit). It did not isolate quiet's causal effect.
The prior cap ranges18..30ms; this20ms cap is shorter whenP>10ms, equal at10,
and longer forP<10ms. Do not call it uniformly tighter. Earlier gather0
removed existing animation gather quiet; its mixed matrix did not establish
a general fix. Neither experiment proves65 failed or succeeds.

Current61 WAN random candidate1% lows29.7/31.6 vs B40.2/41.6 are not causal
evidence: A is28.9/33.2, candidate incomplete state~0.55% resembles A~0.4–0.57%
while B is0, and A/candidate share the odd-slot pair under the existing
two-evaluation crossing. Root's64 reversal diagnostic is separate evidence;
do not dismiss it, pool it, infer CPU-core cause, or add a tolerance. No
producer-frame lineage from scalar PTY events has been proved here.

Three synthetic fixture methods are planned, not run: expired/recent margin
deadline boundaries and repeated sub-ms writes; unaffected nonanimation/
bothUDP/knownslow/highqueue paths; TAIL/unsent/shadow/repaint and immediate
long-idle limitation. Same fixtures on unchanged parent60 are planned to
fail the first extension assertion and pass the other two. Parent tests and
all other32 files remain byte-identical after scoped inverse. No build,
capture, benchmark or Rust test has run. This is not a PR/product change;
any future experiment waits for Root review and uses four workers×two
evaluations, body/header/partial/freshness and all adverse results intact.

V2 retained-fixture correction, adopted before any build: parent60's retained
quarter fixture reset the rate, set output_at=last+P, then asserted only the
unknown-rate margin deadline. The new quiet result intentionally differs.
Both candidate/control now set output_at=last in that setup while preserving
the original assertion; new65 fixture1 explicitly covers fresh output. This
isolates the old test's purpose instead of removing the failing assertion.
Scoped inverse restores last+P before comparing all32 parent files. Complete
v1 UNBUILT source/fixture/proof/input snapshots are preserved separately.
