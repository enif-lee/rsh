# Benchmark report

Recorded comparisons of rsh (UDP), rsh with UDP blocked (TCP fallback),
OpenSSH, Eternal Terminal (ET), and mosh. All tables and charts below are
generated from the [committed measurement snapshots](docs/bench/data).

## Measurement scope

| Dataset | Conditions | Samples | Version provenance |
| --- | --- | --- | --- |
| Static links, 2026-10-28 | Five transports; 100×30 terminal; requested 60 FPS; 8 s render window | Six render/switch runs per case; three other tests; two ten-session runs | Predates 0.2.0. Original run JSON does not embed an rsh build ID; exact binary cannot be certified. |
| Moving link, 2026-10-29 | SSH/mosh/rsh; 158×43 terminal; requested 60 FPS; 60 s per scene | One run, two 30 s cycles per scene | Development build `7a1d9454d5b1`, with the transport implementation subsequently shipped in 0.2.0. |

The static snapshot restores the original README comparison. It is not a
new benchmark of release build `6729d398d4f1`. The moving measurement is
reported separately because its window, screen size and impairment
semantics differ. No measurements were rerun to produce this report.

### Environment and workloads

The recorded environment was an Apple M5 (10 cores, 32 GB), macOS 27.0.1,
OrbStack/Docker, and Ubuntu 24.04 Linux arm64 containers. The lab image
installs OpenSSH, mosh and ET from Ubuntu/PPA packages
([Dockerfile](benchmark/harness/Dockerfile)). The contemporaneous environment
record lists OpenSSH 9.6p1, mosh 1.4.0 and ET 7.0.0; the saved run JSON does
not independently pin the image digest or package versions.

Each job has a server/client container pair and an isolated shaped link;
jobs share host CPUs. Static batches used ten parallel jobs. This can
affect frame timing. `compress` is a compressible full-screen dashboard;
`bandwidth` changes every cell with random content. Both request 60 FPS.
The client reconstructs terminal output and observes frame headers; this
does not measure GPU presentation or prove that every cell was repainted.

### Metrics and aggregation

- **Avg FPS** is the arithmetic mean of each run's displayed frames divided
  by its observation duration. **Median FPS** is the median of those run
  averages; the original opening table and FPS charts use this median.
- **1% / 5% low** are `1000 / p99(gap_ms)` and `1000 / p95(gap_ms)` using
  nearest-rank percentiles over all six runs' inter-frame gaps pooled
  together. They are percentile reciprocals, not averages of the slowest
  one/five percent. Startup/end boundary gaps are excluded from lows.
- **Age** is the median of each run's median generation-to-observation
  time; it includes processing and queueing and is not RTT. Static windows
  begin at the first observed frame; late backlog is not drained into FPS.
- **Network** is received interface traffic at the client, in decimal
  kbit/s, including protocol overhead. KiB/frame is computed per run and
  then medianed. Packet counts include inbound and outbound interface
  traffic. These are not payload-only byte counts.
- **RSS** is a halfway-through-render sample, in MiB (KiB / 1024), medianed
  across runs; not peak RSS or PSS. Client and server are separate. Server
  process selection includes sshd's listener, ET's shared daemon and
  terminal, mosh-server, or rsh's shared serve process; shells are excluded.
- Other scalar values are medians of available results. Successful/total
  counts, timeouts and collection failures are preserved below. A missing
  observation is never a zero or proof of successful recovery.
- CPU is percent of one core over a render window, not total machine load.
  The original collector's `torn` counter can double-count observations;
  it is not used as a count of visible defects in this report.

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
| lowbw | 1 Mbit/s · RTT 100 ms |
| verylow | 256 kbit/s · RTT 100 ms |
| slow64k | 64 kbit/s · RTT 100 ms |
| mobile | 512 kbit/s · RTT 200 ms · 1% loss |
| badmobile | 256 kbit/s · RTT 400 ms · 2% loss |
| highlat | RTT 300 ms |
| lossy | 2% loss · RTT 100 ms |

## 1. Dashboard rendering

Light bars show median FPS, medium bars 5% low and solid bars 1% low.
All five transports, all nine network profiles and all six runs are included.


![Dashboard FPS, 1% low and 5% low by link and transport](docs/bench/fps-dashboard.svg)

