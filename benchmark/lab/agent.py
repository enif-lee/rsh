#!/usr/bin/env python3
"""The lab's client side, in the client container (see lab.py).

  agent.py run '<job json>'                   one test; one JSON line out
  agent.py watch <log> <status> <frames>      a demo pane's frames and rate

`run` starts the tool's client in a pty and drains it on its own thread,
so the client never waits on this script (a slow reader holds the client's
writes, and with them its acks, which would measure the reader, not the
link). Frames (row 0 of scene.py: `FRAME n T=ms`) are found in the raw
bytes for ssh and et, which pass the program's bytes; mosh and rsh may
send screen differences (only the digits that changed), so their screen
is rebuilt with pyte, on another thread. Steps the host must take (link changes, a move to another
address) are asked for on stdout (`REQ ...`) and confirmed on stdin.
"""
import fcntl, json, math, os, pty, queue, random, re, signal, statistics, struct, subprocess, sys, termios, threading, time
import pyte

COMMAND = {
    "ssh": ["ssh", "-tt", "dev@server"],
    "et": ["et", "dev@server:2022"],
    "mosh": ["mosh", "dev@server"],
    "rsh": ["rsh", "server"],
    # Over TCP: the host drops UDP at the server (lab.py `prepare`).
    "rsh-tcp": ["rsh", "server"],
}
FRAME = re.compile(rb"FRAME (\d+) T=(\d+) (\w+)")
FRAME_TEXT = re.compile(r"FRAME (\d+) T=(\d+) (\w+)")
# The end of a synchronized update: a terminal shows the screen as it is
# there, not what came between (rsh wraps each write in one).
SYNC_END = b"\x1b[?2026l"
SYNC_BEGIN = b"\x1b[?2026h"


class Client:
    """A tool's client in a pty: a reader thread drains it at once; an
    emulator thread rebuilds the screen when asked (prompts, echoes, and
    mosh's frames)."""

    def __init__(self, tool, name, cols, rows):
        self.tool, self.cols, self.rows = tool, cols, rows
        # Tools that may send differences: frames from the rebuilt screen.
        self.diffs = tool in ("mosh", "rsh", "rsh-tcp")
        argv = COMMAND[tool] + ([name] if tool in ("rsh", "rsh-tcp") else [])
        env = dict(os.environ, TERM="xterm-256color", LANG="C.UTF-8", RSH_PEER="server:2222")
        pid, fd = pty.fork()
        if pid == 0:
            os.execvpe(argv[0], argv, env)
        fcntl.ioctl(fd, termios.TIOCSWINSZ, struct.pack("HHHH", rows, cols, 0, 0))
        self.pid, self.fd = pid, fd
        self.alive = True
        self.bytes = 0
        self.tail = b""
        self.frames = []  # (shown at, n, made at, scene)
        # Each screen the terminal would show while a scene runs: (at, torn,
        # blank), see `look`.
        self.states = []
        self.watching = False
        self.lock = threading.Lock()
        self.emulate = True
        # Inside a synchronized update (pyte does not keep mode 2026).
        self.sync_open = False
        self.screen = pyte.Screen(cols, rows)
        self.stream = pyte.ByteStream(self.screen)
        self.chunks = queue.Queue()
        threading.Thread(target=self.read, daemon=True).start()
        threading.Thread(target=self.feed, daemon=True).start()

    def read(self):
        carry = b""
        while True:
            try:
                data = os.read(self.fd, 1 << 20)
            except OSError:
                data = b""
            if not data:
                self.alive = False
                return
            now = time.time()
            self.bytes += len(data)
            buf = carry + data
            self.tail = buf[-512:]
            if not self.diffs:
                found = list(FRAME.finditer(buf))
                if found:
                    self.frame(now, int(found[-1].group(1)), int(found[-1].group(2)), found[-1].group(3).decode())
            carry = buf[-64:]
            self.chunks.put((now, data))

    def feed(self):
        while True:
            now, data = self.chunks.get()
            if not self.emulate:
                continue
            with self.lock:
                # What a terminal shows: the screen at each end of a
                # synchronized update, or, outside one, after each read.
                pieces = data.split(SYNC_END)
                for i, piece in enumerate(pieces):
                    ended = i < len(pieces) - 1
                    self.stream.feed(piece + (SYNC_END if ended else b""))
                    if ended:
                        self.sync_open = False
                    elif SYNC_BEGIN in piece:
                        self.sync_open = True
                    if self.sync_open:
                        continue
                    if self.diffs:
                        row = self.screen.buffer[0]
                        m = FRAME_TEXT.search("".join(row[x].data for x in range(min(self.cols, 60))))
                        if m:
                            self.frame(now, int(m.group(1)), int(m.group(2)), m.group(3))
                    if self.watching:
                        self.look(now)

    def look(self, now):
        """The screen as shown now: torn (rows of different frames) or
        blank (most rows without their frame number, as after a clear)."""
        tags = []
        for y in range(1, self.rows):
            row = self.screen.buffer[y]
            text = "".join(row[x].data for x in range(6))
            tags.append(text if text.isdigit() else None)
        shown = [t for t in tags if t]
        self.states.append((now, len(set(shown)) > 1, len(shown) < len(tags) / 2))

    def frame(self, now, n, made_ms, scene):
        if not self.frames or self.frames[-1][1] != n or self.frames[-1][3] != scene:
            self.frames.append((now, n, made_ms / 1000.0, scene))

    def settle(self, timeout=2.0):
        """Until the emulator has taken everything read so far."""
        end = time.time() + timeout
        while not self.chunks.empty() and time.time() < end:
            time.sleep(0.01)

    def fresh(self):
        """A new screen from what comes next (after output it skipped)."""
        with self.lock:
            self.sync_open = False
            self.screen = pyte.Screen(self.cols, self.rows)
            self.stream = pyte.ByteStream(self.screen)

    def send(self, text):
        os.write(self.fd, text.encode() if isinstance(text, str) else text)

    def wait(self, pred, timeout, step=0.01):
        end = time.time() + timeout
        while True:
            with self.lock:
                if pred():
                    return time.time()
            if time.time() >= end:
                return None
            time.sleep(step)

    def lines(self):
        return [line.rstrip() for line in self.screen.display]

    def has_line(self, text):
        return any(line == text or line.startswith(text + " ") for line in self.lines())

    def kill(self):
        try:
            os.kill(self.pid, signal.SIGKILL)
            os.waitpid(self.pid, 0)
        except (ProcessLookupError, ChildProcessError):
            pass
        self.alive = False

    def finish(self):
        if self.alive:
            self.emulate = False
            self.send("\r exit\r")
            end = time.time() + 1
            while self.alive and time.time() < end:
                time.sleep(0.05)
        self.kill()


