# Benchmark report

OpenSSH, Eternal Terminal (ET), mosh and rsh over UDP/TCP are compared on
isolated Docker server/client links. All tables and charts use the
[committed measurement snapshots](docs/bench/data/v0.4.0).

## Measurement scope

Static tests use **four parallel jobs and two evaluations per case**.
Ten network profiles and three full-screen workloads (`compress`, `bandwidth`,
and `agent`) request 60 FPS in 100×30 terminals. The 700 jobs comprise
render 300, link-switch 150, echo 100, bulk-output 100,
connectivity 40 and ten-session memory 10 observations.
Every attempted job remains in the data, including failures and missing values.

Moving observations are separate: two evaluations per scene/cohort, each 60 s
with two shared 30-second cycles. Four sessions run concurrently. One cohort
compares SSH/ET/mosh/rsh UDP, the other SSH/ET/mosh/rsh TCP; their shared
reference tools are not pooled across cohorts. A 403×32 virtual viewer is
drained continuously, with actual 100×30 workload PTYs checked separately.

**1% low, 5% low and average FPS are primary; network efficiency and CPU/RSS are secondary.**
No aggregate score or universal advantage follows from one metric.

### Environment and workloads

The shared host has an Apple M5 with four performance cores, six efficiency
cores and 32 GiB RAM. Four jobs do not establish physical core affinity. The
[environment manifest](docs/bench/data/v0.4.0/environment.json) records the
host/VM, package versions, executable identities and actual test commands.
Other local services remain running; host scheduling can vary.

Each job has its own shaped link:
- `compress`: a compressible full-screen dashboard table with changing counters and bars.
- `bandwidth`: an incompressible full-screen stress test changing every cell with random characters and colors.
- `agent`: a Claude Code / Ink-style coding agent TUI cycling through Thinking spinner updates, token-by-token 24-bit RGB code and prose streaming, and scrolling diff review above a bordered prompt box and status footer.

SSH/ET frame headers in static tests are observed in the raw-output reader;
mosh/rsh headers are observed after pyte reconstructs terminal differences.
Header arrival can precede remaining cells, and emulator scheduling contributes
to these measurements. These are terminal-observer results, not GPU presentation
or a complete-body oracle. No historical dataset supplies values to the tables below.

### Metrics and aggregation

- **Paced FPS & burst exclusion:** Buffered frames flushed in a post-stall burst
  do not count as real-time frame delivery. A single-frame-credit leaky bucket
  at the target cadence (`60 FPS`) absorbs normal OS/PTY scheduling jitter while
  discarding buffered catch-up bursts, and caps all reported FPS and percentile
  lows at the requested `60.0 FPS` ceiling.
- **Avg FPS** is the arithmetic mean of each run's paced displayed frames divided
  by its observation duration. **Median FPS** is the median of those run
  averages. The opening table shows both; FPS charts use the median.
- **1% / 5% low** are `min(scene_fps, 1000 / p99(gap_ms))` and
  `min(scene_fps, 1000 / p95(gap_ms))` using nearest-rank percentiles over
  available runs' paced inter-frame gaps pooled together. Startup/end boundary
  gaps are excluded from lows.
- **Age** is the median of each run's median generation-to-observation
  time; it includes processing and queueing and is not RTT. Static windows
  begin at the first observed frame; late backlog is not drained into FPS.
- **Network** is received interface traffic at the client, in decimal
  kbit/s, including protocol overhead. KiB/frame is computed per run against
  paced displayed frames and then medianed. Packet counts include inbound and
  outbound interface traffic.
- **RSS** is a halfway-through-render sample, in MiB (KiB / 1024), medianed
  across runs; not peak RSS or PSS. Client and server are separate. Server
  process selection includes sshd's listener, ET's shared daemon and
  terminal, mosh-server, or rsh's shared serve process; shells are excluded.
- Other scalar values are medians of available results. Successful/total
  counts, timeouts and collection failures are preserved below. A missing
  observation is never a zero or proof of successful recovery.
- CPU is percent of one core over a render window, not total machine load.

Metric implementations: [agent.py](benchmark/lab/agent.py),
[pooled lows and summaries](benchmark/lab/lab.py),
[original charts](benchmark/lab/charts.py), and
[report generator](benchmark/lab/report.py).

### Static network profiles

`tc netem` applies delay and independent random loss in both directions;
rate limits below apply server → client. RTT is the nominal delay total,
before queueing. No bandwidth cap means no configured rate limit.


| Profile | Conditions |
| --- | --- |
| lan | LAN |
| wan | RTT 100 ms |
| mid5 | 5 Mbit/s · RTT 100 ms |
| lowbw | 1 Mbit/s · RTT 100 ms |
| verylow | 256 kbit/s · RTT 100 ms |
| slow64k | 64 kbit/s · RTT 100 ms |
| mobile | 512 kbit/s · RTT 200 ms · 1% loss |
| badmobile | 256 kbit/s · RTT 400 ms · 2% loss |
| highlat | RTT 300 ms |
| lossy | 2% loss · RTT 100 ms |

## 1. Dashboard rendering

Full-strength color shows median FPS; 5% and 1% lows use progressively softer shades.
All five transports and all ten static profiles are included; two runs per case were attempted.


![Dashboard FPS, 1% low and 5% low by link and transport](docs/bench/v0.4.0/fps-dashboard.svg)

| Link | Tool | Completed | Avg FPS | Median FPS | 1% low | 5% low | Age ms | RX kbit/s | Client RSS MiB | Server RSS MiB |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| LAN | rsh (UDP) | 2/2 | 60.0 | 60.0 | 51.3 | 53.8 | 1 | 167 | 3.03 | 5.73 |
| LAN | rsh (TCP) | 2/2 | 60.0 | 60.0 | 50.8 | 53.5 | 1 | 179 | 3.11 | 5.96 |
| LAN | OpenSSH | 2/2 | 60.0 | 60.0 | 51.3 | 53.2 | 1 | 2478 | 7.36 | 18.42 |
| LAN | ET | 2/2 | 60.0 | 60.0 | 51.3 | 54.1 | 1 | 2445 | 18.08 | 15.37 |
| LAN | mosh | 2/2 | 46.3 | 46.3 | 37.3 | 40.7 | 10 | 236 | 15.34 | 10.48 |
| RTT 100 ms | rsh (UDP) | 2/2 | 59.8 | 59.8 | 46.7 | 50.0 | 52 | 167 | 3.06 | 5.92 |
| RTT 100 ms | rsh (TCP) | 2/2 | 59.9 | 59.9 | 47.6 | 50.5 | 52 | 178 | 3.12 | 5.73 |
| RTT 100 ms | OpenSSH | 2/2 | 59.8 | 59.8 | 46.1 | 50.3 | 52 | 2510 | 7.38 | 18.38 |
| RTT 100 ms | ET | 2/2 | 59.0 | 59.0 | 45.9 | 50.3 | 52 | 2466 | 17.65 | 15.79 |
| RTT 100 ms | mosh | 2/2 | 18.7 | 18.7 | 17.4 | 17.8 | 60 | 102 | 14.35 | 10.45 |
| 5 Mbit/s · RTT 100 ms | rsh (UDP) | 2/2 | 59.9 | 59.9 | 46.7 | 49.8 | 53 | 168 | 3.05 | 5.96 |
| 5 Mbit/s · RTT 100 ms | rsh (TCP) | 2/2 | 59.9 | 59.9 | 46.7 | 49.0 | 53 | 178 | 3.12 | 5.89 |
| 5 Mbit/s · RTT 100 ms | OpenSSH | 2/2 | 59.7 | 59.7 | 46.1 | 51.0 | 59 | 2510 | 7.38 | 18.38 |
| 5 Mbit/s · RTT 100 ms | ET | 2/2 | 59.0 | 59.0 | 45.5 | 50.3 | 60 | 2468 | 14.74 | 14.99 |
| 5 Mbit/s · RTT 100 ms | mosh | 2/2 | 18.6 | 18.6 | 17.2 | 17.8 | 62 | 102 | 14.47 | 10.49 |
| 1 Mbit/s · RTT 100 ms | rsh (UDP) | 2/2 | 59.6 | 59.6 | 45.9 | 50.8 | 56 | 169 | 3.06 | 5.89 |
| 1 Mbit/s · RTT 100 ms | rsh (TCP) | 2/2 | 57.8 | 57.8 | 25.2 | 50.0 | 56 | 174 | 3.12 | 5.47 |
| 1 Mbit/s · RTT 100 ms | OpenSSH | 2/2 | 17.0 | 17.0 | 6.4 | 6.9 | 1825 | 977 | 7.49 | 19.22 |
| 1 Mbit/s · RTT 100 ms | ET | 2/2 | 19.4 | 19.4 | 6.9 | 7.5 | 2027 | 976 | 17.09 | 15.17 |
| 1 Mbit/s · RTT 100 ms | mosh | 2/2 | 17.9 | 17.9 | 16.6 | 17.2 | 66 | 98 | 14.45 | 10.48 |
| 256 kbit/s · RTT 100 ms | rsh (UDP) | 2/2 | 59.5 | 59.5 | 43.7 | 49.5 | 64 | 170 | 3.04 | 5.99 |
| 256 kbit/s · RTT 100 ms | rsh (TCP) | 2/2 | 59.7 | 59.7 | 45.0 | 48.3 | 65 | 184 | 3.10 | 5.95 |
| 256 kbit/s · RTT 100 ms | OpenSSH | 2/2 | 6.0 | 6.0 | 4.9 | 5.1 | 3808 | 250 | 7.37 | 19.32 |
| 256 kbit/s · RTT 100 ms | ET | 2/2 | 6.2 | 6.2 | 4.2 | 4.2 | 3818 | 250 | 18.07 | 13.93 |
| 256 kbit/s · RTT 100 ms | mosh | 2/2 | 15.6 | 15.6 | 14.4 | 14.8 | 83 | 90 | 14.35 | 10.45 |
| 64 kbit/s · RTT 100 ms | rsh (UDP) | 2/2 | 15.7 | 15.7 | 10.8 | 12.2 | 118 | 59 | 3.05 | 5.99 |
| 64 kbit/s · RTT 100 ms | rsh (TCP) | 2/2 | 15.3 | 15.3 | 4.5 | 11.3 | 255 | 63 | 3.12 | 5.44 |
| 64 kbit/s · RTT 100 ms | OpenSSH | 2/2 | 1.4 | 1.4 | 1.0 | 1.0 | 4269 | 61 | 7.37 | 19.35 |
| 64 kbit/s · RTT 100 ms | ET | 2/2 | 1.6 | 1.6 | 0.9 | 0.9 | 4652 | 62 | 16.54 | 14.56 |
| 64 kbit/s · RTT 100 ms | mosh | 2/2 | 10.1 | 10.1 | 7.1 | 7.9 | 157 | 60 | 14.41 | 10.37 |
| 512 kbit/s · RTT 200 ms · 1% loss | rsh (UDP) | 2/2 | 57.5 | 57.5 | 44.1 | 50.5 | 109 | 308 | 3.12 | 6.50 |
| 512 kbit/s · RTT 200 ms · 1% loss | rsh (TCP) | 2/2 | 32.6 | 32.6 | 3.6 | 7.9 | 112 | 155 | 3.16 | 5.52 |
| 512 kbit/s · RTT 200 ms · 1% loss | OpenSSH | 2/2 | 4.7 | 4.7 | 1.0 | 1.7 | 2576 | 368 | 7.41 | 19.53 |
| 512 kbit/s · RTT 200 ms · 1% loss | ET | 2/2 | 9.4 | 9.4 | 1.1 | 4.0 | 2900 | 500 | 17.71 | 14.42 |
| 512 kbit/s · RTT 200 ms · 1% loss | mosh | 2/2 | 9.1 | 9.1 | 4.7 | 8.9 | 121 | 48 | 14.37 | 10.47 |
| 256 kbit/s · RTT 400 ms · 2% loss | rsh (UDP) | 2/2 | 24.5 | 24.5 | 1.2 | 15.6 | 220 | 159 | 3.29 | 6.21 |
| 256 kbit/s · RTT 400 ms · 2% loss | rsh (TCP) | 2/2 | 19.4 | 19.4 | 1.8 | 3.9 | 228 | 138 | 3.20 | 7.01 |
| 256 kbit/s · RTT 400 ms · 2% loss | OpenSSH | 2/2 | 2.3 | 2.3 | 0.4 | 0.4 | 4348 | 237 | 7.51 | 19.21 |
| 256 kbit/s · RTT 400 ms · 2% loss | ET | 2/2 | 3.1 | 3.1 | 0.3 | 1.6 | 2896 | 242 | 17.23 | 13.94 |
| 256 kbit/s · RTT 400 ms · 2% loss | mosh | 2/2 | 4.1 | 4.1 | 1.2 | 4.6 | 234 | 23 | 14.37 | 10.54 |
| RTT 300 ms | rsh (UDP) | 2/2 | 59.7 | 59.7 | 44.4 | 48.1 | 153 | 167 | 3.05 | 5.74 |
| RTT 300 ms | rsh (TCP) | 2/2 | 58.5 | 58.5 | 41.2 | 49.5 | 152 | 177 | 3.12 | 5.84 |
| RTT 300 ms | OpenSSH | 2/2 | 58.0 | 58.0 | 24.1 | 46.7 | 152 | 2507 | 7.45 | 18.48 |
| RTT 300 ms | ET | 2/2 | 50.1 | 50.1 | 3.8 | 49.0 | 152 | 2461 | 14.08 | 15.71 |
| RTT 300 ms | mosh | 2/2 | 6.5 | 6.5 | 6.4 | 6.4 | 160 | 36 | 14.29 | 10.36 |
| 2% loss · RTT 100 ms | rsh (UDP) | 2/2 | 59.2 | 59.2 | 45.5 | 49.0 | 52 | 322 | 3.09 | 5.52 |
| 2% loss · RTT 100 ms | rsh (TCP) | 2/2 | 45.8 | 45.8 | 6.8 | 19.7 | 54 | 170 | 3.15 | 6.52 |
| 2% loss · RTT 100 ms | OpenSSH | 2/2 | 14.8 | 14.8 | 4.8 | 5.0 | 1640 | 1159 | 7.53 | 19.57 |
| 2% loss · RTT 100 ms | ET | 2/2 | 17.1 | 17.1 | 3.4 | 5.2 | 764 | 1359 | 16.81 | 14.36 |
| 2% loss · RTT 100 ms | mosh | 2/2 | 17.6 | 17.6 | 3.7 | 17.5 | 61 | 102 | 14.51 | 10.74 |

