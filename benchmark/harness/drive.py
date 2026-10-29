#!/usr/bin/env python3
"""Drives one remote-shell client in a pty, as a user at a 120x40 terminal
would, and reports what the screen showed and when, as JSON.

  drive.py run <tool> <test>...    tests: echo flood render idle
  drive.py outage|roam|reattach|udp <tool>

The screen is rebuilt with pyte from what the client writes, so a tool
that sends screen diffs (mosh) is measured the same way as one that sends
the program's bytes (ssh, et, rsh)."""
import fcntl, json, os, pty, random, re, select, signal, struct, subprocess, sys, termios, time
import pyte

COLS, ROWS = 120, 40


def command(tool, name):
    return {
        "ssh": ["ssh", "-tt", "dev@server"],
        "mosh": ["mosh", "dev@server"],
        "mosh-np": ["mosh", "--predict=never", "dev@server"],
        "et": ["et", "dev@server:2022"],
        "rsh": ["rsh", "server", name],
        "rsh-np": ["rsh", "server", name],
        "rsh-tcp": ["rsh", "server", name],
    }[tool]


class Term:
    def __init__(self, tool, name):
        env = dict(os.environ, TERM="xterm-256color", LANG="C.UTF-8", RSH_PEER="server:2222")
        if tool == "rsh-np":
            env["RSH_PREDICTION"] = "never"
        argv = command(tool, name)
        pid, fd = pty.fork()
        if pid == 0:
            os.execvpe(argv[0], argv, env)
        fcntl.ioctl(fd, termios.TIOCSWINSZ, struct.pack("HHHH", ROWS, COLS, 0, 0))
        self.pid, self.fd = pid, fd
        self.fresh_screen()
        self.tail = b""
        self.bytes = 0
        self.alive = True
        self.on_data = None

    def fresh_screen(self):
        self.screen = pyte.Screen(COLS, ROWS)
        self.stream = pyte.ByteStream(self.screen)
        self.emulate = True

    def pump(self, timeout):
        if not self.alive:
            time.sleep(timeout)
            return
        ready, _, _ = select.select([self.fd], [], [], max(0.0, timeout))
        if not ready:
            return
        try:
            data = os.read(self.fd, 1 << 16)
        except OSError:
            data = b""
        if not data:
            self.alive = False
            return
        self.bytes += len(data)
        self.tail = (self.tail + data)[-256:]
        if self.emulate:
            self.stream.feed(data)
        if self.on_data:
            self.on_data()

    def send(self, text):
        os.write(self.fd, text.encode() if isinstance(text, str) else text)

    def wait(self, pred, timeout):
        """When `pred` first held, or None."""
        end = time.time() + timeout
        while True:
            if pred():
                return time.time()
            left = end - time.time()
            if left <= 0:
                return None
            self.pump(min(0.02, left))

    def lines(self):
        return [line.rstrip() for line in self.screen.display]

    def has_line(self, text):
        """A line that starts with `text` as a word (rsh draws its header at
        the right end of row 0, so lines are not compared whole)."""
        return any(line == text or line.startswith(text + " ") for line in self.lines())

    def run_until_marker(self, cmd, marker, timeout):
        """Types `cmd` and waits for `marker` to be printed on its own."""
        self.send(cmd + "\r")
        return self.wait(lambda: self.has_line(marker), timeout)

    def finish(self):
        if self.alive:
            self.send("\r exit\r")
            self.wait(lambda: not self.alive, 8)
        if self.alive:
            os.kill(self.pid, signal.SIGKILL)
        try:
            os.waitpid(self.pid, 0)
        except ChildProcessError:
            pass


def connect(tool, name, timeout=30):
    """Starts the client; the time is until the remote shell's prompt
    (`...$ `) is on the screen, before anything is typed."""
    t = Term(tool, name)
    t0 = time.time()
    prompt = t.wait(lambda: t.screen.display[t.screen.cursor.y][: t.screen.cursor.x].rstrip().endswith("$"), timeout)
    if prompt is None:
        return t, None
    # Then the shell is set up; typed again only if a key was lost.
    ready = None
    while time.time() - t0 < timeout and ready is None:
        t.send("export PS1='$ ' PROMPT_COMMAND=; echo RE''ADY\r")
        ready = t.wait(lambda: t.has_line("READY"), 5)
    if ready is None:
        return t, None
    t.send("clear\r")
    t.wait(lambda: not t.has_line("READY"), 3)
    return t, prompt - t0