| Link | Tool | Runs | Avg FPS | Median FPS | 1% low | 5% low | Age ms | RX kbit/s | Client RSS MiB | Server RSS MiB |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| LAN | rsh (UDP) | 6 | 60.0 | 60.0 | 46.1 | 49.8 | 5 | 218 | 2.42 | 5.54 |
| LAN | rsh (TCP) | 6 | 60.0 | 60.0 | 45.5 | 49.5 | 4 | 228 | 2.43 | 5.79 |
| LAN | OpenSSH | 6 | 60.0 | 60.0 | 50.3 | 52.6 | 1 | 2480 | 7.37 | 18.39 |
| LAN | ET | 6 | 60.0 | 60.0 | 50.3 | 52.6 | 1 | 2445 | 18.11 | 14.43 |
| LAN | mosh | 6 | 46.8 | 46.7 | 38.6 | 41.3 | 10 | 237 | 15.42 | 10.52 |
| RTT 100 ms | rsh (UDP) | 6 | 60.0 | 60.0 | 44.1 | 48.3 | 56 | 217 | 2.43 | 5.54 |
| RTT 100 ms | rsh (TCP) | 6 | 56.7 | 56.7 | 42.7 | 46.3 | 60 | 243 | 2.45 | 5.68 |
| RTT 100 ms | OpenSSH | 6 | 60.0 | 60.0 | 45.9 | 50.0 | 52 | 2507 | 7.38 | 18.39 |
| RTT 100 ms | ET | 6 | 60.0 | 60.0 | 45.7 | 50.5 | 51 | 2468 | 17.68 | 14.53 |
| RTT 100 ms | mosh | 6 | 19.1 | 19.1 | 17.6 | 18.1 | 60 | 104 | 14.47 | 10.47 |
| 1 Mbit/s · RTT 100 ms | rsh (UDP) | 6 | 60.0 | 60.0 | 43.9 | 47.8 | 59 | 217 | 2.44 | 5.55 |
| 1 Mbit/s · RTT 100 ms | rsh (TCP) | 6 | 56.2 | 56.4 | 29.3 | 45.2 | 63 | 235 | 2.44 | 5.88 |
| 1 Mbit/s · RTT 100 ms | OpenSSH | 6 | 23.7 | 23.7 | 6.9 | 7.0 | 2546 | 978 | 7.55 | 19.25 |
| 1 Mbit/s · RTT 100 ms | ET | 6 | 23.9 | 23.9 | 6.5 | 7.6 | 2517 | 976 | 18.06 | 14.38 |
| 1 Mbit/s · RTT 100 ms | mosh | 6 | 18.1 | 18.1 | 16.9 | 17.3 | 66 | 100 | 14.53 | 10.54 |
| 256 kbit/s · RTT 100 ms | rsh (UDP) | 6 | 53.2 | 54.3 | 36.1 | 41.8 | 76 | 220 | 2.45 | 5.77 |
| 256 kbit/s · RTT 100 ms | rsh (TCP) | 6 | 35.7 | 35.3 | 25.1 | 27.4 | 71 | 158 | 2.44 | 5.71 |
| 256 kbit/s · RTT 100 ms | OpenSSH | 6 | 6.0 | 6.0 | 1.9 | 4.9 | 3782 | 250 | 7.37 | 19.43 |
| 256 kbit/s · RTT 100 ms | ET | 6 | 6.1 | 6.1 | 4.2 | 4.3 | 3754 | 249 | 18.07 | 13.76 |
| 256 kbit/s · RTT 100 ms | mosh | 6 | 15.9 | 15.9 | 14.7 | 15.0 | 82 | 88 | 14.54 | 10.43 |
| 64 kbit/s · RTT 100 ms | rsh (UDP) | 6 | 12.9 | 13.1 | 8.5 | 10.8 | 122 | 54 | 2.44 | 6.13 |
| 64 kbit/s · RTT 100 ms | rsh (TCP) | 6 | 13.8 | 13.8 | 7.3 | 11.4 | 184 | 63 | 2.45 | 5.69 |
| 64 kbit/s · RTT 100 ms | OpenSSH | 6 | 1.5 | 1.5 | 1.0 | 1.1 | 4304 | 62 | 7.37 | 19.31 |
| 64 kbit/s · RTT 100 ms | ET | 6 | 1.6 | 1.6 | 0.9 | 1.1 | 4633 | 61 | 18.06 | 13.51 |
| 64 kbit/s · RTT 100 ms | mosh | 6 | 10.3 | 10.4 | 6.9 | 9.4 | 153 | 59 | 14.47 | 10.39 |
| 512 kbit/s · RTT 200 ms · 1% loss | rsh (UDP) | 6 | 47.9 | 47.8 | 4.9 | 44.4 | 118 | 234 | 2.79 | 6.04 |
| 512 kbit/s · RTT 200 ms · 1% loss | rsh (TCP) | 6 | 36.6 | 38.0 | 7.2 | 26.2 | 116 | 186 | 2.62 | 5.73 |
| 512 kbit/s · RTT 200 ms · 1% loss | OpenSSH | 6 | 11.2 | 11.3 | 2.1 | 3.2 | 3533 | 474 | 7.58 | 19.63 |
| 512 kbit/s · RTT 200 ms · 1% loss | ET | 6 | 11.8 | 11.9 | 2.3 | 3.8 | 3322 | 499 | 17.36 | 13.74 |
| 512 kbit/s · RTT 200 ms · 1% loss | mosh | 6 | 9.2 | 9.2 | 7.9 | 8.9 | 121 | 50 | 14.43 | 10.39 |
| 256 kbit/s · RTT 400 ms · 2% loss | rsh (UDP) | 6 | 26.3 | 24.2 | 1.7 | 31.2 | 228 | 182 | 3.09 | 5.74 |
| 256 kbit/s · RTT 400 ms · 2% loss | rsh (TCP) | 6 | 21.9 | 21.7 | 1.8 | 9.3 | 230 | 146 | 2.70 | 5.72 |
| 256 kbit/s · RTT 400 ms · 2% loss | OpenSSH | 6 | 5.1 | 5.2 | 0.9 | 1.4 | 4160 | 222 | 7.57 | 19.53 |
| 256 kbit/s · RTT 400 ms · 2% loss | ET | 6 | 5.5 | 5.8 | 0.6 | 1.7 | 4024 | 246 | 17.32 | 13.58 |
| 256 kbit/s · RTT 400 ms · 2% loss | mosh | 6 | 4.5 | 4.7 | 2.4 | 4.6 | 232 | 26 | 14.54 | 10.32 |
| RTT 300 ms | rsh (UDP) | 6 | 57.3 | 59.4 | 35.2 | 42.4 | 156 | 217 | 2.42 | 5.60 |
| RTT 300 ms | rsh (TCP) | 6 | 55.0 | 55.0 | 41.0 | 46.5 | 161 | 237 | 2.50 | 5.96 |
| RTT 300 ms | OpenSSH | 6 | 60.0 | 60.0 | 23.3 | 48.3 | 152 | 2507 | 7.45 | 18.40 |
| RTT 300 ms | ET | 6 | 59.8 | 59.8 | 4.2 | 50.3 | 152 | 2466 | 15.98 | 14.58 |
| RTT 300 ms | mosh | 6 | 6.6 | 6.6 | 6.4 | 6.4 | 160 | 37 | 14.54 | 10.30 |
| 2% loss · RTT 100 ms | rsh (UDP) | 6 | 58.8 | 58.7 | 42.6 | 48.3 | 56 | 424 | 2.43 | 5.60 |
| 2% loss · RTT 100 ms | rsh (TCP) | 6 | 27.9 | 27.4 | 6.2 | 12.5 | 57 | 128 | 2.56 | 6.21 |
| 2% loss · RTT 100 ms | OpenSSH | 6 | 33.0 | 30.2 | 4.9 | 7.5 | 1111 | 1246 | 7.64 | 19.16 |
| 2% loss · RTT 100 ms | ET | 6 | 38.1 | 37.8 | 4.9 | 9.8 | 268 | 1558 | 17.24 | 14.27 |
| 2% loss · RTT 100 ms | mosh | 6 | 17.1 | 16.7 | 3.7 | 17.6 | 60 | 100 | 14.54 | 10.87 |

## 2. Random full-screen rendering

Every cell changes each frame. This workload compresses less effectively
and exposes a different bandwidth and frame-gap tradeoff than the dashboard.


![Random-screen FPS, 1% low and 5% low by link and transport](docs/bench/fps-random.svg)

