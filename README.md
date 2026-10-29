<p align="center">
  <img src="docs/assets/rsh-icon.png" alt="rsh icon: a terminal prompt with a reconnect arrow" width="128" height="128">
</p>

<h1 align="center">rsh</h1>

<p align="center">
  <a href="Cargo.toml"><img src="https://img.shields.io/badge/rust-2021-orange?logo=rust" alt="Rust"></a>
  <a href="https://github.com/enif-lee/rsh/releases/latest"><img src="https://img.shields.io/github/v/release/enif-lee/rsh" alt="Release"></a>
  <a href="#installation"><img src="https://img.shields.io/badge/platforms-linux%20%7C%20macOS-lightgrey" alt="Platforms"></a>
  <a href="#architecture"><img src="https://img.shields.io/badge/transport-UDP%20%2B%20TCP%20fallback-blue" alt="Transport"></a>
</p>

<h3 align="center">The remote shell for bad networks.</h3>

<p align="center">
  <em>Local key prediction, persistent sessions, and a current screen when the link cannot carry every byte.<br>
  Built for 3G, ADSL, trains, planes and hotel Wi-Fi, and everyday work on a LAN.</em>
</p>

rsh opens a shell on a host the way `ssh` does, but the session lives on the server: close the laptop, change networks, or kill the client, and `rsh <host>` brings you back to the same shell. Over UDP it echoes your keys at once, measures the link as it goes, and when the link cannot carry every byte it sends the latest screen instead of a backlog. Where only TCP gets through, it falls back to TCP on the same port.