def echo_test(t, n=20):
    """Keystroke to its echo on the screen, n times."""
    lat, typed = [], ""
    random.seed(7)
    for _ in range(n):
        ch = random.choice("abcdefghijklmnopqrstuvwxyz")
        typed += ch
        start = time.time()
        t.send(ch)
        seen = t.wait(lambda: t.screen.display[t.screen.cursor.y][: t.screen.cursor.x].endswith(typed), 5)
        lat.append(None if seen is None else round((seen - start) * 1000, 1))
        # Let the echo settle, so the next key is not typed into a burst.
        t.wait(lambda: False, 0.25 + random.random() * 0.1)
    t.send("\x15")
    t.wait(lambda: False, 0.5)
    return lat


def flood_test(t, count=200000, timeout=120):
    """`seq 1 count` (1.3 MB for 200000): until the line after it is shown."""
    t.emulate = False
    start = time.time()
    before = t.bytes
    t.send("seq 1 %d; printf 'FL%%sNE7\\n' XD\r" % count)
    done = t.wait(lambda: b"FLXDNE7" in t.tail, timeout)
    took = None if done is None else round(done - start, 2)
    got = t.bytes - before
    t.wait(lambda: False, 1)
    t.fresh_screen()
    t.send("clear\r")
    t.wait(lambda: False, 1)
    return {"seconds": took, "bytes_to_terminal": got}


def render_test(t, seconds=8, fps=30):
    """A 30 fps full-screen animation: frames seen, gaps, frame age."""
    seen = []
    frame_re = re.compile(r"FRAME (\d+) T=(\d+)")

    def watch():
        m = frame_re.search(t.screen.display[0])
        if m:
            n = int(m.group(1))
            if not seen or seen[-1][1] != n:
                seen.append((time.time(), n, int(m.group(2)) / 1000.0))

    t.on_data = watch
    before = t.bytes
    t.send("python3 /usr/local/bin/anim.py %s %s\r" % (seconds, fps))
    t.wait(lambda: t.has_line("frames %d" % int(seconds * fps)) or any(l.startswith("frames ") for l in t.lines()), seconds + 30)
    t.on_data = None
    got = t.bytes - before
    produced = int(seconds * fps)
    for line in t.lines():
        if line.startswith("frames "):
            produced = int(line.split()[1])
    gaps = [round((b[0] - a[0]) * 1000, 1) for a, b in zip(seen, seen[1:])]
    ages = [round((s[0] - s[2]) * 1000, 1) for s in seen]
    t.send("clear\r")
    t.wait(lambda: False, 1)
    return {
        "produced": produced,
        "shown": len(seen),
        "gaps_ms": gaps,
        "ages_ms": ages,
        "last_frame": seen[-1][1] if seen else None,
        "bytes_to_terminal": got,
    }


def block(on):
    """Cuts (or restores) everything between this container and the server."""
    action = "-I" if on else "-D"
    for chain, flag in (("OUTPUT", "-d"), ("INPUT", "-s")):
        subprocess.run(["iptables", action, chain, flag, "server", "-j", "DROP"], check=False, capture_output=True)


def outage_test(t, seconds=20):
    """Network gone for `seconds`; keys typed meanwhile run once it is back."""
    block(True)
    t.wait(lambda: False, 2)
    t.send("echo DUR''ING\r")
    t.wait(lambda: False, seconds - 2)
    block(False)
    back = time.time()
    seen = t.wait(lambda: t.has_line("DURING"), 60)
    after = None
    if t.alive:
        after = t.run_until_marker("echo AF''TER", "AFTER", 20)
    return {
        "typed_while_down_ran": seen is not None,
        "seconds_to_show_it": None if seen is None else round(seen - back, 2),
        "usable_after": after is not None,
        "client_alive": t.alive,
    }