| Link | Tool | Runs | Avg FPS | Median FPS | 1% low | 5% low | Age ms | RX kbit/s | Client RSS MiB | Server RSS MiB |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| LAN | rsh (UDP) | 6 | 59.8 | 59.7 | 41.0 | 46.5 | 9 | 2308 | 2.44 | 5.83 |
| LAN | rsh (TCP) | 6 | 59.9 | 59.9 | 42.0 | 46.9 | 9 | 2352 | 2.44 | 5.82 |
| LAN | OpenSSH | 6 | 60.0 | 60.0 | 49.8 | 52.6 | 2 | 5782 | 7.38 | 18.45 |
| LAN | ET | 6 | 60.0 | 60.0 | 50.0 | 52.6 | 2 | 5722 | 18.10 | 16.03 |
| LAN | mosh | 6 | 46.8 | 46.7 | 37.7 | 41.0 | 11 | 2027 | 15.47 | 10.51 |
| RTT 100 ms | rsh (UDP) | 6 | 57.8 | 57.8 | 23.7 | 44.8 | 60 | 2307 | 2.57 | 5.64 |
| RTT 100 ms | rsh (TCP) | 6 | 37.2 | 37.8 | 14.2 | 18.6 | 60 | 1483 | 2.45 | 5.92 |
| RTT 100 ms | OpenSSH | 6 | 60.0 | 60.0 | 29.1 | 49.0 | 52 | 5798 | 7.65 | 18.46 |
| RTT 100 ms | ET | 6 | 60.0 | 60.0 | 13.0 | 49.3 | 52 | 5754 | 18.20 | 16.25 |
| RTT 100 ms | mosh | 6 | 19.0 | 19.0 | 17.5 | 17.8 | 62 | 821 | 14.43 | 10.44 |
| 1 Mbit/s · RTT 100 ms | rsh (UDP) | 6 | 23.9 | 23.9 | 13.0 | 20.6 | 101 | 928 | 2.46 | 5.99 |
| 1 Mbit/s · RTT 100 ms | rsh (TCP) | 6 | 20.3 | 20.4 | 10.1 | 15.6 | 98 | 822 | 2.46 | 5.77 |
| 1 Mbit/s · RTT 100 ms | OpenSSH | 6 | 10.0 | 10.0 | 6.2 | 6.7 | 3432 | 978 | 7.41 | 19.95 |
| 1 Mbit/s · RTT 100 ms | ET | 6 | 10.1 | 10.1 | 4.6 | 5.9 | 3232 | 978 | 17.22 | 14.13 |
| 1 Mbit/s · RTT 100 ms | mosh | 6 | 16.4 | 16.4 | 15.1 | 15.5 | 106 | 707 | 14.76 | 10.44 |
| 256 kbit/s · RTT 100 ms | rsh (UDP) | 6 | 6.2 | 6.2 | 5.4 | 5.6 | 216 | 243 | 2.43 | 6.06 |
| 256 kbit/s · RTT 100 ms | rsh (TCP) | 6 | 5.7 | 5.7 | 2.9 | 5.5 | 385 | 256 | 2.46 | 6.36 |
| 256 kbit/s · RTT 100 ms | OpenSSH | 6 | 2.5 | 2.5 | 1.3 | 1.4 | 3917 | 248 | 7.38 | 19.62 |
| 256 kbit/s · RTT 100 ms | ET | 6 | 2.6 | 2.6 | 1.4 | 1.7 | 3976 | 250 | 18.10 | 13.62 |
| 256 kbit/s · RTT 100 ms | mosh | 6 | 5.5 | 5.5 | 4.0 | 4.0 | 504 | 240 | 15.03 | 10.89 |
| 64 kbit/s · RTT 100 ms | rsh (UDP) | 6 | 1.5 | 1.5 | 1.2 | 1.2 | 534 | 64 | 2.43 | 6.56 |
| 64 kbit/s · RTT 100 ms | rsh (TCP) | 6 | 1.6 | 1.6 | 1.1 | 1.2 | 1414 | 64 | 2.43 | 5.98 |
| 64 kbit/s · RTT 100 ms | OpenSSH | 6 | 0.6 | 0.6 | 0.5 | 0.5 | 4058 | 62 | 7.37 | 19.59 |
| 64 kbit/s · RTT 100 ms | ET | 6 | 0.6 | 0.6 | 0.4 | 0.4 | 4652 | 62 | 18.05 | 13.45 |
| 64 kbit/s · RTT 100 ms | mosh | 6 | 1.5 | 1.5 | 1.4 | 1.5 | 3978 | 63 | 14.95 | 13.00 |
| 512 kbit/s · RTT 200 ms · 1% loss | rsh (UDP) | 6 | 9.8 | 9.5 | 2.7 | 4.1 | 184 | 418 | 2.57 | 6.43 |
| 512 kbit/s · RTT 200 ms · 1% loss | rsh (TCP) | 6 | 9.5 | 9.1 | 3.1 | 3.5 | 216 | 443 | 2.54 | 6.27 |
| 512 kbit/s · RTT 200 ms · 1% loss | OpenSSH | 6 | 4.9 | 5.0 | 0.5 | 2.3 | 4401 | 500 | 7.45 | 19.46 |
| 512 kbit/s · RTT 200 ms · 1% loss | ET | 6 | 5.2 | 5.2 | 0.9 | 1.8 | 3760 | 501 | 17.75 | 13.67 |
| 512 kbit/s · RTT 200 ms · 1% loss | mosh | 6 | 7.5 | 7.5 | 2.6 | 7.5 | 197 | 346 | 14.67 | 10.41 |
| 256 kbit/s · RTT 400 ms · 2% loss | rsh (UDP) | 6 | 3.6 | 3.5 | 0.5 | 1.4 | 373 | 196 | 2.54 | 5.76 |
| 256 kbit/s · RTT 400 ms · 2% loss | rsh (TCP) | 6 | 3.8 | 3.8 | 1.0 | 1.6 | 500 | 206 | 2.48 | 6.48 |
| 256 kbit/s · RTT 400 ms · 2% loss | OpenSSH | 6 | 2.2 | 2.2 | 0.4 | 1.1 | 4310 | 216 | 7.41 | 19.60 |
| 256 kbit/s · RTT 400 ms · 2% loss | ET | 6 | 2.2 | 2.2 | 0.9 | 1.0 | 4712 | 232 | 17.29 | 13.30 |
| 256 kbit/s · RTT 400 ms · 2% loss | mosh | 6 | 4.2 | 4.3 | 1.1 | 2.3 | 383 | 192 | 14.45 | 10.38 |
| RTT 300 ms | rsh (UDP) | 6 | 56.7 | 56.7 | 27.6 | 44.4 | 160 | 2308 | 2.62 | 5.71 |
| RTT 300 ms | rsh (TCP) | 6 | 46.8 | 50.5 | 14.6 | 29.2 | 164 | 2006 | 2.56 | 5.90 |
| RTT 300 ms | OpenSSH | 6 | 60.0 | 60.0 | 14.5 | 47.2 | 152 | 5798 | 7.82 | 18.66 |
| RTT 300 ms | ET | 6 | 60.0 | 60.0 | 4.1 | 48.8 | 153 | 5740 | 16.28 | 16.32 |
| RTT 300 ms | mosh | 6 | 6.6 | 6.6 | 6.4 | 6.4 | 162 | 282 | 14.89 | 10.21 |
| 2% loss · RTT 100 ms | rsh (UDP) | 6 | 56.1 | 56.2 | 19.3 | 44.2 | 60 | 4370 | 2.62 | 5.89 |
| 2% loss · RTT 100 ms | rsh (TCP) | 6 | 13.1 | 12.7 | 3.0 | 4.1 | 77 | 714 | 2.58 | 5.81 |
| 2% loss · RTT 100 ms | OpenSSH | 6 | 15.3 | 14.6 | 4.3 | 5.0 | 2607 | 1413 | 7.57 | 19.99 |
| 2% loss · RTT 100 ms | ET | 6 | 24.1 | 27.0 | 4.9 | 9.3 | 427 | 2598 | 18.18 | 15.68 |
| 2% loss · RTT 100 ms | mosh | 6 | 15.2 | 15.0 | 3.7 | 6.3 | 61 | 802 | 14.40 | 10.90 |

## 3. Frame age

![Dashboard median frame age by link and transport](docs/bench/age.svg)

The rendering tables above include age for both scenes. The chart uses a log scale.

## 4. Network traffic and CPU

![Dashboard received KiB per displayed frame](docs/bench/network.svg)

![Dashboard inbound and outbound packets per displayed frame](docs/bench/packets.svg)

The render tables give RX kbit/s. Below: median per-run bytes/frame, packets/frame and CPU for each scene.

### Dashboard