def connect(tool, name, cols, rows, timeout=30):
    """The client, and seconds until the remote prompt; the shell is set
    up with a plain prompt."""
    c = Client(tool, name, cols, rows)
    t0 = time.time()
    at = c.wait(lambda: c.screen.display[c.screen.cursor.y][: c.screen.cursor.x].rstrip().endswith("$"), timeout, 0.005)
    if at is None:
        return c, None
    ready = None
    while time.time() - t0 < timeout and ready is None:
        c.send("export PS1='$ ' PROMPT_COMMAND=; echo RE''ADY\r")
        ready = c.wait(lambda: c.has_line("READY"), 5)
    if ready is None:
        return c, None
    c.send("clear\r")
    c.wait(lambda: not c.has_line("READY"), 3)
    return c, round(at - t0, 3)


def ask(request):
    """A step only the host can take; returns once it is done, with what the
    host answered after its OK."""
    print("REQ " + request, flush=True)
    line = sys.stdin.readline()
    if not line.startswith("OK"):
        raise SystemExit("host: " + line.strip())
    return line[2:].strip()


# Each tool's client process, as `ps -C` names it.
PROCESS = {"ssh": "ssh", "et": "et", "mosh": "mosh-client", "rsh": "rsh", "rsh-tcp": "rsh"}


def rss_kb(names):
    """Resident memory of this container's processes so named, KB."""
    out = subprocess.run(["ps", "-o", "rss=", "-C", names], capture_output=True, text=True).stdout
    sizes = [int(x) for x in out.split() if x.isdigit()]
    return sum(sizes) if sizes else None


def block(on, udp_only=False):
    """Cuts (or restores) this container's link to the server."""
    action = "-I" if on else "-D"
    rules = [["OUTPUT", "-p", "udp", "-d", "server"]] if udp_only else [["OUTPUT", "-d", "server"], ["INPUT", "-s", "server"]]
    for rule in rules:
        subprocess.run(["iptables", action, *rule, "-j", "DROP"], check=False, capture_output=True)


