# Dashboard frame drops on a long, unconstrained path

Investigation: 2026-10-29. Diagnostic baseline: `cpu-fix` checkpoint `6f3e3bb`
(the earlier `gate2` experiment, on top of `cpu` `c9901b8`).

## Reproduction and cause

The demo uses a 158×43 viewer, OpenSSH, mosh and rsh panes, 60 fps,
4-second measurements, and `lan,wan,highlat,lossy` in that order, with
`compress,bandwidth` on each. Each scene has its usual countdown and an
unmeasured first second. Network changes and the preceding scene matter:
starting each scene on a fresh connection does not reliably reproduce it.
Investigation runs use a separate copy of the lab with different container,
network and tmux names, preserving the scene and measurement code.

Two instrumented baseline runs established the failure:

| Run | RTT 300 ms dashboard, rsh | OpenSSH | Server observation |
| --- | ---: | ---: | --- |
| `demo-20261029-012327` | 56.0 fps | 60.0 fps | Four exits immediately followed by lag-triggered full repaints during the measurement |
| `demo-20261029-013129` | 48.25 fps | 60.0 fps | Unnecessary screen mode throughout the measurement, paced by a low rate estimate |

In the first run, each repaint was triggered by 5,656–5,868 pending bytes
at an estimated 55,589 wire bytes/s. The measured standing queue was only
0.13–1.40 ms. In the second run, screen mode started with 998 stream bytes
in flight and an incoming 387 bytes, at 23,980 wire bytes/s and a 0.48 ms
standing queue. The recorded mode produced 193 differences in the four
measured seconds, matching the 48.25 fps seen by the client.

`Sender::pending()` includes sent bytes awaiting their ACK. Both rate-based
entry checks treated these bytes as waiting for serialization on the link:

- Proactive entry: `(in_flight + incoming) / rate > 50 ms`.
- Lag entry: `(pending + gathered + incoming) / rate > 100 ms`.

A long path normally has bytes in flight for a full round trip. A low
packet-train rate sample therefore started screen mode even while the
path kept up. Its rate-based pacing then skipped frames; leaving the mode
could immediately trigger another full repaint. Requiring two rate samples
in `gate2` did not solve this: the 48.25 fps case had three samples, and the
separate lag entry did not use that gate.

The lag formulas predate the CPU optimization branch. That branch changed
the deflater, level policy, screen differences and output processing costs.
Changes in packet sizes and timing can expose this latent error. The traces
establish the bad entry and resulting frame loss; they do not identify a
single optimization as the sole cause of the misleading rate samples.

## Correction

For a path with arrival reports, a bandwidth-delay product (BDP) of data
can be in flight without queueing. Exclude that normal flight from the two
rate-based entry checks. Charge datagram overhead against the BDP first:

```text
normal_flight = min(in_flight, max(BDP - in_flight_overhead, 0))
backlog = max(pending_with_new_output - normal_flight, 0)
```

Unsent bytes and flight beyond the BDP still count. The existing window,
standing-queue and memory bounds retain their original quantities. Clients
without usable path reports, including TCP, retain their ACK-based behavior.
The path rate itself remains available to transport and screen pacing.

## Verification

The behavioral regression test calls `Session::output()` with normal
flight, with a single rate sample to exercise the later lag entry, with an
unsent backlog, and with flight exceeding a real slow link's BDP. The
normal-flight case fails on the checkpoint and passes with the correction;
actual backlogs still start screen updates.

`NO_COLOR= cargo test --release` passes: 92 unit tests, 4 binary CLI tests,
25 session tests; 2 deliberately ignored tests. The empty `NO_COLOR` value
keeps the existing badge-color assertion enabled in this shell environment.
The regression case also failed on the Linux checkpoint and passed with
the fix, independently of the native run.

Two checkpoint demos with server tracing were followed by three corrected
diagnostic demos interleaved with two checkpoint demos using the same
server/client tracing. All use the full scene sequence above; tracing is
absent from the committed source.

| Run | Build | RTT 100 ms dashboard | RTT 300 ms dashboard |
| --- | --- | ---: | ---: |
| `012203` | checkpoint, server trace | 60.00 | 60.25 |
| `012327` | checkpoint, server trace | 60.00 | 56.00 |
| `013003` | corrected | 60.00 | 60.25 |
| `013129` | checkpoint | 60.25 | 48.25 |
| `013257` | corrected | 60.25 | 60.00 |
| `013424` | checkpoint | 60.00 | 60.00 |
| `013552` | corrected | 60.00 | 59.75 |

Values are fps. The full IDs are `demo-20261029-<time>`. The corrected
runs have no repeated reentry/repaint cycles in the measured RTT 100/300 ms
dashboard windows. In `013552`, a mode entered 0.6 seconds before the RTT
300 ms measurement exited 0.4 seconds into it; that run still reached
59.75 fps. A path change can start screen mode during the countdown, when
the previous delay floor makes a real rise look like a queue; that behavior
is preserved.