| Link | Tool | RX KiB/frame | Packets in/frame | Packets out/frame | Client CPU % | Server CPU % |
| --- | --- | --- | --- | --- | --- | --- |
| lan | rsh (UDP) | 0.44 | 1.00 | 1.00 | 0.56 | 0.88 |
| lan | rsh (TCP) | 0.46 | 1.00 | 1.00 | 0.56 | 0.91 |
| lan | OpenSSH | 5.05 | 2.21 | 1.32 | 0.16 | 0.21 |
| lan | ET | 4.97 | 1.54 | 1.03 | 0.34 | 0.41 |
| lan | mosh | 0.62 | 1.00 | 0.20 | 0.70 | 1.62 |
| wan | rsh (UDP) | 0.44 | 1.00 | 0.99 | 0.56 | 0.89 |
| wan | rsh (TCP) | 0.52 | 1.09 | 1.00 | 0.76 | 3.16 |
| wan | OpenSSH | 5.10 | 3.13 | 1.39 | 0.17 | 0.23 |
| wan | ET | 5.02 | 2.53 | 1.92 | 0.25 | 0.37 |
| wan | mosh | 0.66 | 0.99 | 0.48 | 0.36 | 1.40 |
| lowbw | rsh (UDP) | 0.44 | 1.00 | 0.99 | 0.56 | 0.86 |
| lowbw | rsh (TCP) | 0.52 | 1.08 | 1.00 | 0.67 | 2.60 |
| lowbw | OpenSSH | 5.03 | 1.92 | 2.09 | 0.11 | 0.37 |
| lowbw | ET | 4.98 | 1.82 | 1.62 | 0.17 | 0.38 |
| lowbw | mosh | 0.67 | 0.99 | 0.50 | 0.34 | 1.32 |
| verylow | rsh (UDP) | 0.50 | 1.00 | 1.00 | 0.80 | 3.20 |
| verylow | rsh (TCP) | 0.54 | 1.15 | 1.00 | 0.57 | 2.19 |
| verylow | OpenSSH | 5.09 | 2.33 | 2.38 | 0.06 | 0.27 |
| verylow | ET | 4.98 | 1.90 | 1.73 | 0.14 | 0.34 |
| verylow | mosh | 0.67 | 0.99 | 0.50 | 0.35 | 1.43 |
| slow64k | rsh (UDP) | 0.51 | 1.01 | 1.02 | 0.27 | 1.27 |
| slow64k | rsh (TCP) | 0.56 | 1.39 | 1.04 | 0.28 | 1.31 |
| slow64k | OpenSSH | 5.13 | 3.72 | 3.87 | 0.03 | 0.23 |
| slow64k | ET | 4.81 | 1.97 | 2.00 | 0.12 | 0.34 |
| slow64k | mosh | 0.69 | 1.00 | 0.55 | 0.29 | 1.58 |
| mobile | rsh (UDP) | 0.59 | 1.17 | 1.06 | 0.81 | 3.23 |
| mobile | rsh (TCP) | 0.59 | 1.29 | 1.06 | 0.61 | 2.38 |
| mobile | OpenSSH | 5.05 | 2.34 | 2.48 | 0.08 | 0.46 |
| mobile | ET | 5.06 | 2.04 | 1.90 | 0.17 | 0.40 |
| mobile | mosh | 0.65 | 0.99 | 0.96 | 0.22 | 1.31 |
| badmobile | rsh (UDP) | 0.91 | 1.73 | 1.16 | 0.57 | 2.60 |
| badmobile | rsh (TCP) | 0.82 | 1.55 | 1.23 | 0.46 | 2.00 |
| badmobile | OpenSSH | 5.39 | 3.32 | 3.09 | 0.04 | 0.33 |
| badmobile | ET | 5.02 | 2.33 | 2.21 | 0.12 | 0.33 |
| badmobile | mosh | 0.69 | 0.99 | 0.98 | 0.14 | 1.33 |
| highlat | rsh (UDP) | 0.45 | 1.00 | 0.99 | 0.55 | 0.98 |
| highlat | rsh (TCP) | 0.53 | 1.07 | 0.99 | 0.71 | 2.96 |
| highlat | OpenSSH | 5.10 | 3.01 | 1.70 | 0.16 | 0.20 |
| highlat | ET | 5.03 | 2.39 | 1.69 | 0.22 | 0.31 |
| highlat | mosh | 0.68 | 0.98 | 0.97 | 0.15 | 1.13 |
| lossy | rsh (UDP) | 0.87 | 1.97 | 0.99 | 0.56 | 0.91 |
| lossy | rsh (TCP) | 0.58 | 1.28 | 1.10 | 0.45 | 1.71 |
| lossy | OpenSSH | 5.08 | 2.21 | 1.35 | 0.09 | 0.54 |
| lossy | ET | 5.04 | 2.13 | 1.42 | 0.18 | 0.33 |
| lossy | mosh | 0.73 | 1.08 | 0.48 | 0.34 | 1.33 |

### Random screen

| Link | Tool | RX KiB/frame | Packets in/frame | Packets out/frame | Client CPU % | Server CPU % |
| --- | --- | --- | --- | --- | --- | --- |
| lan | rsh (UDP) | 4.72 | 4.02 | 1.00 | 1.39 | 11.37 |
| lan | rsh (TCP) | 4.79 | 4.01 | 2.34 | 1.36 | 11.34 |
| lan | OpenSSH | 11.76 | 3.21 | 1.98 | 0.24 | 0.30 |
| lan | ET | 11.64 | 1.92 | 1.84 | 0.47 | 0.60 |
| lan | mosh | 5.30 | 4.99 | 0.20 | 2.84 | 4.53 |
| wan | rsh (UDP) | 4.87 | 4.15 | 1.01 | 1.35 | 11.34 |
| wan | rsh (TCP) | 4.79 | 4.06 | 2.02 | 0.91 | 8.86 |
| wan | OpenSSH | 11.80 | 3.37 | 1.73 | 0.24 | 0.27 |
| wan | ET | 11.71 | 3.06 | 2.03 | 0.36 | 0.52 |
| wan | mosh | 5.28 | 4.99 | 0.51 | 1.21 | 3.40 |
| lowbw | rsh (UDP) | 4.72 | 4.03 | 4.00 | 0.81 | 5.88 |
| lowbw | rsh (TCP) | 4.95 | 5.86 | 4.06 | 0.71 | 5.22 |
| lowbw | OpenSSH | 11.91 | 4.54 | 5.10 | 0.11 | 0.23 |
| lowbw | ET | 11.81 | 4.36 | 3.81 | 0.16 | 0.37 |
| lowbw | mosh | 5.26 | 4.97 | 0.50 | 1.06 | 3.13 |
| verylow | rsh (UDP) | 4.78 | 4.98 | 5.01 | 0.33 | 2.23 |
| verylow | rsh (TCP) | 5.48 | 8.72 | 7.46 | 0.38 | 2.17 |
| verylow | OpenSSH | 12.13 | 5.48 | 5.72 | 0.06 | 0.20 |
| verylow | ET | 11.74 | 4.90 | 4.53 | 0.15 | 0.38 |
| verylow | mosh | 5.30 | 5.06 | 1.07 | 0.46 | 2.98 |
| slow64k | rsh (UDP) | 5.21 | 10.90 | 10.83 | 0.15 | 1.29 |
| slow64k | rsh (TCP) | 4.84 | 7.28 | 5.03 | 0.08 | 1.25 |
| slow64k | OpenSSH | 12.31 | 6.50 | 8.17 | 0.03 | 0.12 |
| slow64k | ET | 12.51 | 4.92 | 5.08 | 0.12 | 0.34 |
| slow64k | mosh | 5.13 | 4.87 | 1.07 | 0.14 | 3.18 |
| mobile | rsh (UDP) | 5.28 | 4.53 | 4.16 | 0.45 | 3.23 |
| mobile | rsh (TCP) | 5.73 | 5.81 | 4.64 | 0.36 | 3.11 |
| mobile | OpenSSH | 12.00 | 5.72 | 5.72 | 0.05 | 0.18 |
| mobile | ET | 11.77 | 5.15 | 4.85 | 0.15 | 0.33 |
| mobile | mosh | 5.57 | 5.36 | 1.03 | 0.57 | 2.92 |
| badmobile | rsh (UDP) | 6.68 | 5.82 | 4.41 | 0.20 | 2.04 |
| badmobile | rsh (TCP) | 6.65 | 6.47 | 4.96 | 0.20 | 2.03 |
| badmobile | OpenSSH | 11.79 | 6.60 | 6.14 | 0.04 | 0.20 |
| badmobile | ET | 11.24 | 4.98 | 4.22 | 0.12 | 0.30 |
| badmobile | mosh | 5.54 | 5.39 | 1.01 | 0.38 | 3.15 |
| highlat | rsh (UDP) | 4.97 | 4.24 | 1.04 | 1.35 | 11.11 |
| highlat | rsh (TCP) | 4.86 | 4.06 | 2.06 | 1.22 | 11.72 |
| highlat | OpenSSH | 11.80 | 3.40 | 2.03 | 0.24 | 0.27 |
| highlat | ET | 11.68 | 2.66 | 1.99 | 0.33 | 0.51 |
| highlat | mosh | 5.22 | 4.91 | 0.97 | 0.43 | 2.83 |
| lossy | rsh (UDP) | 9.52 | 8.11 | 1.02 | 1.35 | 11.12 |
| lossy | rsh (TCP) | 6.57 | 5.11 | 3.56 | 0.53 | 5.38 |
| lossy | OpenSSH | 11.88 | 4.88 | 2.98 | 0.09 | 0.34 |
| lossy | ET | 11.74 | 4.47 | 2.20 | 0.20 | 0.42 |
| lossy | mosh | 6.53 | 6.15 | 0.51 | 0.92 | 3.33 |

