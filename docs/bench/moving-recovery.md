# Moving-network recovery (2026-10-29)

The moving demo exposed a feedback loop in the path estimator: a small
dashboard update delayed by jitter looked like a low-capacity link. Pacing
subsequent updates at that rate reduced the delivered rate too, so normal
traffic could not demonstrate the recovered capacity. Keeping samples for
ten seconds did not help when later jitter continually supplied new low
samples. The initial independent-netem trace reached roughly 1.8 KB/s on
the recovering/clear path (`moving-baseline-trace-demo-20261029-124103`).

## Transport change

- Finish a back-to-back packet pair before a later frame, even when that
  frame's delay would join it to a busy train. Otherwise the inter-frame
  delay hides the pair's serialization measurement.
- When an estimated slow path sees only small, spaced output, send at most
  one additional short duplicate per second, with at most 64 stream bytes.
  Full-sized chunks and multi-datagram screen updates are excluded, including
  their short final tail. A small tail in ordinary byte-stream mode can
  still probe. Recent genuine serialization samples, including short packet pairs,
  suppress additional probes. Spaced-frame jitter does not. Idle sessions and paths
  without a slow estimate do not probe.
- Two unspread pairs showing over twice the estimate invalidate that
  estimate. A genuinely spread packet pair cancels this recovery evidence;
  a later frame's jitter does not. Small pairs can invalidate a low estimate
  without declaring the link faster than the existing 250 KB/s threshold.
- On modelled UDP paths, proactive and reactive screen-mode entry require two
  rate samples or a standing queue. The reactive path previously omitted
  this check: on RTT 300 ms, a single host-delayed pair with under 1 ms of
  standing queue started screen mode and reduced FPS for its hold interval.
  Window, actual queue and memory safeguards remain independent.
- TCP and independently deflated datagrams do not use prefix probes.
  A compressed stream can accept a duplicate prefix at its ordinary stream
  offset; an independently compressed packet must remain whole.

The deterministic path test covers both a recovered fast path and an
unchanged 64-kbit/s path, with delayed frames between probes. Existing
path/window/queue tests remain in place. Recovery evidence is reset whenever its rate estimate is cleared or expires;
a newly learned rate needs two fresh pairs. The wire protocol is unchanged.

## Common conditions and observation window

Each tool retains its own link and capacity, while every link receives the
same precomputed delay/rate/outage schedule against the Docker VM clock.
Workers are ready before choosing the start. A seeded normal delay sample
changes every 250 ms; nominal loss percentages are now **outage fractions
of time**, implemented as shared 100%-loss intervals. Packet counts no
longer advance independent impairment RNGs. The trace repeats every 30 s.

Source scenes are scheduled against the same clock. Their end agrees with
the common measurement end; a late source skips elapsed frame numbers
rather than extending the test. Moving summaries use that common interval,
excluding drain time. Static summaries retain their previous convention.
`network_trace` preserves the complete schedule and each container's
post-application timestamp, since OS scheduling is not simultaneous.

Retained demo slots receive the current scene script before clients start,
so an earlier script cannot silently ignore the scheduled-start argument.

The demo has one top profile header. Pane headers contain program, FPS,
displayed-frame age and received kbit/s. Frame age is not an RTT measurement.

## Measurement

Both builds used the final common source/impairment schedule, SSH/mosh/rsh,
60 FPS, a 158×43 attached terminal, and 60 seconds per scene (two cycles).
The baseline was installed build `a5bb6cae5bf0`; the candidate was
`7a1d9454d5b1`. Tests used isolated `rmv-*` containers and the `rshmoving`
tmux socket. The ordinary command is:

```sh
python3 benchmark/lab/lab.py demo --tools ssh,mosh,rsh --events '' \
  --nets moving --scenes compress,bandwidth --fps 60 --duration 60
```

Local raw artifacts under `benchmark/profile/out/investigation/`:

- `moving-baseline-demo-20261029-130341`
- `moving-dedup-demo-20261029-140913`

