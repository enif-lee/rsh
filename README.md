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

Version **0.2.0** supports Linux and macOS on x86_64 and arm64. It uses UDP
with TCP fallback on the same port.

<!-- benchmark-summary:start -->
### Performance at a glance

Freshly measured **2026-10-29**, using **rsh 0.2.0**
(build `6729d398d4f1`). The table uses a 256 kbit/s link at RTT 100 ms,
except the ten-session memory row. Rendering requests 60 FPS in a
100×30 terminal; four jobs run in parallel on the shared host.

| Measurement | rsh (UDP) | mosh | OpenSSH | Eternal Terminal |
| --- | --- | --- | --- | --- |
| Dashboard displayed FPS (average) | 57.8 | 15.7 | 6.0 | 6.1 |
| Dashboard displayed FPS (median) | 59.9 | 15.7 | 6.0 | 6.1 |
| 1% / 5% low FPS | 39.4 / 45.0 | 14.1 / 14.9 | 2.6 / 4.9 | 3.5 / 4.2 |
| Frame age, ms | 70 | 82 | 3783 | 3762 |
| Received network, kbit/s | 218 | 87 | 248 | 249 |
| Key echo, ms | 7 | 8 | 120 | 111 |
| Bulk output to prompt, s | 0.16 | 0.22 | 23.11 | 22.74 |
| Ten idle LAN sessions, client + server MiB | 33.2 | 242.5 | 221.9 | 210.2 |

Six render attempts; pooled frame-gap lows; three echo/bulk attempts; two memory
runs. The [full report](BENCHMARK.md) includes rsh TCP fallback, all links,
raw measurements, aggregation rules and collection failures.
<!-- benchmark-summary:end -->

## Installation

```sh
curl -fsSL https://raw.githubusercontent.com/enif-lee/rsh/main/install.sh | sh
```

The installer verifies the release's SHA-256 checksum, installs
`~/.rsh/bin/rsh`, and creates `~/.local/bin/rsh`. Add `~/.local/bin` to
`PATH` if needed. To install a specific release:

```sh
curl -fsSL https://raw.githubusercontent.com/enif-lee/rsh/main/install.sh | RSH_VERSION=v0.2.0 sh
```

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

Distribution builds rebuild the `rsh` release package and verify each binary's
source build ID before writing checksums. Check an externally built binary
against its source tree with `scripts/check-build.sh /path/to/source /path/to/binary`.
The check reads the embedded ID, so the binary can target another platform.

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

