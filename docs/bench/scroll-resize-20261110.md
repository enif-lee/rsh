# Scroll / resize verification — 2026-11-10

Local PTY measurements passed all 27 runs: every final screen matched the
reference. Physical trackpad feel, GUI rendering/GPU behavior, WAN latency
and loss were **not** tested. The measured time includes the PTY, rsh,
workload redraw, Python scheduling and independent terminal parsing.

## Environment and commands

- macOS 27.0.1 (26A434), arm64; rustc 1.99.1; tmux 3.7c; pyte 0.8.2.
- Release binary: `rsh 0.3.0 build 45b87e08e73a protocol 3 handoff 3`.
- Loopback UDP; default `compress: always`; prediction disabled to isolate rendering.
- Temporary private auth/home and an isolated `TMUX_TMPDIR`; existing sessions untouched.

```sh
cargo build --release
python3 -m venv /tmp/rsh-scroll-venv
/tmp/rsh-scroll-venv/bin/pip install pyte==0.8.2
/tmp/rsh-scroll-venv/bin/python benchmark/harness/scroll_resize.py \
  --binary target/release/rsh --repeats 3 --output /tmp/rsh-scroll-direct-final.json
/tmp/rsh-scroll-venv/bin/python benchmark/harness/scroll_resize.py \
  --binary target/release/rsh --tmux --repeats 3 --output /tmp/rsh-scroll-tmux-final.json
```

## Workloads and measurements

Each workload runs for 2 seconds, repeated three times in separate sessions.

- `infinite`: continuous numbered output (8 lines per 2 ms scheduling step).
- `wheel`: 120 SGR wheel events/s, alternating directions. Every run received all 240 events.
- `resize`: 60 size changes/s, 80–144 columns and 20–38 rows.
- `combined`: wheel plus resize, both at 120 events/s; all 240 wheel events received per run.
- `copy`: tmux copy-mode over 5,000 numbered history lines, wheel-up plus resize at 120/s.

At the end, the terminal changes to 120×36. `settle_ms` measures from that
change/query until all rows match. The top row's intentional rsh badge is
excluded. For normal workloads, expected rows are generated independently
from the reported frame counter and dimensions. For copy-mode, the reference
is tmux's history capture at its current `scroll_position`.

| Path | Workload | Settle median ms | Settle max ms | Largest run's output-gap p95 ms | Final match |
| --- | --- | ---: | ---: | ---: | --- |
| direct | infinite | 31.94 | 36.93 | 11.56 | 3/3 |
| direct | wheel | 25.13 | 25.64 | 10.17 | 3/3 |
| direct | resize | 26.76 | 27.28 | 16.94 | 3/3 |
| direct | combined | 19.20 | 27.30 | 14.18 | 3/3 |
| tmux | infinite | 37.77 | 42.18 | 10.19 | 3/3 |
| tmux | wheel | 30.73 | 32.76 | 10.23 | 3/3 |
| tmux | resize | 39.79 | 39.83 | 13.79 | 3/3 |
| tmux | combined | 18.56 | 22.41 | 13.65 | 3/3 |
| tmux | copy | 69.72 | 76.52 | 10.48 | 3/3 |

Raw results: [direct](data/scroll-resize-20261110/direct.json),
[tmux](data/scroll-resize-20261110/tmux.json).

`output_gap_p95_ms` measures spacing between received synchronized updates,
not per-input latency. Frame and byte totals include startup and final
settling. Completed synchronized frames were sampled for consistent rows;
these samples had zero inconsistent frames in the final runs. During size
storms, frames for the previous dimensions still arrived (see
`stale_size_frames` in the raw data). Those are recorded separately and do
not establish correctness while the window is changing. Final full-screen
comparison is the stronger check. No persistent partial rendering defect
was reproduced in these final runs.

The probe preserves partial UTF-8 and synchronized-update delimiters across
reads. tmux `capture-pane -M` returns copy-mode's backing grid, so the probe
uses `scroll_position` and explicit `-S`/`-E` bounds for the visible viewport.
This follows [tmux's capture implementation](https://github.com/tmux/tmux/blob/3.7c/cmd-capture-pane.c#L164)
and [copy-mode's backing-screen accessor](https://github.com/tmux/tmux/blob/3.7c/window-copy.c#L978).

## Resize delay correction

Earlier copy-mode trials occasionally took about 1.1 s to settle. Code
inspection found a concrete race: `SIGWINCH` received during drawing sets
an atomic flag, but the flag was checked only after the next `poll`. Since
the signal had already been delivered, it could not interrupt that poll;
a quiet connection could then wait until its 1 s active/5 s idle heartbeat.

The client now writes a byte to a nonblocking, close-on-exec pipe from the
signal handler. The pipe stays alive for the process lifetime and is part
of the client's poll set. A signal delivered before polling therefore
remains readable. The handler preserves errno and the loop drains the pipe
before processing signal flags. The regression test queues a wake before
polling and verifies immediate readiness, draining, nonblocking mode and
close-on-exec flags (`src/client.rs`, `a_signal_queued_before_poll_wakes_it_and_can_be_drained`).

Final tmux copy-mode settling was 65.55–76.52 ms across three runs. These
small local trials establish the observed final behavior; they do not prove
that this race explains every reported GUI lag or provide a controlled
percentage speedup.

## Feature review and checks

The reviewer reproduced three defects, all accepted and fixed:

1. Closing a busy name editor and submitting another rename discarded the
   first result. New submissions retain the existing receiver; delayed
   callback regression verifies one request and matching local/server names.
2. Unreachable hosts disappeared from the interactive list. They now appear
   below selectable rows; an entirely unreachable list reports its errors
   and exits with `no host answers`.
3. A rename from another client left stale detach/resume advice. Advice
   now refreshes the name by stable session ID; current tmux rename also
   resolves the current target using its stable wrapper ID.

No reviewer findings were rejected. Follow-up review found no new defects.
Other coverage includes authenticated rsh/tmux rename, name collisions,
Unicode editing, input arriving in the same read as `~m`/`~,`, fragmented
arrow keys, settings persistence, and retaining the same tmux wrapper ID
when reconnecting after rename.

```sh
env -u NO_COLOR cargo test --all-targets
cargo clippy --all-targets -- -D warnings -A clippy::double_ended_iterator_last
git diff --check
```

Final result: 157 passed, 2 existing tests ignored; release build, Clippy with
the exclusion below, and whitespace checks passed.

`NO_COLOR=1` in the agent shell causes an existing badge-color assertion to
fail, so tests run with it unset. Clippy's existing
`double_ended_iterator_last` warning in `src/terminal.rs` is excluded; the
remaining warnings are treated as errors. The verification run did not deploy, push, or install into the user's PATH.


## Subsequent local installation

At the user's request, build `45b87e08e73a` was subsequently installed at
`~/.rsh/bin/rsh`, reached through the existing `~/.local/bin/rsh` link.
All four matching distribution binaries and `SHA256SUMS` were installed
under `~/.rsh/dist`. The previous installation is backed up under
`~/.rsh/backups/install-20261110-192323`.

- Installed CLI, macOS arm64 and both Linux binaries reported the matching build.
- All four manifest checksums and source build markers passed verification.
- macOS x86_64 passed build-marker verification; execution was unavailable
  on this machine (`bad CPU type in executable`).
- No remote host was contacted or upgraded during this installation.