| rsh measurement | Baseline | Candidate |
| --- | ---: | ---: |
| Dashboard, whole 60 s | 29.23 FPS | 37.72 FPS |
| Dashboard, first clear recovery, seconds 25–30 | 8.6 FPS | 58.0 FPS |
| Dashboard, second clear recovery, seconds 55–60 | 58.8 FPS | 57.6 FPS |
| Random screen, whole 60 s | 30.47 FPS | 30.22 FPS |
| Random screen, seconds 25–30 | 55.4 FPS | 56.2 FPS |
| Random screen, seconds 55–60 | 55.6 FPS | 56.4 FPS |

Recovery means completion of the first full one-second window with at least
54 displayed frames after the clear phase starts (23/53 s). Dashboard
recovery was absent during the first baseline clear phase and took 1.0 s
in the second; the candidate took 1.2/2.45 s. The already-healthy second baseline
cycle recovered sooner; improvement is not uniform across cycles. Random-screen
recovery was 1.2/1.8 s versus 1.75/1.9 s: no consistent gain is claimed for that scene.
All three tools reached both scenes' final markers in both runs. These are
two cycles per build, not a guarantee for all real networks.

Candidate maximum application skew between containers was 26.75 ms for the
dashboard and 32.91 ms for random screens; maximum lateness from schedule
was 51.71/42.63 ms. Baseline maximum skew was 16.79/4.43 ms. The intended
conditions are shared; actual packet losses and application times are not
claimed to be identical.

## Review and checks

Reviewer findings adopted: exclude independently compressed packet prefixes;
close all worker input pipes on interruption; align source and measurement
ends; preserve the end epoch when source launch is late; corroborate
reactive screen-mode entry after a static-link regression was reproduced;
reset recovery evidence when its estimate is invalidated. No outstanding
blocking finding remained after the follow-up review.

`NO_COLOR= cargo test --release`: 125 passed, two ignored. Python lab tests:
10 passed, including common trace replay, blackout restoration, interruption,
worker disconnect, common summary windows and late source startup.

The Docker interruption check stopped a two-slot replay during the shared
100%-loss handover. All four links returned to 35 ms and no replay workers
remained (`/tmp/rsh-moving-interrupt.log`). A retained-slot launch also
verified that both server copies of `scene.py` matched the host SHA-256.

All four distribution binaries were built and their checksums verified.
Darwin arm64 and both Linux architectures execute the version check;
Darwin x86_64 is compile/checksum verified, without a runtime check here.

## Static-link regression checks

The rate-entry guard was checked at RTT 300 ms: baseline dashboard
60.0/59.2 FPS versus 60.0/60.0 after the fix (12 s, two repeats,
`run-20261029-135425.json`). The fresh-evidence fix was separately repeated
three times on `badmobile` and `slow64k` (`run-20261029-134706.json`):
mean dashboard FPS was 26.0/13.43 before and 27.87/14.17 after.

The final short-pair suppression was checked for 20 s twice at 256 kbit/s,
RTT 400 ms, with and without random loss (`run-20261029-140423.json`).
The lossless check was 47.4/56.7 FPS before and 51.4/59.9 after. With
nominal 2% loss, results were 26.4/35.4 versus 22.8/25.4 FPS, but server
qdisc counters showed different realized packet losses: 18/931 and 13/975
before, 21/860 and 21/973 after. Those random-loss FPS samples do not
establish equivalent realized conditions or a uniform non-regression claim.
Raw qdisc counters are retained with the results rather than treating
these differences as proof of a code regression or discarding them.

A broad exploratory run covered LAN, RTT 100/300 ms, 1 Mbit/s, 256 kbit/s
and 64 kbit/s; later runs added mobile, badmobile and 2% loss.
Fourteen switch/visible-echo cases completed without agent errors
(`run-20261029-131012.json`). Probes add at most one small duplicate per
second when no recent useful pair exists; this is a bounded cost, not
zero-cost measurement. Large screen updates do not pay it.