The first corrected demo also contained a LAN slowdown in both rsh
(58 fps) and OpenSSH (57 fps); the rsh trace shows seven frames batched
into one write. This observation is retained, rather than presenting all
runs as a stable machine. The RTT 100/300 ms controls remained at 60 fps.

The final, uninstrumented build is also compared with `final3` (`cpu`
`c9901b8`) on the low-rate links and on path changes, six iterations each:

```sh
python3 lab.py run --rsh ../profile/out/builds/final3 \
  --rsh-b ../profile/out/builds/flight --tools rsh,rsh-b \
  --tests render,switch \
  --nets lowbw,verylow,slow64k,mobile,badmobile,mid5 \
  --switches lan:lowbw,wan:verylow,lan:highlat \
  --iterations 6 --duration 8 --fps 60 --parallel 4
```

The final build is `f73e643831d3`, compared with `final3`
`adcdc66e964b`, both Linux aarch64, protocol/handoff 3/3. The run is
`run-20261029-014539`: 216 jobs, no job errors. These are medians of six
runs; CPU is percent of one core.

| Dashboard link | `final3` fps | Corrected fps | Server CPU, before → after |
| --- | ---: | ---: | ---: |
| 1 Mbit/s, RTT 100 ms | 60.00 | 60.00 | 0.85 → 0.84 |
| 256 kbit/s, RTT 100 ms | 52.70 | 60.00 | 3.15 → 1.14 |
| 64 kbit/s, RTT 100 ms | 13.75 | 13.55 | 1.99 → 2.11 |
| 512 kbit/s, RTT 200 ms, 1% loss | 46.60 | 50.65 | 3.23 → 1.15 |
| 256 kbit/s, RTT 400 ms, 2% loss | 26.45 | 26.25 | 2.84 → 2.72 |
| 5 Mbit/s, RTT 100 ms | 60.00 | 60.00 | 0.85 → 0.84 |

Across the six path-change scene pairs, median FPS after the change differs
by at most 0.4 fps. LAN→1 Mbit/s dashboard adaptation is 0.5→1 s by the
lab's whole-second measure; its subsequent FPS is 58.45→58.85.

The initial run's random screen at 512 kbit/s/1% loss was 9.9→9.3 fps;
the 64 kbit/s dashboard also had occasional long gaps in both builds.
Each was repeated 12 times, swapping the builds between `rsh` and `rsh-b`
to check assignment and loss variability. The repeat results are:

| Workload | `final3` fps | Corrected fps | Pooled 1% low, before → after |
| --- | ---: | ---: | ---: |
| 512 kbit/s/1% loss, random (`014822`) | 9.95 | 10.00 | 2.72 → 2.74 |
| 64 kbit/s, dashboard (`014924`) | 15.60 | 15.50 | 11.10 → 10.95 |

The initial FPS difference did not repeat. At 64 kbit/s the corrected
12 runs span 15.1–15.9 fps, versus 14.6–16.0 for `final3`. These finite
runs do not establish identical behavior under every loss sequence.

A final **uninstrumented** demo (`demo-20261029-014651`) uses the same
158×43 viewer and LAN→RTT 100 ms→RTT 300 ms scene sequence. Its dashboard
rates are 59.75, 60.25 and 60.00 fps respectively; random screens are
59.75, 60.00 and 59.75 fps.

## Other observations and measurement limits

- The first 216-job run shows higher server RSS on the two dashboard links
  that now mostly use the byte stream: 5,788→7,618 KiB at 256 kbit/s and
  5,894→7,380 KiB at 512 kbit/s/1% loss. No memory-invariance claim is made.
- The original collector reports more torn screen observations in some
  random-screen cases. Its `Client.feed` checks the same screen again for
  an empty trailing piece after a sync-end marker, and does not retain a
  sync marker split between reads. Thus its `torn` counter is not a count
  of distinct visible events. Stored `torn_at` values are also capped at
  eight; they cannot establish that all occurrences are at startup.
- A separate six-iteration comparison (`015406`) uses a temporary collector
  that retains split sync markers and skips empty trailing pieces. At
  1 Mbit/s the torn states are 17/1,154 before and 24/1,158 after; at
  512 kbit/s/1% loss, 26/461 before and 32/529 after. Both builds expose
  partial screens; several corrected observations are in the first
  0.3 seconds, with some later observations too. The large difference in
  the original counts is not sufficient evidence of a new product defect.
  This temporary measurement adjustment is not part of the product fix.
- The byte-stream client waits up to 10 ms for remaining data, whereas
  screen updates may wait 300 ms. Spending longer in the byte-stream mode
  can affect partial-screen exposure. This mechanism is consistent with
  the observations, but a frame-by-frame real-terminal comparison was not
  performed; no assertion of tear-free rendering is made.

Raw JSON for these run IDs is retained locally under `benchmark/lab/out/`
(ignored, as the other lab measurements are); diagnostic server logs,
frame lists and raw client output are under
`benchmark/profile/out/investigation/`.