## 2. Random full-screen rendering

Every cell changes each frame. This workload compresses less effectively
and exposes a different bandwidth and frame-gap tradeoff than the dashboard.


![Random-screen FPS, 1% low and 5% low by link and transport](docs/bench/v0.4.0/fps-random.svg)

| Link | Tool | Completed | Avg FPS | Median FPS | 1% low | 5% low | Age ms | RX kbit/s | Client RSS MiB | Server RSS MiB |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| LAN | rsh (UDP) | 2/2 | 59.8 | 59.8 | 46.5 | 49.8 | 3 | 2670 | 3.07 | 6.12 |
| LAN | rsh (TCP) | 2/2 | 59.7 | 59.7 | 46.1 | 50.3 | 3 | 2722 | 3.17 | 6.09 |
| LAN | OpenSSH | 2/2 | 60.0 | 60.0 | 51.0 | 53.2 | 2 | 5790 | 7.38 | 18.37 |
| LAN | ET | 2/2 | 60.0 | 60.0 | 51.3 | 53.5 | 2 | 5720 | 17.56 | 16.90 |
| LAN | mosh | 2/2 | 46.5 | 46.5 | 37.9 | 41.5 | 12 | 2019 | 15.47 | 10.47 |
| RTT 100 ms | rsh (UDP) | 2/2 | 59.1 | 59.1 | 39.2 | 46.5 | 54 | 2678 | 3.08 | 5.84 |
| RTT 100 ms | rsh (TCP) | 2/2 | 57.3 | 57.3 | 25.7 | 46.3 | 54 | 2686 | 3.18 | 6.04 |
| RTT 100 ms | OpenSSH | 2/2 | 59.0 | 59.0 | 35.6 | 49.0 | 52 | 5797 | 7.48 | 18.49 |
| RTT 100 ms | ET | 2/2 | 57.0 | 57.0 | 13.1 | 49.5 | 52 | 5754 | 16.24 | 16.52 |
| RTT 100 ms | mosh | 2/2 | 18.9 | 18.9 | 17.4 | 18.0 | 62 | 816 | 14.43 | 10.53 |
| 5 Mbit/s · RTT 100 ms | rsh (UDP) | 2/2 | 58.9 | 58.9 | 30.2 | 47.4 | 64 | 2679 | 3.08 | 5.99 |
| 5 Mbit/s · RTT 100 ms | rsh (TCP) | 2/2 | 57.7 | 57.7 | 28.2 | 46.9 | 64 | 2714 | 3.16 | 5.61 |
| 5 Mbit/s · RTT 100 ms | OpenSSH | 2/2 | 49.6 | 49.6 | 29.2 | 40.7 | 778 | 4850 | 7.40 | 18.46 |
| 5 Mbit/s · RTT 100 ms | ET | 2/2 | 45.3 | 45.3 | 23.5 | 28.5 | 822 | 4804 | 17.71 | 17.11 |
| 5 Mbit/s · RTT 100 ms | mosh | 2/2 | 17.6 | 17.6 | 16.3 | 16.6 | 70 | 764 | 14.45 | 10.45 |
| 1 Mbit/s · RTT 100 ms | rsh (UDP) | 2/2 | 23.9 | 23.9 | 19.5 | 20.8 | 101 | 932 | 3.14 | 5.59 |
| 1 Mbit/s · RTT 100 ms | rsh (TCP) | 2/2 | 24.1 | 24.1 | 17.2 | 19.4 | 126 | 963 | 3.14 | 5.71 |
| 1 Mbit/s · RTT 100 ms | OpenSSH | 2/2 | 8.2 | 8.2 | 5.1 | 6.3 | 3037 | 979 | 7.41 | 19.57 |
| 1 Mbit/s · RTT 100 ms | ET | 2/2 | 8.9 | 8.9 | 4.4 | 5.5 | 3096 | 978 | 15.42 | 15.61 |
| 1 Mbit/s · RTT 100 ms | mosh | 2/2 | 15.4 | 15.4 | 12.3 | 13.2 | 108 | 667 | 14.54 | 10.46 |
| 256 kbit/s · RTT 100 ms | rsh (UDP) | 2/2 | 6.2 | 6.2 | 5.5 | 5.6 | 220 | 244 | 3.06 | 6.00 |
| 256 kbit/s · RTT 100 ms | rsh (TCP) | 2/2 | 5.8 | 5.8 | 3.0 | 5.4 | 388 | 256 | 3.14 | 6.90 |
| 256 kbit/s · RTT 100 ms | OpenSSH | 2/2 | 2.5 | 2.5 | 1.5 | 1.5 | 3876 | 249 | 7.39 | 19.43 |
| 256 kbit/s · RTT 100 ms | ET | 2/2 | 2.3 | 2.3 | 1.5 | 1.5 | 3806 | 249 | 18.11 | 13.87 |
| 256 kbit/s · RTT 100 ms | mosh | 2/2 | 5.6 | 5.6 | 3.9 | 4.0 | 448 | 240 | 15.04 | 10.91 |
| 64 kbit/s · RTT 100 ms | rsh (UDP) | 2/2 | 1.4 | 1.4 | 0.9 | 1.0 | 570 | 64 | 3.02 | 5.53 |
| 64 kbit/s · RTT 100 ms | rsh (TCP) | 2/2 | 1.4 | 1.4 | 0.8 | 1.0 | 1690 | 64 | 3.11 | 6.34 |
| 64 kbit/s · RTT 100 ms | OpenSSH | 2/2 | 0.6 | 0.6 | — | — | 3966 | 62 | 7.38 | 19.44 |
| 64 kbit/s · RTT 100 ms | ET | 2/2 | 0.6 | 0.6 | — | — | 4793 | 62 | 17.66 | 14.60 |
| 64 kbit/s · RTT 100 ms | mosh | 2/2 | 1.5 | 1.5 | 1.5 | 1.5 | 3992 | 64 | 14.82 | 13.34 |
| 512 kbit/s · RTT 200 ms · 1% loss | rsh (UDP) | 2/2 | 8.8 | 8.8 | 1.9 | 3.1 | 214 | 446 | 3.41 | 6.10 |
| 512 kbit/s · RTT 200 ms · 1% loss | rsh (TCP) | 2/2 | 8.0 | 8.0 | 2.7 | 3.1 | 208 | 420 | 3.32 | 5.58 |
| 512 kbit/s · RTT 200 ms · 1% loss | OpenSSH | 2/2 | 4.0 | 4.0 | 0.9 | 1.9 | 4088 | 500 | 7.48 | 19.61 |
| 512 kbit/s · RTT 200 ms · 1% loss | ET | 2/2 | 4.2 | 4.2 | 0.5 | 3.3 | 3378 | 500 | 17.70 | 14.60 |
| 512 kbit/s · RTT 200 ms · 1% loss | mosh | 2/2 | 7.3 | 7.3 | 2.6 | 7.6 | 198 | 340 | 14.43 | 10.71 |
| 256 kbit/s · RTT 400 ms · 2% loss | rsh (UDP) | 2/2 | 3.8 | 3.8 | 0.9 | 1.4 | 391 | 204 | 3.27 | 7.10 |
| 256 kbit/s · RTT 400 ms · 2% loss | rsh (TCP) | 2/2 | 3.5 | 3.5 | 1.0 | 1.6 | 1146 | 218 | 3.32 | 5.74 |
| 256 kbit/s · RTT 400 ms · 2% loss | OpenSSH | 2/2 | 1.9 | 1.9 | 0.4 | 1.2 | 4546 | 244 | 7.53 | 19.49 |
| 256 kbit/s · RTT 400 ms · 2% loss | ET | 2/2 | 1.7 | 1.7 | 0.7 | 0.9 | 3936 | 218 | 17.30 | 13.73 |
| 256 kbit/s · RTT 400 ms · 2% loss | mosh | 2/2 | 4.0 | 4.0 | 2.2 | 2.3 | 386 | 187 | 14.45 | 10.33 |
| RTT 300 ms | rsh (UDP) | 2/2 | 59.2 | 59.2 | 33.6 | 46.5 | 154 | 2716 | 3.09 | 6.11 |
| RTT 300 ms | rsh (TCP) | 2/2 | 51.4 | 51.4 | 15.0 | 42.0 | 155 | 2508 | 3.25 | 6.17 |
| RTT 300 ms | OpenSSH | 2/2 | 55.0 | 55.0 | 12.4 | 43.7 | 152 | 5792 | 7.83 | 18.65 |
| RTT 300 ms | ET | 2/2 | 47.2 | 47.2 | 3.7 | 47.4 | 152 | 5744 | 16.07 | 16.84 |
| RTT 300 ms | mosh | 2/2 | 6.6 | 6.6 | 6.2 | 6.4 | 162 | 284 | 14.38 | 10.26 |
| 2% loss · RTT 100 ms | rsh (UDP) | 2/2 | 56.8 | 56.8 | 29.3 | 44.6 | 54 | 5232 | 3.27 | 6.69 |
| 2% loss · RTT 100 ms | rsh (TCP) | 2/2 | 8.3 | 8.3 | 2.6 | 3.1 | 116 | 796 | 3.33 | 6.78 |
| 2% loss · RTT 100 ms | OpenSSH | 2/2 | 13.3 | 13.3 | 4.6 | 5.0 | 1723 | 1824 | 7.60 | 19.53 |
| 2% loss · RTT 100 ms | ET | 2/2 | 6.9 | 6.9 | 2.4 | 3.3 | 1538 | 922 | 18.11 | 14.69 |
| 2% loss · RTT 100 ms | mosh | 2/2 | 15.2 | 15.2 | 3.1 | 9.2 | 63 | 796 | 14.48 | 10.99 |