The table below is an [earlier five-tool snapshot](#earlier-five-tool-comparison).
[Latest branch measurements](#latest-branch-measurements-2026-10-29) cover recovery on a changing link.

| On a 256 kbit/s link (RTT 100 ms) | rsh | mosh | OpenSSH | Eternal Terminal |
| --- | --- | --- | --- | --- |
| A 60 fps dashboard, frames shown a second | **54.3** | 15.9 | 6.0 | 6.1 |
| Its 1% low (fps of the slowest 1% of frames) | **36.1** | 14.7 | 1.9 | 4.2 |
| How old the screen is | **76 ms** | 82 ms | 3.8 s | 3.8 s |
| A key to its echo | **4 ms** | 4 ms | 120 ms | 108 ms |
| `seq 1 100000` until the prompt is back | **0.16 s** | 0.24 s | 23 s | 23 s |
| Memory for 10 idle sessions on a LAN, clients + server | **35 MB** | 242 MB | 222 MB | 207 MB |

## Installation

```sh
curl -fsSL https://raw.githubusercontent.com/enif-lee/rsh/main/install.sh | bash
```

This puts the latest release for your machine (Linux or macOS, x86_64 or arm64), checked against the release's SHA-256 sums, at `~/.rsh/bin/rsh` and links it from `~/.local/bin/rsh`. `RSH_VERSION=v0.1.0` picks a release; running it again upgrades.

The first `rsh <host>` sets the host up over `ssh`: it copies the same build there (a binary for the host's platform comes from this version's GitHub release and is kept in `~/.rsh/dist`), starts `rsh serve` and saves a token. A host already running `rsh serve` can be used directly with `--peer <host>:2222`.

> [!NOTE]
> **Upgrades.** Once a day, when you connect, rsh looks for a newer release. If there is one it downloads the binary for your platform, checks it, replaces `~/.rsh/bin/rsh` and runs the new one with the same arguments; the next connection to each host then brings its server to the same build, in place, keeping its sessions. `RSH_NO_UPDATE=1` turns this off, and a build run from anywhere else than `~/.rsh/bin` never replaces itself.

### From source

Requires Rust (stable), and Docker for the static Linux builds that rsh copies to servers.

```sh
git clone https://github.com/enif-lee/rsh.git
cd rsh
scripts/dist.sh                         # ~/.rsh/dist/rsh-<platform> for Linux and macOS, and SHA256SUMS
install -m 755 target/release/rsh ~/.local/bin/rsh
```

A build of your own looks in `~/.rsh/dist` first for the hosts' binaries, so run `scripts/dist.sh` again after changing the source.

## Features

- **Sessions live on the server** and outlast the client: detach with `~.`, reattach by name, from another machine too. A killed client is back in the same shell in 0.5 s.
- **Roaming and outages are a notice, not a disconnect.** One UDP socket per host; a new IP address or a link that comes back is picked up within about 2 s.
- **Predictive local echo**, as in mosh, that steps aside on fast links.
- **Adapts to the link as it changes.** The server measures the bottleneck's rate, the queue in front of it, the round trip and loss from the client's arrival reports, keeps the queue short, and switches to sending screen differences when the link cannot keep up.
- **Light.** One `rsh serve` process carries every session of a host; ten sessions take 11 MB on the server and 24 MB of clients. Quiet sessions give memory back, and idle ones keep alive every 5 s.
- **Deflate-compressed output stream** by default (`--compress always|adaptive|none`).
- **TCP fallback on the same port** for networks that drop UDP.
- **Zero setup on the server**: the first connection installs the same build over `ssh`, starts `rsh serve`, saves a token. A server running another build is upgraded in place, keeping its sessions.
- `-L` and `-R` port forwarding, `--tmux` integration, a session list across hosts.

## Usage

```sh
rsh myhost                  # open a shell; with live sessions, a menu to pick one
rsh myhost work             # open or resume the session called "work"
rsh myhost -- make test     # run a command in a new session, exit with its status
rsh myhost work --tmux      # join (or create) the host's tmux session "work"
rsh myhost -L 8080:localhost:80
rsh ls                      # sessions on every known host
```

Keys, typed at the start of a line as in ssh:

| Key | Action |
| --- | --- |
| `~.` | detach; the session keeps running |
| `~l` | sessions of every known host; Enter switches |
| `~r` | repaint the screen from the server |
| `~h` | list these keys |
| `~~` | send `~` |

> [!TIP]
> `RSH_ESCAPE_KEY` sets another escape character (empty disables it), `RSH_PREDICTION=never|always` overrides local echo prediction, and `~/.rsh/config.yaml` takes `compress: always|adaptive|none` and `clipboard_read: false` (blocks OSC 52 clipboard reads by remote programs).

## Benchmarks

### Latest branch measurements (2026-10-29)

These development-build comparisons cover changing-link recovery:
checkpoint `a5bb6cae5bf0` versus `7a1d9454d5b1` (Linux arm64).
The moving demo ran OpenSSH, mosh and rsh at a requested 60 FPS in a
158×43 terminal, for 60 seconds per scene: two cycles through congestion,
weak signal, a one-second handover outage and recovery. All tools use the
same scheduled observation window and time-based impairment trace on
independent links.

| rsh workload / observation window | Checkpoint | Updated build |
| --- | ---: | ---: |
| Dashboard, whole 60 seconds | 29.23 FPS | 37.72 FPS |
| Dashboard, first clear phase, last 5 seconds (25–30) | 8.6 FPS | 58.0 FPS |
| Dashboard, second clear phase, last 5 seconds (55–60) | 58.8 FPS | 57.6 FPS |
| Random full screen, whole 60 seconds | 30.47 FPS | 30.22 FPS |

The dashboard's first clear phase previously never completed a full
one-second window with at least 54 displayed frames; the updated build
completed one 1.2 seconds after the phase began. In the second cycle the
checkpoint was already recovering faster: 1.0 versus 2.45 seconds.
Random-screen recovery was 1.2/1.8 seconds before and 1.75/1.9 after.
Recovery is therefore not uniformly faster across scenes or cycles.
The fix addresses sparse output paced at an old, low capacity estimate.
It measures actual packet pairs separately from later frames and keeps
later frame jitter from cancelling the pair's recovery evidence.

Some static loss tests also showed lower FPS. At 256 kbit/s, RTT 400 ms
and nominal 2% random loss, two 20-second dashboard checks were 26.4/35.4
FPS before and 22.8/25.4 after. The qdisc counters showed different
realized losses, so the cause of this decrease is unconfirmed; these
results do not establish non-regression on every lossy link. The lossless
variant was 47.4/56.7 versus 51.4/59.9 FPS.

The common schedule does not make actual packet outcomes or application
times identical: the moving run recorded up to 32.91 ms of application
skew between containers. These are two cycles per build on one shared host.
[Measurement details, run IDs and limitations](docs/bench/moving-recovery.md)
include the full results, timing errors and regression checks.
The [long-RTT investigation](docs/bench/backlog-fix.md) records the backlog
and coalescing fixes and the 1%/5% low comparisons; it does not establish
a consistent percentile advantage over SSH.

### Earlier five-tool comparison

The opening comparison table and the charts and numerical comparisons
below are the earlier static-link snapshot, not measurements of build
`7a1d9454d5b1`.

Same machine, same simulated links, five clients: rsh over UDP, rsh over TCP (UDP blocked), OpenSSH, Eternal Terminal (ET) and mosh. Each scene draws 60 frames a second for 8 s; values are the median of six runs for the screen tests and three for the others. The frame-rate charts carry each tool's 1% and 5% lows (the frame rate its slowest 1% and 5% of frames would give), taken over all six runs' frame gaps together: light is the frame rate, mid the 5% low, solid the 1% low.

<p align="center">
  <img src="docs/bench/fps-dashboard.svg" alt="Frames shown per second, with 1% and 5% lows, for a full-screen dashboard at 60 fps, per network and tool" width="760">
</p>

<p align="center">
  <img src="docs/bench/age.svg" alt="Median age of what is on screen for a full-screen dashboard at 60 fps, per network and tool" width="760">
</p>

- **The screen stays current.** From 1 Mbit/s down to 64 kbit/s, and at 1% to 2% loss, rsh over UDP shows a 60 fps dashboard at 48 to 60 fps on every link but 64 kbit/s (13.1) and 256 kbit/s with RTT 400 ms and 2% loss (24.2), and what is on screen is 56 to 228 ms old. At 1 Mbit/s, at 256 kbit/s and at 2% loss its slowest 1% of frames still come at 36 to 44 fps. OpenSSH and ET show 1.5 to 38 fps of frames that are 0.3 to 4.6 s old, because every byte waits its turn on the link; mosh 4.7 to 18 fps with 1% lows of 2.4 to 17, and about 47 fps even on a LAN.
- **Keys echo at once.** Local echo prediction answers a key in 4 to 7 ms on every link. OpenSSH and ET wait a round trip: 105 to 527 ms on the links with delay.
- **Big output does not block you.** `seq 1 100000` returns the prompt in 0.05 to 0.46 s on every link. OpenSSH and ET take 6 to 24 s on slow links, and more than 30 s at 64 kbit/s.
- **Sessions survive.** After a 5 s outage rsh is back 0.1 s after the link returns (mosh 1.0 s, OpenSSH and ET about 5 s). After the client moves to another IP address it carries on in 2.1 s (mosh 3.1 s; OpenSSH and ET not within the 8 s watched). A client killed and started again is back in the same shell in 0.5 s.

#### Memory

<p align="center">
  <img src="docs/bench/memory-sessions.svg" alt="Resident memory with 10 sessions at once, clients and server side, per tool" width="760">
</p>

<p align="center">
  <img src="docs/bench/memory.svg" alt="Client and server resident memory while drawing at 60 fps, per tool" width="760">
</p>

- **Ten sessions at once** (each shell having printed 3000 lines, then quiet): the ten rsh clients hold 24 MB and the server 11 MB, all ten sessions in one `rsh serve`. OpenSSH: 74 MB of clients and 148 MB of `sshd`; mosh 144 and 98 MB (a `mosh-server` per session); ET 171 and 36 MB. Idle, the ten rsh sessions cost 2 packets a second each way (mosh 3.4 to 3.6, ET 7.5 to 7.9; OpenSSH sends none).
- **One session.** While drawing at 60 fps the rsh client holds 2.4 MB (OpenSSH 7.4, mosh 14.5, ET 18.0 MB) and `rsh serve` 5.7 MB (`mosh-server` 10.4, ET 14.2, `sshd` 19.4 MB with its listener).

#### Network

<p align="center">
  <img src="docs/bench/network.svg" alt="Network data received per frame shown for a full-screen dashboard, per network and tool" width="760">
</p>

<p align="center">
  <img src="docs/bench/packets.svg" alt="Packets in and out for each frame shown for a full-screen dashboard, per network and tool" width="760">
</p>

- **Data.** A dashboard frame on screen costs rsh 0.44 to 0.51 KB on the wire on clean links and 0.59 to 0.91 KB on lossy ones (datagrams go twice there while the link has room), mosh 0.62 to 0.73 KB, and OpenSSH and ET 4.8 to 5.4 KB, as they carry every byte the program writes.
- **Packets.** On clean links rsh takes one datagram a frame and answers each; OpenSSH takes 1.9 to 3.7 packets a frame and ET 1.5 to 2.5, as TCP segments a frame's bytes; mosh takes one and answers about every second one at RTT 100 ms (every fifth on a LAN, each at RTT 300 ms). On lossy links rsh takes 1.2 to 2.0, the copies. A typed key costs rsh 1.1 to 2.0 packets in and 2.1 to 2.7 out, about as many as ET and mosh; OpenSSH 16 to 31 in and 16 to 35 out, as it sends keystroke-timing chaff.

<details>
<summary>More charts: random full screen, bulk output</summary>

<p align="center">
  <img src="docs/bench/fps-random.svg" alt="Frames shown per second, with 1% and 5% lows, for a random full screen at 60 fps" width="760">
</p>

<p align="center">
  <img src="docs/bench/flood.svg" alt="Time to finish seq 1 100000 per network and tool" width="760">
</p>

</details>

> [!NOTE]
> These come from [`benchmark/lab`](benchmark/lab): two Docker containers per client on one machine, with `tc netem` for delay, loss and rate. Real radio links add jitter and reordering that netem only partly imitates. Where rsh is behind:
> - With loss on a slow, far link, a dashboard frame now and then waits for a lost datagram's resend: at 512 kbit/s, RTT 200 ms and 1% loss rsh's 1% low is 4.9 (mosh 7.9), at 256 kbit/s, RTT 400 ms and 2% loss 1.7 (mosh 2.4), though it shows 47.8 and 24.2 fps against mosh's 9.2 and 4.7. Under a random full screen (every cell changes each frame) there mosh shows more frames (4.3 against 3.5 fps) with a higher 1% low (1.1 against 0.5), and at 64 kbit/s about as many (1.5 each) with a higher 1% low (1.4 against 1.2). At 1 Mbit/s mosh's random-screen frames come at more even gaps: a 1% low of 15.1 against 13.0, at 16.4 fps against 23.9.
> - On near links OpenSSH and ET keep a steady 60 fps where rsh's frames come a little less evenly. On a LAN their 1% lows are 50.3 on the dashboard and 49.8 to 50.0 on the random screen, against rsh's 46.1 and 41.0; at RTT 100 ms 45.7 to 45.9 and (OpenSSH) 29.1 against 44.1 and 23.7. Their 5% lows are higher on a LAN and at RTT 100 and 300 ms, and on the random screen rsh shows 56.7 to 59.7 fps there against their 60. After a switch from a LAN to RTT 300 ms rsh shows 56.1 to 56.8 fps, OpenSSH and ET 59.
> - When the link drops from RTT 100 ms to 256 kbit/s under a random full screen, rsh shows 4.7 fps afterwards (mosh 5.3) with a 1% low of 2.5 (mosh 3.9) and about 40 stalls a run (mosh 1), though its screen is 0.2 s old against mosh's 0.95 s.
> - On a LAN OpenSSH's and ET's screen is 1 to 2 ms old, rsh's 5 to 9 ms; at RTT 100 ms 51 to 52 ms against 56 to 60 ms. On a LAN ET echoes a key in 5 ms, rsh in 7 ms. On clean links the rsh client sends a packet for every frame it takes, mosh's one for every one to five.
> - rsh over TCP is back about 5 s after an outage, as OpenSSH is, and does not follow a move within the 8 s watched. Where UDP is blocked rsh connects in 2.0 s (it tries UDP first), OpenSSH in 1.4 s and ET in 1.9 s.
>
> The earlier screen tests (render, switch) come from two `lab.py run --fps 60`s of three iterations each, the other tests from the first of them; memory with 10 sessions from a separate run of the `sessions` test (two runs, median).

## Feature matrix

| | OpenSSH (sshd) | Eternal Terminal | mosh | rsh |
| --- | --- | --- | --- | --- |
| Transport | TCP | TCP | UDP | UDP, or TCP on the same port when UDP is blocked |
| Works where only TCP gets through | ✅ | ✅ | ❌ cannot connect | ✅ connects over TCP in 2.0 s |
| Client changes IP address | ❌ connection drops | ⚠️ reconnects over a new TCP connection (not within the 8 s watched) | ✅ ~3.1 s | ✅ ~2.1 s (UDP) |
| Link down for a few seconds | ⚠️ only while TCP survives its backoff (~5 s after the link) | ⚠️ reconnects (~5 s after the link) | ✅ back ~1.0 s after the link | ✅ back ~0.1 s after the link (UDP) |
| Client killed, started again | ❌ new shell | ❌ new shell | ❌ new shell | ✅ same shell in 0.5 s (`rsh <host> <name>`) |
| Local echo prediction | ❌ | ❌ | ✅ adaptive | ✅ adaptive |
| Slow link: shows the latest screen, not a backlog | ❌ | ❌ | ✅ | ✅ |
| Program's bytes reach your terminal (scrollback, OSC) | ✅ | ✅ | ❌ screen only, 5 of 18 OSC tested | ✅ 18 of 18 OSC tested; screen updates only while the link falls behind |
| Compression | ⚠️ `-C`, off by default | not checked | ✅ zlib | ✅ deflate stream, on by default |
| Port forwarding | ✅ `-L` `-R` `-D`, X11, agent | ✅ `-t` `-r`, agent | ❌ | ⚠️ `-L` `-R` |
| File transfer | ✅ scp, sftp | ❌ | ❌ | ❌ |
| List and switch sessions across hosts | ❌ | ❌ | ❌ | ✅ `rsh ls`, `~l` |
| tmux | run it yourself | run it yourself | run it yourself | ✅ `--tmux` joins or creates a tmux session |
| Server side | `sshd` | `sshd` + `etserver` daemon | `sshd` + a `mosh-server` per session (UDP 60000–61000) | one `rsh serve` for every session, port 2222, installed and upgraded over SSH by the client |
| Memory, 10 sessions (clients + server) | 222 MB | 207 MB | 242 MB | 35 MB |
| Upgrade the server without dropping sessions | n/a | not checked | n/a | ✅ hands sessions to the new binary |
| Authentication | SSH keys, passwords | via SSH | via SSH | per-client token issued over SSH; X25519 per attach, XChaCha20-Poly1305 |

Times and memory in this matrix come from the earlier five-tool lab runs above. The OSC counts come from an earlier run of [`benchmark/harness`](benchmark/harness); the other cells from each project's documentation and source.

## Architecture

rsh is one Rust binary that runs as the client (`rsh <host>`) and, on the host, as the server (`rsh serve`). Everything below is in [`src/`](src).

```mermaid
flowchart LR
  subgraph C["Client: rsh &lt;host&gt;"]
    direction TB
    KB[keys] --> PR["predictor<br/>local echo"]
    KB --> TXS["input stream<br/>reliable, ordered"]
    RX["receive<br/>kernel arrival time"] --> MIR["mirror terminal<br/>what the screen holds"]
    MIR --> PW["paced writer<br/>one synchronized update a frame"]
    PR --> PW
    PW --> TTY[your terminal]
    RX --> FB["arrival reports<br/>in every ack"]
  end
  subgraph S["Server: rsh serve, one process for every session"]
    direction TB
    PTY[shell in a PTY] --> EMU["terminal emulator<br/>screen and history"]
    EMU --> MODE{"link keeping up?"}
    MODE -- yes --> RAW["the program's bytes<br/>deflate stream"]
    MODE -- no --> DIFF["screen difference<br/>from what the client has"]
    RAW --> SND
    DIFF --> SND["sender<br/>window, ack clock, pacing,<br/>resends, copies"]
    PM["path model<br/>rate, queue, round trip, loss"] --> SND
    PM --> MODE
  end
  TXS -- "sealed datagrams, UDP :2222 (TCP on the same port when UDP is blocked)" --> PTY
  SND -- "sealed, numbered datagrams" --> RX
  FB --> PM
```

### Server

- **One process, one socket.** `rsh serve` holds every session in one process on one UDP port, polls the PTYs and the socket, and on upgrade hands all of it to the new binary (`src/upgrade.rs`) without dropping a session.
- **A terminal per session.** Each session's output goes through a terminal emulator (`alacritty_terminal`), so the server always knows the screen and the last 1000 lines: a client that attaches gets a repaint with its scrollback, and a slow link can be sent the screen instead of the backlog. Once output is quiet, history older than the last 100 lines moves out of the emulator's cells (24 bytes each) into the bytes that draw it, and an idle session gives back what its buffers grew to (`src/terminal.rs`, `src/server.rs`).
- **Two ways to send output.** While the link keeps up, the program's bytes go as they are (one deflate stream; with `--compress adaptive` only on far, slow or lagging links), so scrollback and OSC sequences reach your terminal. When they would take longer than 100 ms to drain, the server switches to *screen updates*: the difference between the screen now and the screen the client was last sent, at the program's own frame rate, until the link has kept up for a second.

### Transmission

```mermaid
flowchart LR
  OUT["output or screen update"] --> W{"window open and<br/>link reckoned free?"}
  W -- wait --> W
  W -- go --> CH["chunks of up to 1100 bytes<br/>numbered, sealed"]
  CH --> CP{"datagrams lost<br/>and room?"}
  CP -- yes --> DUP[a copy of each]
  CP --> NET(("UDP"))
  DUP --> NET
  NET --> CL["client: kernel arrival time<br/>of each datagram"]
  CL --> ACK["ack + arrival reports"]
  ACK --> MOD["path model"]
  MOD -- "bottleneck rate, least round trip, queue" --> W
  MOD -- "share of datagrams lost" --> CP
```

- **Wire.** Every datagram is sealed with XChaCha20-Poly1305 under keys from an X25519 handshake on each attach, which a token issued over SSH authorizes. Without UDP the same packets go over a TCP connection to the same port, each with a 2-byte length.
- **A reliable stream with its own congestion control** (`src/proto.rs`): cumulative acks by byte offset; fast retransmit after duplicate acks, early retransmit when few datagrams are in flight, a retransmission timer with an Eifel-style undo of spurious timeouts, and a window that grows while round trips show no queue.
- **Path model** (`src/path.rs`): the client stamps each numbered datagram with the time its kernel received it (`SO_TIMESTAMP`) and reports those times in its acks. From them the server measures the bottleneck's rate from trains of datagrams sent back to back (packet-train dispersion), the queue in front of it as one-way delay over its least, the round trip less the client's hold, and loss. It tracks changes in delay and capacity; sparse output can send a short packet-pair probe when no recent useful serialization sample exists. Recovery evidence is reset when its estimate is invalidated. With UDP arrival reports, rate-based screen-mode entry requires corroborating samples or a queue.
- **Pacing to the link.** On a slow link the next screen update goes when the previous one is reckoned off the link, so updates never queue there and what you see stays young. On a fast path the window opens to the program's own pace.
- **Loss protection and probes.** Observed loss and retransmissions enable duplicate datagrams while capacity permits. Separate measurement probes can send one short copy per second when a slow capacity estimate lacks recent useful packet-pair samples.

### Client and rendering

- **Prediction.** Typed characters show at once, underlined on far links (round trip over 80 ms) until the server confirms them, as in mosh; they step aside on links fast enough not to need them.
- **Mirror.** The client keeps its own terminal emulator of what your screen holds, which predictions and the session badge are drawn against.
- **Paced writer.** Output goes to your terminal as whole frames, each one synchronized update (mode 2026), at the program's frame rate, waiting a moment for the rest of a frame split over datagrams.
- **Keepalive.** Every second while in use, every 5 seconds once no key or output for 10 s; a key wakes it.

### References

rsh follows published work; it copies no code from it.

- K. Winstein, H. Balakrishnan. *Mosh: An Interactive Remote Shell for Mobile Clients.* USENIX ATC 2012: state synchronization over UDP, predictive local echo, roaming.
- N. Cardwell, Y. Cheng, C. S. Gunn, S. H. Yeganeh, V. Jacobson. *BBR: Congestion-Based Congestion Control.* ACM Queue, 2016: the bottleneck's rate and the least round trip as the model of a path.
- G. Carlucci, L. De Cicco, S. Holmer, S. Mascolo. *Analysis and Design of the Google Congestion Control for Web Real-time Communication (WebRTC).* MMSys 2016, and the transport-wide congestion control feedback: per-packet arrival times reported to the sender.
- D. Ray, C. Fraser, et al. *SQP: Congestion Control for Low-Latency Interactive Video Streaming.* 2022: bandwidth from the dispersion of a frame's packets.
- S. Fouladi, J. Emmons, E. Orbay, C. Wu, R. S. Wahby, K. Winstein. *Salsify: Low-Latency Network Video through Tighter Integration between a Video Codec and a Transport Protocol.* NSDI 2018: skipping frames the link cannot carry rather than queueing them.
- S. Keshav. *A Control-Theoretic Approach to Flow Control.* SIGCOMM 1991; C. Dovrolis, P. Ramanathan, D. Moore. *Packet-Dispersion Techniques and a Capacity-Estimation Methodology.* IEEE/ACM Transactions on Networking, 2004: packet pairs and trains.
- RFC 9002 (QUIC loss detection), RFC 6298 (retransmission timer), RFC 5681 and RFC 5827 (fast and early retransmit), RFC 3522 (Eifel detection).
- RFC 1951 (DEFLATE), RFC 7748 (X25519), XChaCha20-Poly1305, and synchronized output (mode 2026) as modern terminals implement it.

## Benchmark lab

[`benchmark/lab`](benchmark/lab) runs the comparison in parallel container pairs, each with its own link:

```sh
scripts/dist.sh && benchmark/harness/run.py build  # the image with sshd, ET, mosh and this rsh (from ~/.rsh/dist)
python3 benchmark/lab/lab.py run --iterations 3 --duration 8 --fps 60 \
    --tests render,switch,echo,flood,outage,roam,reattach,udpblock \
    --nets lan,wan,lowbw,verylow,slow64k,highlat,lossy,mobile,badmobile \
    --switches lan:highlat,lan:lowbw,lowbw:highlat,highlat:lossy,wan:verylow
                                                    # earlier five-tool survey (render and switch twice); results in benchmark/lab/out/run-<time>.json
python3 benchmark/lab/lab.py run --tools ssh,et,mosh,rsh --tests sessions --sessions 10 --iterations 2
                                                    # memory with 10 sessions at once
python3 benchmark/lab/lab.py demo                   # the tools side by side in tmux, then a summary
python3 benchmark/lab/lab.py demo --tools ssh,mosh,rsh --events '' \
    --nets moving --scenes compress,bandwidth --fps 60 --duration 60
                                                    # latest recovery comparison: two shared 30 s cycles per scene
python3 benchmark/lab/lab.py stop                   # stop a run or demo from another shell
python3 benchmark/lab/charts.py benchmark/lab/out/run-<a>.json,benchmark/lab/out/run-<b>.json docs/bench
                                                    # every chart the runs have data for
python3 benchmark/lab/charts.py benchmark/lab/out/run-<a>.json docs/bench memory   # only these
```

Render runs record frames, their age, every frame gap (for the 1% and 5% lows), the client's bytes and packets a second, the client's and the server's memory halfway through each scene, and their CPU over it. `--tests`, `--tools`, `--nets`, `--scenes`, `--switches`, `--fps` and `--duration` pick what runs (without `--tests`, render and switch only); `--rsh <linux binary>` tests another build without rebuilding the image, and `--rsh-b <linux binary>` installs a second build as the tool `rsh-b` (add it to `--tools`), to compare two builds in one run.

The demo-only `moving` profile is a synthetic commute, not a captured
mobile-network trace. Each scene runs at least one 30-second cycle
(`--duration 60` runs two), starting over for the next scene:

| Seconds | Conditions | Nominal RTT / one-way jitter | Outage time each way | Down / up |
| --- | --- | --- | --- | --- |
| 0–6 | Clear signal | 70 / 5 ms | 0.1% | 8 / 2 Mbit/s |
| 6–10 | Congestion | 140 / 20 ms | 0.8% | 2 / 0.5 Mbit/s |
| 10–14 | Weak signal | 320 / 60 ms | 3% | 512 / 128 kbit/s |
| 14–17 | Deep fade | 600 / 120 ms | 8% | 128 / 32 kbit/s |
| 17–18 | Handover | 360 / 60 ms | 100% (one second) | 128 / 32 kbit/s |
| 18–23 | Recovering | 180 / 30 ms | 1% | 1 / 0.25 Mbit/s |
| 23–30 | Clear again | 80 / 8 ms | 0.1% | 5 / 1.25 Mbit/s |

All tools replay the same deterministic time-based trace on separate links,
with a common Docker VM clock. Every 250 ms a seeded, correlated normal
sample changes the delay (the jitter column is its standard deviation).
Each phase's outage-time percentage becomes one shared interval of 100%
packet loss; outside it, packet loss is zero. The one-second handover is a
shared full outage. These are time fractions, **not random per-packet loss
probabilities**: protocols with different packet counts experience the same
conditions rather than consuming independent random sequences. The 30-second
trace repeats identically and restarts for each scene.

Workers are loaded before choosing the common start. Transitions keep the
queue with `tc qdisc change`. OS scheduling and qdisc application take time:
result JSON preserves both the scheduled `network_trace.offset_s` and each
container's actual `applied_at`, alongside the phase-level `network_changes`.
The top demo header shows the current profile, phase and conditions. Each
session header shows only program, FPS, displayed-frame age in ms (not a
network RTT measurement), and received network kbit/s.
For `moving`, the FPS summary uses the common scheduled start and end for
every tool, excluding the drain. Static profiles retain their window from
each tool's first displayed frame.
At the end (including interruption), the link returns to the first phase
so queued output can drain. The profile
is supported by `demo --nets`, rather than the static `run` tests or
connection events. See [netem's delay and loss options](https://man7.org/linux/man-pages/man8/tc-netem.8.html).
The [recovery investigation](docs/bench/moving-recovery.md) records the
capacity-estimator fix, before/after measurements and timing limits.

## Development

```sh
cargo test --release              # unit and session tests
tests/container/e2e.sh            # end to end against a container (see the script for its setup)
tests/container/onboard.sh        # first setup over SSH, upgrade in place
```
