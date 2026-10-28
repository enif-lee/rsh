#!/usr/bin/env python3
"""The lab's scenes (lab.py), run on the server in each tool's shell.

  scene.py <compress|bandwidth|ticker> <seconds> [fps] [warm-up seconds] [label]

Each keeps row 0 as `FRAME nnnnnn T=<ms since epoch> <name>` (row 1 in a
pane under 43 columns, where rsh's badge covers the end of row 0), so the
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

A warm-up (the demo's) first draws the scene as it will run, so each
tool's link is warm when the measured part starts, with a box over its
middle: the label (the network and the scene) and a count down to 0.00 in
large digits. Its frames are named `warmup` in the header, so none of them
counts as the scene's.
"""
import os, random, sys, time

scene, seconds = sys.argv[1], float(sys.argv[2])
fps = float(sys.argv[3]) if len(sys.argv) > 3 else 30.0
warmup = float(sys.argv[4]) if len(sys.argv) > 4 else 0.0
label = sys.argv[5] if len(sys.argv) > 5 else ""
cols, rows = os.get_terminal_size()
out = sys.stdout.buffer
rng = random.Random(1)
# The header's row: rsh's badge (13 columns at the right of row 0 from 33
# columns) would cover its frame time and name in a narrower pane.
top = 1 if cols >= 43 else 2


# The scene's name in the header: `warmup` while the warm-up runs.
name = scene


def header(n):
    text = "FRAME %06d T=%d %s" % (n, int(time.time() * 1000), name)
    return "\x1b[%d;1H\x1b[7m" % top + text.ljust(cols)[:cols] + "\x1b[0m"


def compress(n):
    """About cols×rows characters a frame in a fixed layout."""
    parts = [header(n)]
    bar_width = max(10, cols - 53)
    for r in range(top + 1, rows + 1):
        i = r - top - 1
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
    for r in range(top + 1, rows + 1):
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
    return header(n) + "".join("\x1b[%d;3H\x1b[K%s" % (r + top + 2, l) for r, l in enumerate(lines))


# Digits five rows high, for the warm-up's count down.
GLYPHS = {
    "0": ["███", "█ █", "█ █", "█ █", "███"],
    "1": [" █ ", "██ ", " █ ", " █ ", "███"],
    "2": ["███", "  █", "███", "█  ", "███"],
    "3": ["███", "  █", "███", "  █", "███"],
    "4": ["█ █", "█ █", "███", "  █", "  █"],
    "5": ["███", "█  ", "███", "  █", "███"],
    "6": ["███", "█  ", "███", "█ █", "███"],
    "7": ["███", "  █", "  █", "  █", "  █"],
    "8": ["███", "█ █", "███", "█ █", "███"],
    "9": ["███", "█ █", "███", "  █", "███"],
    ".": ["   ", "   ", "   ", "   ", " █ "],
    "s": ["   ", "███", "█  ", "  █", "███"],
}


def big(text):
    """`text` in large digits: each glyph doubled in width, its five rows
    halved to three with half blocks."""
    rows5 = ["  ".join("".join(c * 2 for c in GLYPHS[ch][row]) for ch in text) for row in range(5)]
    rows5.append(" " * len(rows5[0]))
    half = {(True, True): "█", (True, False): "▀", (False, True): "▄", (False, False): " "}
    return ["".join(half[(a != " ", b != " ")] for a, b in zip(rows5[r], rows5[r + 1])) for r in (0, 2, 4)]


def warm(left):
    """The warm-up's box over the middle of the scene: the label's parts,
    then the time left."""
    lines = [("\x1b[1;97m", part) for part in label.split(" · ") if part] + [("", "")] + [("\x1b[1;93m", row) for row in big("%.2fs" % left)]
    width = min(cols, max(len(text) for _, text in lines) + 6)
    lines = [("", "")] + lines + [("", "")]
    first = max(1, (rows - len(lines)) // 2 + 1)
    left_col = max(1, (cols - width) // 2 + 1)
    parts = []
    for i, (style, text) in enumerate(lines):
        text = text[:width]
        pad = (width - len(text)) // 2
        parts.append("\x1b[%d;%dH\x1b[48;5;236m%s%s%s\x1b[0m" % (first + i, left_col, " " * pad, style + text + "\x1b[22;39m", " " * (width - pad - len(text))))
    return "".join(parts)


draw = {"compress": compress, "bandwidth": bandwidth, "ticker": ticker}[scene]
# No autowrap: in a pane narrower than a row the row is cut at the edge
# instead of wrapping and scrolling the header away.
out.write(b"\x1b[?25l\x1b[?7l\x1b[2J")
start = time.time()
n = 0
try:
    name = "warmup"
    while warmup > 0 and time.time() - start < warmup:
        out.write((draw(n) + warm(max(0.0, warmup - (time.time() - start)))).encode())
        out.flush()
        n += 1
        time.sleep(max(0.0, start + n / fps - time.time()))
    name = scene
    if warmup > 0:
        start = time.time()
        n = 0
    while time.time() - start < seconds:
        out.write(draw(n).encode())
        out.flush()
        n += 1
        time.sleep(max(0.0, start + n / fps - time.time()))
finally:
    # Also when stopped (Ctrl-C): the shell goes on with wrap and cursor.
    out.write(b"\x1b[?7h\x1b[?25h\x1b[2J\x1b[H")
    out.flush()
print("frames %d" % n)