## 3. Coding agent TUI rendering & network efficiency

The `agent` profile models an Ink-based coding assistant (Claude Code style):
synchronized redraws of the active response block, 24-bit RGB syntax-highlighted
code, scrolling diff hunks, and a dynamic spinner/prompt/status footer. Because
rsh hashes unchanged rows, scrolls moved transcript regions in place, and jumps
across unchanged columns (`CHA`) to patch only modified cells before `zstream`
compression, wire traffic drops by an order of magnitude compared with byte-stream
transports while sustaining full frame rate.


![Coding agent TUI FPS, 1% low and 5% low by link and transport](docs/bench/v0.4.0/fps-agent.svg)

| Link | Tool | Completed | Avg FPS | Median FPS | 1% low | 5% low | Age ms | RX kbit/s | Client RSS MiB | Server RSS MiB |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| LAN | rsh (UDP) | 2/2 | 60.0 | 60.0 | 51.8 | 53.8 | 1 | 83 | 2.98 | 6.73 |
| LAN | rsh (TCP) | 2/2 | 60.0 | 60.0 | 51.0 | 53.2 | 1 | 94 | 3.05 | 5.88 |
| LAN | OpenSSH | 2/2 | 60.0 | 60.0 | 50.8 | 53.5 | 1 | 1864 | 7.38 | 18.39 |
| LAN | ET | 2/2 | 60.0 | 60.0 | 50.3 | 52.6 | 1 | 1842 | 16.18 | 14.88 |
| LAN | mosh | 2/2 | 45.8 | 45.8 | 37.3 | 39.4 | 10 | 133 | 15.21 | 10.41 |
| RTT 100 ms | rsh (UDP) | 2/2 | 59.9 | 59.9 | 45.9 | 49.5 | 52 | 83 | 3.00 | 5.95 |
| RTT 100 ms | rsh (TCP) | 2/2 | 59.5 | 59.5 | 46.1 | 49.5 | 52 | 93 | 3.07 | 5.87 |
| RTT 100 ms | OpenSSH | 2/2 | 59.6 | 59.6 | 43.7 | 50.0 | 51 | 1886 | 7.36 | 18.43 |
| RTT 100 ms | ET | 2/2 | 59.2 | 59.2 | 43.7 | 51.0 | 51 | 1860 | 17.61 | 14.69 |
| RTT 100 ms | mosh | 2/2 | 18.7 | 18.7 | 17.1 | 17.5 | 60 | 57 | 14.75 | 10.55 |
| 5 Mbit/s · RTT 100 ms | rsh (UDP) | 2/2 | 59.9 | 59.9 | 46.3 | 49.8 | 52 | 83 | 2.99 | 5.49 |
| 5 Mbit/s · RTT 100 ms | rsh (TCP) | 2/2 | 59.9 | 59.9 | 46.7 | 49.8 | 52 | 93 | 3.07 | 5.68 |
| 5 Mbit/s · RTT 100 ms | OpenSSH | 2/2 | 59.9 | 59.9 | 47.2 | 50.3 | 58 | 1886 | 7.38 | 18.39 |
| 5 Mbit/s · RTT 100 ms | ET | 2/2 | 59.5 | 59.5 | 46.3 | 51.0 | 58 | 1860 | 18.06 | 15.28 |
| 5 Mbit/s · RTT 100 ms | mosh | 2/2 | 18.9 | 18.9 | 17.5 | 17.9 | 61 | 57 | 14.76 | 10.41 |
| 1 Mbit/s · RTT 100 ms | rsh (UDP) | 2/2 | 59.9 | 59.9 | 47.2 | 50.3 | 53 | 84 | 2.99 | 5.67 |
| 1 Mbit/s · RTT 100 ms | rsh (TCP) | 2/2 | 59.8 | 59.8 | 46.7 | 50.3 | 54 | 93 | 3.07 | 5.95 |
| 1 Mbit/s · RTT 100 ms | OpenSSH | 2/2 | 27.9 | 27.9 | 6.8 | 16.0 | 1098 | 978 | 7.39 | 18.67 |
| 1 Mbit/s · RTT 100 ms | ET | 2/2 | 31.8 | 31.8 | 16.3 | 20.0 | 1226 | 978 | 17.13 | 14.49 |
| 1 Mbit/s · RTT 100 ms | mosh | 2/2 | 18.4 | 18.4 | 16.9 | 17.3 | 64 | 54 | 14.76 | 10.47 |
| 256 kbit/s · RTT 100 ms | rsh (UDP) | 2/2 | 59.0 | 59.0 | 34.6 | 50.0 | 58 | 84 | 3.00 | 6.84 |
| 256 kbit/s · RTT 100 ms | rsh (TCP) | 2/2 | 59.3 | 59.3 | 43.3 | 51.0 | 58 | 94 | 3.07 | 6.86 |
| 256 kbit/s · RTT 100 ms | OpenSSH | 2/2 | 9.7 | 9.7 | 5.2 | 5.3 | 3103 | 248 | 7.49 | 19.17 |
| 256 kbit/s · RTT 100 ms | ET | 2/2 | 9.9 | 9.9 | 5.3 | 6.9 | 3122 | 250 | 18.04 | 14.14 |
| 256 kbit/s · RTT 100 ms | mosh | 2/2 | 17.1 | 17.1 | 14.0 | 14.7 | 72 | 48 | 14.75 | 10.42 |
| 64 kbit/s · RTT 100 ms | rsh (UDP) | 2/2 | 43.0 | 43.0 | 14.6 | 35.1 | 116 | 62 | 2.99 | 6.92 |
| 64 kbit/s · RTT 100 ms | rsh (TCP) | 2/2 | 36.5 | 36.5 | 13.2 | 27.0 | 160 | 63 | 3.05 | 6.24 |
| 64 kbit/s · RTT 100 ms | OpenSSH | 2/2 | 2.5 | 2.5 | 1.3 | 1.3 | 3796 | 62 | 7.38 | 19.32 |
| 64 kbit/s · RTT 100 ms | ET | 2/2 | 2.8 | 2.8 | 1.8 | 2.6 | 4116 | 60 | 17.60 | 13.68 |
| 64 kbit/s · RTT 100 ms | mosh | 2/2 | 13.1 | 13.1 | 7.9 | 9.6 | 100 | 40 | 14.79 | 10.40 |
| 512 kbit/s · RTT 200 ms · 1% loss | rsh (UDP) | 2/2 | 58.5 | 58.5 | 40.3 | 50.3 | 106 | 146 | 3.01 | 5.54 |
| 512 kbit/s · RTT 200 ms · 1% loss | rsh (TCP) | 2/2 | 46.9 | 46.9 | 8.3 | 29.8 | 106 | 88 | 3.07 | 6.03 |
| 512 kbit/s · RTT 200 ms · 1% loss | OpenSSH | 2/2 | 11.8 | 11.8 | 0.6 | 10.7 | 3220 | 500 | 7.62 | 18.92 |
| 512 kbit/s · RTT 200 ms · 1% loss | ET | 2/2 | 15.7 | 15.7 | 3.5 | 10.6 | 2807 | 498 | 17.33 | 14.67 |
| 512 kbit/s · RTT 200 ms · 1% loss | mosh | 2/2 | 9.2 | 9.2 | 8.7 | 8.9 | 117 | 29 | 14.78 | 10.40 |
| 256 kbit/s · RTT 400 ms · 2% loss | rsh (UDP) | 2/2 | 51.5 | 51.5 | 24.3 | 38.6 | 208 | 154 | 3.01 | 6.02 |
| 256 kbit/s · RTT 400 ms · 2% loss | rsh (TCP) | 2/2 | 22.2 | 22.2 | 1.9 | 5.2 | 217 | 81 | 3.12 | 5.55 |
| 256 kbit/s · RTT 400 ms · 2% loss | OpenSSH | 2/2 | 3.4 | 3.4 | 0.4 | 0.5 | 4484 | 244 | 7.51 | 19.20 |
| 256 kbit/s · RTT 400 ms · 2% loss | ET | 2/2 | 5.8 | 5.8 | 0.6 | 1.0 | 3231 | 248 | 16.65 | 14.19 |
| 256 kbit/s · RTT 400 ms · 2% loss | mosh | 2/2 | 4.5 | 4.5 | 1.2 | 4.5 | 224 | 25 | 14.77 | 10.38 |
| RTT 300 ms | rsh (UDP) | 2/2 | 59.8 | 59.8 | 45.0 | 48.5 | 152 | 83 | 2.99 | 6.01 |
| RTT 300 ms | rsh (TCP) | 2/2 | 58.5 | 58.5 | 41.3 | 48.8 | 152 | 93 | 3.08 | 5.47 |
| RTT 300 ms | OpenSSH | 2/2 | 59.5 | 59.5 | 42.6 | 49.5 | 152 | 1885 | 7.36 | 18.38 |
| RTT 300 ms | ET | 2/2 | 55.2 | 55.2 | 32.2 | 48.8 | 152 | 1860 | 17.31 | 14.87 |
| RTT 300 ms | mosh | 2/2 | 6.5 | 6.5 | 6.4 | 6.4 | 161 | 22 | 14.77 | 10.33 |
| 2% loss · RTT 100 ms | rsh (UDP) | 2/2 | 58.9 | 58.9 | 42.7 | 48.5 | 52 | 161 | 2.98 | 6.66 |
| 2% loss · RTT 100 ms | rsh (TCP) | 2/2 | 39.7 | 39.7 | 6.5 | 14.8 | 54 | 78 | 3.09 | 6.53 |
| 2% loss · RTT 100 ms | OpenSSH | 2/2 | 16.2 | 16.2 | 3.5 | 5.0 | 750 | 1128 | 7.70 | 19.34 |
| 2% loss · RTT 100 ms | ET | 2/2 | 24.0 | 24.0 | 4.9 | 7.5 | 106 | 1289 | 18.14 | 14.62 |
| 2% loss · RTT 100 ms | mosh | 2/2 | 18.4 | 18.4 | 9.3 | 17.5 | 60 | 56 | 14.59 | 10.47 |

### Coding agent network efficiency vs OpenSSH, ET and mosh

For `RX vs SSH` and `RX vs ET`, ratios are `other / rsh` using unrounded median
interface bitrates (note that on bandwidth-capped links where SSH/ET saturate
the bottleneck at lower FPS, per-frame efficiency `KiB/frame` reflects the true
per-update payload ratio). Compared with mosh, `KiB/frame` compares wire bytes
spent per displayed update because mosh caps its frame cadence on high-RTT links.


![Coding agent TUI received KiB per displayed frame](docs/bench/v0.4.0/network-agent.svg)

![Coding agent TUI inbound and outbound packets per displayed frame](docs/bench/v0.4.0/packets-agent.svg)

