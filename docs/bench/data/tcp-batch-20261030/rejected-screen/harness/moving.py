"""Replay a common, time-based netem trace on an independent link.

The controller imports `timeline`; each container runs this file as a
worker. Workers share the Docker VM's clock, not Docker exec arrival times.
"""
import json
import math
import random
import signal
import subprocess
import sys
import threading
import time


def timeline(phases, seconds):
    """Quarter-second delay samples and one shared blackout per phase.

    Loss is a fraction of *time*, not independently drawn per packet. No
    protocol's packet count can advance another protocol's impairment RNG.
    """
    cycle = sum(p[0] for p in phases)
    result = []
    base = 0
    while base < seconds:
        rng = random.Random(0)
        offset = 0
        for phase, (duration, delay, jitter, loss, down, label) in enumerate(phases):
            cut = duration * 0.6
            end = cut + duration * loss / 100
            if loss == 100:
                cut, end = 0, duration
            points = {i / 4 for i in range(math.ceil(duration * 4))}
            if loss:
                points.update((cut, end))
            noise = 0.0
            sampled = None
            for at in sorted(points):
                if at >= duration or base + offset + at >= seconds:
                    continue
                # Blackout edges retain the current delay sample.
                if at * 4 == int(at * 4):
                    noise = 0.25 * noise + math.sqrt(1 - 0.25 ** 2) * rng.gauss(0, jitter)
                    sampled = max(0, round(delay + noise))
                result.append(dict(offset_s=round(base + offset + at, 6), phase=phase,
                                   delay_ms=sampled, loss_pct=100 if cut <= at < end else 0,
                                   down_kbit=down, up_kbit=down // 4))
            offset += duration
        base += cycle
    return result


def worker(device, direction):
    stopped = False

    def stop(*_):
        nonlocal stopped
        stopped = True

    signal.signal(signal.SIGTERM, stop)
    print("ready", flush=True)
    line = sys.stdin.readline()
    if not line:
        return
    job = json.loads(line)
    # Closing the controller's stdin also stops a worker inside docker
    # exec; terminating the host docker process alone does not do that.
    def disconnected():
        sys.stdin.read(1)
        stop()
    threading.Thread(target=disconnected, daemon=True).start()
    # Convert once; clock adjustments cannot shift one worker mid-replay.
    start = time.monotonic() + job["start"] - time.time()
    applied = []

    def shape(point):
        subprocess.run(["tc", "qdisc", "change", "dev", device, "root", "netem",
                        "delay", "%dms" % point["delay_ms"], "loss", "%d%%" % point["loss_pct"],
                        "rate", "%dkbit" % point[direction + "_kbit"]], check=True)

    try:
        for point in job["trace"]:
            due = start + point["offset_s"]
            while not stopped and time.monotonic() < due:
                time.sleep(max(0, min(0.02, due - time.monotonic())))
            if stopped:
                break
            shape(point)
            applied.append(time.time())
        until = start + job["seconds"]
        while not stopped and time.monotonic() < until:
            time.sleep(max(0, min(0.02, until - time.monotonic())))
    finally:
        shape(job["restore"])
    print(json.dumps(applied), flush=True)


if __name__ == "__main__":
    worker(*sys.argv[1:])
