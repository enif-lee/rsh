<p align="center">
  <img src="docs/assets/rsh-icon.png" alt="rsh terminal icon" width="128" height="128">
</p>

<h1 align="center">rsh</h1>

<p align="center">
  <a href="https://github.com/enif-lee/rsh/releases/latest"><img src="https://img.shields.io/github/v/release/enif-lee/rsh" alt="Release"></a>
  <a href="#installation"><img src="https://img.shields.io/badge/platforms-linux%20%7C%20macOS-lightgrey" alt="Platforms"></a>
  <a href="#how-it-works"><img src="https://img.shields.io/badge/transport-UDP%20%2B%20TCP%20fallback-blue" alt="Transport"></a>
</p>

<p align="center"><strong>A remote shell for unreliable networks.</strong></p>

rsh keeps your shell on the server so you can detach, reconnect, or change
networks and return to the same session. It predicts keystrokes locally,
adapts output to the available link, and sends the current screen when
sending every byte would leave you behind.

rsh supports Linux and macOS on x86_64 and arm64. It uses UDP with TCP
fallback on the same port.

<!-- benchmark-summary:start -->
### Performance at a glance

A 256 kbit/s link at RTT 100 ms, requesting 60 FPS in a 100×30 terminal.
Four jobs run in parallel; each case has two evaluations. The memory row
uses ten simultaneous idle LAN sessions.

| Measurement | **rsh (UDP)** | **vs SSH** | mosh | OpenSSH | Eternal Terminal |
| --- | --- | --- | --- | --- | --- |
| Dashboard observed FPS (average) | **57.0** | **9.42×** | 15.6 | 6.0 | 6.1 |
| Dashboard observed FPS (median) | **57.0** | **9.42×** | 15.6 | 6.0 | 6.1 |
| 1% / 5% low FPS | **39.4 / 44.2** | **9.52× / 8.63×** | 13.9 / 14.8 | 4.1 / 5.1 | 4.2 / 4.3 |
| Frame age, ms | **72** | **52.17×** | 84 | 3782 | 3726 |
| Received network, kbit/s | **211** | **1.18×** | 88 | 248 | 248 |
| Packets per frame (in + out) | **2.01** | **2.34×** | 1.61 | 4.71 | 3.65 |
| Key echo, ms | **7** | **17.42×** | 8 | 120 | 112 |
| Bulk output to prompt, s | **0.16** | **143.94×** | 0.19 | 23.03 | 22.71 |
| Ten idle LAN sessions, client + server MiB | **33.7** | **6.55×** | 243.1 | 220.6 | 201.4 |

For FPS, **vs SSH** is rsh/SSH. For age, RX, packets, echo, bulk time and RSS it is
SSH/rsh: a larger factor favors rsh, while a factor below 1 favors SSH.
The 1%/5% factors are separate. Resource ratios describe usage, not speed;
ratios use unrounded statistics. Packets per frame is the median of each
run's client inbound plus outbound packet rate divided by observed FPS.
[Full results and methodology](BENCHMARK.md)
include both rsh transports, every profile, CPU/RSS and collection failures.
<!-- benchmark-summary:end -->

## Installation

```sh
curl -fsSL https://raw.githubusercontent.com/enif-lee/rsh/main/install.sh | sh
```

The installer verifies the release's SHA-256 checksum, installs
`~/.rsh/bin/rsh`, and creates `~/.local/bin/rsh`. Add `~/.local/bin` to
`PATH` if needed.

On the first connection, rsh uses your SSH access to install the matching
server binary and save an authentication token. Allow UDP port **2222**
for the normal transport, or TCP port **2222** for fallback. SSH is needed
for initial setup and server installation or upgrades.

Installed clients check for a new release once a day when connecting.
Servers are brought to the client's build on connection, with live session
handoff when supported. Set `RSH_NO_UPDATE=1` to disable client auto-update.

### Build from source

Requires Rust and Docker with support for the Linux architectures you build.

```sh
git clone https://github.com/enif-lee/rsh.git
cd rsh
scripts/dist.sh
./target/release/rsh user@host
```

`scripts/dist.sh` writes platform binaries and checksums to `~/.rsh/dist`.
Rebuild them after changing the source so the client can install a matching
build on remote hosts. Binaries stored outside `~/.rsh/bin` do not
auto-update themselves.