| Link | rsh RX kbit/s | SSH RX kbit/s | ET RX kbit/s | mosh RX kbit/s | RX vs SSH | RX vs ET | rsh KiB/frame | KiB/frame vs SSH / ET / mosh |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| LAN | **83** | 1864 | 1842 | 133 | **22.46×** | **22.19×** | **0.17** | 22.48× / 22.21× / 2.10× |
| RTT 100 ms | **83** | 1886 | 1860 | 57 | **22.72×** | **22.40×** | **0.17** | 22.81× / 22.65× / 2.20× |
| 5 Mbit/s · RTT 100 ms | **83** | 1886 | 1860 | 57 | **22.72×** | **22.41×** | **0.17** | 22.72× / 22.56× / 2.18× |
| 1 Mbit/s · RTT 100 ms | **84** | 978 | 978 | 54 | **11.65×** | **11.64×** | **0.17** | 24.97× / 21.97× / 2.11× |
| 256 kbit/s · RTT 100 ms | **84** | 248 | 250 | 48 | **2.94×** | **2.96×** | **0.17** | 17.90× / 17.65× / 1.96× |
| 64 kbit/s · RTT 100 ms | **62** | 62 | 60 | 40 | **0.98×** | **0.97×** | **0.18** | 16.58× / 15.16× / 2.07× |
| 512 kbit/s · RTT 200 ms · 1% loss | **146** | 500 | 498 | 29 | **3.43×** | **3.41×** | **0.30** | 17.10× / 12.74× / 1.26× |
| 256 kbit/s · RTT 400 ms · 2% loss | **154** | 244 | 248 | 25 | **1.59×** | **1.62×** | **0.36** | 24.49× / 14.38× / 1.86× |
| RTT 300 ms | **83** | 1885 | 1860 | 22 | **22.71×** | **22.41×** | **0.17** | 22.81× / 24.26× / 2.44× |
| 2% loss · RTT 100 ms | **161** | 1128 | 1289 | 56 | **7.01×** | **8.01×** | **0.33** | 25.89× / 19.65× / 1.12× |

## 4. Frame age

![Dashboard median frame age by link and transport](docs/bench/v0.4.0/age.svg)

The rendering tables above include age for all scenes. The chart uses a log scale.

## 5. Network traffic and CPU

![Dashboard received KiB per displayed frame](docs/bench/v0.4.0/network.svg)

![Dashboard inbound and outbound packets per displayed frame](docs/bench/v0.4.0/packets.svg)

The render tables give RX kbit/s. Below: median per-run bytes/frame, packets/frame and CPU for each scene.

### Dashboard

| Link | Tool | RX KiB/frame | Packets in/frame | Packets out/frame | Client CPU % | Server CPU % |
| --- | --- | --- | --- | --- | --- | --- |
| lan | rsh (UDP) | 0.34 | 1.00 | 1.00 | 1.67 | 2.21 |
| lan | rsh (TCP) | 0.36 | 1.02 | 1.01 | 1.25 | 1.69 |
| lan | OpenSSH | 5.04 | 2.14 | 1.25 | 0.34 | 0.47 |
| lan | ET | 4.97 | 1.52 | 1.04 | 0.57 | 0.82 |
| lan | mosh | 0.62 | 1.02 | 0.21 | 1.44 | 3.25 |
| wan | rsh (UDP) | 0.34 | 1.00 | 1.00 | 2.48 | 3.31 |
| wan | rsh (TCP) | 0.36 | 1.00 | 1.00 | 1.46 | 1.83 |
| wan | OpenSSH | 5.12 | 3.15 | 1.78 | 0.32 | 0.36 |
| wan | ET | 5.10 | 2.49 | 1.99 | 0.47 | 0.69 |
| wan | mosh | 0.67 | 1.00 | 0.50 | 0.71 | 2.71 |
| mid5 | rsh (UDP) | 0.34 | 1.01 | 1.00 | 2.59 | 3.44 |
| mid5 | rsh (TCP) | 0.36 | 1.00 | 1.00 | 2.17 | 2.85 |
| mid5 | OpenSSH | 5.13 | 3.12 | 2.09 | 0.48 | 0.35 |
| mid5 | ET | 5.10 | 2.53 | 1.96 | 0.54 | 0.61 |
| mid5 | mosh | 0.67 | 1.01 | 0.49 | 0.80 | 3.04 |
| lowbw | rsh (UDP) | 0.35 | 1.02 | 1.01 | 3.21 | 4.17 |
| lowbw | rsh (TCP) | 0.37 | 1.02 | 1.01 | 3.29 | 3.84 |
| lowbw | OpenSSH | 7.02 | 2.68 | 2.85 | 0.28 | 0.80 |
| lowbw | ET | 6.14 | 2.26 | 2.00 | 0.41 | 0.82 |
| lowbw | mosh | 0.67 | 0.99 | 0.50 | 0.94 | 3.54 |
| verylow | rsh (UDP) | 0.35 | 1.03 | 1.01 | 3.80 | 4.86 |
| verylow | rsh (TCP) | 0.38 | 1.19 | 1.01 | 3.42 | 4.47 |
| verylow | OpenSSH | 5.03 | 2.20 | 2.27 | 0.11 | 0.67 |
| verylow | ET | 4.97 | 1.99 | 1.79 | 0.35 | 0.75 |
| verylow | mosh | 0.70 | 1.13 | 0.56 | 1.02 | 4.62 |
| slow64k | rsh (UDP) | 0.46 | 1.07 | 1.03 | 1.11 | 2.29 |
| slow64k | rsh (TCP) | 0.50 | 1.51 | 1.02 | 1.16 | 2.19 |
| slow64k | OpenSSH | 5.14 | 3.33 | 3.16 | 0.07 | 0.72 |
| slow64k | ET | 4.85 | 2.17 | 2.27 | 0.43 | 0.85 |
| slow64k | mosh | 0.72 | 1.14 | 0.63 | 0.65 | 3.65 |
| mobile | rsh (UDP) | 0.65 | 1.92 | 1.01 | 3.65 | 4.69 |
| mobile | rsh (TCP) | 0.58 | 1.40 | 1.21 | 1.98 | 3.58 |
| mobile | OpenSSH | 9.58 | 4.90 | 5.35 | 0.14 | 0.83 |
| mobile | ET | 6.53 | 2.69 | 2.54 | 0.39 | 0.73 |
| mobile | mosh | 0.65 | 1.00 | 0.99 | 0.56 | 3.50 |
| badmobile | rsh (UDP) | 0.82 | 2.07 | 1.34 | 2.15 | 4.24 |
| badmobile | rsh (TCP) | 0.90 | 2.22 | 1.75 | 1.66 | 3.90 |
| badmobile | OpenSSH | 12.67 | 5.78 | 5.56 | 0.08 | 0.61 |
| badmobile | ET | 9.51 | 3.81 | 3.68 | 0.46 | 0.91 |
| badmobile | mosh | 0.68 | 1.02 | 1.02 | 0.33 | 3.22 |
| highlat | rsh (UDP) | 0.34 | 1.01 | 0.99 | 3.12 | 4.14 |
| highlat | rsh (TCP) | 0.37 | 1.01 | 1.00 | 2.08 | 2.82 |
| highlat | OpenSSH | 5.28 | 3.14 | 2.17 | 0.34 | 0.38 |
| highlat | ET | 6.00 | 2.76 | 2.10 | 0.45 | 0.67 |
| highlat | mosh | 0.68 | 1.00 | 1.00 | 0.36 | 2.51 |
| lossy | rsh (UDP) | 0.66 | 1.96 | 0.99 | 2.84 | 3.87 |
| lossy | rsh (TCP) | 0.46 | 1.27 | 1.18 | 2.12 | 3.15 |
| lossy | OpenSSH | 9.98 | 4.56 | 2.70 | 0.17 | 1.08 |
| lossy | ET | 10.03 | 4.01 | 2.24 | 0.37 | 0.65 |
| lossy | mosh | 0.71 | 1.10 | 0.54 | 0.89 | 3.52 |

### Random screen

| Link | Tool | RX KiB/frame | Packets in/frame | Packets out/frame | Client CPU % | Server CPU % |
| --- | --- | --- | --- | --- | --- | --- |
| lan | rsh (UDP) | 5.45 | 4.89 | 1.03 | 2.43 | 3.04 |
| lan | rsh (TCP) | 5.57 | 4.92 | 2.09 | 2.52 | 3.05 |
| lan | OpenSSH | 11.78 | 3.17 | 1.83 | 0.26 | 0.34 |
| lan | ET | 11.64 | 1.90 | 1.86 | 0.48 | 0.61 |
| lan | mosh | 5.30 | 4.99 | 0.20 | 2.74 | 4.45 |
| wan | rsh (UDP) | 5.53 | 4.98 | 1.06 | 2.56 | 2.96 |
| wan | rsh (TCP) | 5.72 | 5.09 | 2.12 | 2.62 | 3.02 |
| wan | OpenSSH | 11.99 | 3.36 | 1.85 | 0.27 | 0.29 |
| wan | ET | 12.32 | 3.21 | 2.12 | 0.40 | 0.53 |
| wan | mosh | 5.27 | 4.97 | 0.51 | 1.23 | 3.51 |
| mid5 | rsh (UDP) | 5.56 | 5.01 | 2.12 | 2.75 | 3.05 |
| mid5 | rsh (TCP) | 5.74 | 5.94 | 4.09 | 2.92 | 3.14 |
| mid5 | OpenSSH | 11.94 | 4.57 | 4.10 | 0.52 | 0.35 |
| mid5 | ET | 12.95 | 4.86 | 4.13 | 0.48 | 0.53 |
| mid5 | mosh | 5.28 | 4.97 | 0.50 | 1.17 | 3.33 |
| lowbw | rsh (UDP) | 4.75 | 4.10 | 4.00 | 1.86 | 3.35 |
| lowbw | rsh (TCP) | 4.88 | 5.68 | 4.01 | 2.17 | 3.92 |
| lowbw | OpenSSH | 14.57 | 5.46 | 5.84 | 0.17 | 0.46 |
| lowbw | ET | 13.34 | 4.93 | 4.26 | 0.27 | 0.59 |
| lowbw | mosh | 5.27 | 4.97 | 0.49 | 2.09 | 4.83 |
| verylow | rsh (UDP) | 4.80 | 5.03 | 5.06 | 0.84 | 2.38 |
| verylow | rsh (TCP) | 5.38 | 8.41 | 7.25 | 1.07 | 2.80 |
| verylow | OpenSSH | 12.16 | 5.32 | 5.58 | 0.11 | 0.40 |
| verylow | ET | 13.24 | 5.20 | 4.80 | 0.30 | 0.62 |
| verylow | mosh | 5.22 | 4.91 | 0.99 | 0.89 | 5.84 |
| slow64k | rsh (UDP) | 5.58 | 13.21 | 12.57 | 0.52 | 2.19 |
| slow64k | rsh (TCP) | 5.35 | 7.17 | 5.39 | 0.31 | 2.00 |
| slow64k | OpenSSH | 12.61 | 7.00 | 8.67 | 0.09 | 0.30 |
| slow64k | ET | 11.70 | 5.23 | 5.30 | 0.38 | 0.31 |
| slow64k | mosh | 5.21 | 4.93 | 1.00 | 0.24 | 5.99 |
| mobile | rsh (UDP) | 6.30 | 5.47 | 4.27 | 1.16 | 2.74 |
| mobile | rsh (TCP) | 6.40 | 6.24 | 4.97 | 1.01 | 2.57 |
| mobile | OpenSSH | 15.25 | 7.50 | 7.94 | 0.14 | 0.40 |
| mobile | ET | 14.72 | 5.67 | 5.07 | 0.31 | 0.71 |
| mobile | mosh | 5.66 | 5.32 | 0.97 | 1.18 | 6.15 |
| badmobile | rsh (UDP) | 6.54 | 6.02 | 4.90 | 0.61 | 2.31 |
| badmobile | rsh (TCP) | 7.71 | 5.77 | 5.39 | 0.57 | 2.19 |
| badmobile | OpenSSH | 15.79 | 6.32 | 6.24 | 0.09 | 0.33 |
| badmobile | ET | 16.11 | 6.27 | 5.76 | 0.34 | 0.71 |
| badmobile | mosh | 5.64 | 5.31 | 1.00 | 0.73 | 5.73 |
| highlat | rsh (UDP) | 5.60 | 5.06 | 1.03 | 2.73 | 3.04 |
| highlat | rsh (TCP) | 5.96 | 5.30 | 2.21 | 2.60 | 3.00 |
| highlat | OpenSSH | 12.87 | 3.63 | 2.14 | 0.30 | 0.33 |
| highlat | ET | 14.88 | 3.60 | 2.59 | 0.40 | 0.59 |
| highlat | mosh | 5.25 | 5.06 | 1.12 | 0.50 | 3.19 |
| lossy | rsh (UDP) | 11.26 | 10.15 | 1.08 | 2.83 | 3.38 |
| lossy | rsh (TCP) | 11.62 | 7.18 | 5.75 | 1.01 | 2.73 |
| lossy | OpenSSH | 16.32 | 6.64 | 3.57 | 0.16 | 0.53 |
| lossy | ET | 16.27 | 6.54 | 3.60 | 0.23 | 0.46 |
| lossy | mosh | 6.43 | 6.07 | 0.50 | 1.27 | 4.31 |