def roam_test(t):
    """The client's address changes, as when Wi-Fi gives way to LTE: the
    orchestrator moves this container to another address of the network
    (`docker network disconnect` / `connect --ip`) when asked."""
    open("/tmp/roam-request", "w").close()
    while not os.path.exists("/tmp/roam-done"):
        t.pump(0.1)
    moved = time.time()
    seen = t.run_until_marker("echo RO''AMED", "ROAMED", 45)
    return {
        "usable_after": seen is not None,
        "seconds": None if seen is None else round(seen - moved, 2),
        "client_alive": t.alive,
    }


def main():
    mode, tool = sys.argv[1], sys.argv[2]
    name = "b%d" % (time.time() * 1000 % 1e9)
    out = {"tool": tool, "mode": mode, "phases": []}
    if mode == "udp" or tool == "rsh-tcp":
        # A network that passes TCP only (many corporate VPNs and firewalls);
        # rsh-tcp runs every test that way.
        subprocess.run(["iptables", "-I", "OUTPUT", "-p", "udp", "-d", "server", "-j", "DROP"], check=False)
    t, connected = connect(tool, name, 20 if mode == "udp" else 30)
    if mode == "udp" and tool != "rsh-tcp":
        subprocess.run(["iptables", "-D", "OUTPUT", "-p", "udp", "-d", "server", "-j", "DROP"], check=False)
    out["connect_s"] = None if connected is None else round(connected, 2)
    if connected is None:
        out["error"] = "no prompt"
        t.finish()
        print(json.dumps(out))
        return

    def phase(label, fn):
        start = time.time()
        result = fn()
        out["phases"].append({"name": label, "start": start, "end": time.time()})
        out[label] = result

    if mode == "run":
        for test in sys.argv[3:]:
            if test == "echo":
                phase("echo", lambda: echo_test(t))
            elif test == "flood":
                phase("flood", lambda: flood_test(t))
            elif test == "bigflood":
                # 3,000,000 lines, 21 MB: CPU per megabyte on each end.
                phase("bigflood", lambda: flood_test(t, 3000000, 600))
            elif test == "render":
                phase("render", lambda: render_test(t))
            elif test == "idle":
                phase("idle", lambda: t.wait(lambda: False, 10) or "")
    elif mode == "connect":
        pass
    elif mode == "osc":
        # Which OSC sequences reach this terminal, by their payloads.
        sys.path.insert(0, "/usr/local/bin")
        from oscs import SEQS
        raw = bytearray()
        t.emulate = False
        t.on_data = lambda: None
        start = t.bytes
        t.send("python3 /usr/local/bin/oscs.py\r")
        end = time.time() + 15
        while time.time() < end and b"OSCDONE" not in raw:
            before = len(raw)
            ready, _, _ = select.select([t.fd], [], [], 0.1)
            if ready:
                try:
                    raw += os.read(t.fd, 1 << 16)
                except OSError:
                    break
        t.wait(lambda: False, 1.5)
        out["osc"] = {name: key.encode() in raw for name, _, key in SEQS}
        out["osc_exact"] = {name: ("\x1b]" + body).encode() in raw for name, body, _ in SEQS}
    elif mode == "many":
        # Ten sessions open at once, idle: memory per session on each end.
        others = []
        for i in range(9):
            other, ok = connect(tool, "%s-%d" % (name, i))
            others.append(other)
        start = time.time()
        t.wait(lambda: False, 5)
        out["phases"].append({"name": "many", "start": start, "end": time.time()})
        out["many"] = {"sessions": 1 + sum(1 for o in others if o.alive)}
        for other in others:
            other.finish()
    elif mode == "outage":
        phase("outage", lambda: outage_test(t))
    elif mode == "roam":
        phase("roam", lambda: roam_test(t))
    elif mode == "reattach":
        t.run_until_marker("export KEEP=ke''pt; echo SE''T", "SET", 10)
        os.kill(t.pid, signal.SIGKILL)
        os.waitpid(t.pid, 0)
        t.alive = False
        time.sleep(2)
        t2, ok = connect(tool, name)
        same = None
        if ok is not None:
            same = t2.run_until_marker("echo K=$KEEP", "K=kept", 8) is not None
        out["reattach"] = {"reconnected": ok is not None, "same_shell": same}
        t = t2
    t.finish()
    if tool == "rsh-tcp":
        subprocess.run(["iptables", "-D", "OUTPUT", "-p", "udp", "-d", "server", "-j", "DROP"], check=False)
    print(json.dumps(out))


main()