These comparisons were freshly measured on **2026-10-29** with the
published **0.2.0** binary, build `6729d398d4f1`.
Avg FPS is the mean of six per-run frame rates; 1%/5% lows are reciprocals
of pooled p99/p95 frame gaps. Network is client RX including protocol
overhead; RSS is a sampled resident size, not a peak. See
[methodology and version scope](BENCHMARK.md#measurement-scope).

### Dashboard on slow, distant and lossy links

![Dashboard frame rates with 1% and 5% lows for all five transports](docs/bench/fps-dashboard.svg)

| Link | Tool | Avg FPS | 1% low | 5% low | Age ms | RX kbit/s | Client RSS MiB | Server RSS MiB |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1 Mbit/s · RTT 100 ms | rsh (UDP) | 60.0 | 41.8 | 46.7 | 60 | 218 | 2.45 | 5.21 |
| 1 Mbit/s · RTT 100 ms | OpenSSH | 23.7 | 6.8 | 7.1 | 2542 | 976 | 7.51 | 19.22 |
| 1 Mbit/s · RTT 100 ms | mosh | 17.9 | 16.8 | 17.1 | 66 | 98 | 14.36 | 10.48 |
| 256 kbit/s · RTT 100 ms | rsh (UDP) | 57.8 | 39.4 | 45.0 | 70 | 218 | 2.45 | 5.30 |
| 256 kbit/s · RTT 100 ms | OpenSSH | 6.0 | 2.6 | 4.9 | 3783 | 248 | 7.39 | 19.36 |
| 256 kbit/s · RTT 100 ms | mosh | 15.7 | 14.1 | 14.9 | 82 | 87 | 14.37 | 10.43 |
| RTT 300 ms | rsh (UDP) | 59.9 | 42.7 | 47.2 | 155 | 217 | 2.44 | 5.24 |
| RTT 300 ms | OpenSSH | 60.0 | 23.4 | 47.8 | 152 | 2502 | 7.44 | 18.49 |
| RTT 300 ms | mosh | 6.6 | 6.3 | 6.4 | 159 | 37 | 14.48 | 10.28 |
| 2% loss · RTT 100 ms | rsh (UDP) | 59.6 | 42.4 | 47.2 | 55 | 420 | 2.45 | 5.21 |
| 2% loss · RTT 100 ms | OpenSSH | 32.0 | 4.9 | 7.3 | 1782 | 1297 | 7.51 | 19.39 |
| 2% loss · RTT 100 ms | mosh | 17.2 | 3.7 | 17.6 | 60 | 102 | 14.53 | 10.66 |
| 256 kbit/s · RTT 400 ms · 2% loss | rsh (UDP) | 32.1 | 1.8 | 36.8 | 226 | 174 | 2.93 | 5.49 |
| 256 kbit/s · RTT 400 ms · 2% loss | OpenSSH | 5.2 | 1.7 | 1.8 | 3882 | 249 | 7.52 | 19.33 |
| 256 kbit/s · RTT 400 ms · 2% loss | mosh | 4.3 | 1.2 | 4.6 | 233 | 25 | 14.34 | 10.36 |

### Random full screen

All cells change each frame. These results expose frame-gap and bandwidth
limits that a compressible dashboard can hide.

![Random-screen frame rates with 1% and 5% lows](docs/bench/fps-random.svg)

| Link | Tool | Avg FPS | 1% low | 5% low | Age ms | RX kbit/s | Client RSS MiB | Server RSS MiB |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1 Mbit/s · RTT 100 ms | rsh (UDP) | 24.1 | 13.2 | 20.4 | 100 | 931 | 2.46 | 5.44 |
| 1 Mbit/s · RTT 100 ms | OpenSSH | 10.1 | 6.2 | 6.7 | 3460 | 978 | 7.40 | 19.58 |
| 1 Mbit/s · RTT 100 ms | mosh | 16.2 | 15.1 | 15.4 | 106 | 701 | 14.59 | 10.47 |
| RTT 300 ms | rsh (UDP) | 56.1 | 26.5 | 42.9 | 156 | 2709 | 2.57 | 5.46 |
| RTT 300 ms | OpenSSH | 60.0 | 14.3 | 46.9 | 153 | 5795 | 7.82 | 18.63 |
| RTT 300 ms | mosh | 6.6 | 6.3 | 6.4 | 161 | 282 | 14.32 | 10.19 |
| 2% loss · RTT 100 ms | rsh (UDP) | 57.1 | 24.1 | 44.8 | 56 | 5240 | 2.69 | 5.56 |
| 2% loss · RTT 100 ms | OpenSSH | 13.6 | 3.3 | 4.9 | 3205 | 1095 | 7.50 | 19.82 |
| 2% loss · RTT 100 ms | mosh | 14.9 | 3.7 | 6.1 | 62 | 794 | 14.45 | 10.90 |
| 256 kbit/s · RTT 400 ms · 2% loss | rsh (UDP) | 4.2 | 1.0 | 1.6 | 370 | 196 | 2.69 | 5.74 |
| 256 kbit/s · RTT 400 ms · 2% loss | OpenSSH | 2.3 | 0.6 | 1.0 | 4146 | 250 | 7.47 | 19.51 |
| 256 kbit/s · RTT 400 ms · 2% loss | mosh | 4.1 | 1.6 | 2.3 | 385 | 188 | 14.42 | 10.42 |

### Network and memory

![Received KiB per displayed dashboard frame](docs/bench/network.svg)

![Client and server RSS for ten idle sessions](docs/bench/memory-sessions.svg)

The [full benchmark report](BENCHMARK.md) includes client/server RSS and CPU
for both workloads on every link, packet counts, all five transports,
connection/echo, bulk output, link changes and session recovery, each with
comparison tables and charts. Raw job results are committed with the report.

### Moving network

The same **0.2.0** binary was measured with a **158×43 viewer**, for
60 s per scene and two shared 30 s impairment cycles. The three workload
panes are narrower than the viewer. These results are separate from the
static tests.

![Moving-link average FPS with 1% and 5% lows](docs/bench/moving.svg)

rsh displayed **35.73 FPS** across the dashboard trace and **30.88 FPS**
across the random-screen trace. The dashboard reached **58.4 / 58.6 FPS**
in the final five seconds of the two clear phases. The
[moving comparison](BENCHMARK.md#10-moving-network) includes SSH/mosh,
lows, scene traffic and RSS. Shared scheduling still recorded up to
27.47 ms of application skew between containers.

Performance varies by workload, host scheduling and loss sequence.
No uniform FPS or 1%/5% low advantage is claimed. These synthetic tests do not establish fairness with competing
flows or performance on every real radio network.
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
  --scenes compress,bandwidth --fps 60 --duration 8 --iterations 3 --parallel 4
```

Published benchmarks always use four parallel jobs (the default), with
three repeats per case. Development checks use `--iterations 1 --parallel 8`;
their results are kept separate from published comparisons.
The `sessions` test defaults to one repeat; `--session-iterations` overrides it.

The [twelve-worker validation](docs/bench/parallel12-20261029.md) shortened
the matched 750-job batch from 33.17 to 11.02 minutes. It compares the same
three repeats per case: mean FPS is close on lossless rsh UDP links, while
frame-gap lows, lossy-link results and RSS vary between runs.

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
NO_COLOR= cargo test --release --locked
python3 -m unittest discover -s benchmark/lab -p 'test_*.py' -v
```

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