## 5. Resident memory

![Median client and server RSS while rendering](docs/bench/memory.svg)

Per-link RSS for both scenes is in sections 1–2. The chart pools the render samples across links and scenes.

### Ten simultaneous sessions

![Combined client and server RSS for ten idle sessions](docs/bench/memory-sessions.svg)

| Tool | Runs | All clients MiB | Server MiB | Idle packets in/s | Idle packets out/s |
| --- | --- | --- | --- | --- | --- |
| rsh (UDP) | 2 | 24.35 | 11.09 | 2.00 | 2.00 |
| OpenSSH | 2 | 73.73 | 148.18 | 0.00 | 0.00 |
| ET | 2 | 171.25 | 36.17 | 7.45 | 7.85 |
| mosh | 2 | 144.38 | 98.12 | 3.40 | 3.60 |

Each shell prints 3,000 lines, then becomes quiet. Packet rates are totals for the container's ten sessions. No rsh-TCP ten-session result was recorded.

## 6. Connection and key echo

![Connection time in echo jobs](docs/bench/connect.svg)

![Key echo latency](docs/bench/echo.svg)

| Link | Tool | Echo runs | Connect s | Echo ms | Echo max ms | Packets in/key | Packets out/key |
| --- | --- | --- | --- | --- | --- | --- | --- |
| lan | rsh (UDP) | 3/3 | 0.014 | 6.8 | 11.6 | 1.1 | 2.2 |
| lan | rsh (TCP) | 3/3 | 1.514 | 8.2 | 13.1 | 3.3 | 2.2 |
| lan | OpenSSH | 3/3 | 0.146 | 16.4 | 24.2 | 16.2 | 16.2 |
| lan | ET | 3/3 | 0.157 | 5.1 | 8.6 | 2.2 | 3.0 |
| lan | mosh | 3/3 | 0.232 | 15.7 | 19.2 | 2.3 | 2.2 |
| wan | rsh (UDP) | 3/3 | 0.320 | 7.0 | 109.8 | 1.2 | 2.3 |
| wan | rsh (TCP) | 3/3 | 2.024 | 5.4 | 109.5 | 3.3 | 2.3 |
| wan | OpenSSH | 3/3 | 1.396 | 115.8 | 126.8 | 21.4 | 24.4 |
| wan | ET | 3/3 | 1.893 | 107.4 | 113.9 | 1.2 | 2.4 |
| wan | mosh | 3/3 | 1.583 | 6.5 | 119.0 | 2.3 | 2.9 |
| lowbw | rsh (UDP) | 3/3 | 0.325 | 3.8 | 111.4 | 1.1 | 2.3 |
| lowbw | rsh (TCP) | 3/3 | 2.033 | 5.0 | 111.5 | 3.3 | 2.3 |
| lowbw | OpenSSH | 3/3 | 1.516 | 119.6 | 126.8 | 20.6 | 20.9 |
| lowbw | ET | 3/3 | 1.944 | 105.3 | 108.2 | 1.1 | 2.4 |
| lowbw | mosh | 3/3 | 1.746 | 4.5 | 121.7 | 2.5 | 2.9 |
| verylow | rsh (UDP) | 3/3 | 0.352 | 4.5 | 112.6 | 1.5 | 2.7 |
| verylow | rsh (TCP) | 3/3 | 2.063 | 4.0 | 116.6 | 3.7 | 2.9 |
| verylow | OpenSSH | 3/3 | 1.553 | 120.4 | 129.7 | 20.9 | 22.6 |
| verylow | ET | 3/3 | 2.084 | 108.0 | 112.0 | 1.2 | 2.4 |
| verylow | mosh | 3/3 | 1.925 | 4.3 | 120.5 | 2.4 | 2.7 |
| slow64k | rsh (UDP) | 3/3 | 0.483 | 4.3 | 126.1 | 1.1 | 2.3 |
| slow64k | rsh (TCP) | 3/3 | 2.201 | 3.5 | 134.5 | 3.9 | 3.0 |
| slow64k | OpenSSH | 3/3 | 2.113 | 129.9 | 139.5 | 21.2 | 21.7 |
| slow64k | ET | 3/3 | 2.673 | 117.7 | 122.2 | 1.2 | 2.4 |
| slow64k | mosh | 3/3 | 2.364 | 4.8 | 140.3 | 2.3 | 2.1 |
| mobile | rsh (UDP) | 3/3 | 0.636 | 4.9 | 212.7 | 1.2 | 2.2 |
| mobile | rsh (TCP) | 3/3 | 2.542 | 5.7 | 213.0 | 3.4 | 2.6 |
| mobile | OpenSSH | 3/3 | 2.666 | 218.8 | 487.6 | 29.5 | 33.5 |
| mobile | ET | 3/3 | 4.707 | 207.2 | 209.0 | 1.1 | 2.3 |
| mobile | mosh | 3/3 | 2.916 | 4.4 | 302.8 | 2.6 | 3.1 |
| badmobile | rsh (UDP) | 3/3 | 1.252 | 5.0 | 412.9 | 1.5 | 2.1 |
| badmobile | rsh (TCP) | 2/3 | 3.564 | 4.9 | 459.4 | 2.8 | 3.0 |
| badmobile | OpenSSH | 3/3 | 5.173 | 526.8 | 949.2 | 30.4 | 35.2 |
| badmobile | ET | 3/3 | 8.230 | 409.1 | 1148.4 | 1.2 | 2.4 |
| badmobile | mosh | 3/3 | 5.849 | 4.5 | 583.2 | 1.3 | 1.9 |
| highlat | rsh (UDP) | 3/3 | 0.918 | 3.9 | 307.7 | 1.2 | 2.4 |
| highlat | rsh (TCP) | 3/3 | 3.024 | 4.9 | 311.0 | 2.9 | 2.6 |
| highlat | OpenSSH | 3/3 | 3.776 | 317.6 | 326.5 | 30.8 | 31.5 |
| highlat | ET | 3/3 | 5.747 | 305.9 | 309.5 | 1.2 | 2.2 |
| highlat | mosh | 3/3 | 4.362 | 7.1 | 399.0 | 1.8 | 2.2 |
| lossy | rsh (UDP) | 3/3 | 0.317 | 4.5 | 109.4 | 2.0 | 2.3 |
| lossy | rsh (TCP) | 3/3 | 2.026 | 4.2 | 110.4 | 3.4 | 2.5 |
| lossy | OpenSSH | 3/3 | 1.385 | 119.5 | 242.7 | 21.4 | 24.7 |
| lossy | ET | 3/3 | 2.007 | 106.2 | 414.9 | 1.3 | 2.4 |
| lossy | mosh | 3/3 | 1.983 | 6.5 | 121.0 | 2.4 | 2.5 |