### Coding agent TUI

| Link | Tool | RX KiB/frame | Packets in/frame | Packets out/frame | Client CPU % | Server CPU % |
| --- | --- | --- | --- | --- | --- | --- |
| lan | rsh (UDP) | 0.17 | 1.00 | 1.00 | 1.46 | 2.27 |
| lan | rsh (TCP) | 0.19 | 1.02 | 1.01 | 1.45 | 2.17 |
| lan | OpenSSH | 3.80 | 1.60 | 1.27 | 0.18 | 0.23 |
| lan | ET | 3.75 | 1.23 | 1.02 | 0.48 | 0.62 |
| lan | mosh | 0.35 | 1.00 | 0.20 | 0.86 | 2.79 |
| wan | rsh (UDP) | 0.17 | 1.00 | 1.00 | 1.77 | 2.54 |
| wan | rsh (TCP) | 0.19 | 1.01 | 1.01 | 2.06 | 2.92 |
| wan | OpenSSH | 3.86 | 2.38 | 1.50 | 0.20 | 0.23 |
| wan | ET | 3.83 | 2.00 | 1.67 | 0.32 | 0.42 |
| wan | mosh | 0.37 | 1.01 | 0.49 | 0.53 | 2.81 |
| mid5 | rsh (UDP) | 0.17 | 1.00 | 1.00 | 1.44 | 2.23 |
| mid5 | rsh (TCP) | 0.19 | 1.00 | 1.00 | 1.40 | 2.12 |
| mid5 | OpenSSH | 3.84 | 2.38 | 1.61 | 0.31 | 0.27 |
| mid5 | ET | 3.82 | 2.03 | 1.54 | 0.44 | 0.49 |
| mid5 | mosh | 0.37 | 0.99 | 0.49 | 0.36 | 1.95 |
| lowbw | rsh (UDP) | 0.17 | 1.01 | 1.00 | 1.40 | 2.04 |
| lowbw | rsh (TCP) | 0.19 | 1.00 | 1.00 | 1.28 | 1.97 |
| lowbw | OpenSSH | 4.27 | 1.91 | 1.76 | 0.21 | 0.27 |
| lowbw | ET | 3.76 | 1.49 | 1.23 | 0.34 | 0.55 |
| lowbw | mosh | 0.36 | 0.99 | 0.50 | 0.40 | 2.28 |
| verylow | rsh (UDP) | 0.17 | 1.04 | 1.02 | 1.96 | 2.73 |
| verylow | rsh (TCP) | 0.19 | 1.03 | 1.01 | 1.81 | 2.58 |
| verylow | OpenSSH | 3.13 | 1.51 | 1.67 | 0.14 | 0.63 |
| verylow | ET | 3.08 | 1.18 | 1.07 | 0.34 | 0.75 |
| verylow | mosh | 0.34 | 1.00 | 0.49 | 0.48 | 2.92 |
| slow64k | rsh (UDP) | 0.18 | 1.03 | 1.01 | 1.72 | 2.68 |
| slow64k | rsh (TCP) | 0.21 | 1.16 | 1.00 | 1.40 | 2.25 |
| slow64k | OpenSSH | 2.94 | 2.20 | 2.43 | 0.08 | 0.72 |
| slow64k | ET | 2.69 | 1.06 | 1.09 | 0.42 | 0.82 |
| slow64k | mosh | 0.37 | 1.00 | 0.51 | 0.43 | 3.12 |
| mobile | rsh (UDP) | 0.30 | 1.80 | 1.00 | 2.09 | 3.32 |
| mobile | rsh (TCP) | 0.23 | 1.24 | 1.13 | 1.42 | 2.31 |
| mobile | OpenSSH | 5.20 | 2.72 | 2.80 | 0.15 | 0.64 |
| mobile | ET | 3.88 | 1.65 | 1.58 | 0.38 | 0.65 |
| mobile | mosh | 0.38 | 1.10 | 0.94 | 0.30 | 3.24 |
| badmobile | rsh (UDP) | 0.36 | 2.12 | 1.02 | 2.75 | 3.13 |
| badmobile | rsh (TCP) | 0.47 | 2.11 | 1.86 | 0.97 | 2.77 |
| badmobile | OpenSSH | 8.92 | 4.60 | 4.66 | 0.10 | 0.79 |
| badmobile | ET | 5.24 | 2.34 | 2.15 | 0.39 | 0.81 |
| badmobile | mosh | 0.68 | 1.83 | 1.20 | 0.16 | 2.74 |
| highlat | rsh (UDP) | 0.17 | 1.00 | 0.99 | 1.51 | 2.23 |
| highlat | rsh (TCP) | 0.19 | 1.01 | 1.00 | 1.47 | 2.25 |
| highlat | OpenSSH | 3.87 | 2.38 | 1.42 | 0.26 | 0.28 |
| highlat | ET | 4.11 | 2.12 | 1.59 | 0.43 | 0.59 |
| highlat | mosh | 0.41 | 1.15 | 1.15 | 0.20 | 2.35 |
| lossy | rsh (UDP) | 0.33 | 1.97 | 1.00 | 1.69 | 2.60 |
| lossy | rsh (TCP) | 0.24 | 1.33 | 1.13 | 1.13 | 2.21 |
| lossy | OpenSSH | 8.64 | 4.09 | 2.87 | 0.14 | 0.83 |
| lossy | ET | 6.56 | 2.77 | 1.75 | 0.36 | 0.57 |
| lossy | mosh | 0.37 | 1.00 | 0.49 | 0.50 | 2.80 |

## 6. Resident memory

![Median client and server RSS while rendering](docs/bench/v0.4.0/memory.svg)

Per-link RSS for each scene is in sections 1–3. The chart pools the render samples across links and scenes.

### Ten simultaneous sessions

![Combined client and server RSS for ten idle sessions](docs/bench/v0.4.0/memory-sessions.svg)

| Tool | Runs | All clients MiB | Server MiB | Idle packets in/s | Idle packets out/s |
| --- | --- | --- | --- | --- | --- |
| rsh (UDP) | 2 | 25.89 | 10.33 | 2.00 | 2.00 |
| rsh (TCP) | 2 | 30.61 | 10.95 | 2.00 | 4.00 |
| OpenSSH | 2 | 73.75 | 148.22 | 0.00 | 0.00 |
| ET | 2 | 171.46 | 36.09 | 6.80 | 7.35 |
| mosh | 2 | 144.47 | 98.06 | 3.30 | 3.55 |

Each shell prints 3,000 lines, then becomes quiet. Packet rates are totals for the container's ten sessions. Both rsh transports have ten-session observations.

## 7. Connection and key echo

![Connection time in echo jobs](docs/bench/v0.4.0/connect.svg)

![Key echo latency](docs/bench/v0.4.0/echo.svg)

| Link | Tool | Echo runs | Connect s | Echo ms | Echo max ms | Packets in/key | Packets out/key |
| --- | --- | --- | --- | --- | --- | --- | --- |
| lan | rsh (UDP) | 2/2 | 0.007 | 6.8 | 15.6 | 1.1 | 2.2 |
| lan | rsh (TCP) | 2/2 | 1.516 | 6.8 | 13.9 | 3.3 | 2.2 |
| lan | OpenSSH | 2/2 | 0.146 | 14.8 | 25.9 | 15.6 | 15.6 |
| lan | ET | 2/2 | 0.157 | 6.7 | 9.0 | 2.0 | 3.0 |
| lan | mosh | 2/2 | 0.242 | 16.6 | 23.5 | 2.5 | 2.2 |
| wan | rsh (UDP) | 2/2 | 0.327 | 6.2 | 108.2 | 1.1 | 2.5 |
| wan | rsh (TCP) | 2/2 | 2.034 | 6.8 | 110.3 | 3.5 | 2.4 |
| wan | OpenSSH | 2/2 | 1.408 | 115.8 | 127.1 | 21.1 | 21.3 |
| wan | ET | 2/2 | 1.929 | 106.2 | 110.3 | 1.2 | 2.3 |
| wan | mosh | 2/2 | 1.623 | 7.8 | 119.8 | 2.3 | 2.8 |
| mid5 | rsh (UDP) | 2/2 | 0.325 | 6.4 | 107.8 | 1.1 | 2.4 |
| mid5 | rsh (TCP) | 2/2 | 2.024 | 6.2 | 107.7 | 3.3 | 2.4 |
| mid5 | OpenSSH | 2/2 | 1.433 | 115.0 | 125.6 | 21.0 | 22.3 |
| mid5 | ET | 2/2 | 1.930 | 108.5 | 115.5 | 1.1 | 2.3 |
| mid5 | mosh | 2/2 | 1.671 | 7.5 | 122.0 | 2.3 | 2.8 |
| lowbw | rsh (UDP) | 2/2 | 0.333 | 6.9 | 109.9 | 1.3 | 2.4 |
| lowbw | rsh (TCP) | 2/2 | 2.045 | 6.5 | 107.4 | 3.3 | 2.4 |
| lowbw | OpenSSH | 2/2 | 1.472 | 117.7 | 129.3 | 20.1 | 20.5 |
| lowbw | ET | 2/2 | 1.949 | 108.6 | 111.1 | 1.2 | 2.3 |
| lowbw | mosh | 2/2 | 1.668 | 7.8 | 121.8 | 2.3 | 2.8 |
| verylow | rsh (UDP) | 2/2 | 0.358 | 6.6 | 111.3 | 1.6 | 2.5 |
| verylow | rsh (TCP) | 2/2 | 2.072 | 5.5 | 114.8 | 3.9 | 2.9 |
| verylow | OpenSSH | 2/2 | 1.583 | 119.8 | 128.8 | 20.8 | 22.4 |
| verylow | ET | 2/2 | 2.110 | 109.2 | 114.1 | 1.6 | 2.5 |
| verylow | mosh | 2/2 | 1.803 | 7.6 | 135.9 | 2.3 | 2.5 |
| slow64k | rsh (UDP) | 2/2 | 0.529 | 7.2 | 128.4 | 1.8 | 2.8 |
| slow64k | rsh (TCP) | 2/2 | 2.210 | 6.4 | 140.2 | 3.5 | 2.7 |
| slow64k | OpenSSH | 2/2 | 2.159 | 132.2 | 143.1 | 21.9 | 21.9 |
| slow64k | ET | 2/2 | 2.714 | 121.0 | 122.7 | 1.2 | 2.5 |
| slow64k | mosh | 2/2 | 2.403 | 7.7 | 152.5 | 2.3 | 2.1 |
| mobile | rsh (UDP) | 2/2 | 0.638 | 7.0 | 210.6 | 1.9 | 2.4 |
| mobile | rsh (TCP) | 2/2 | 2.548 | 6.8 | 332.4 | 3.6 | 2.6 |
| mobile | OpenSSH | 2/2 | 2.699 | 237.6 | 401.6 | 25.6 | 29.6 |
| mobile | ET | 2/2 | 4.071 | 210.3 | 423.0 | 1.6 | 2.5 |
| mobile | mosh | 2/2 | 2.948 | 7.5 | 297.6 | 2.1 | 2.5 |
| badmobile | rsh (UDP) | 2/2 | 1.459 | 6.6 | 413.9 | 1.8 | 2.2 |
| badmobile | rsh (TCP) | 2/2 | 3.575 | 5.8 | 666.6 | 3.9 | 2.7 |
| badmobile | OpenSSH | 2/2 | 5.195 | 509.9 | 842.3 | 23.9 | 25.8 |
| badmobile | ET | 2/2 | 8.421 | 411.1 | 861.6 | 1.1 | 2.2 |
| badmobile | mosh | 2/2 | 5.437 | 6.8 | 589.4 | 1.4 | 1.8 |
| highlat | rsh (UDP) | 2/2 | 0.923 | 7.3 | 309.0 | 1.2 | 2.5 |
| highlat | rsh (TCP) | 2/2 | 3.038 | 7.7 | 308.9 | 3.1 | 3.0 |
| highlat | OpenSSH | 2/2 | 3.808 | 315.4 | 328.7 | 29.9 | 30.6 |
| highlat | ET | 2/2 | 5.340 | 308.6 | 313.9 | 1.1 | 2.2 |
| highlat | mosh | 2/2 | 4.080 | 7.5 | 400.1 | 1.9 | 2.3 |
| lossy | rsh (UDP) | 2/2 | 0.420 | 6.5 | 107.8 | 2.0 | 2.3 |
| lossy | rsh (TCP) | 2/2 | 2.546 | 5.4 | 109.9 | 3.7 | 3.2 |
| lossy | OpenSSH | 2/2 | 1.720 | 143.9 | 268.2 | 21.0 | 23.6 |
| lossy | ET | 2/2 | 2.095 | 107.8 | 112.1 | 1.2 | 2.3 |
| lossy | mosh | 2/2 | 1.774 | 6.9 | 125.0 | 2.2 | 2.5 |