def rx_bytes():
    """Bytes in on every interface but loopback."""
    total = 0
    for dev in os.listdir("/sys/class/net"):
        if dev != "lo":
            try:
                with open("/sys/class/net/%s/statistics/rx_bytes" % dev) as f:
                    total += int(f.read())
            except OSError:
                pass  # an interface going away as the client moves
    return total


def frames_between(c, scene, start, end):
    return [f for f in c.frames if f[3] == scene and start <= f[0] <= end]


def screen_stats(c, start, end, seen):
    """Screens shown that were torn or blank, up to the last frame (the
    scene clears the screen as it ends)."""
    end = seen[-1][0] if seen else end
    states = [s for s in c.states if start <= s[0] <= end]
    return {"torn": sum(1 for s in states if s[1]), "blank": sum(1 for s in states if s[2]), "screens": len(states),
            "blank_at": [round(s[0] - start, 2) for s in states if s[2]][:8], "torn_at": [round(s[0] - start, 2) for s in states if s[1]][:8]}


def stalls(seen):
    gaps = [b[0] - a[0] for a, b in zip(seen, seen[1:])]
    if len(gaps) < 4:
        return None
    usual = statistics.median(gaps)
    return sum(1 for g in gaps if g > 2.5 * usual)


def stall_times(seen):
    """When each stall ended (epoch s), its length (ms) and the frames it
    skipped, for matching with the server's log."""
    gaps = [(b[0], (b[0] - a[0]) * 1000, b[1] - a[1]) for a, b in zip(seen, seen[1:])]
    if len(gaps) < 4:
        return []
    usual = statistics.median(g[1] for g in gaps)
    return [(round(at, 3), round(ms), skipped) for at, ms, skipped in gaps if ms > 2.5 * usual][:10]


def render_stats(seen, start, end):
    """fps, frame age (made on the server, shown here; the containers
    share a clock), the longest gap (edges included), fps each second."""
    span = max(end - start, 1e-6)
    ages = sorted((f[0] - f[2]) * 1000 for f in seen)
    times = [start] + [f[0] for f in seen] + [end]
    return {
        "frames": len(seen),
        "fps": round(len(seen) / span, 1),
        "age_ms": round(statistics.median(ages)) if ages else None,
        "age_p95_ms": round(ages[math.ceil(len(ages) * 0.95) - 1]) if len(ages) >= 20 else None,
        "max_gap_ms": round(max(b - a for a, b in zip(times, times[1:])) * 1000),
        "per_s": [sum(1 for f in seen if start + k <= f[0] < start + k + 1) for k in range(int(span))],
        # The first second alone: how fast the tool settles at the start.
        "first_s_fps": sum(1 for f in seen if f[0] < start + 1),
        "first_s_age_ms": round(statistics.median((f[0] - f[2]) * 1000 for f in seen if f[0] < start + 1)) if any(f[0] < start + 1 for f in seen) else None,
    }


def run_scene(c, scene, seconds, fps, steps=(), tool=None):
    """Runs scene.py for `seconds`; `steps` are (at seconds, request) for
    the host during it. With `tool`, the client's and the server's memory
    halfway through. Returns the frames' window."""
    # Every tool's screen is rebuilt while a scene runs: what it shows is
    # checked for tearing and flashes (`Client.look`).
    c.emulate = True
    c.send("clear; python3 /tmp/scene.py %s %g %g\r" % (scene, seconds, fps))
    first = None
    end = time.time() + 15
    while first is None and time.time() < end:
        got = [f for f in c.frames if f[3] == scene]
        first = got[0][0] if got else None
        time.sleep(0.01)
    start = first or time.time()
    c.watching = True
    rx = rx_bytes()
    marks = {}
    for at, request in steps:
        time.sleep(max(0.0, start + at - time.time()))
        if request.startswith("block"):
            block(request == "block on")
        else:
            ask(request)
        # From when the change is made.
        marks[request] = time.time()
    if tool:
        time.sleep(max(0.0, start + seconds / 2 - time.time()))
        marks["rss_client_kb"] = rss_kb(PROCESS[tool])
        server = ask("rss")
        marks["rss_server_kb"] = int(server) if server.isdigit() else None
    time.sleep(max(0.0, start + seconds - time.time()))
    stop = time.time()
    c.watching = False
    marks["kbit_s"] = round((rx_bytes() - rx) * 8 / 1000 / (stop - start))
    # Frames still on their way (ssh, et on a slow link) are not counted;
    # a clean screen for what follows.
    c.send("\x03")
    time.sleep(0.3)
    c.emulate = True
    c.fresh()
    c.send("clear\r")
    c.settle()
    return start, stop, marks