Ten keys per run. Echo includes predictive local output for mosh/rsh. Connection times come from these same echo jobs, including their startup path; they are not a new dedicated cold-install test.

## 7. Bulk output

![Time until the prompt returns after seq 1 100000](docs/bench/flood.svg)

| Link | Tool | Completed | Completion s | 30 s timeouts | Errors | Terminal bytes |
| --- | --- | --- | --- | --- | --- | --- |
| lan | rsh (UDP) | 3/3 | 0.05 | 0 | 0 | 690327 |
| lan | rsh (TCP) | 3/3 | 0.06 | 0 | 0 | 691285 |
| lan | OpenSSH | 3/3 | 0.05 | 0 | 0 | 688960 |
| lan | ET | 3/3 | 0.05 | 0 | 0 | 688960 |
| lan | mosh | 3/3 | 0.06 | 0 | 0 | 672 |
| wan | rsh (UDP) | 3/3 | 0.15 | 0 | 0 | 4059 |
| wan | rsh (TCP) | 3/3 | 0.21 | 0 | 0 | 26388 |
| wan | OpenSSH | 3/3 | 0.74 | 0 | 0 | 688960 |
| wan | ET | 3/3 | 0.76 | 0 | 0 | 688960 |
| wan | mosh | 3/3 | 0.17 | 0 | 0 | 471 |
| lowbw | rsh (UDP) | 3/3 | 0.15 | 0 | 0 | 4061 |
| lowbw | rsh (TCP) | 3/3 | 0.15 | 0 | 0 | 4006 |
| lowbw | OpenSSH | 3/3 | 6.03 | 0 | 0 | 688960 |
| lowbw | ET | 3/3 | 6.00 | 0 | 0 | 688960 |
| lowbw | mosh | 3/3 | 0.17 | 0 | 0 | 470 |
| verylow | rsh (UDP) | 3/3 | 0.16 | 0 | 0 | 4064 |
| verylow | rsh (TCP) | 3/3 | 0.17 | 0 | 0 | 2796 |
| verylow | OpenSSH | 3/3 | 23.18 | 0 | 0 | 688960 |
| verylow | ET | 3/3 | 22.89 | 0 | 0 | 688960 |
| verylow | mosh | 3/3 | 0.24 | 0 | 0 | 670 |
| slow64k | rsh (UDP) | 3/3 | 0.19 | 0 | 0 | 3102 |
| slow64k | rsh (TCP) | 3/3 | 0.20 | 0 | 0 | 1841 |
| slow64k | OpenSSH | 0/3 | — | 3 | 0 | 205053 |
| slow64k | ET | 0/3 | — | 3 | 0 | 218943 |
| slow64k | mosh | 3/3 | 0.23 | 0 | 0 | 458 |
| mobile | rsh (UDP) | 3/3 | 0.25 | 0 | 0 | 4061 |
| mobile | rsh (TCP) | 2/3 | 0.26 | 1 | 0 | 4091 |
| mobile | OpenSSH | 3/3 | 12.40 | 0 | 0 | 688960 |
| mobile | ET | 3/3 | 11.59 | 0 | 0 | 688960 |
| mobile | mosh | 3/3 | 0.40 | 0 | 0 | 453 |
| badmobile | rsh (UDP) | 3/3 | 0.46 | 0 | 0 | 3634 |
| badmobile | rsh (TCP) | 2/3 | 0.59 | 0 | 1 | 13322 |
| badmobile | OpenSSH | 3/3 | 23.89 | 0 | 0 | 688960 |
| badmobile | ET | 3/3 | 24.36 | 0 | 0 | 688960 |
| badmobile | mosh | 3/3 | 0.72 | 0 | 0 | 508 |
| highlat | rsh (UDP) | 3/3 | 0.35 | 0 | 0 | 2767 |
| highlat | rsh (TCP) | 3/3 | 0.35 | 0 | 0 | 2757 |
| highlat | OpenSSH | 3/3 | 1.53 | 0 | 0 | 688960 |
| highlat | ET | 3/3 | 2.73 | 0 | 0 | 688960 |
| highlat | mosh | 3/3 | 0.56 | 0 | 0 | 553 |
| lossy | rsh (UDP) | 3/3 | 0.15 | 0 | 0 | 4061 |
| lossy | rsh (TCP) | 3/3 | 0.15 | 0 | 0 | 4035 |
| lossy | OpenSSH | 3/3 | 0.84 | 0 | 0 | 688960 |
| lossy | ET | 3/3 | 1.17 | 0 | 0 | 688960 |
| lossy | mosh | 3/3 | 0.17 | 0 | 0 | 462 |

Completion is the marker after `seq 1 100000`, watched for 30 s. When a case is mixed, completion time covers only completed runs; the counts remain explicit. mosh and rsh may skip intermediate output, so prompt recovery does not imply delivery of the complete byte stream.

## 8. Link changes

Each scene runs two seconds on the first link and eight seconds after the change. FPS before/after is medianed; post-change lows pool only post-change inter-frame gaps. Adaptation is the first whole-second bucket reaching 90% of that tool's own settled FPS, not time to recover 60 FPS.

### Dashboard

![Dashboard FPS and lows after each link change](docs/bench/switch-dashboard.svg)