Ten keys per run. Echo includes predictive local output for mosh/rsh. Connection times come from these same echo jobs, including their startup path; they are not a new dedicated cold-install test.

## 8. Bulk output

![Time until the prompt returns after seq 1 100000](docs/bench/v0.4.0/flood.svg)

| Link | Tool | Completed | Completion s | 30 s timeouts | Errors | Terminal bytes |
| --- | --- | --- | --- | --- | --- | --- |
| lan | rsh (UDP) | 2/2 | 0.04 | 0 | 0 | 1620 |
| lan | rsh (TCP) | 2/2 | 0.04 | 0 | 0 | 1774 |
| lan | OpenSSH | 2/2 | 0.04 | 0 | 0 | 688960 |
| lan | ET | 2/2 | 0.07 | 0 | 0 | 688960 |
| lan | mosh | 2/2 | 0.06 | 0 | 0 | 659 |
| wan | rsh (UDP) | 2/2 | 0.15 | 0 | 0 | 1987 |
| wan | rsh (TCP) | 2/2 | 0.15 | 0 | 0 | 1969 |
| wan | OpenSSH | 2/2 | 0.61 | 0 | 0 | 688960 |
| wan | ET | 2/2 | 0.68 | 0 | 0 | 688960 |
| wan | mosh | 2/2 | 0.22 | 0 | 0 | 456 |
| mid5 | rsh (UDP) | 2/2 | 0.15 | 0 | 0 | 1957 |
| mid5 | rsh (TCP) | 2/2 | 0.15 | 0 | 0 | 1972 |
| mid5 | OpenSSH | 2/2 | 1.36 | 0 | 0 | 688960 |
| mid5 | ET | 2/2 | 1.42 | 0 | 0 | 688960 |
| mid5 | mosh | 2/2 | 0.20 | 0 | 0 | 542 |
| lowbw | rsh (UDP) | 2/2 | 0.15 | 0 | 0 | 2182 |
| lowbw | rsh (TCP) | 2/2 | 0.15 | 0 | 0 | 2100 |
| lowbw | OpenSSH | 2/2 | 6.06 | 0 | 0 | 688960 |
| lowbw | ET | 2/2 | 5.97 | 0 | 0 | 688960 |
| lowbw | mosh | 2/2 | 0.21 | 0 | 0 | 522 |
| verylow | rsh (UDP) | 2/2 | 0.15 | 0 | 0 | 1968 |
| verylow | rsh (TCP) | 2/2 | 0.17 | 0 | 0 | 2026 |
| verylow | OpenSSH | 2/2 | 23.25 | 0 | 0 | 688960 |
| verylow | ET | 2/2 | 23.14 | 0 | 0 | 688960 |
| verylow | mosh | 2/2 | 0.23 | 0 | 0 | 454 |
| slow64k | rsh (UDP) | 2/2 | 0.24 | 0 | 0 | 2180 |
| slow64k | rsh (TCP) | 2/2 | 0.30 | 0 | 0 | 2426 |
| slow64k | OpenSSH | 0/2 | — | 2 | 0 | 199100 |
| slow64k | ET | 0/2 | — | 2 | 0 | 216466 |
| slow64k | mosh | 2/2 | 0.26 | 0 | 0 | 444 |
| mobile | rsh (UDP) | 2/2 | 0.26 | 0 | 0 | 2185 |
| mobile | rsh (TCP) | 2/2 | 0.33 | 0 | 0 | 1224 |
| mobile | OpenSSH | 2/2 | 11.77 | 0 | 0 | 688960 |
| mobile | ET | 2/2 | 11.77 | 0 | 0 | 688960 |
| mobile | mosh | 2/2 | 0.35 | 0 | 0 | 532 |
| badmobile | rsh (UDP) | 2/2 | 0.46 | 0 | 0 | 2005 |
| badmobile | rsh (TCP) | 2/2 | 0.46 | 0 | 0 | 1969 |
| badmobile | OpenSSH | 2/2 | 23.84 | 0 | 0 | 688960 |
| badmobile | ET | 2/2 | 24.09 | 0 | 0 | 688960 |
| badmobile | mosh | 2/2 | 1.00 | 0 | 0 | 522 |
| highlat | rsh (UDP) | 2/2 | 0.35 | 0 | 0 | 1790 |
| highlat | rsh (TCP) | 2/2 | 0.34 | 0 | 0 | 1562 |
| highlat | OpenSSH | 2/2 | 1.48 | 0 | 0 | 688960 |
| highlat | ET | 2/2 | 2.58 | 0 | 0 | 688960 |
| highlat | mosh | 2/2 | 0.51 | 0 | 0 | 496 |
| lossy | rsh (UDP) | 2/2 | 0.15 | 0 | 0 | 1982 |
| lossy | rsh (TCP) | 2/2 | 0.14 | 0 | 0 | 1750 |
| lossy | OpenSSH | 2/2 | 2.98 | 0 | 0 | 688960 |
| lossy | ET | 2/2 | 3.49 | 0 | 0 | 688960 |
| lossy | mosh | 2/2 | 0.22 | 0 | 0 | 570 |

Completion is the marker after `seq 1 100000`, watched for 30 s. When a case is mixed, completion time covers only completed runs; the counts remain explicit. mosh and rsh may skip intermediate output, so prompt recovery does not imply delivery of the complete byte stream.

## 9. Link changes

Each scene runs two seconds on the first link and eight seconds after the change. FPS before/after is medianed; post-change lows pool only post-change inter-frame gaps. Adaptation is the first whole-second bucket reaching 90% of that tool's own settled FPS, not time to recover 60 FPS.

### Dashboard

![Dashboard FPS and lows after each link change](docs/bench/v0.4.0/switch-dashboard.svg)

| Change | Tool | Runs | FPS before | FPS after | 1% low after | 5% low after | Age after ms | Adapt s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| wan → verylow | rsh (UDP) | 2/2 | 58.9 | 57.0 | 37.2 | 46.9 | 64 | 1.0 |
| wan → verylow | rsh (TCP) | 2/2 | 59.0 | 57.2 | 30.0 | 48.1 | 65 | 1.0 |
| wan → verylow | OpenSSH | 2/2 | 58.4 | 4.0 | 1.7 | 1.8 | 3782 | 1.5 |
| wan → verylow | ET | 2/2 | 55.8 | 4.0 | 1.7 | 1.9 | 4136 | 2.0 |
| wan → verylow | mosh | 2/2 | 18.5 | 15.6 | 14.6 | 14.8 | 84 | 0.0 |
| lan → highlat | rsh (UDP) | 2/2 | 59.2 | 57.0 | 35.2 | 49.0 | 153 | 1.0 |
| lan → highlat | rsh (TCP) | 2/2 | 59.0 | 56.6 | 34.2 | 47.6 | 152 | 1.0 |
| lan → highlat | OpenSSH | 2/2 | 59.1 | 52.1 | 15.0 | 49.0 | 152 | 2.0 |
| lan → highlat | ET | 2/2 | 59.0 | 50.5 | 5.0 | 48.5 | 152 | 2.0 |
| lan → highlat | mosh | 2/2 | 45.8 | 10.2 | 6.5 | 6.5 | 162 | 0.0 |
| lan → lowbw | rsh (UDP) | 2/2 | 58.8 | 59.5 | 46.3 | 50.8 | 55 | 0.0 |
| lan → lowbw | rsh (TCP) | 2/2 | 58.9 | 59.8 | 49.0 | 51.5 | 56 | 0.0 |
| lan → lowbw | OpenSSH | 2/2 | 59.1 | 8.6 | 1.9 | 2.4 | 2006 | 0.0 |
| lan → lowbw | ET | 2/2 | 59.0 | 6.9 | 1.9 | 2.0 | 1997 | 0.0 |
| lan → lowbw | mosh | 2/2 | 46.0 | 21.5 | 17.1 | 17.5 | 65 | 0.0 |
| lowbw → highlat | rsh (UDP) | 2/2 | 58.1 | 55.8 | 42.7 | 48.3 | 153 | 1.0 |
| lowbw → highlat | rsh (TCP) | 2/2 | 58.7 | 43.2 | 8.7 | 26.9 | 152 | 3.0 |
| lowbw → highlat | OpenSSH | 2/2 | 23.2 | 8.8 | 3.3 | 3.4 | 3236 | 1.5 |
| lowbw → highlat | ET | 2/2 | 23.8 | 5.8 | 3.3 | 3.3 | 2264 | 2.5 |
| lowbw → highlat | mosh | 2/2 | 17.8 | 7.7 | 6.4 | 6.5 | 162 | 0.0 |
| highlat → lossy | rsh (UDP) | 2/2 | 56.0 | 56.4 | 33.8 | 48.8 | 53 | 1.0 |
| highlat → lossy | rsh (TCP) | 2/2 | 54.6 | 45.7 | 6.9 | 27.4 | 54 | 1.0 |
| highlat → lossy | OpenSSH | 2/2 | 51.2 | 27.4 | 6.5 | 9.6 | 125 | 0.0 |
| highlat → lossy | ET | 2/2 | 20.0 | 29.2 | 6.2 | 8.4 | 110 | 0.0 |
| highlat → lossy | mosh | 2/2 | 6.8 | 13.8 | 3.6 | 8.6 | 60 | 3.5 |

