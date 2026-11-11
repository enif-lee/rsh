#!/usr/bin/env python3
"""Local PTY regression/latency probe for rsh scrolling and resize storms.

Install pyte in a temporary venv, build `cargo build --release`, then run:
  python scroll_resize.py --binary ../../target/release/rsh --output /tmp/scroll.json
Only loopback servers, temporary auth files and child PTYs are used. This
measures rsh plus a PTY/terminal parser, not a GUI compositor or physical trackpad.
"""
import argparse
import codecs
import fcntl
import json
import os
from pathlib import Path
import pty
import re
import select
import shlex
import signal
import socket
import struct
import subprocess
import sys
import tempfile
import termios
import time


def canvas(cols, rows, count):
    lines = []
    for row in range(rows):
        text = f"FRAME {count:06d} {cols}x{rows} ROW {row:03d} "
        lines.append((text + chr(65 + row % 26) * cols)[:cols - 1])
    return lines


def remote(mode):
    import tty
    tty.setraw(0)
    if mode == "copy":
        for row in range(5000):
            os.write(1, f"HISTORY {row:06d} ".encode() + b"z" * 60 + b"\r\n")
        while os.read(0, 1024):
            pass
        return
    os.write(1, b"\x1b[?1049h\x1b[?1000h\x1b[?1006h")
    count = 0
    changed = True

    def resized(*_):
        nonlocal changed
        changed = True

    signal.signal(signal.SIGWINCH, resized)

    def draw():
        cols, rows = os.get_terminal_size(0)
        body = "\x1b[?2026h\x1b[2J" + "".join(f"\x1b[{row+1};1H{text}" for row, text in enumerate(canvas(cols, rows, count)))
        os.write(1, (body + "\x1b[?2026l").encode())

    pending = b""
    scrolling = mode == "infinite"
    while True:
        ready, _, _ = select.select([0], [], [], 0.002 if scrolling else 0.02)
        if ready:
            data = os.read(0, 65536)
            if not data or b"x" in data:
                return
            if b"s" in data:
                scrolling = False
            pending += data
            events = list(re.finditer(rb"\x1b\[<6[45];\d+;\d+M", pending))
            if events:
                count += len(events)
                pending = pending[events[-1].end():]
            # 's' is a barrier/query after all earlier input.
            changed = bool(events) or b"s" in data or changed
        if scrolling:
            os.write(1, ("".join(f"scroll {count+i:08d} " + "z" * 64 + "\r\n" for i in range(8))).encode())
            count += 8
        elif changed:
            changed = False
            draw()


