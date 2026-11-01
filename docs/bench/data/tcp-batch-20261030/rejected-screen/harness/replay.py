#!/usr/bin/env python3
"""Replay captured reads after measurement, using common terminal criteria.

The full-body criterion compares every body character with scene.py's
deterministic source frame; it excludes the header/badge and SGR attributes.
It is not a monitor scan-out or GPU rendering measurement.
"""
import collections
import concurrent.futures
import json
import math
from pathlib import Path
import re
import statistics
import struct
import sys

import pyte
import agent

BEGIN, END = b"\x1b[?2026h", b"\x1b[?2026l"
TOKENS = (BEGIN, END, b"\x1b[0m", b"\x1b[K")
BOUNDARY = re.compile(rb"\x1b\[(?:\?2026[hl]|0m|K)")


class Source:
    def __init__(self, scene, cols, rows):
        text = (Path(__file__).parent / "scene.py").read_text().split('\ndraw = {')[0]
        text = text.replace("cols, rows = os.get_terminal_size()", f"cols, rows = {cols}, {rows}")
        args = sys.argv
        try:
            sys.argv = ["scene.py", scene, "8", "60"]
            self.namespace = {}
            exec(compile(text, "scene.py", "exec"), self.namespace)
        finally:
            sys.argv = args
        self.draw = self.namespace[scene]
        self.screen = pyte.Screen(cols, rows)
        self.stream = pyte.ByteStream(self.screen)
        self.stream.feed(b"\x1b[?7l")
        self.frames = []

    def body(self, n):
        while len(self.frames) <= n:
            self.stream.feed(self.draw(len(self.frames)).encode())
            self.frames.append(tuple(self.screen.display[1:]))
        return self.frames[n]


def chunks(path):
    with path.open("rb") as f:
        while head := f.read(12):
            at, size = struct.unpack("!dI", head)
            data = f.read(size)
            assert len(data) == size
            yield at, data


def state_time(visible, lo, hi):
    durations = {None: 0., False: 0., True: 0.}
    last_at, state = lo, None
    for at, following in visible:
        if at < lo:
            state = following
            continue
        if at > hi:
            break
        durations[state] += max(0., at - last_at)
        last_at, state = at, following
    durations[state] += max(0., hi - last_at)
    span = max(hi - lo, 1e-9)
    return {"incomplete_pct": durations[False] / span * 100,
            "unknown_pct": durations[None] / span * 100,
            "complete_pct": durations[True] / span * 100}


def observe(path, job, source, include_frames=False):
    screen = pyte.Screen(job["cols"], job["rows"])
    stream = pyte.ByteStream(screen)
    pending, opened = b"", False
    header, events, full = [], [], []
    visible = []
    mismatch, checked = 0, 0
    def add(seq, frame):
        if not seq or frame[1] > seq[-1][1]:
            seq.append(frame)
    def check(at, observe_header=True):
        nonlocal mismatch, checked
        match = agent.header(screen, 60)
        if not match or match.group(3) != job["scene"]:
            if observe_header:
                visible.append((at, None))
            return
        n, made = int(match.group(1)), int(match.group(2)) / 1000
        frame = (at, n, made, job["scene"])
        if observe_header:
            add(header, frame)
        add(events, frame)
        body = tuple(screen.display[1:])
        checked += 1
        complete = body == source.body(n)
        if observe_header:
            # A sync-end or the final state of an unwrapped read, rather
            # than every intermediate SGR/EL boundary in that same read.
            visible.append((at, complete))
        if complete:
            add(full, frame)
        else:
            mismatch += 1
    for at, data in chunks(path):
        data = pending + data
        pending = b""
        # Keep incomplete markers across reads. A frame may finish and the
        # next start in ONE read: check the final row's reset/erase boundary
        # too, rather than silently discarding that completed body.
        keep = max((n for n in range(1, len(BEGIN))
                    if any(n < len(token) and token.startswith(data[-n:]) for token in TOKENS)), default=0)
        if keep:
            data, pending = data[:-keep], data[-keep:]
        pos = 0
        for match in BOUNDARY.finditer(data):
            stream.feed(data[pos:match.end()])
            token = match.group()
            if token in (BEGIN, END):
                opened = token == BEGIN
            if token == END:
                check(at)
            elif not opened and screen.cursor.y == job["rows"] - 1:
                check(at, observe_header=False)
            pos = match.end()
        if pos < len(data):
            stream.feed(data[pos:])
        if not opened:
            check(at)
    lo, hi = job["window_start"], job["window_end"]
    trim = lambda frames: [f for f in frames if lo <= f[0] <= hi]
    after = {}
    if "switch_moved" in job:
        moved = job["switch_moved"]
        after["complete_body_after"] = agent.render_stats([f for f in full if moved <= f[0] <= hi], moved, hi)
        after["visible_state_after"] = state_time(visible, moved, hi)
    detail = {"complete_frames": trim(full)} if include_frames else {}
    return dict(uniform_header=agent.render_stats(trim(header), lo, hi),
                uniform_events=agent.render_stats(trim(events), lo, hi),
                complete_body=agent.render_stats(trim(full), lo, hi),
                visible_state=state_time(visible, lo, hi),
                checked_states=checked, incomplete_states=mismatch, **after, **detail)


def trace_stats(job):
    values = collections.defaultdict(list)
    for side in ("srv", "cli"):
        for line in job.get("trace_" + side, "").splitlines():
            at, kind, us, size = line.split(",")
            values[kind].append(float(us) / 1000)
    def quantile(xs, q):
        return sorted(xs)[min(len(xs)-1, math.ceil(len(xs)*q)-1)]
    return {k: dict(count=len(v), p50_ms=statistics.median(v),
                   p95_ms=quantile(v, .95), p99_ms=quantile(v, .99)) for k, v in values.items()}


SOURCES = {}


def replay_one(item):
    root, tag, parallel, job = item
    if job.get("fps") is None:
        return dict(batch=tag, **job)
    key = (job["scene"], job["cols"], job["rows"])
    if key not in SOURCES:
        SOURCES[key] = Source(*key)
    out = {k: v for k, v in job.items() if not k.startswith("trace_")}
    out.update(observe(root / job["capture"], job, SOURCES[key]), trace=trace_stats(job),
               batch=tag, parallel=parallel)
    return out


def main(root):
    output = []
    with concurrent.futures.ProcessPoolExecutor(8) as ex:
        for batch in sorted(root.glob("*/batch.json")):
            data = json.loads(batch.read_text())
            jobs = [(root, data["tag"], data["parallel"], job) for job in data["results"]]
            results = list(ex.map(replay_one, jobs))
            output.extend(results)
            (batch.parent / "replay.json").write_text(json.dumps(results, indent=1) + "\n")
            print("replayed", data["tag"], flush=True)
    (root / "summary.json").write_text(json.dumps(output, indent=1) + "\n")


if __name__ == "__main__":
    main(Path(sys.argv[1]))