### Random screen

![Random screen FPS and lows after each link change](docs/bench/v0.4.0/switch-random.svg)

| Change | Tool | Runs | FPS before | FPS after | 1% low after | 5% low after | Age after ms | Adapt s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| wan → verylow | rsh (UDP) | 2/2 | 58.7 | 3.9 | 0.8 | 3.0 | 218 | 3.5 |
| wan → verylow | rsh (TCP) | 2/2 | 47.4 | 5.0 | 3.4 | 4.7 | 391 | 1.5 |
| wan → verylow | OpenSSH | 2/2 | 55.3 | 0.7 | — | — | 7258 | — |
| wan → verylow | ET | 2/2 | 49.5 | 0.6 | — | — | 7520 | — |
| wan → verylow | mosh | 2/2 | 18.6 | 5.3 | 3.9 | 4.0 | 814 | 0.5 |
| lan → highlat | rsh (UDP) | 2/2 | 57.4 | 56.1 | 30.6 | 45.7 | 154 | 1.0 |
| lan → highlat | rsh (TCP) | 2/2 | 59.2 | 47.0 | 4.0 | 43.9 | 154 | 2.0 |
| lan → highlat | OpenSSH | 2/2 | 59.2 | 50.1 | 4.4 | 46.9 | 152 | 2.5 |
| lan → highlat | ET | 2/2 | 59.2 | 50.3 | 4.6 | 48.1 | 152 | 2.0 |
| lan → highlat | mosh | 2/2 | 44.6 | 10.1 | 6.5 | 6.5 | 164 | 0.0 |
| lan → lowbw | rsh (UDP) | 2/2 | 59.2 | 23.6 | 18.9 | 20.8 | 100 | 1.0 |
| lan → lowbw | rsh (TCP) | 2/2 | 58.6 | 22.8 | 11.5 | 19.6 | 138 | 1.0 |
| lan → lowbw | OpenSSH | 2/2 | 59.5 | 6.8 | 3.2 | 3.7 | 3028 | 0.0 |
| lan → lowbw | ET | 2/2 | 59.1 | 4.5 | 1.6 | 1.9 | 2356 | 0.0 |
| lan → lowbw | mosh | 2/2 | 45.5 | 13.6 | 6.7 | 8.4 | 110 | 0.0 |
| lowbw → highlat | rsh (UDP) | 2/2 | 23.6 | 53.5 | 32.6 | 45.0 | 155 | 1.0 |
| lowbw → highlat | rsh (TCP) | 2/2 | 23.2 | 10.7 | 0.9 | 3.3 | 181 | 4.0 |
| lowbw → highlat | OpenSSH | 2/2 | 10.0 | 8.3 | 3.4 | 3.5 | 3774 | 1.0 |
| lowbw → highlat | ET | 2/2 | 10.1 | 6.0 | 3.3 | 3.3 | 2401 | 1.5 |
| lowbw → highlat | mosh | 2/2 | 16.6 | 7.5 | 6.5 | 6.5 | 160 | 0.0 |
| highlat → lossy | rsh (UDP) | 2/2 | 58.8 | 53.8 | 26.9 | 43.7 | 55 | 1.0 |
| highlat → lossy | rsh (TCP) | 2/2 | 27.8 | 16.0 | 3.1 | 4.5 | 62 | 1.5 |
| highlat → lossy | OpenSSH | 2/2 | 38.6 | 23.1 | 5.5 | 8.7 | 254 | 0.0 |
| highlat → lossy | ET | 2/2 | 8.3 | 14.7 | 4.9 | 6.5 | 1936 | 1.0 |
| highlat → lossy | mosh | 2/2 | 6.8 | 11.1 | 2.3 | 3.7 | 62 | 2.5 |

### Coding agent TUI

![Coding agent TUI FPS and lows after each link change](docs/bench/v0.4.0/switch-agent.svg)

| Change | Tool | Runs | FPS before | FPS after | 1% low after | 5% low after | Age after ms | Adapt s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| wan → verylow | rsh (UDP) | 2/2 | 58.9 | 56.8 | 29.4 | 47.8 | 58 | 1.0 |
| wan → verylow | rsh (TCP) | 2/2 | 58.6 | 56.8 | 25.8 | 48.3 | 58 | 1.0 |
| wan → verylow | OpenSSH | 2/2 | 58.9 | 5.9 | 4.2 | 5.2 | 3820 | 1.0 |
| wan → verylow | ET | 2/2 | 58.1 | 5.7 | 4.2 | 4.3 | 3924 | 1.0 |
| wan → verylow | mosh | 2/2 | 18.4 | 17.0 | 13.9 | 14.9 | 72 | 0.0 |
| lan → highlat | rsh (UDP) | 2/2 | 58.9 | 57.0 | 32.7 | 49.8 | 152 | 1.0 |
| lan → highlat | rsh (TCP) | 2/2 | 59.2 | 57.8 | 44.1 | 48.1 | 152 | 1.0 |
| lan → highlat | OpenSSH | 2/2 | 59.1 | 51.5 | 14.2 | 49.0 | 151 | 2.0 |
| lan → highlat | ET | 2/2 | 59.0 | 50.2 | 4.1 | 49.5 | 151 | 2.0 |
| lan → highlat | mosh | 2/2 | 45.1 | 10.1 | 6.4 | 6.5 | 160 | 0.0 |
| lan → lowbw | rsh (UDP) | 2/2 | 58.8 | 59.5 | 45.2 | 50.0 | 54 | 0.0 |
| lan → lowbw | rsh (TCP) | 2/2 | 58.4 | 59.5 | 46.9 | 50.0 | 54 | 0.0 |
| lan → lowbw | OpenSSH | 2/2 | 59.0 | 6.6 | 1.9 | 2.0 | 2034 | 0.0 |
| lan → lowbw | ET | 2/2 | 59.2 | 6.7 | 1.9 | 1.9 | 2023 | 0.0 |
| lan → lowbw | mosh | 2/2 | 44.8 | 21.9 | 17.2 | 17.6 | 64 | 0.0 |
| lowbw → highlat | rsh (UDP) | 2/2 | 59.0 | 56.9 | 42.7 | 48.1 | 152 | 1.0 |
| lowbw → highlat | rsh (TCP) | 2/2 | 58.1 | 42.2 | 6.8 | 19.8 | 152 | 3.5 |
| lowbw → highlat | OpenSSH | 2/2 | 40.0 | 8.6 | 3.2 | 3.4 | 2231 | 2.5 |
| lowbw → highlat | ET | 2/2 | 38.1 | 10.3 | 3.6 | 4.6 | 2078 | 2.0 |
| lowbw → highlat | mosh | 2/2 | 17.4 | 7.5 | 6.2 | 6.5 | 162 | 0.0 |
| highlat → lossy | rsh (UDP) | 2/2 | 57.6 | 57.6 | 44.1 | 48.5 | 52 | 1.0 |
| highlat → lossy | rsh (TCP) | 2/2 | 54.0 | 40.9 | 6.6 | 16.0 | 56 | 1.5 |
| highlat → lossy | OpenSSH | 2/2 | 56.9 | 21.4 | 4.9 | 7.2 | 159 | 0.0 |
| highlat → lossy | ET | 2/2 | 43.7 | 27.9 | 6.0 | 8.8 | 229 | 0.0 |
| highlat → lossy | mosh | 2/2 | 6.8 | 14.1 | 6.1 | 9.2 | 60 | 3.0 |

## 10. Outage, roaming, reattachment and UDP block

Two runs each on RTT 100 ms. Outage is five seconds; roaming is watched for eight seconds. Reattachment restarts the client. UDP-block connection is watched for 15 seconds.

![Recovery and connection times for connectivity events](docs/bench/v0.4.0/events.svg)

| Event | Tool | Observed | Time s | Same shell | Errors |
| --- | --- | --- | --- | --- | --- |
| After 5 s outage | rsh (UDP) | 2/2 | 0.07 | — | 0 |
| After 5 s outage | rsh (TCP) | 2/2 | 5.23 | — | 0 |
| After 5 s outage | OpenSSH | 2/2 | 5.02 | — | 0 |
| After 5 s outage | ET | 2/2 | 5.09 | — | 0 |
| After 5 s outage | mosh | 2/2 | 1.01 | — | 0 |
| After IP change | rsh (UDP) | 2/2 | 2.00 | — | 0 |
| After IP change | rsh (TCP) | 0/2 | — | — | 0 |
| After IP change | OpenSSH | 0/2 | — | — | 0 |
| After IP change | ET | 0/2 | — | — | 0 |
| After IP change | mosh | 2/2 | 3.09 | — | 0 |
| Client restart | rsh (UDP) | 2/2 | 0.56 | 2/2 | 0 |
| Client restart | rsh (TCP) | 2/2 | 2.26 | 2/2 | 0 |
| Client restart | OpenSSH | 2/2 | 1.65 | 0/2 | 0 |
| Client restart | ET | 2/2 | 2.17 | 0/2 | 0 |
| Client restart | mosh | 2/2 | 1.90 | 0/2 | 0 |
| UDP silently blocked | rsh (UDP) | 2/2 | 2.04 | — | 0 |
| UDP silently blocked | rsh (TCP) | 2/2 | 2.03 | — | 0 |
| UDP silently blocked | OpenSSH | 2/2 | 1.42 | — | 0 |
| UDP silently blocked | ET | 2/2 | 1.93 | — | 0 |
| UDP silently blocked | mosh | 0/2 | — | — | 2 |

Restart time alone does not establish session continuity: see Same shell. A missing roaming response within eight seconds is a censored observation, not a measured infinite recovery time. UDP-block mosh failures are retained, consistent with its UDP-only transport.

## 11. Moving network

Each scene runs for 60 seconds with two shared 30-second delay/rate/outage
cycles. There are two evaluations per cohort. Four sessions run concurrently
in 100×30 workload PTYs; the 403×32 virtual viewer is continuously drained.
One cohort uses rsh UDP and another rsh TCP alongside SSH, ET and mosh.
All moving tools use the same 400×200 pyte header observer with leaky-bucket
catch-up burst filtering at the target 60 FPS cap.
Shared tools are kept separate across cohorts. Moving is descriptive and is not pooled with static results. Replay application
skew is recorded; exact global timing fidelity is not assumed.
Loss fractions here describe intervals of complete outage, not independent
per-packet random loss. No GPU presentation or radio-network claim follows.

![Moving UDP cohort FPS and lows](docs/bench/v0.4.0/moving.svg)