def echo_test(c, n):
    """Keystroke to its echo on the screen, n times."""
    c.emulate = True
    lat, typed = [], ""
    rng = random.Random(7)
    for _ in range(n):
        ch = rng.choice("abcdefghijklmnopqrstuvwxyz")
        typed += ch
        start = time.time()
        c.send(ch)
        seen = c.wait(lambda: c.screen.display[c.screen.cursor.y][: c.screen.cursor.x].endswith(typed), 5, 0.002)
        lat.append(None if seen is None else round((seen - start) * 1000, 1))
        time.sleep(0.25 + rng.random() * 0.1)
    c.send("\x15")
    time.sleep(0.3)
    ok = sorted(v for v in lat if v is not None)
    return {"echo_ms": round(statistics.median(ok), 1) if ok else None, "echo_max_ms": max(ok) if ok else None, "lost": lat.count(None)}


def flood_test(c, lines, timeout=30):
    """`seq 1 lines`, until the line after it is shown (None: not within
    `timeout` s, as ssh and et on a slow link)."""
    c.emulate = False
    start, before = time.time(), c.bytes
    c.send("seq 1 %d; printf 'FL%%sNE7\\n' XD\r" % lines)
    end = start + timeout
    while b"FLXDNE7" not in c.tail and time.time() < end:
        time.sleep(0.005)
    took = time.time() - start if b"FLXDNE7" in c.tail else None
    got = c.bytes - before
    time.sleep(0.2)
    c.emulate = True
    c.fresh()
    c.send("clear\r")
    c.settle()
    return {"flood_s": None if took is None else round(took, 2), "bytes_to_terminal": got}


def ticker_event(c, job, during):
    """A ticker (10 a second) on the server; `during` acts after 2 s and
    returns when the link is back. The longest freeze and how long after
    the link came back the ticker moved again."""
    seconds = job["duration"]
    c.emulate = c.diffs
    c.send("clear; python3 /tmp/scene.py ticker %d 10\r" % (seconds + 120))
    end = time.time() + 15
    while not [f for f in c.frames if f[3] == "ticker"] and time.time() < end:
        time.sleep(0.01)
    start = time.time()
    time.sleep(1)
    back = during()
    time.sleep(max(seconds, 2.0))
    stop = time.time()
    seen = frames_between(c, "ticker", start, stop)
    after = [f[0] for f in seen if f[0] > back]
    times = [start] + [f[0] for f in seen] + [stop]
    c.send("\x03")
    time.sleep(0.3)
    c.emulate = True
    c.fresh()
    c.send("clear\r")
    c.settle()
    return {
        "freeze_s": round(max(b - a for a, b in zip(times, times[1:])), 2),
        "resumed_s": round(after[0] - back, 2) if after else None,
    }


