# rsh

[![Rust](https://img.shields.io/badge/rust-2021-orange?logo=rust)](Cargo.toml)
[![Platforms](https://img.shields.io/badge/platforms-linux%20%7C%20macOS-lightgrey)](#installation)
[![Transport](https://img.shields.io/badge/transport-UDP%20%2B%20TCP%20fallback-blue)](#how-it-works)

> A remote shell that keeps your session through disconnects and keeps the screen current on slow, far and lossy links.

rsh opens a shell on a host the way `ssh` does, but the session lives on the server: close the laptop, change networks, or kill the client, and `rsh <host>` brings you back to the same shell. Over UDP it echoes your keys at once and, when the link cannot carry every byte, it sends the latest screen instead of a backlog. Where only TCP gets through, it falls back to TCP on the same port.

## Benchmarks

Same machine, same simulated links, five clients: rsh over UDP, rsh over TCP (UDP blocked), OpenSSH, Eternal Terminal (ET) and mosh. Each value is the median of three runs.

<p align="center">
  <img src="docs/bench/fps-dashboard.svg" alt="Frames shown per second for a full-screen dashboard at 30 fps, per network and tool" width="760">
</p>

<p align="center">
  <img src="docs/bench/age.svg" alt="Median age of what is on screen for a full-screen dashboard at 30 fps, per network and tool" width="760">
</p>

- **The screen stays current.** On 1 Mbit/s to 64 kbit/s links rsh over UDP shows a 30 fps dashboard at 14 to 30 fps, and what is on screen is 56 to 132 ms old (over TCP 13 to 30 fps, up to 207 ms). OpenSSH and ET show 1.5 to 24 fps of frames that are 0.5 to 2.4 s old, because every byte waits its turn on the link.
- **Keys echo at once.** Local echo prediction answers a key in about 4 to 6 ms on every link. OpenSSH and ET wait a round trip: 107 to 319 ms.
- **Big output does not block you.** `seq 1 100000` returns the prompt in 0.13 to 0.41 s on every link. OpenSSH and ET take 1.6 to 23 s, and more than 30 s at 64 kbit/s.
- **Sessions survive.** After a 5 s outage rsh is back about 1 s (median 0.95 s) after the link returns. After the client moves to another IP address it carries on in about 2 s. A client killed and started again returns to the same shell in 0.5 s.

<details>
<summary>More charts: random full screen, bulk output</summary>

<p align="center">
  <img src="docs/bench/fps-random.svg" alt="Frames shown per second for a random full screen at 30 fps" width="760">
</p>

<p align="center">
  <img src="docs/bench/flood.svg" alt="Time to finish seq 1 100000 per network and tool" width="760">
</p>

</details>

> [!NOTE]
> These come from [`benchmark/lab`](benchmark/lab): two Docker containers per client on one machine, with `tc netem` for delay, loss and rate. Real radio links add jitter and reordering that netem only partly imitates. Where rsh is behind:
> - On the dashboard at RTT 300 ms and 2% loss, OpenSSH and ET reach 29.5 to 30 fps against rsh's 28 to 30 (they keep a backlog in flight, so their screen is older on slow links).
> - On a random full screen at 256 kbit/s and 64 kbit/s, mosh shows slightly more frames (6.0 and 1.7 against 5.5 and 1.5).
> - When the link changes under a random full screen, rsh takes longer to adapt: from RTT 100 ms to 256 kbit/s it shows 1.8 fps afterwards (OpenSSH and ET 2.5, mosh and rsh over TCP 5.3); from a LAN to RTT 300 ms, 18.0 fps (OpenSSH 29.0, ET 29.2).
> - rsh over TCP does not resume after an outage or a move within the 4 s the test watches.
>
> One of ET's three bulk-output runs at 512 kbit/s did not connect; its value is the median of two.

## Feature matrix

| | OpenSSH (sshd) | Eternal Terminal | mosh | rsh |
| --- | --- | --- | --- | --- |
| Transport | TCP | TCP | UDP | UDP, or TCP on the same port when UDP is blocked |
| Works where only TCP gets through | ✅ | ✅ | ❌ cannot connect | ✅ connects over TCP in 2.0 s |
| Client changes IP address | ❌ connection drops | ⚠️ reconnects over a new TCP connection (about 23 s in an earlier run) | ✅ ~3.0 s | ✅ ~2.1 s (UDP) |
| Link down for a few seconds | ⚠️ only while TCP survives its backoff (not back within 4 s here) | ⚠️ reconnects (not within 4 s here) | ✅ back ~1.0 s after the link | ✅ back ~0.95 s after the link (UDP) |
| Client killed, started again | ❌ new shell | ❌ new shell | ❌ new shell | ✅ same shell (`rsh <host> <name>`) |
| Local echo prediction | ❌ | ❌ | ✅ adaptive | ✅ adaptive |
| Slow link: shows the latest screen, not a backlog | ❌ | ❌ | ✅ | ✅ |
| Program's bytes reach your terminal (scrollback, OSC) | ✅ | ✅ | ❌ screen only, 5 of 18 OSC tested | ✅ 18 of 18 OSC tested; screen updates only while the link falls behind |
| Compression | ⚠️ `-C`, off by default | not checked | ✅ zlib | ✅ deflate stream, on by default |
| Port forwarding | ✅ `-L` `-R` `-D`, X11, agent | ✅ `-t` `-r`, agent | ❌ | ⚠️ `-L` `-R` |
| File transfer | ✅ scp, sftp | ❌ | ❌ | ❌ |
| List and switch sessions across hosts | ❌ | ❌ | ❌ | ✅ `rsh ls`, `~l` |
| tmux | run it yourself | run it yourself | run it yourself | ✅ `--tmux` joins or creates a tmux session |
| Server side | `sshd` | `sshd` + `etserver` daemon | `sshd` + a `mosh-server` per session (UDP 60000–61000) | `rsh serve` on port 2222, installed and upgraded over SSH by the client |
| Upgrade the server without dropping sessions | n/a | not checked | n/a | ✅ hands sessions to the new binary |
| Authentication | SSH keys, passwords | via SSH | via SSH | per-client token issued over SSH; X25519 per attach, XChaCha20-Poly1305 |

Times come from the lab run above. The OSC counts and ET's reconnect time come from an earlier run of [`benchmark/harness`](benchmark/harness); the other cells from each project's documentation and source.

## Features

- Sessions live on the server and outlast the client: detach with `~.`, reattach by name, from another machine too.
- One UDP socket per host; roaming and short outages are invisible except for a notice.
- Predictive local echo, as in mosh, that steps aside on fast links.
- Adapts to the link as it changes: measures delay and delivery, holds the queue short on slow links, and switches to sending screen differences when the link cannot keep up.
- Deflate-compressed output stream by default (`--compress always|adaptive|none`).
- TCP fallback on the same port for networks that drop UDP.
- First connection sets the host up over `ssh`: copies this build, starts `rsh serve`, saves a token. A server running another build is upgraded in place, keeping its sessions.
- `-L` and `-R` port forwarding, `--tmux` integration, a session list across hosts.

## Installation

Requires Rust (stable) to build, Docker for the static Linux builds that rsh copies to servers, and `ssh` access to each host for its first setup.

```sh
git clone git@github.com:enif-lee/rsh.git
cd rsh
scripts/dist.sh                         # builds ~/.rsh/dist/rsh-{linux-x86_64,linux-aarch64,<this platform>}
install -m 755 target/release/rsh ~/.local/bin/rsh
```

`rsh <host>` copies the matching binary from `~/.rsh/dist` to `~/.rsh/bin/rsh` on the host the first time, so run `scripts/dist.sh` again after changing the source. A host already running `rsh serve` can be used directly with `--peer <host>:2222`.

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

## How it works

- **Wire.** Sealed datagrams (XChaCha20-Poly1305) on one UDP port; keys from an X25519 handshake on each attach, authorized by a token the client got over SSH. Without UDP, the same packets go length-prefixed over a TCP connection to the same port.
- **Stream.** Output is a reliable, ordered byte stream with its own acks, repairs and congestion control: a window and an ack clock bounded by recent delivery, a slow-link rate estimate that holds the queue to about two round trips, and a least round trip that is learned again when the path changes.
- **Screens.** The server keeps a terminal emulator (`alacritty_terminal`) per session. When output would take long to drain, it stops sending bytes and sends the difference from the screen the client has, at the program's frame rate; when the link catches up, bytes again. Reattaching repaints the screen and recent scrollback.
- **Client.** Draws through a paced writer that follows the program's frame rate, shows a session badge, predicts echo, and reports disconnects without taking over the screen.

## Benchmark lab

[`benchmark/lab`](benchmark/lab) runs the comparison in parallel container pairs, each with its own link:

```sh
benchmark/harness/run.py build                      # the image with sshd, ET, mosh and this rsh
python3 benchmark/lab/lab.py run --iterations 3 \
    --tests render,switch,echo,flood,outage,roam,reattach,udpblock \
    --nets lowbw,verylow,slow64k,highlat,lossy,mobile \
    --switches lan:highlat,lan:lowbw,lowbw:highlat,highlat:lossy,wan:verylow
                                                    # the README's run; results in benchmark/lab/out/run-<time>.json
python3 benchmark/lab/lab.py demo                   # the tools side by side in tmux, then a summary
python3 benchmark/lab/lab.py stop                   # stop a run or demo from another shell
python3 benchmark/lab/charts.py benchmark/lab/out/run-<time>.json docs/bench
```

`--tests`, `--tools`, `--nets`, `--scenes`, `--switches`, `--fps` and `--duration` pick what runs (without `--tests`, render and switch only); `--rsh <linux binary>` tests another build without rebuilding the image.

## Development

```sh
cargo test --release              # unit and session tests
tests/container/e2e.sh            # end to end against a container (see the script for its setup)
tests/container/onboard.sh        # first setup over SSH, upgrade in place
```