| Cohort | Evaluation | Scene | Tool | Avg FPS | 1% low | 5% low | Age ms | Maxgap ms | Scene RX MiB | Client RSS MiB | Server RSS MiB | End marker |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| UDP | 0 | Dashboard | OpenSSH | 20.65 | 3.45 | 10.42 | 297 | 13528 | 13.16 | 8.13 | 20.70 | seen |
| UDP | 0 | Dashboard | ET | 20.20 | 2.76 | 8.77 | 274 | 12667 | 18.28 | 16.17 | 32.68 | seen |
| UDP | 0 | Dashboard | mosh | 14.50 | 2.92 | 5.81 | 58 | 2085 | 0.60 | 14.44 | 10.97 | seen |
| UDP | 0 | Dashboard | rsh (UDP) | 33.87 | 4.10 | 13.51 | 50 | 1249 | 1.01 | 3.64 | 6.03 | seen |
| UDP | 0 | Random screen | OpenSSH | 16.93 | 2.60 | 4.90 | 1065 | 2380 | 17.34 | 8.21 | 20.70 | seen |
| UDP | 0 | Random screen | ET | 20.25 | 2.22 | 8.93 | 3498 | 5224 | 22.85 | 40.07 | 66.32 | seen |
| UDP | 0 | Random screen | mosh | 13.05 | 2.87 | 3.82 | 69 | 751 | 4.21 | 13.47 | 11.44 | seen |
| UDP | 0 | Random screen | rsh (UDP) | 26.75 | 2.58 | 7.41 | 58 | 1310 | 10.21 | 3.96 | 5.99 | seen |
| UDP | 0 | Coding agent TUI | OpenSSH | 12.48 | 3.50 | 12.05 | 88 | 23552 | 5.54 | 4.39 | 12.37 | seen |
| UDP | 0 | Coding agent TUI | ET | 31.38 | 5.21 | 16.39 | 189 | 5960 | 13.43 | 35.57 | 75.53 | seen |
| UDP | 0 | Coding agent TUI | mosh | 14.72 | 2.21 | 6.29 | 57 | 2013 | 0.34 | 10.24 | 9.84 | seen |
| UDP | 0 | Coding agent TUI | rsh (UDP) | 37.38 | 4.88 | 14.93 | 49 | 1316 | 0.52 | 3.95 | 5.98 | seen |
| UDP | 1 | Dashboard | OpenSSH | 20.35 | 1.79 | 8.40 | 253 | 3659 | 13.23 | 7.91 | 19.39 | seen |
| UDP | 1 | Dashboard | ET | 20.27 | 2.08 | 7.58 | 394 | 2321 | 15.42 | 18.21 | 94.61 | seen |
| UDP | 1 | Dashboard | mosh | 14.57 | 2.22 | 5.46 | 58 | 1074 | 0.61 | 14.31 | 20.61 | seen |
| UDP | 1 | Dashboard | rsh (UDP) | 34.27 | 4.48 | 14.49 | 51 | 1510 | 1.03 | 3.74 | 6.07 | seen |
| UDP | 1 | Random screen | OpenSSH | 15.88 | 1.78 | 6.67 | 1079 | 8326 | 18.01 | 7.31 | 19.14 | seen |
| UDP | 1 | Random screen | ET | 7.08 | 0.52 | 3.30 | 58 | 20536 | 123.47 | 35.55 | 181.43 | not seen |
| UDP | 1 | Random screen | mosh | 13.07 | 2.88 | 4.02 | 70 | 749 | 4.23 | 13.42 | 20.97 | seen |
| UDP | 1 | Random screen | rsh (UDP) | 27.87 | 2.88 | 9.43 | 58 | 1524 | 10.43 | 3.96 | 5.59 | seen |
| UDP | 1 | Coding agent TUI | OpenSSH | 32.18 | 3.48 | 14.71 | 237 | 3809 | 13.40 | 4.43 | 11.12 | seen |
| UDP | 1 | Coding agent TUI | ET | 0.00 | — | — | — | 60000 | 130.18 | 48.62 | 485.18 | not seen |
| UDP | 1 | Coding agent TUI | mosh | 14.28 | 2.12 | 6.02 | 56 | 2122 | 0.34 | 9.89 | 19.30 | seen |
| UDP | 1 | Coding agent TUI | rsh (UDP) | 40.02 | 4.95 | 26.32 | 51 | 1330 | 0.58 | 3.95 | 5.42 | seen |

UDP cohort maximum replay application skew: 4.27ms.

![Moving TCP cohort FPS and lows](docs/bench/v0.4.0/moving-tcp.svg)

| Cohort | Evaluation | Scene | Tool | Avg FPS | 1% low | 5% low | Age ms | Maxgap ms | Scene RX MiB | Client RSS MiB | Server RSS MiB | End marker |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| TCP | 0 | Dashboard | OpenSSH | 21.90 | 3.40 | 10.53 | 688 | 6574 | 13.74 | 7.79 | 18.93 | seen |
| TCP | 0 | Dashboard | ET | 21.62 | 2.16 | 10.00 | 405 | 6648 | 17.68 | 17.60 | 602.65 | seen |
| TCP | 0 | Dashboard | mosh | 14.58 | 2.56 | 5.62 | 58 | 1074 | 0.61 | 13.80 | 29.57 | seen |
| TCP | 0 | Dashboard | rsh (TCP) | 26.88 | 2.87 | 9.35 | 49 | 2190 | 0.72 | 2.68 | 5.59 | seen |
| TCP | 0 | Random screen | OpenSSH | 12.30 | 1.31 | 5.21 | 795 | 10358 | 13.76 | 7.19 | 18.50 | seen |
| TCP | 0 | Random screen | ET | 7.33 | 0.75 | 1.33 | 58 | 13642 | 118.50 | 28.96 | 645.21 | not seen |
| TCP | 0 | Random screen | mosh | 12.75 | 2.86 | 3.62 | 70 | 1306 | 4.17 | 12.55 | 28.88 | seen |
| TCP | 0 | Random screen | rsh (TCP) | 17.87 | 2.11 | 5.85 | 64 | 1765 | 7.16 | 2.81 | 5.71 | seen |
| TCP | 0 | Coding agent TUI | OpenSSH | 27.82 | 5.13 | 10.87 | 213 | 6389 | 12.81 | 4.74 | 11.28 | seen |
| TCP | 0 | Coding agent TUI | ET | 0.00 | — | — | — | 60000 | 124.28 | 60.29 | 725.57 | not seen |
| TCP | 0 | Coding agent TUI | mosh | 14.45 | 2.21 | 6.02 | 57 | 2147 | 0.34 | 10.16 | 28.13 | seen |
| TCP | 0 | Coding agent TUI | rsh (TCP) | 26.95 | 3.37 | 9.26 | 48 | 1261 | 0.37 | 2.78 | 5.51 | seen |
| TCP | 1 | Dashboard | OpenSSH | 20.67 | 1.76 | 8.33 | 319 | 3601 | 13.20 | 8.02 | 20.23 | seen |
| TCP | 1 | Dashboard | ET | 17.52 | 1.76 | 8.47 | 403 | 9129 | 17.21 | 18.38 | 736.40 | seen |
| TCP | 1 | Dashboard | mosh | 14.57 | 3.39 | 5.71 | 58 | 1075 | 0.60 | 14.27 | 38.71 | seen |
| TCP | 1 | Dashboard | rsh (TCP) | 26.95 | 3.75 | 8.62 | 49 | 1273 | 0.71 | 2.69 | 5.58 | seen |
| TCP | 1 | Random screen | OpenSSH | 14.35 | 1.68 | 5.49 | 1072 | 12660 | 18.46 | 8.14 | 20.23 | seen |
| TCP | 1 | Random screen | ET | 20.28 | 2.20 | 10.42 | 3050 | 5164 | 22.65 | 31.29 | 758.80 | seen |
| TCP | 1 | Random screen | mosh | 12.58 | 2.82 | 3.46 | 69 | 1290 | 4.15 | 14.36 | 38.58 | seen |
| TCP | 1 | Random screen | rsh (TCP) | 19.83 | 2.23 | 6.76 | 61 | 2044 | 7.74 | 2.82 | 5.76 | seen |
| TCP | 1 | Coding agent TUI | OpenSSH | 27.67 | 5.41 | 12.99 | 316 | 9299 | 13.31 | 5.12 | 12.78 | seen |
| TCP | 1 | Coding agent TUI | ET | 31.53 | 3.47 | 14.71 | 280 | 5448 | 13.42 | 24.69 | 770.91 | seen |
| TCP | 1 | Coding agent TUI | mosh | 14.37 | 2.10 | 6.45 | 57 | 2102 | 0.34 | 9.88 | 37.25 | seen |
| TCP | 1 | Coding agent TUI | rsh (TCP) | 28.73 | 3.44 | 11.11 | 49 | 1277 | 0.39 | 2.80 | 5.62 | seen |

TCP cohort maximum replay application skew: 7.14ms.

Scene RX includes warmup/drain outside the common FPS window, so it is
not a 60-second RX rate. Moving RSS is the larger of midpoint/end samples,
not peak RSS; static RSS uses the midpoint sample. Buffered catch-up bursts
arriving faster than the target cadence are excluded from FPS and capped at
the requested 60 FPS ceiling. Sparse lows remain missing.
The end-marker column preserves clients that did not show the scene end within
the 120-second drain limit; that drain is outside the FPS window.

## Collection failures and limitations

| Source | Test | Link | Scene | Tool | Iteration | Error |
| --- | --- | --- | --- | --- | --- | --- |
| full-suite | udpblock | wan | — | mosh | 0 | no prompt |
| full-suite | udpblock | wan | — | mosh | 1 | no prompt |

Static render jobs: 300/300 measured; switch jobs: 150/150 measured. All tests use the same full batch. Collection failures: 2; expected mosh UDP-block no-prompt cases: 2. No failed job was silently converted to a numeric result.

Results cover synthetic isolated paths on one shared host, not fairness with cross traffic, real radio traces, independent hardware replications or statistical confidence intervals. Static nominal loss percentages do not guarantee equal realized drops across protocols. High average FPS can coexist with worse tails; compare the average and percentile columns separately. No universal win or non-regression claim is made.

## Data and reproduction

| Source run | Jobs | Used for | Original SHA-256 |
| --- | --- | --- | --- |
| full-suite.json | 700 | echo, flood, outage, reattach, render, roam, sessions, switch, udpblock | 2430daf66fc030d7c336f1bf15c940668e3bcb2954cfc32f936810ef535e765a |

The [static snapshot](docs/bench/data/v0.4.0/static.json) contains the complete
batch, options, frame gaps, resource samples and errors. Separate moving
[UDP](docs/bench/data/v0.4.0/moving-udp.json) and
[TCP](docs/bench/data/v0.4.0/moving-tcp.json) snapshots retain both evaluations,
raw frame observations, windows, scene counters and network replay traces.
The [environment manifest](docs/bench/data/v0.4.0/environment.json) records
host/VM configuration, package and executable identities, source/harness hashes
and actual test commands. No older comparison supplies values to this report.

Regenerate this report, the README tables and every linked chart from the
committed snapshots:

```sh
python3 benchmark/lab/report.py
```

Collect the full static workload with four jobs and two evaluations:

```sh
scripts/dist.sh
python3 benchmark/harness/run.py build
python3 benchmark/lab/lab.py run --tools ssh,et,mosh,rsh,rsh-tcp \
  --tests render,switch,echo,flood,outage,roam,reattach,udpblock,sessions \
  --nets lan,wan,lowbw,mid5,verylow,slow64k,highlat,lossy,mobile,badmobile \
  --switches lan:highlat,lan:lowbw,lowbw:highlat,highlat:lossy,wan:verylow \
  --scenes compress,bandwidth,agent --fps 60 --duration 8 --iterations 2 \
  --sessions 10 --session-iterations 2 --cols 100 --rows 30 --parallel 4
```

For moving, run each transport cohort twice, with a 403×32 viewer producing
four 100×30 workload panes. The actual pane/source dimensions are recorded;
`--cols`/`--rows` alone do not set demo geometry.

```sh
python3 benchmark/lab/lab.py demo --tools ssh,et,mosh,rsh --events '' \
  --nets moving --scenes compress,bandwidth,agent --fps 60 --duration 60
python3 benchmark/lab/lab.py demo --tools ssh,et,mosh,rsh-tcp --events '' \
  --nets moving --scenes compress,bandwidth,agent --fps 60 --duration 60
```

## References

The [README references](README.md#references) connect sender/receiver mechanisms
to their research sources. These measurements come from the repository's
Docker/netem/terminal collectors.