| Change | Tool | Runs | FPS before | FPS after | 1% low after | 5% low after | Age after ms | Adapt s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| wan → verylow | rsh (UDP) | 6/6 | 59.0 | 52.4 | 32.4 | 43.7 | 76 | 1.0 |
| wan → verylow | rsh (TCP) | 6/6 | 55.2 | 44.8 | 20.3 | 28.5 | 77 | 0.5 |
| wan → verylow | OpenSSH | 6/6 | 59.2 | 5.8 | 1.8 | 1.8 | 3684 | 2.0 |
| wan → verylow | ET | 6/6 | 59.1 | 6.1 | 4.2 | 4.3 | 3717 | 2.0 |
| wan → verylow | mosh | 6/6 | 18.8 | 15.9 | 14.8 | 15.1 | 82 | 0.0 |
| lan → highlat | rsh (UDP) | 6/6 | 59.5 | 56.8 | 43.5 | 47.6 | 156 | 1.0 |
| lan → highlat | rsh (TCP) | 6/6 | 59.4 | 52.0 | 24.0 | 44.6 | 161 | 1.0 |
| lan → highlat | OpenSSH | 6/6 | 59.0 | 59.1 | 25.4 | 50.0 | 152 | 1.0 |
| lan → highlat | ET | 6/6 | 59.1 | 59.1 | 7.9 | 49.5 | 152 | 1.0 |
| lan → highlat | mosh | 6/6 | 46.0 | 10.2 | 6.5 | 6.5 | 160 | 0.0 |
| lan → lowbw | rsh (UDP) | 6/6 | 59.2 | 59.0 | 42.9 | 47.8 | 60 | 1.0 |
| lan → lowbw | rsh (TCP) | 6/6 | 59.5 | 56.5 | 43.7 | 46.9 | 64 | 0.0 |
| lan → lowbw | OpenSSH | 6/6 | 59.0 | 23.4 | 1.9 | 3.8 | 2580 | 1.0 |
| lan → lowbw | ET | 5/6 | 59.1 | 22.4 | 1.9 | 3.0 | 2515 | 0.0 |
| lan → lowbw | mosh | 6/6 | 46.0 | 21.6 | 17.1 | 17.5 | 66 | 0.0 |
| lowbw → highlat | rsh (UDP) | 6/6 | 58.3 | 50.7 | 41.7 | 46.7 | 161 | 1.0 |
| lowbw → highlat | rsh (TCP) | 6/6 | 49.4 | 37.0 | 5.0 | 6.5 | 162 | 3.0 |
| lowbw → highlat | OpenSSH | 6/6 | 23.4 | 44.2 | 3.3 | 3.6 | 3252 | 3.0 |
| lowbw → highlat | ET | 6/6 | 23.5 | 33.6 | 3.3 | 3.5 | 2188 | 3.0 |
| lowbw → highlat | mosh | 6/6 | 17.9 | 7.7 | 6.5 | 6.5 | 160 | 0.0 |
| highlat → lossy | rsh (UDP) | 6/6 | 59.2 | 57.2 | 44.4 | 48.3 | 55 | 1.0 |
| highlat → lossy | rsh (TCP) | 6/6 | 51.8 | 34.5 | 6.1 | 14.1 | 59 | 0.0 |
| highlat → lossy | OpenSSH | 6/6 | 59.0 | 45.8 | 7.1 | 10.0 | 134 | 0.0 |
| highlat → lossy | ET | 6/6 | 58.4 | 46.6 | 6.6 | 10.0 | 114 | 0.0 |
| highlat → lossy | mosh | 6/6 | 6.7 | 14.9 | 3.8 | 9.2 | 60 | 2.5 |

### Random screen

![Random screen FPS and lows after each link change](docs/bench/switch-random.svg)

| Change | Tool | Runs | FPS before | FPS after | 1% low after | 5% low after | Age after ms | Adapt s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| wan → verylow | rsh (UDP) | 6/6 | 51.9 | 4.7 | 2.5 | 3.1 | 219 | 3.0 |
| wan → verylow | rsh (TCP) | 6/6 | 50.8 | 5.3 | 4.1 | 5.5 | 421 | 1.0 |
| wan → verylow | OpenSSH | 6/6 | 59.2 | 2.5 | 1.6 | 2.4 | 7015 | 5.0 |
| wan → verylow | ET | 6/6 | 59.0 | 2.5 | 1.6 | 1.8 | 6922 | 5.0 |
| wan → verylow | mosh | 6/6 | 18.7 | 5.3 | 3.9 | 4.0 | 950 | 1.0 |
| lan → highlat | rsh (UDP) | 6/6 | 59.1 | 56.1 | 34.8 | 45.5 | 160 | 1.0 |
| lan → highlat | rsh (TCP) | 6/6 | 59.4 | 46.0 | 4.3 | 44.2 | 164 | 2.0 |
| lan → highlat | OpenSSH | 6/6 | 59.0 | 59.1 | 6.1 | 49.8 | 153 | 1.0 |
| lan → highlat | ET | 6/6 | 59.0 | 59.0 | 4.9 | 49.5 | 153 | 1.0 |
| lan → highlat | mosh | 6/6 | 46.0 | 10.0 | 6.5 | 6.5 | 161 | 0.0 |
| lan → lowbw | rsh (UDP) | 6/6 | 59.4 | 23.1 | 7.7 | 20.3 | 102 | 1.0 |
| lan → lowbw | rsh (TCP) | 6/6 | 59.5 | 20.9 | 10.3 | 18.3 | 99 | 0.0 |
| lan → lowbw | OpenSSH | 6/6 | 59.0 | 9.9 | 3.0 | 4.0 | 3371 | 0.0 |
| lan → lowbw | ET | 5/6 | 59.1 | 9.6 | 1.9 | 1.9 | 3224 | 0.0 |
| lan → lowbw | mosh | 6/6 | 46.0 | 13.8 | 7.0 | 8.7 | 109 | 0.0 |
| lowbw → highlat | rsh (UDP) | 6/6 | 23.1 | 51.9 | 38.8 | 46.3 | 160 | 1.0 |
| lowbw → highlat | rsh (TCP) | 6/6 | 19.7 | 15.6 | 3.4 | 8.4 | 159 | 4.0 |
| lowbw → highlat | OpenSSH | 6/6 | 10.1 | 19.6 | 3.3 | 3.5 | 3826 | 3.0 |
| lowbw → highlat | ET | 6/6 | 10.2 | 13.4 | 3.3 | 3.3 | 2262 | 3.0 |
| lowbw → highlat | mosh | 6/6 | 16.6 | 7.5 | 6.4 | 6.5 | 162 | 0.0 |
| highlat → lossy | rsh (UDP) | 6/6 | 47.7 | 56.2 | 29.1 | 45.7 | 60 | 1.0 |
| highlat → lossy | rsh (TCP) | 6/6 | 34.2 | 15.6 | 3.3 | 5.5 | 67 | 0.0 |
| highlat → lossy | OpenSSH | 6/6 | 59.2 | 32.1 | 4.9 | 8.4 | 155 | 0.0 |
| highlat → lossy | ET | 6/6 | 31.0 | 36.6 | 4.9 | 9.5 | 820 | 0.0 |
| highlat → lossy | mosh | 6/6 | 6.7 | 12.5 | 2.9 | 6.1 | 61 | 2.5 |

## 9. Outage, roaming, reattachment and UDP block

Three runs each on RTT 100 ms. Outage is five seconds; roaming is watched for eight seconds. Reattachment restarts the client. UDP-block connection is watched for 15 seconds.

![Recovery and connection times for connectivity events](docs/bench/events.svg)

| Event | Tool | Observed | Time s | Same shell | Errors |
| --- | --- | --- | --- | --- | --- |
| After 5 s outage | rsh (UDP) | 3/3 | 0.06 | — | 0 |
| After 5 s outage | rsh (TCP) | 3/3 | 5.24 | — | 0 |
| After 5 s outage | OpenSSH | 3/3 | 5.11 | — | 0 |
| After 5 s outage | ET | 3/3 | 4.95 | — | 0 |
| After 5 s outage | mosh | 3/3 | 1.02 | — | 0 |
| After IP change | rsh (UDP) | 3/3 | 2.08 | — | 0 |
| After IP change | rsh (TCP) | 0/3 | — | — | 0 |
| After IP change | OpenSSH | 0/3 | — | — | 0 |
| After IP change | ET | 0/3 | — | — | 0 |
| After IP change | mosh | 3/3 | 3.10 | — | 0 |
| Client restart | rsh (UDP) | 3/3 | 0.54 | 3/3 | 0 |
| Client restart | rsh (TCP) | 3/3 | 2.25 | 3/3 | 0 |
| Client restart | OpenSSH | 3/3 | 1.60 | 0/3 | 0 |
| Client restart | ET | 3/3 | 2.09 | 0/3 | 0 |
| Client restart | mosh | 3/3 | 1.79 | 0/3 | 0 |
| UDP silently blocked | rsh (UDP) | 3/3 | 2.03 | — | 0 |
| UDP silently blocked | rsh (TCP) | 3/3 | 2.03 | — | 0 |
| UDP silently blocked | OpenSSH | 3/3 | 1.38 | — | 0 |
| UDP silently blocked | ET | 3/3 | 1.88 | — | 0 |
| UDP silently blocked | mosh | 0/3 | — | — | 3 |