def main_run(job):
    tool, test = job["tool"], job["test"]
    cols, rows = job.get("cols", 100), job.get("rows", 30)
    name = "lab%d" % (time.time() * 1000 % 1e9)
    out = {"tool": tool, "test": test}
    c, out["connect_s"] = connect(tool, name, cols, rows, job.get("connect_timeout", 30))
    if out["connect_s"] is None:
        out["error"] = "no prompt"
        c.finish()
        return out
    if test == "render":
        start, stop, marks = run_scene(c, job["scene"], job["duration"], job["scene_fps"], tool=job["tool"])
        seen = frames_between(c, job["scene"], start, stop)
        out.update(render_stats(seen, start, stop), kbit_s=marks["kbit_s"], stalls=stalls(seen), stall_at=stall_times(seen), **screen_stats(c, start, stop, seen),
                   rss_client_kb=marks["rss_client_kb"], rss_server_kb=marks["rss_server_kb"])
    elif test == "switch":
        # The link changes during the scene: how fast the tool follows.
        at = job["switch_at"]
        start, stop, marks = run_scene(c, job["scene"], at + job["duration"], job["scene_fps"], [(at, "shape " + job["to"])])
        seen = frames_between(c, job["scene"], start, stop)
        out.update(render_stats(seen, start, stop), stalls=stalls(seen), **screen_stats(c, start, stop, seen))
        moved = marks["shape " + job["to"]]
        before = render_stats([f for f in seen if f[0] < moved], start, moved)
        after = render_stats([f for f in seen if f[0] >= moved], moved, stop)
        out.update(fps_before=before["fps"], fps_after=after["fps"], age_after_ms=after["age_ms"])
        # Seconds from the change until a second at 90% of the rate the
        # tool settles at (the median of the last half).
        per = after["per_s"]
        steady = statistics.median(per[len(per) // 2:]) if per else 0
        out["adapt_s"] = next((k for k, v in enumerate(per) if v >= 0.9 * steady), None) if steady else None
    elif test == "echo":
        out.update(echo_test(c, job.get("keys", 20)))
    elif test == "flood":
        out.update(flood_test(c, job.get("lines", 200000)))
    elif test == "outage":
        def outage():
            block(True)
            time.sleep(job.get("down", 10))
            block(False)
            return time.time()
        out.update(ticker_event(c, job, outage))
    elif test == "roam":
        def roam():
            ask("roam")
            return time.time()
        out.update(ticker_event(c, job, roam))
    elif test == "reattach":
        c.send("export KEEP=ke''pt; echo SE''T\r")
        c.wait(lambda: c.has_line("SET"), 10)
        c.kill()
        time.sleep(0.3)
        start = time.time()
        c, ok = connect(tool, name, cols, rows)
        out["reconnect_s"] = None if ok is None else round(time.time() - start, 2)
        if ok is not None:
            c.send("echo K=$KEEP\r")
            out["same_shell"] = c.wait(lambda: c.has_line("K=kept"), 8) is not None
    c.finish()
    return out


def main_watch(log, status_path, frames_path):
    """A demo pane: follows what the client wrote (`script -f` keeps it in
    the log), rebuilds the screen with pyte, and every quarter second
    writes frames a second, frame age, how long the screen stood still and
    the download rate; each new frame goes to the frames log."""
    screen = pyte.Screen(400, 200)
    stream = pyte.ByteStream(screen)

    while not os.path.exists(log):
        time.sleep(0.1)
    src = open(log, "rb")
    frames = open(frames_path, "a")
    seen = []
    sync_open = False
    rx = [(time.time(), rx_bytes())]
    last_status = 0.0
    while True:
        data = src.read(65536)
        if data:
            # As a terminal shows it: at each end of a synchronized update,
            # or, outside one, after each read.
            pieces = data.split(SYNC_END)
            for i, piece in enumerate(pieces):
                ended = i < len(pieces) - 1
                stream.feed(piece + (SYNC_END if ended else b""))
                if ended:
                    sync_open = False
                elif SYNC_BEGIN in piece:
                    sync_open = True
                if sync_open:
                    continue
                row = "".join(screen.buffer[0][x].data for x in range(min(screen.columns, 80)))
                m = FRAME_TEXT.search(row)
                if m:
                    n, made, scene = int(m.group(1)), int(m.group(2)) / 1000.0, m.group(3)
                    if not seen or seen[-1][1] != n:
                        now = time.time()
                        seen.append((now, n, made))
                        frames.write("%.3f %d %.3f %s\n" % (now, n, made, scene))
                        frames.flush()
        else:
            time.sleep(0.005)
        now = time.time()
        if now - last_status >= 0.25:
            last_status = now
            rx.append((now, rx_bytes()))
            rx = [r for r in rx if now - r[0] <= 1.0] or rx[-1:]
            recent = [s for s in seen if now - s[0] <= 1.0]
            span = rx[-1][0] - rx[0][0]
            status = {
                "fps": float(len(recent)),
                "still_s": now - seen[-1][0] if seen else None,
                "age_ms": statistics.median((s[0] - s[2]) * 1000 for s in recent) if recent else None,
                "kbit_s": (rx[-1][1] - rx[0][1]) * 8 / 1000 / span if span > 0 else 0.0,
            }
            with open(status_path + ".new", "w") as f:
                json.dump(status, f)
            os.replace(status_path + ".new", status_path)


if __name__ == "__main__":
    if sys.argv[1] == "run":
        print(json.dumps(main_run(json.loads(sys.argv[2]))), flush=True)
    else:
        main_watch(*sys.argv[2:5])
