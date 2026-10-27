#!/usr/bin/env python3
"""The lab's scenes (lab.py), run on the server in each tool's shell.

  scene.py <compress|bandwidth|ticker> <seconds> [fps]

Each keeps row 0 as `FRAME nnnnnn T=<ms since epoch> <name>`, so the
client side can count the frames that reach the screen and how old they
are when they do (agent.py). `compress` and `bandwidth` also start every
other row with the frame's number: a screen whose rows disagree shows
parts of two frames at once (tearing), one without them was cleared
(a flash).

  compress   a dashboard table: every row rewritten each frame, but in the
             same layout with a few changing numbers and bars, so it
             compresses well (deflate shrinks a frame many times over)
  bandwidth  random characters in random colours on every cell: nothing
             to compress, so how much the link carries decides
  ticker     a clock and a counter (10 a second by default) for the
             connection scenarios: it stops on screen while the link is
             gone and shows how it comes back; it names the shell it runs
             in, so a reattach shows whether it is the same shell
"""
import os, random, sys, time

scene, seconds = sys.argv[1], float(sys.argv[2])
fps = float(sys.argv[3]) if len(sys.argv) > 3 else 30.0
cols, rows = os.get_terminal_size()
out = sys.stdout.buffer
rng = random.Random(1)


def header(n):
    text = "FRAME %06d T=%d %s" % (n, int(time.time() * 1000), scene)
    return "\x1b[1;1H\x1b[7m" + text.ljust(cols)[:cols] + "\x1b[0m"


def compress(n):
    """About cols×rows characters a frame in a fixed layout."""
    parts = [header(n)]
    bar_width = max(10, cols - 53)
    for r in range(2, rows + 1):
        i = r - 2
        load = (n * (i % 7 + 1) + i * 13) % 100
        bar = "█" * (load * bar_width // 100)
        state = "\x1b[32m OK \x1b[0m" if load < 80 else "\x1b[31mBUSY\x1b[0m"
        line = "%06d server-%03d  cpu %3d%%  \x1b[36m%-*s\x1b[0m  %s  req %6d" % (n % 1000000, i, load, bar_width, bar, state, n * 17 + i)
        parts.append("\x1b[%d;1H%s\x1b[K" % (r, line))
    return "".join(parts)


CHARS = "".join(chr(c) for c in range(33, 127))


def bandwidth(n):
    """Every cell a random character, colour runs of random length."""
    parts = [header(n)]
    for r in range(2, rows + 1):
        line = ["\x1b[0m%06d " % (n % 1000000)]
        c = 7
        while c < cols:
            run = rng.randint(1, 6)
            line.append("\x1b[38;5;%dm" % rng.randint(16, 231) + "".join(rng.choice(CHARS) for _ in range(min(run, cols - c))))
            c += run
        parts.append("\x1b[%d;1H%s\x1b[0m" % (r, "".join(line)))
    return "".join(parts)


def ticker(n):
    elapsed = n / fps
    lines = [
        "the program keeps running on the server",
        "",
        "  counter   %8d" % n,
        "  running   %8.1f s" % elapsed,
        "  clock     %s" % time.strftime("%H:%M:%S"),
        "  shell     pid %d" % os.getppid(),
    ]
    return header(n) + "".join("\x1b[%d;3H\x1b[K%s" % (r + 3, l) for r, l in enumerate(lines))


draw = {"compress": compress, "bandwidth": bandwidth, "ticker": ticker}[scene]
out.write(b"\x1b[?25l\x1b[2J")
start = time.time()
n = 0
while time.time() - start < seconds:
    out.write(draw(n).encode())
    out.flush()
    n += 1
    time.sleep(max(0.0, start + n / fps - time.time()))
out.write(b"\x1b[?25h\x1b[2J\x1b[H")
out.flush()
print("frames %d" % n)
