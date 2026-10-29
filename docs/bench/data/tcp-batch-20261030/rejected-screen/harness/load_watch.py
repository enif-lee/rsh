"""Low-rate macOS load evidence for development experiments.

These are conservative screening guards, not CPU isolation or proof that
every frame window is free from contention. Other user processes are never
stopped. iostat's first (since-boot) row is discarded.
"""
import json
from pathlib import Path
import re
import subprocess
import threading
import time


def parse_iostat(text):
    rows = [line.split() for line in text.splitlines() if re.match(r"^\s*\d+\s+\d+\s+\d+\s+[\d.]+", line)]
    assert len(rows) >= 2, "iostat did not return an interval sample"
    user, system, idle, *loads = map(float, rows[-1])
    return dict(user_pct=user, system_pct=system, idle_pct=idle, load_average=loads)


def sample():
    began = time.time()
    cpu = parse_iostat(subprocess.check_output(["iostat", "-C", "-n", "0", "-c", "2", "-w", "1"], text=True, timeout=10))
    processes = []
    for line in subprocess.check_output(["ps", "-Ao", "pid=,pcpu=,comm=", "-r"], text=True, timeout=10).splitlines():
        pid, usage, name = line.strip().split(None, 2)
        processes.append(dict(pid=int(pid), cpu_pct=float(usage), program=Path(name).name))
    external = [p for p in processes if p["program"] != "OrbStack Helper"]
    return dict(at=time.time(), began=began, **cpu, processes=processes[:12],
                max_external_process_cpu_pct=max((p["cpu_pct"] for p in external), default=0))


def quiet(s):
    return s["idle_pct"] >= 70 and s["max_external_process_cpu_pct"] < 100


def wait_quiet(path, seconds=10):
    """Wait for sampled headroom; abort after ten minutes rather than run busy."""
    started, since, previous = time.time(), None, None
    with Path(path).open("a") as log:
        while time.time() - started < 600:
            began = time.time()
            s = sample()
            log.write(json.dumps(s) + "\n")
            log.flush()
            if previous is not None and not 0 <= s["at"] - previous <= 15:
                since = None
            previous = s["at"]
            if quiet(s):
                since = since if since is not None else s["at"]
                if s["at"] - since >= seconds:
                    return
            else:
                since = None
            time.sleep(max(0, 5 - (time.time() - began)))
    raise RuntimeError("Host did not reach sampled headroom; no new performance phase was started")


def assess(samples, phase_start=None, phase_end=None, monotonic_elapsed=None):
    reasons, low, external = [], 0, 0
    if len(samples) < 2:
        reasons.append("insufficient host samples")
    if samples and phase_start is not None and samples[0]["at"] - phase_start > 15:
        reasons.append("unobserved phase start >15s")
    if samples and phase_end is not None and phase_end - samples[-1]["at"] > 15:
        reasons.append("unobserved phase end >15s")
    if phase_start is not None and phase_end is not None:
        wall_elapsed = phase_end - phase_start
        if wall_elapsed < 0 or monotonic_elapsed is not None and abs(wall_elapsed-monotonic_elapsed) > 2:
            reasons.append("wall/monotonic clock discontinuity")
    previous = None
    for s in samples:
        if s.get("error"):
            reasons.append("host sampler failed")
            continue
        if previous is not None and not 0 <= s["at"] - previous <= 15:
            reasons.append("host sampling gap >15s (sleep/stall/clock change)")
        if s["at"] - s.get("began", s["at"]) > 15:
            reasons.append("host sampler interval stalled >15s")
        previous = s["at"]
        low = low + 1 if s["idle_pct"] < 30 else 0
        external = external + 1 if s["max_external_process_cpu_pct"] >= 100 else 0
        if low >= 3:
            reasons.append("CPU idle <30% in three consecutive samples")
        if external >= 2:
            reasons.append("a competing host process >=100% CPU in two consecutive samples")
    return dict(accepted=not reasons, reasons=sorted(set(reasons)), samples=len(samples),
                phase_start=phase_start, phase_end=phase_end, monotonic_elapsed=monotonic_elapsed,
                rules="5s sampling; phase includes setup/drain; guards are heuristic and do not establish CPU-core isolation")


class Watch:
    def __init__(self, path):
        self.path, self.samples, self.stop = Path(path), [], threading.Event()

    def __enter__(self):
        self.began, self.mono = time.time(), time.monotonic()
        def run():
            with self.path.open("w") as log:
                while not self.stop.is_set():
                    began = time.time()
                    try:
                        s = sample()
                    except Exception as error:
                        s = dict(at=time.time(), error=str(error))
                    self.samples.append(s)
                    log.write(json.dumps(s) + "\n")
                    log.flush()
                    self.stop.wait(max(0, 5 - (time.time() - began)))
        self.thread = threading.Thread(target=run, daemon=True)
        self.thread.start()
        return self

    def __exit__(self, *exc):
        self.ended, self.elapsed = time.time(), time.monotonic()-self.mono
        self.stop.set()
        self.thread.join(timeout=25)
        if self.thread.is_alive():
            self.samples.append(dict(at=time.time(), error="sampler did not terminate"))

    def status(self):
        return assess(self.samples, self.began, self.ended, self.elapsed)