Restart time alone does not establish session continuity: see Same shell. A missing roaming response within eight seconds is a censored observation, not a measured infinite recovery time. All three UDP-block mosh jobs reported no prompt, consistent with its UDP-only transport.

## 10. Moving network

This is the later development-build dataset, not pooled into the static comparison. Three independent links replay a common delay/rate/outage schedule on the Docker VM clock. Each 60 s scene contains two 30 s cycles. Nominal loss fractions are shared intervals of complete outage, unlike static independent per-packet random loss.

| Cycle seconds | Phase | One-way delay / jitter ms | Outage time % | Down kbit/s | Up kbit/s |
| --- | --- | --- | --- | --- | --- |
| 0–6 | clear signal | 35 ± 5 | 0.1 | 8000 | 2000 |
| 6–10 | congestion | 70 ± 20 | 0.8 | 2000 | 500 |
| 10–14 | weak signal | 160 ± 60 | 3 | 512 | 128 |
| 14–17 | deep fade | 300 ± 120 | 8 | 128 | 32 |
| 17–18 | handover: link down | 180 ± 60 | 100 | 128 | 32 |
| 18–23 | recovering | 90 ± 30 | 1 | 1000 | 250 |
| 23–30 | clear again | 40 ± 8 | 0.1 | 5000 | 1250 |

Delay jitter is sampled every 250 ms; the ± column gives the configured jitter parameter, not a hard bound. The committed trace includes exact scheduled values and actual application timestamps.

![Moving-link displayed FPS and frame-gap lows](docs/bench/moving.svg)

| Scene | Tool | Avg FPS | 1% low | 5% low | Age ms | Scene RX MiB | Reported KiB/frame | Client RSS MiB | Server RSS MiB |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Dashboard | OpenSSH | 29.05 | 2.68 | 13.33 | 117 | 13.55 | 7.91 | 7.82 | 19.80 |
| Dashboard | mosh | 14.48 | 2.55 | 5.75 | 57 | 0.54 | 0.63 | 14.30 | 10.68 |
| Dashboard | rsh (UDP) | 37.72 | 4.74 | 19.23 | 58 | 1.40 | 0.63 | 3.30 | 4.98 |
| Random screen | OpenSSH | 15.27 | 0.91 | 4.35 | 1033 | 15.84 | 17.48 | 6.55 | 16.56 |
| Random screen | mosh | 13.58 | 3.24 | 4.17 | 66 | 3.09 | 3.88 | 12.36 | 10.50 |
| Random screen | rsh (UDP) | 30.22 | 3.19 | 10.53 | 58 | 7.79 | 4.39 | 2.94 | 4.61 |

The demo's scene RX counters include warmup/drain outside the common FPS window; reported KiB/frame also uses the collector's broader frame count. They are preserved as recorded and must not be read as 60-second RX rates or compared directly with static KiB/frame. RSS uses the demo's recorded samples rather than a peak-memory trace.

Dashboard rsh averaged 58.0 / 57.6 FPS in the final five seconds of each clear phase. Its first full one-second window at ≥54 frames completed 1.2 / 2.45 s after those phases began. Random screens averaged 56.2 / 56.4 FPS in those final windows. Maximum application skew between containers was 26.75 / 32.91 ms (dashboard/random). See the [measurement record](docs/bench/moving-recovery.md#measurement) for timing limits.

## Collection failures and limitations

| Source | Test | Link | Scene | Tool | Iteration | Error |
| --- | --- | --- | --- | --- | --- | --- |
| run-20261028-183821 | flood | badmobile | — | rsh (TCP) | 2 | no prompt |
| run-20261028-183821 | udpblock | wan | — | mosh | 0 | no prompt |
| run-20261028-183821 | udpblock | wan | — | mosh | 1 | no prompt |
| run-20261028-183821 | udpblock | wan | — | mosh | 2 | no prompt |
| run-20261028-183821 | switch | lan → lowbw | bandwidth | ET | 2 | no prompt |
| run-20261028-183821 | echo | badmobile | — | rsh (TCP) | 2 | no prompt |
| run-20261028-184730 | switch | lan → lowbw | compress | ET | 2 | no prompt |

All 540 static render jobs completed. Two of 300 switch jobs lacked a prompt; their groups contain five measured runs rather than six. Other tests use the first main batch only. Of seven collection failures, three are the expected mosh UDP-block no-prompt cases. No failed job was silently converted to a numeric result.

Results cover synthetic isolated paths on one shared host, not fairness with cross traffic, real radio traces, independent hardware replications or statistical confidence intervals. Static nominal loss percentages do not guarantee equal realized drops across protocols. High average FPS can coexist with worse tails: for example, the static random-screen `badmobile` case favors mosh over rsh in both average delivery and 1% low. No universal win or non-regression claim is made.

## Data and reproduction

| Source run | Jobs | Used for | Original SHA-256 |
| --- | --- | --- | --- |
| run-20261028-183821.json | 750 | render, switch + other tests | 8b465f5ac31a9a7df3979b2a396db6373f06fb8f01f014df123e706998384d70 |
| run-20261028-184730.json | 420 | render, switch | 0aef9de0c63fcc6a8e97c1cd4e42161196e45bf5c72de9eedb47d033f6c3b073 |
| run-20261028-174440.json | 8 | ten sessions | e73400cf384269ec7f68d7593822876fea7ebfb95e68605ae2dda8495c8847ed |

The [static snapshot](docs/bench/data/static-20261028.json) contains all three batches, original options, per-job results, frame gaps and errors. The [moving snapshot](docs/bench/data/moving-20261029.json) includes all six results, window timestamps, RSS/byte counters and the complete replay trace. SHA-256 fields identify the original local files; JSON is repackaged without changing measurement values.

Regenerate this report, the README tables and all charts from committed data:

```sh
python3 benchmark/lab/report.py
```

Run a new static comparison with the original workload settings:

```sh
scripts/dist.sh
python3 benchmark/harness/run.py build
python3 benchmark/lab/lab.py run --tools ssh,et,mosh,rsh,rsh-tcp \
  --tests render,switch,echo,flood,outage,roam,reattach,udpblock \
  --nets lan,wan,lowbw,verylow,slow64k,highlat,lossy,mobile,badmobile \
  --switches lan:highlat,lan:lowbw,lowbw:highlat,highlat:lossy,wan:verylow \
  --scenes compress,bandwidth --fps 60 --duration 8 --iterations 3 \
  --cols 100 --rows 30 --parallel 10
# Repeat render,switch only with the same options for six screen samples.
python3 benchmark/lab/lab.py run --tools ssh,et,mosh,rsh \
  --tests sessions --sessions 10 --iterations 2 --nets lan
```

These commands run the current installed lab image, not the unidentified
historical binary. For cleaner host-timing experiments reduce `--parallel`;
that changes the original conditions. Moving demos need a 158×43 terminal
to reproduce the recorded viewer size:

```sh
python3 benchmark/lab/lab.py demo --tools ssh,mosh,rsh --events '' \
  --nets moving --scenes compress,bandwidth --fps 60 --duration 60
```

## References

The [README references](README.md#references) identify the papers and
protocol specifications considered for rsh's sender/receiver design,
with links to the corresponding implementation. Those publications do
not supply the measurements in this report. The benchmark uses its own
Docker/netem/terminal collector described above.