## Usage

```sh
rsh user@host                         # open a shell or choose a live session
rsh user@host work                    # open or resume the session named work
rsh user@host -- make test            # run a command and return its exit status
rsh user@host work --tmux             # join or create a remote tmux session
rsh user@host -L 8080:localhost:80     # forward a local port
rsh user@host -R 8080:localhost:80     # forward a remote port
rsh ls                               # list and switch sessions across hosts
```

Detach with `~.` and reconnect by session name. Sessions remain on the
server until their shell exits or the server is stopped; they are not
saved across host reboots. UDP sessions can follow a client to a new IP
address. TCP fallback provides connectivity on networks that block UDP,
but does not provide the same roaming behavior.

Type these keys at the start of a line:

| Key | Action |
| --- | --- |
| `~.` | Detach and leave the session running |
| `~l` | Open the session list |
| `~r` | Request a repaint |
| `~h` | Show escape-key help |
| `~~` | Send a literal tilde |

Use `rsh --help` for connection, server and forwarding options.

## Configuration

`~/.rsh/config.yaml`:

```yaml
compress: always
clipboard_read: false
```

`compress` accepts `always` (the default), `adaptive`, or `none`.
`--compress` overrides the file setting. `clipboard_read: false` blocks
remote OSC 52 clipboard reads; the default is `true`.

| Environment variable | Purpose |
| --- | --- |
| `RSH_NO_UPDATE=1` | Disable automatic client updates |
| `RSH_PREDICTION=never` or `always` | Override adaptive local echo prediction |
| `RSH_ESCAPE_KEY` | Change the escape character; an empty value disables it |
| `RSH_AUTH` | Use another client token file instead of `~/.rsh/known` |
| `RSH_PEER` | Connect directly to an existing server at `HOST:PORT` |

## How it works

One `rsh serve` process holds the host's sessions and PTYs. The server
maintains a terminal model, while the client maintains a mirror for local
echo prediction and rendering.

```mermaid
flowchart LR
  Keys[Keyboard] --> Predict[Local echo prediction]
  Keys --> Input[Reliable input stream]
  Input --> Shell[Remote shell / PTY]
  Shell --> Model[Server terminal model]
  Model --> Output[Output bytes or screen differences]
  Output --> Transport[UDP / TCP fallback]
  Transport --> Terminal[Client terminal]
  Predict --> Terminal
  Terminal --> Feedback[Arrival reports]
  Feedback --> Path[Path model]
  Path --> Output
```

- **Output:** by default, while the link keeps up, the program's bytes
  travel as a deflate stream, preserving scrollback and terminal sequences. When the
  link falls behind, screen differences replace intermediate output.
- **Adaptation:** arrival reports measure capacity, delay, queueing and
  loss. The server uses them for its send window, output pacing and
  loss-protection copies. Sparse traffic can use short measurement copies
  when recent packet-pair measurements are unavailable.
- **Rendering:** the client combines output into synchronized terminal
  updates and provides adaptive local echo prediction.
- **Resources:** UDP sessions share a server process and socket. Quiet
  terminal history and buffers are compacted, and idle keepalives slow
  down after inactivity.
- **Authentication:** SSH bootstraps per-client tokens. Each attachment
  uses X25519 key exchange and XChaCha20-Poly1305 encryption.

The implementation is in [`src/server.rs`](src/server.rs),
[`src/client.rs`](src/client.rs), [`src/path.rs`](src/path.rs),
[`src/proto.rs`](src/proto.rs), and [`src/terminal.rs`](src/terminal.rs).

<!-- benchmark-details:start -->
## Performance

