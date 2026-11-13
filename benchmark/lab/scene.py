#!/usr/bin/env python3
"""The lab's scenes (lab.py), run on the server in each tool's shell.

  scene.py <compress|bandwidth|agent|ticker> <seconds> [fps] [warm-up seconds] [label] [start epoch]

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
  agent      Claude Code / Ink-style coding agent TUI: Thinking spinner,
             token-by-token RGB code & prose streaming, and scrolling diff
             review above a bordered prompt box and status footer
  ticker     a clock and a counter (10 a second by default) for the
             connection scenarios: it stops on screen while the link is
             gone and shows how it comes back; it names the shell it runs
             in, so a reattach shows whether it is the same shell

A count down (the demo's) first shows the label (the network and the
scene) and the seconds left, to 0.0, in large digits, centred: no frame
header, so nothing of it counts as a frame. (The demo leaves the scene's
first second out of its results: the links warm up on the scene itself.)
"""
import os, random, sys, time

scene, seconds = sys.argv[1], float(sys.argv[2])
fps = float(sys.argv[3]) if len(sys.argv) > 3 else 30.0
warmup = float(sys.argv[4]) if len(sys.argv) > 4 else 0.0
label = sys.argv[5] if len(sys.argv) > 5 else ""
start_at = float(sys.argv[6]) if len(sys.argv) > 6 else None
cols, rows = os.get_terminal_size()
out = sys.stdout.buffer
rng = random.Random(1)
# The header's row: rsh's badge (13 columns at the right of row 0 from 33
# columns) would cover its frame time and name in a narrower pane.
top = 1 if cols >= 43 else 2


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


SPINNER = ("⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏")

AGENT_HISTORY = (
    "\x1b[38;2;152;195;121m●\x1b[0m \x1b[1mRead\x1b[0m(\x1b[36msrc/terminal.rs\x1b[0m) \x1b[90m· lines 1140-1350 (211 lines, 0.02s)\x1b[0m",
    "\x1b[38;2;152;195;121m●\x1b[0m \x1b[1mRead\x1b[0m(\x1b[36msrc/client/viewport.rs\x1b[0m) \x1b[90m· lines 310-520 (211 lines, 0.01s)\x1b[0m",
    "\x1b[38;2;97;175;239m●\x1b[0m Checking row hashes first, then choosing cell jumps (`CHA`) vs span merge:",
    "  \x1b[90m 1 │\x1b[0m \x1b[38;2;198;120;221mpub(crate) fn\x1b[0m \x1b[38;2;97;175;239mplan_row_diff\x1b[0m(row: \x1b[38;2;229;192;123musize\x1b[0m, cur: &\x1b[38;2;229;192;123mRow\x1b[0m, pen: \x1b[38;2;229;192;123mAttr\x1b[0m)",
    "  \x1b[90m 2 │\x1b[0m     \x1b[38;2;198;120;221mlet\x1b[0m end = \x1b[38;2;97;175;239mrow_content_end\x1b[0m(cur, cols);",
    "  \x1b[90m 3 │\x1b[0m     \x1b[38;2;198;120;221mlet\x1b[0m atomic = \x1b[38;2;97;175;239matomic_runs\x1b[0m(cur, old?, cols, last_end);",
    "  \x1b[90m 4 │\x1b[0m     \x1b[38;2;198;120;221mlet\x1b[0m merged = \x1b[38;2;97;175;239mmerge_runs\x1b[0m(cur, cols, &atomic, \x1b[38;2;209;154;102mNone\x1b[0m, pen);",
    "  \x1b[90m 5 │\x1b[0m     \x1b[38;2;97;175;239memit_runs\x1b[0m(row, cur, cols, &merged, erase_tail, pen)",
)

AGENT_STREAM = (
    ("prose", "● Instead of resending `first..=last` on every edit, we compare exact wire byte costs per gap."),
    ("code", "let jump = cha_bytes(next_col + 1) + sgr_delta_bytes(pen_after, next_cell) + MERGE_MARGIN;"),
    ("code", "let bridge = emit_span_bytes(unchanged_gap, pen_after) + sgr_delta_bytes(pen_gap, next_cell);"),
    ("code", "if bridge <= jump { merged.last_mut().unwrap().1 = next_end; } else { merged.push(next); }"),
    ("prose", "● Shrunken line tails emit `CSI K` (`EL`) when shorter than trailing spaces, bounded by full-row sweep."),
    ("code", "if last_end > content_end && with_el.len() <= best.bytes.len() { best = with_el; }"),
    ("prose", "● Inline OSC 8 links and incremental SGR transitions avoid falling back to full-screen repaints."),
)