def run_case(binary, mode, rate, duration, tmux=False):
    import pyte
    with tempfile.TemporaryDirectory(prefix="rsh-scroll-", dir="/tmp") as tmp:
        home = Path(tmp)
        auth = home / "known"
        auth.write_text("scroll.example probe-token\n")
        auth.chmod(0o600)
        udp = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        udp.bind(("127.0.0.1", 0))
        port = udp.getsockname()[1]
        udp.close()
        env = dict(os.environ, HOME=tmp, TERM="xterm-256color", RSH_NO_UPDATE="1", RSH_PREDICTION="never")
        env.pop("NO_COLOR", None)
        env["TMUX_TMPDIR"] = tmp
        env.pop("TMUX", None)
        command = "exec " + " ".join(map(shlex.quote, [sys.executable, str(Path(__file__).resolve()), "--remote", mode]))
        if tmux:
            subprocess.run(["tmux", "new-session", "-d", "-s", "probe", "-x", "100", "-y", "30", command], env=env, check=True)
            subprocess.run(["tmux", "set-option", "-g", "mouse", "on"], env=env, check=True)
            subprocess.run(["tmux", "set-option", "-g", "status", "off"], env=env, check=True)
        server = subprocess.Popen([binary, "serve", "--bind", f"127.0.0.1:{port}", "--authorized", str(auth)], env=env, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        child = None
        master = None
        try:
            time.sleep(0.1)
            child, master = pty.fork()
            if child == 0:
                args = [binary, "--auth", str(auth), "--peer", f"127.0.0.1:{port}", "scroll.example", "probe"]
                args += ["--tmux"] if tmux else ["--", command]
                os.execve(binary, args, env)
            cols, rows = 100, 30
            class ProbeScreen(pyte.Screen):
                def write_process_input(self, text):
                    os.write(master, text.encode())

                def report_device_status(self, mode, **kwargs):
                    # tmux's private cursor-position query is outside pyte's
                    # public signature; answer it as a terminal would.
                    if kwargs.get("private") and mode == 6:
                        self.write_process_input(f"\x1b[?{self.cursor.y+1};{self.cursor.x+1}R")
                    else:
                        super().report_device_status(mode)

            screen = ProbeScreen(cols, rows)
            stream = pyte.Stream(screen)
            decoder = codecs.getincrementaldecoder("utf-8")("replace")
            received = 0
            frames = 0
            consistent = 0
            inconsistent = 0
            stale_size = 0
            gaps = []
            last_frame = None
            sync_tail = ""

            def pump(wait=0):
                nonlocal received, frames, last_frame, consistent, inconsistent, stale_size, sync_tail
                ready, _, _ = select.select([master], [], [], max(0, wait))
                if ready:
                    data = os.read(master, 65536)
                    received += len(data)
                    now = time.monotonic()
                    # Retain only a possible delimiter prefix across reads.
                    # Every completed 2026l is counted even when fragmented.
                    delimiter = "\x1b[?2026l"
                    text = sync_tail + decoder.decode(data)
                    chunks = text.split(delimiter)
                    for chunk in chunks[:-1]:
                        stream.feed(chunk + delimiter)
                        frames += 1
                        if last_frame is not None:
                            gaps.append((now - last_frame) * 1000)
                        last_frame = now
                        display = screen.display
                        marker = re.search(r"FRAME (\d+) (\d+)x(\d+) ROW 001", display[1])
                        if marker:
                            if (int(marker[2]), int(marker[3])) != (cols, rows):
                                stale_size += 1
                            else:
                                expected = canvas(cols, rows, int(marker[1]))
                                if all(display[row].rstrip() == expected[row].rstrip() for row in range(1, rows)):
                                    consistent += 1
                                else:
                                    inconsistent += 1
                    tail = chunks[-1]
                    keep = next((size for size in range(min(len(tail), len(delimiter)-1), 0, -1) if tail.endswith(delimiter[:size])), 0)
                    stream.feed(tail[:-keep] if keep else tail)
                    sync_tail = tail[-keep:] if keep else ""
                    return True
                return False

            def resize(width, height):
                nonlocal cols, rows
                # Model the actual terminal resizing before notifying rsh.
                cols, rows = width, height
                screen.resize(lines=rows, columns=cols)
                fcntl.ioctl(master, termios.TIOCSWINSZ, struct.pack("HHHH", rows, cols, 0, 0))
                os.kill(child, signal.SIGWINCH)

            resize(cols, rows)
            ready_at = time.monotonic() + 5
            while time.monotonic() < ready_at:
                pump(0.02)
                if any("FRAME" in line or "scroll 0000" in line or "HISTORY" in line for line in screen.display):
                    break
            else:
                raise AssertionError("remote workload did not start: " + repr(screen.display))
            start = time.monotonic()
            next_event = start
            events = 0
            while time.monotonic() - start < duration:
                now = time.monotonic()
                if now >= next_event:
                    if mode in ("resize", "combined", "copy"):
                        resize(80 + events % 9 * 8, 20 + events % 7 * 3)
                    if mode in ("wheel", "combined", "copy"):
                        os.write(master, b"\x1b[<64;12;8M" if mode == "copy" or events % 2 == 0 else b"\x1b[<65;12;8M")
                    events += 1
                    next_event += 1 / rate
                pump(min(0.002, max(0, next_event - time.monotonic())))
            resize(120, 36)
            # Stop infinite scrolling, or request an authoritative final frame.
            barrier = time.monotonic()
            if mode != "copy":
                os.write(master, b"s")
            expected_count = events if mode in ("wheel", "combined") else None
            recovered = None
            mismatch = []
            while time.monotonic() - barrier < 5:
                pump(0.01)
                display = screen.display
                if mode == "copy":
                    state = subprocess.check_output(["tmux", "display-message", "-p", "-t", "probe", "#{pane_in_mode} #{scroll_position} #{pane_width} #{pane_height}"], env=env, text=True).split()
                    if state[0] != "1" or len(state) != 4 or tuple(map(int, state[2:])) != (cols, rows):
                        continue
                    offset = int(state[1])
                    # capture-pane -M returns copy-mode's backing grid, not
                    # the visible viewport (tmux 3.7c window_copy_get_screen).
                    capture = subprocess.run(["tmux", "capture-pane", "-p", "-S", str(-offset), "-E", str(rows-1-offset), "-t", "probe"], env=env, text=True, capture_output=True)
                    expected = capture.stdout.splitlines()
                    if capture.returncode == 0 and len(expected) == rows:
                        mismatch = [row for row in range(1, rows) if display[row].rstrip() != expected[row].rstrip()]
                        if not mismatch:
                            recovered = (time.monotonic() - barrier) * 1000
                            break
                    continue
                marker = re.search(r"FRAME (\d+) 120x36 ROW 001", display[1])
                if marker and (expected_count is None or int(marker[1]) == expected_count):
                    expected = canvas(cols, rows, int(marker[1]))
                    # Top row's right edge is the intentional rsh badge.
                    mismatch = [row for row in range(1, rows) if display[row].rstrip() != expected[row].rstrip()]
                    if not mismatch:
                        recovered = (time.monotonic() - barrier) * 1000
                        break
            result = dict(mode=mode, tmux=tmux, rate_hz=rate, duration_s=duration, events=events, bytes=received, sync_frames=frames,
                          consistent_frames=consistent, inconsistent_frames=inconsistent, stale_size_frames=stale_size,
                          output_gap_p95_ms=round(sorted(gaps)[int(len(gaps)*.95)] if gaps else 0, 2),
                          settle_ms=round(recovered, 2) if recovered is not None else None, mismatch_rows=mismatch,
                          final_marker=screen.display[1][:60])
            if recovered is None:
                result["screen"] = screen.display
                if mode == "copy":
                    result["expected"] = expected
                    result["capture_error"] = capture.stderr
            return result
        finally:
            if child:
                try:
                    os.kill(child, signal.SIGKILL)
                    os.waitpid(child, 0)
                except ProcessLookupError:
                    pass
            if master is not None:
                os.close(master)
            server.terminate()
            server.wait(timeout=5)
            if tmux:
                subprocess.run(["tmux", "kill-server"], env=env, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", default="target/release/rsh")
    parser.add_argument("--output")
    parser.add_argument("--tmux", action="store_true", help="use an isolated tmux server with mouse enabled")
    parser.add_argument("--duration", type=float, default=2)
    parser.add_argument("--repeats", type=int, default=3)
    parser.add_argument("--remote", choices=["infinite", "wheel", "resize", "combined", "copy"])
    args = parser.parse_args()
    if args.remote:
        remote(args.remote)
        return
    binary = str(Path(args.binary).resolve())
    results = []
    cases = [("infinite", 120), ("wheel", 120), ("resize", 60), ("combined", 120)]
    if args.tmux:
        cases.append(("copy", 120))
    for repeat in range(args.repeats):
        for mode, rate in cases:
            result = dict(repeat=repeat, **run_case(binary, mode, rate, args.duration, args.tmux))
            results.append(result)
            print(json.dumps(result), flush=True)
    if args.output:
        Path(args.output).write_text(json.dumps({"binary": binary, "version": subprocess.check_output([binary, "--version"], text=True).strip(), "results": results}, indent=2) + "\n")
    if any(row["settle_ms"] is None for row in results):
        sys.exit(1)


if __name__ == "__main__":
    main()