Average FPS is the mean of the two per-run rates;1%/5% lows are reciprocals
of pooled p99/p95 frame gaps. Network is client RX including protocol overhead;
RSS is a sample, not peak memory. Frame observers differ across transports.
See the [measurement scope](BENCHMARK.md#measurement-scope).

### Dashboard on slow, distant and lossy links

![Dashboard FPS and lows for all five transports](docs/bench/v0.3.0/fps-dashboard.svg)

| Link | Tool | Avg FPS | 1% low | 5% low | Age ms | RX kbit/s | Client RSS MiB | Server RSS MiB |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1 Mbit/s · RTT 100 ms | rsh (UDP) | 60.0 | 41.2 | 46.1 | 60 | 218 | 2.49 | 5.03 |
| 1 Mbit/s · RTT 100 ms | OpenSSH | 23.6 | 6.6 | 7.2 | 2542 | 977 | 7.51 | 19.27 |
| 1 Mbit/s · RTT 100 ms | mosh | 18.0 | 16.8 | 17.1 | 66 | 98 | 14.53 | 10.46 |
| 256 kbit/s · RTT 100 ms | rsh (UDP) | 57.0 | 39.4 | 44.2 | 72 | 211 | 2.52 | 5.55 |
| 256 kbit/s · RTT 100 ms | OpenSSH | 6.0 | 4.1 | 5.1 | 3782 | 248 | 7.38 | 19.31 |
| 256 kbit/s · RTT 100 ms | mosh | 15.6 | 13.9 | 14.8 | 84 | 88 | 14.35 | 10.43 |
| RTT 300 ms | rsh (UDP) | 59.0 | 36.2 | 45.5 | 156 | 216 | 2.52 | 5.07 |
| RTT 300 ms | OpenSSH | 59.9 | 23.1 | 48.5 | 152 | 2502 | 7.44 | 18.46 |
| RTT 300 ms | mosh | 6.6 | 6.3 | 6.4 | 161 | 37 | 14.55 | 10.31 |
| 2% loss · RTT 100 ms | rsh (UDP) | 58.6 | 39.4 | 46.5 | 56 | 414 | 2.63 | 5.59 |
| 2% loss · RTT 100 ms | OpenSSH | 33.6 | 5.0 | 9.6 | 1892 | 1373 | 7.52 | 19.31 |
| 2% loss · RTT 100 ms | mosh | 17.9 | 3.8 | 17.8 | 61 | 103 | 14.82 | 10.55 |
| 256 kbit/s · RTT 400 ms · 2% loss | rsh (UDP) | 29.0 | 1.7 | 32.3 | 230 | 174 | 2.95 | 5.71 |
| 256 kbit/s · RTT 400 ms · 2% loss | OpenSSH | 6.0 | 1.0 | 1.7 | 4401 | 249 | 7.58 | 19.50 |
| 256 kbit/s · RTT 400 ms · 2% loss | mosh | 4.5 | 1.2 | 4.6 | 234 | 26 | 14.44 | 10.39 |

### Random full screen

Every cell changes each frame, exposing bandwidth and frame-gap limits.

![Random-screen FPS and lows](docs/bench/v0.3.0/fps-random.svg)

| Link | Tool | Avg FPS | 1% low | 5% low | Age ms | RX kbit/s | Client RSS MiB | Server RSS MiB |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1 Mbit/s · RTT 100 ms | rsh (UDP) | 23.8 | 12.1 | 20.0 | 100 | 916 | 2.52 | 5.74 |
| 1 Mbit/s · RTT 100 ms | OpenSSH | 10.1 | 6.4 | 6.8 | 3496 | 979 | 7.40 | 19.59 |
| 1 Mbit/s · RTT 100 ms | mosh | 16.2 | 15.0 | 15.3 | 106 | 700 | 14.49 | 10.48 |
| RTT 300 ms | rsh (UDP) | 57.4 | 27.6 | 44.2 | 157 | 2710 | 2.64 | 5.75 |
| RTT 300 ms | OpenSSH | 60.0 | 13.6 | 46.7 | 152 | 5794 | 7.80 | 18.62 |
| RTT 300 ms | mosh | 6.6 | 6.3 | 6.4 | 162 | 283 | 14.32 | 10.19 |
| 2% loss · RTT 100 ms | rsh (UDP) | 54.2 | 9.2 | 42.9 | 57 | 5104 | 2.89 | 5.87 |
| 2% loss · RTT 100 ms | OpenSSH | 26.2 | 4.6 | 6.1 | 1658 | 2548 | 7.64 | 19.15 |
| 2% loss · RTT 100 ms | mosh | 15.1 | 3.1 | 6.0 | 62 | 797 | 14.38 | 10.91 |
| 256 kbit/s · RTT 400 ms · 2% loss | rsh (UDP) | 5.2 | 1.5 | 2.2 | 367 | 219 | 2.60 | 5.42 |
| 256 kbit/s · RTT 400 ms · 2% loss | OpenSSH | 2.7 | 0.3 | 0.9 | 4386 | 251 | 7.44 | 19.69 |
| 256 kbit/s · RTT 400 ms · 2% loss | mosh | 4.0 | 2.2 | 2.3 | 382 | 184 | 14.43 | 10.43 |

### Network and memory

![Received KiB per displayed dashboard frame](docs/bench/v0.3.0/network.svg)

![Client and server RSS for ten idle sessions](docs/bench/v0.3.0/memory-sessions.svg)

The [full benchmark report](BENCHMARK.md) includes CPU/RSS, packets, connection,
echo, bulk output, link changes and recovery, with all attempts retained.

### Moving network

Each evaluation contains two common 30-second impairment cycles over 60 seconds.
Four sessions use 100×30 workload PTYs. These observations are separate from
static tests and from the other rsh-transport cohort.

![Moving-link FPS and lows, each evaluation separate](docs/bench/v0.3.0/moving.svg)

Dashboard rsh averages: **41.17 / 32.17 FPS** across the two evaluations.

Random screen rsh averages: **28.40 / 28.07 FPS** across the two evaluations.

High post-outage FPS can include backlog catch-up. Frame age, gaps and replay
application timestamps remain in the [moving comparison](BENCHMARK.md#10-moving-network).
Moving RSS uses the larger of midpoint/end samples; scene traffic includes
warmup/drain. Synthetic links do not establish performance on every real radio
network or fairness with competing traffic.
<!-- benchmark-details:end -->

## Benchmark lab

The lab compares tools in separate Docker server/client pairs, each with
its own shaped link. Build the test image first:

```sh
scripts/dist.sh
python3 benchmark/harness/run.py build
```

Run render and link-switch tests:

```sh
python3 benchmark/lab/lab.py run --tools ssh,mosh,rsh \
  --tests render,switch --nets lan,wan,lowbw,highlat \
  --switches lan:lowbw,lowbw:highlat \
  --scenes compress,bandwidth --fps 60 --duration 8 --iterations 2 --parallel 4
```

Performance measurements use four parallel jobs and two evaluations per case,
including session memory, in both development and published comparisons.
These defaults reflect the measurement host's four performance cores; they do
not pin processes to physical cores. Historical measurements retain their
recorded concurrency and repeat counts and are kept separate from new runs.
`--session-iterations` overrides the session-memory repeat count.

Run the moving-network demo:

```sh
python3 benchmark/lab/lab.py demo --tools ssh,mosh,rsh --events '' \
  --nets moving --scenes compress,bandwidth --fps 60 --duration 60
```

To compare two Linux builds, pass `--rsh /path/to/baseline` and
`--rsh-b /path/to/candidate`. Select `rsh,rsh-b` for UDP or
`rsh-tcp,rsh-b-tcp` for TCP. Both `run` and `demo` install the requested
builds; the second uses port 2223. For example, a four-pane moving demo:

```sh
python3 benchmark/lab/lab.py demo \
  --rsh /path/to/baseline --rsh-b /path/to/candidate \
  --tools ssh,mosh,rsh-tcp,rsh-b-tcp --events '' \
  --nets moving --scenes compress,bandwidth --fps 60
```

The moving profile repeats a 30-second sequence of congestion, weak
signal, a one-second handover outage and recovery. All sessions use the
same intended delay/rate/outage schedule on the Docker VM clock. Each
session has an isolated link with the same configured capacity. Its loss percentages describe shared intervals
of link outage, not independent per-packet random-loss probabilities.

The top header shows the active profile. Each session header shows the
program, FPS, displayed-frame age and received network traffic. Frame age
is not a network RTT measurement. Actual application timestamps are saved
because OS scheduling can introduce differences between containers.

Results are saved under `benchmark/lab/out/`. Use `lab.py --help` for all
profiles and tests. Stop a demo with **Ctrl-b X**, or from another shell:

```sh
python3 benchmark/lab/lab.py stop
```

## Development

```sh
(ulimit -n 4096 && env -u NO_COLOR cargo test --locked --all-targets)
python3 -m unittest discover -s benchmark/lab -p 'test_*.py' -v
```

The Rust suite opens 300 idle TCP connections. The subshell sets its file
descriptor limit and enables the color-dependent UI assertions for this run.

Container integration tests are available in
[`tests/container`](tests/container); read each script's setup requirements
before running it. CPU profiling tools are in
[`benchmark/profile`](benchmark/profile).

## References

The sender/receiver design draws on work on interactive state delivery,
receiver arrival feedback and congestion estimation. The links below
record those influences and the corresponding mechanisms in this code.
rsh uses its own wire protocol and control rules; the mappings describe
ideas, not protocol compatibility or complete implementations of the
referenced algorithms.

| Reference | Connection to rsh |
| --- | --- |
| Winstein and Balakrishnan, [Mosh: An Interactive Remote Shell for Mobile Clients](https://www.usenix.org/conference/atc12/technical-sessions/presentation/winstein), USENIX ATC 2012 | Predictive local echo and sending current terminal state when intermediate output becomes obsolete. See [client prediction](src/client.rs) and [server screen delivery](src/server.rs). rsh also carries a byte stream while the link keeps up. |
| Smith et al., [SQP: Congestion Control for Low-Latency Interactive Video Streaming](https://research.google/pubs/sqp-congestion-control-for-low-latency-interactive-video-streaming/), 2022 | Measuring an application-limited link with the application's own bursts. [PathModel](src/path.rs) estimates rate from packet-train bytes and receiver arrival spacing, with extra handling for sparse terminal updates. |
| Fouladi et al., [Salsify: Low-Latency Network Video through Tighter Integration between a Video Codec and a Transport Protocol](https://www.usenix.org/conference/nsdi18/presentation/fouladi), NSDI 2018 | Jointly considering content production and transmission budget. The [server](src/server.rs) substitutes current-screen differences for obsolete queued output. rsh does not use Salsify's video codec or per-frame encoding search. |
| [WebRTC transport-wide congestion feedback](https://webrtc.googlesource.com/src/+/refs/heads/main/docs/native-code/rtp-hdrext/transport-wide-cc-02/README.md) and [Google Congestion Control draft](https://datatracker.ietf.org/doc/html/draft-ietf-rmcat-gcc-02) | Packet numbers and receiver-side timing inform sender estimates. See [Feedback](src/proto.rs), [client arrival reports](src/client.rs) and [path estimation](src/path.rs). rsh uses delay above a recent floor and packet trains, rather than implementing GCC's full trendline controller or RTCP format. |
| Cardwell et al., [BBR: Congestion-Based Congestion Control](https://research.google/pubs/bbr-congestion-based-congestion-control/), CACM 2017 | Separating bottleneck rate, propagation delay and standing queue, and accounting for bytes in flight. See [path rate/RTT estimates](src/path.rs) and [server drain_backlog](src/server.rs). The sparse terminal workload uses its own estimator; rsh does not implement BBR's complete state machine. |
| [RFC 6298: Computing TCP's Retransmission Timer](https://datatracker.ietf.org/doc/html/rfc6298), 2011 | [Sender](src/proto.rs) maintains smoothed RTT and RTT variation and avoids ambiguous retransmission samples (Karn's rule). The base RTO estimate is clamped to 50–1000 ms; the effective retransmission timeout can extend to 3 s for path RTT and drain time. These are application-specific bounds. |
| [RFC 5827: Early Retransmit for TCP and SCTP](https://www.rfc-editor.org/info/rfc5827/), 2010 | [Sender::on_ack](src/proto.rs) reduces the duplicate-ACK threshold for thin streams with few outstanding datagrams. |
| [RFC 9002: QUIC Loss Detection and Congestion Control](https://datatracker.ietf.org/doc/html/rfc9002), 2021 | A design reference for receiver hold-time/ACK-delay accounting and packet-based loss evidence. [PathModel::feedback](src/path.rs) subtracts reported receiver hold time. rsh's retransmission timer remains its own RFC-6298-derived timer; it does not implement QUIC recovery. |

Additional work considered during design includes
[Pudica (NSDI 2024)](https://www.usenix.org/conference/nsdi24/presentation/wang-shibo)
on low queueing for interactive frames,
[Copa (NSDI 2018)](https://web.mit.edu/copa/)
on delay-based control and competition, and
[Sprout (NSDI 2013)](https://www.usenix.org/system/files/conference/nsdi13/nsdi13-final113.pdf)
on stochastic cellular delivery forecasts. Their complete controllers are
not implemented here. The [benchmark report](BENCHMARK.md) describes the
independent experiments used to evaluate rsh.