def agent_diff_line(idx, width):
    lineno = 1240 + idx
    inner = max(16, width - 6)
    mod = idx % 7
    if mod == 0:
        txt = "Update(src/terminal.rs) · hunk #%d (+18 -6)" % (idx // 7 + 1)
        return "\x1b[38;2;152;195;121m●\x1b[0m \x1b[1m%s\x1b[0m" % txt[: max(10, width - 3)]
    if mod in (1, 2):
        body = ("%4d - out.push_str(&self.span(row, first, last + 1));" % lineno).ljust(inner)[:inner]
        return "  \x1b[48;2;68;24;28m\x1b[38;2;240;128;128m %s \x1b[0m" % body
    if mod in (3, 4, 5):
        body = ("%4d + body.push_str(&plan_row_diff(row, cur, old, cols, pen).bytes);" % lineno).ljust(inner)[:inner]
        return "  \x1b[48;2;22;58;34m\x1b[38;2;130;225;155m %s \x1b[0m" % body
    body = ("%4d │ let (cols, rows) = (now_term.columns(), now_term.screen_lines());" % lineno)[:inner]
    return "  \x1b[90m%s\x1b[0m" % body


AGENT_DIFF_POOL = None


def agent_tui(n):
    """Claude Code / Ink-style TUI cycle:
    1) Thinking spinner (Ink redraws active tail + bottom chrome)
    2) Token-by-token streaming (prose + 24-bit RGB code blocks)
    3) Diff review scrolling up & down between fixed banner and prompt box
    """
    global AGENT_DIFF_POOL
    if AGENT_DIFF_POOL is None:
        AGENT_DIFF_POOL = [agent_diff_line(i, cols) for i in range(96)]

    parts = ["\x1b[?2026h", header(n)]
    inner = max(10, cols - 4)
    banner_r = top + 1
    if rows >= top + 10:
        title = " Claude Code · ~/personal/rsh (main) "[:inner]
        parts.append(
            "\x1b[%d;1H\x1b[38;2;215;119;87m╭─%s%s─╮\x1b[0m\x1b[K"
            % (banner_r, title, "─" * max(0, inner - len(title)))
        )
        prompt_q = "❯ Optimize cell-level terminal diff & verify high-RTT FPS"[: max(6, inner - 1)]
        parts.append(
            "\x1b[%d;1H\x1b[38;2;215;119;87m│\x1b[0m %s%s \x1b[38;2;215;119;87m│\x1b[0m\x1b[K"
            % (banner_r + 1, prompt_q, " " * max(0, inner - 1 - len(prompt_q)))
        )
        parts.append("\x1b[%d;1H\x1b[38;2;215;119;87m╰─%s─╯\x1b[0m\x1b[K" % (banner_r + 2, "─" * inner))
        body_top = banner_r + 3
    else:
        body_top = banner_r

    chrome_h = 6 if rows - body_top >= 9 else 3
    body_bot = max(body_top, rows - chrome_h)
    body_h = body_bot - body_top + 1

    # Cycle length ~4 seconds so even a 4s demo window shows all 3 phases.
    cycle = max(48, int(round(fps * 4.0)))
    step = n % cycle
    p1 = cycle // 5          # ~20%: Thinking spinner
    p2 = (cycle * 3) // 5    # ~40%: Token streaming; remaining ~40%: Diff scroll

    elapsed_s = n / max(fps, 1.0)
    tokens = 840 + n * 7
    spin = SPINNER[n % len(SPINNER)]

    if step < p1:
        verb = "Thinking…"
        if n == 0 or step == 0:
            for i in range(body_h):
                r = body_top + i
                txt = AGENT_HISTORY[i] if i < len(AGENT_HISTORY) else ""
                parts.append("\x1b[%d;1H%s\x1b[0m\x1b[K" % (r, txt))
        else:
            # Ink re-emits the active message tail above the spinner
            tail_start = max(0, min(len(AGENT_HISTORY), body_h) - 2)
            for i in range(tail_start, body_h):
                r = body_top + i
                txt = AGENT_HISTORY[i] if i < len(AGENT_HISTORY) else ""
                parts.append("\x1b[%d;1H%s\x1b[0m\x1b[K" % (r, txt))
    elif step < p2:
        verb = "Generating…"
        prog = (step - p1) / max(1, p2 - p1)
        total_chars = sum(len(s) for _, s in AGENT_STREAM)
        budget = max(1, int(prog * total_chars))
        built = list(AGENT_HISTORY)
        rem = budget
        for kind, full in AGENT_STREAM:
            if rem <= 0:
                break
            piece = full[: min(len(full), rem)]
            rem -= len(piece)
            if kind == "code":
                w = max(12, cols - 8)
                built.append("  \x1b[48;2;30;34;42m\x1b[38;2;171;178;191m %-*s \x1b[0m" % (w, piece[:w]))
            else:
                built.append(piece[: max(10, cols - 2)])
        visible = built[-body_h:]
        # Ink redraws the active assistant response block each token tick
        start_idx = 0 if step == p1 else max(0, len(visible) - min(len(AGENT_STREAM) + 2, body_h))
        for i in range(start_idx, body_h):
            r = body_top + i
            txt = visible[i] if i < len(visible) else ""
            parts.append("\x1b[%d;1H%s\x1b[0m\x1b[K" % (r, txt))
    else:
        verb = "Applying patch & reviewing diff…"
        dstep = step - p2
        span = max(1, cycle - p2)
        half = (span * 2) // 3
        offset = dstep if dstep <= half else max(0, half - (dstep - half) * 2)
        for i in range(body_h):
            r = body_top + i
            parts.append("\x1b[%d;1H%s\x1b[0m\x1b[K" % (r, AGENT_DIFF_POOL[(offset + i) % len(AGENT_DIFF_POOL)]))

    c_top = body_bot + 1
    status_line = (
        "\x1b[38;2;215;119;87m%s\x1b[0m \x1b[1;38;2;230;190;140m%s\x1b[0m \x1b[90m(%.1fs · ↓ %d tokens · esc to interrupt)\x1b[0m"
        % (spin, verb, elapsed_s, tokens)
    )
    parts.append("\x1b[%d;1H%s\x1b[K" % (c_top, status_line))
    if chrome_h >= 6:
        box_w = max(8, cols - 2)
        prompt_in = "❯ Run ssh vs rsh Claude Code TUI traffic & FPS demo"[: max(4, box_w - 2)]
        pad = " " * max(0, box_w - 2 - len(prompt_in))
        parts.append("\x1b[%d;1H\x1b[38;2;110;118;129m╭%s╮\x1b[0m\x1b[K" % (c_top + 1, "─" * box_w))
        parts.append(
            "\x1b[%d;1H\x1b[38;2;110;118;129m│\x1b[0m %s%s \x1b[38;2;110;118;129m│\x1b[0m\x1b[K"
            % (c_top + 2, prompt_in, pad)
        )
        parts.append("\x1b[%d;1H\x1b[38;2;110;118;129m╰%s╯\x1b[0m\x1b[K" % (c_top + 3, "─" * box_w))
        parts.append(
            "\x1b[%d;1H  \x1b[38;2;152;195;121m⏵⏵ accept edits on\x1b[0m \x1b[90m· 6 files (+1359 -240)\x1b[0m\x1b[K"
            % (c_top + 4)
        )
        parts.append(
            "\x1b[%d;1H  \x1b[38;2;97;175;239mclaude-opus-4\x1b[0m \x1b[90m· ~/personal/rsh · ctx %d%% · %dk/200k\x1b[0m\x1b[K"
            % (c_top + 5, 38 + (n // 90) % 20, 42 + (tokens // 500))
        )
    elif c_top + 1 <= rows:
        parts.append(
            "\x1b[%d;1H\x1b[38;2;97;175;239mclaude-opus-4\x1b[0m \x1b[90m· ctx %d%% · %dk/200k\x1b[0m\x1b[K"
            % (c_top + 1, 38 + (n // 90) % 20, 42 + (tokens // 500))
        )
    parts.append("\x1b[?2026l")
    return "".join(parts)


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


def countdown(left):
    """The count down's screen: the label's parts, then the seconds left,
    centred."""
    lines = [("\x1b[1m", part) for part in label.split(" · ") if part] + [("", "")] + [("\x1b[1;33m", row) for row in big("%.1fs" % left)]
    first = max(1, (rows - len(lines)) // 2 + 1)
    parts = []
    for i, (style, text) in enumerate(lines):
        text = text[:cols]
        parts.append("\x1b[%d;%dH\x1b[2K%s%s\x1b[0m" % (first + i, max(1, (cols - len(text)) // 2 + 1), style, text))
    return "".join(parts)


draw = {"compress": compress, "bandwidth": bandwidth, "agent": agent_tui, "ticker": ticker}[scene]
# No autowrap: in a pane narrower than a row the row is cut at the edge
# instead of wrapping and scrolling the header away.
out.write(b"\x1b[?25l\x1b[?7l\x1b[2J")
start = time.time()
if start_at is not None:
    warmup = max(0, start_at - start)
n = 0
try:
    # Ten steps a second: the count down shows tenths.
    while warmup > 0 and time.time() - start < warmup:
        out.write(countdown(max(0.0, warmup - (time.time() - start))).encode())
        out.flush()
        n += 1
        time.sleep(max(0.0, start + n / 10 - time.time()))
    if warmup > 0:
        out.write(countdown(0.0).encode() + b"\x1b[2J")
        start = time.time()
        n = 0
    if start_at is not None:
        start = start_at
        n = max(0, int((time.time() - start) * fps))
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
