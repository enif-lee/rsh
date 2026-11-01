#!/usr/bin/env python3
"""Bounded reason-trace prototype, built only in a disposable source tree.

python3 benchmark/lab/probe.py build --out benchmark/lab/out/reason-probe-20261030
python3 benchmark/lab/probe.py run --out benchmark/lab/out/reason-probe-20261030
python3 benchmark/lab/probe.py analyze --out benchmark/lab/out/reason-probe-20261030

Development only: iterations=1, parallel=8. Source ledgers are enabled for
both trace states. The two off blocks bound temporal variation; these are
descriptive comparisons, not independent repeated host experiments.
"""
import argparse
import concurrent.futures
import fcntl
import hashlib
import json
import os
from pathlib import Path
import shlex
import shutil
import subprocess
import threading
import time

import experiment as ex
import lab
import load_watch

ROOT = Path(__file__).resolve().parents[2]
OUT = Path(lab.HERE) / "out/reason-probe-20261030"


def ledger_scene(text):
    """Record the exact draw result; add no bytes to the terminal stream."""
    old = 'out = sys.stdout.buffer\n'
    assert text.count(old) == 1
    text = text.replace(old, old + '''
import base64, json
ledger = []
ledger_bytes = ledger_dropped = 0
LEDGER_LIMIT = 32 * 1024 * 1024
LEDGER_RECORDS = 4096
''')
    old = '        out.write(draw(n).encode())\n        out.flush()\n'
    assert text.count(old) == 1
    text = text.replace(old, '''        draw_start = time.monotonic_ns()
        payload = draw(n).encode()
        write_start = time.monotonic_ns()
        wall_ns = time.time_ns()
        out.write(payload)
        out.flush()
        write_end = time.monotonic_ns()
        if len(ledger) < LEDGER_RECORDS and ledger_bytes + len(payload) <= LEDGER_LIMIT:
            ledger.append((n, start + n / fps, draw_start, write_start, write_end, wall_ns, payload))
            ledger_bytes += len(payload)
        else:
            ledger_dropped += 1
''')
    old = 'print("frames %d" % n)'
    # A Ctrl-C also executes finally. Dump after the original terminal cleanup
    # and only after the scene has stopped producing measured output.
    assert text.count('    out.flush()\n' + old) == 1
    dump = '''    dump_begin_epoch = time.time()
    with open('/tmp/source-ledger.json.tmp', 'w') as record:
        json.dump(dict(schema=1, scene=scene, cols=cols, rows=rows,
            clock="monotonic_ns; wall_ns sampled immediately before source write",
            capacity_bytes=LEDGER_LIMIT, capacity_records=LEDGER_RECORDS,
            bytes=ledger_bytes, dropped=ledger_dropped, dump_begin_epoch=dump_begin_epoch,
            frames=[dict(n=n, intended_epoch=due, draw_begin_ns=draw,
                write_begin_ns=begin, write_end_ns=end, wall_ns=wall,
                payload_b64=base64.b64encode(payload).decode())
                for n, due, draw, begin, end, wall, payload in ledger]), record)
    os.replace('/tmp/source-ledger.json.tmp', '/tmp/source-ledger.json')
'''
    wait = '''    # Released by the host only after ALL concurrent receive windows close.
    while not os.path.exists('/tmp/source-ledger-flush'):
        time.sleep(.01)
'''
    return text.replace('    out.flush()\n' + old, '    out.flush()\n' + wait + dump + old)


def build():
    import reason_trace
    OUT.mkdir(parents=True, exist_ok=True)
    src = OUT / "source"
    assert not src.exists(), "Use a fresh output directory to preserve the prototype"
    src.mkdir()
    for name in ("Cargo.toml", "Cargo.lock", "build.rs"):
        shutil.copy2(ROOT / name, src / name)
    shutil.copytree(ROOT / "src", src / "src")
    reason_trace.instrument(src)
    diff = subprocess.run(["git", "diff", "--no-index", str(ROOT / "src"), str(src / "src")],
                          capture_output=True, text=True)
    assert diff.returncode in (0, 1), diff.stderr
    (OUT / "prototype.patch").write_text(diff.stdout.replace(str(src / "src"), "/src").replace(str(ROOT / "src"), "/src"))
    subprocess.run(["docker", "run", "--rm", "--platform", "linux/arm64",
        "-v", f"{src}:/src:ro", "-v", "rsh-cargo-aarch64:/usr/local/cargo/registry",
        "-v", "rsh-target-aarch64:/target", "-v", f"{OUT}:/out", "-w", "/src",
        "-e", "CARGO_TARGET_DIR=/target", "rust:1-alpine", "sh", "-c",
        "apk add --no-cache musl-dev >/dev/null && cargo test --locked --lib reason && cargo build --release --locked && cp /target/release/rsh /out/rsh-probe"], check=True)


def configure(traced, flags=""):
    # Capture uses the original online collector plus post-window binary dump.
    ex.OUT = OUT
    agent = ex.agent()
    text = agent.read_text()
    if flags:
        text = text.replace('FRAME = re.compile', "for _tool in ('rsh', 'rsh-tcp'):\n    COMMAND[_tool] = ['env', " + repr("RSH_OPT_FLAGS=" + flags) + "] + COMMAND[_tool]\n\nFRAME = re.compile", 1)
    # All agents rendezvous before any endpoint serializes a memory trace.
    anchor = "    # Frames still on their way (ssh, et on a slow link) are not counted;"
    assert text.count(anchor) == 1
    text = text.replace(anchor, '    ask("probe-end")\n' + anchor)
    anchor = "    c.finish()\n    if test in ('render', 'switch'):"
    assert text.count(anchor) == 1
    text = text.replace(anchor, "    if test not in ('render', 'switch'):\n        ask('probe-end')\n" + anchor)
    if traced:
        text = text.replace('FRAME = re.compile', "for _tool in ('rsh', 'rsh-tcp'):\n    COMMAND[_tool] = ['env', 'RSH_REASON_TRACE=/tmp/reason-trace'] + COMMAND[_tool]\n\nFRAME = re.compile", 1)
        old = '    c.finish()\n    if test in (\'render\', \'switch\'):'
        new = '''    if tool in ('rsh', 'rsh-tcp') and c.alive:
        os.kill(c.pid, signal.SIGUSR1)
        deadline = time.monotonic() + 2
        trace_meta = '/tmp/reason-trace/meta-%d.json' % c.pid
        while not os.path.exists(trace_meta) and time.monotonic() < deadline:
            time.sleep(.01)
        out['trace_flush_confirmed'] = os.path.exists(trace_meta)
    c.finish()
    if test in ('render', 'switch'):'''
        assert text.count(old) == 1
        text = text.replace(old, new)
    agent.write_text(text)
    scene = OUT / "scene-ledger.py"
    scene.write_text(ledger_scene((Path(lab.HERE) / "scene.py").read_text()))
    def one(k):
        for container in (lab.cli(k), lab.srv(k)):
            lab.dexec(container, "sh", "-c", "rm -rf /tmp/reason-trace; mkdir -m 777 /tmp/reason-trace; rm -f /tmp/source-ledger.json /tmp/source-ledger-flush /tmp/capture.bin")
        lab.sh("docker", "cp", str(agent), f"{lab.cli(k)}:/tmp/agent.py")
        lab.sh("docker", "cp", str(scene), f"{lab.srv(k)}:/tmp/scene.py")
        env = "env " + ("RSH_REASON_TRACE=/tmp/reason-trace " if traced else "") + "RSH_OPT_FLAGS=" + shlex.quote(flags) + " "
        command = env + "rsh serve --bind 0.0.0.0:2222 --authorized /home/dev/.rsh/authorized"
        lab.dexec(lab.srv(k), "sh", "-c", "pkill -x rsh; sleep .3; su dev -c " + shlex.quote("nohup " + command + " >/tmp/rsh.log 2>&1 &"))
    with concurrent.futures.ThreadPoolExecutor(8) as pool:
        list(pool.map(one, lab.slots()))
    time.sleep(1)


def jobs():
    matrix = [(n, s, t) for n in ("highlat",) for s in ("bandwidth", "compress")
              for t in ("rsh", "rsh-tcp", "ssh")]
    matrix += [("wan", "bandwidth", "rsh"), ("lowbw", "bandwidth", "rsh")]
    return [dict(test="render", tool=t, net=n, scene=s, iteration=0,
                 duration=8, scene_fps=60, cols=100, rows=30) for n, s, t in matrix]


def run_job(k, job, barrier, ends):
    """The lab render runner with a post-window eight-agent rendezvous."""
    began = time.time()
    lab.prepare(k, job["net"], job["test"] == "udpblock" or job["tool"] in ("rsh-tcp", "rsh-b-tcp"))
    proc = subprocess.Popen(["docker", "exec", "-i", lab.cli(k), "python3", "/tmp/agent.py", "run", json.dumps(job)],
        stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
    timer = threading.Timer(180, proc.kill)
    timer.start()
    result, other = None, []
    try:
        for line in proc.stdout:
            if line.startswith("REQ "):
                words = line.split()[1:]
                what = words[0]
                answer = ""
                if what == "rss":
                    answer = " " + str(lab.rss_kb(lab.srv(k), lab.SERVER_PROCESS[job["tool"]]))
                elif what == "cpu":
                    answer = " " + str(lab.cpu_ns(lab.srv(k), lab.SERVER_PROCESS[job["tool"]]))
                elif what == "probe-end":
                    ends[k] = time.time()
                    barrier.wait(timeout=120)
                    if job["test"] in ("render", "switch"):
                        lab.dexec(lab.srv(k), "touch", "/tmp/source-ledger-flush")
                        deadline = time.monotonic() + 5
                        while time.monotonic() < deadline:
                            if lab.dexec(lab.srv(k), "test", "-f", "/tmp/source-ledger.json", check=False).returncode == 0:
                                break
                            time.sleep(.05)
                        else:
                            raise RuntimeError(f"slot {k}: source ledger did not complete after barrier")
                elif what == "shape":
                    lab.shape(k, words[1])
                elif what == "roam":
                    lab.roam(k, job["net"])
                else:
                    raise RuntimeError(f"Unexpected probe request: {what}")
                proc.stdin.write("OK" + answer + "\n")
                proc.stdin.flush()
            elif line.startswith("{"):
                result = json.loads(line)
            else:
                other.append(line)
        proc.wait()
        if result is None:
            result = dict(tool=job["tool"], test=job["test"], error="agent failed: " + "".join(other)[-500:])
        return dict(job, **result, slot=k, job_s=round(time.time()-began, 1))
    finally:
        timer.cancel()
        # Release peers on any early failure instead of hanging the phase.
        if k not in ends:
            barrier.abort()
        if proc.poll() is None:
            proc.kill()
            proc.wait()


def batch(tag, traced, reverse=False, flags="", specs=None, parallel=8, binary=None):
    folder = OUT / tag
    folder.mkdir()
    if binary:
        lab.install_rsh(str(binary), lab.slots())
    configure(traced, flags)
    spec = list(specs) if specs is not None else jobs()
    assert parallel in (4, 8) and len(spec) == parallel, "Each barrier block must fill its public4/development8 workers"
    if reverse:
        spec.reverse()
    barrier, ends = threading.Barrier(parallel), {}
    def one(pair):
        k, job = pair
        result = run_job(k, job, barrier, ends)
        if result.get("fps") is not None:
            lab.sh("docker", "cp", f"{lab.cli(k)}:/tmp/capture.bin", str(folder / f"{k}.bin"))
            result["capture"] = f"{tag}/{k}.bin"
            copied = lab.sh("docker", "cp", f"{lab.srv(k)}:/tmp/source-ledger.json", str(folder / f"{k}-source.json"), check=False)
            result["source_ledger"] = f"{tag}/{k}-source.json" if copied.returncode == 0 else None
        if traced and job["tool"].startswith("rsh"):
            lab.dexec(lab.srv(k), "pkill", "-USR1", "-x", "rsh")
            # Post-window dump; never counted in performance/network metrics.
            deadline = time.monotonic() + 3
            while time.monotonic() < deadline:
                found = lab.dexec(lab.srv(k), "sh", "-c", "test -n \"$(find /tmp/reason-trace -name 'meta-*.json' -print -quit)\"", check=False)
                if found.returncode == 0:
                    break
                time.sleep(.05)
            for side, container in (("srv", lab.srv(k)), ("cli", lab.cli(k))):
                target = folder / f"{k}-{side}"
                target.mkdir()
                lab.sh("docker", "cp", f"{container}:/tmp/reason-trace/.", str(target))
        result.update(reason_trace=traced, variant=job.get("variant", flags or ("trace-on" if traced else "trace-off")))
        ex.save(folder / f"{k}.json", result)
        print(tag, k, job["tool"], job["net"], job.get("scene", "-"), result.get("fps", result.get("error")), flush=True)
        return result
    began = time.time()
    with concurrent.futures.ThreadPoolExecutor(parallel) as pool:
        results = list(pool.map(one, enumerate(spec)))
    ex.save(folder / "batch.json", dict(tag=tag, parallel=parallel, iterations=1,
        reason_trace=traced, candidate_flags=flags, all_windows_closed_epoch=max(ends.values()),
        window_closed_notifications=ends, elapsed_s=time.time()-began, results=results))


def run():
    assert not lab.slots(), "Existing lab slots must not be overwritten"
    assert not list(OUT.glob("*/batch.json")), "Use a fresh output directory"
    lock_path = Path(lab.HERE) / "out/.lock"
    with lock_path.open("a+") as lock:
        fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        binary = OUT / "rsh-probe"
        ex.save(OUT / "environment.json", dict(date=time.strftime("%Y-%m-%dT%H:%M:%S%z"),
            commit=lab.sh("git", "rev-parse", "HEAD").stdout.strip(),
            binary_sha256=hashlib.sha256(binary.read_bytes()).hexdigest(),
            source_hashes={p: hashlib.sha256((Path(lab.HERE) / p).read_bytes()).hexdigest()
                for p in ("probe.py", "reason_trace.py", "agent.py", "scene.py", "replay.py", "load_watch.py")},
            iterations=1, parallel=8, source_ledger="on in every block",
            schedule=["off-before", "on", "off-after-reversed-slots"],
            note="One case per cell per block; temporal controls, not independent repetitions."))
        load_watch.wait_quiet(OUT / "preflight-host.jsonl", seconds=60)
        try:
            lab.up(8, str(binary))
            for tag, traced, reverse in (("00-off", False, False), ("01-on", True, False), ("02-off", False, True)):
                load_watch.wait_quiet(OUT / (tag + "-quiet.jsonl"))
                with load_watch.Watch(OUT / (tag + "-host.jsonl")) as watch:
                    batch(tag, traced, reverse)
                status = watch.status()
                ex.save(OUT / (tag + "-host-status.json"), status)
                if not status["accepted"]:
                    raise RuntimeError(f"{tag}: rejected host phase: {status['reasons']}")
        finally:
            lab.remove_slots()


def analyze():
    # Freeze parser/oracle code so concurrent development cannot change a
    # running replay through the host bind mount.
    snapshot = OUT / ("analysis-source-" + str(time.time_ns()))
    snapshot.mkdir()
    files = ("probe_report.py", "replay.py", "agent.py", "scene.py", "ledger_oracle.py")
    for name in files:
        shutil.copy2(Path(lab.HERE) / name, snapshot / name)
    ex.save(OUT / "analysis-manifest.json", dict(source=str(snapshot.relative_to(OUT)),
        sha256={name:hashlib.sha256((snapshot/name).read_bytes()).hexdigest() for name in files}))
    subprocess.run(["docker", "run", "--rm", "--cpus", "4",
        "-v", f"{OUT}:/data", "-v", f"{snapshot}:/lab:ro",
        "rsh-bench", "python3", "/lab/probe_report.py", "/data"], check=True)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action", choices=("build", "run", "analyze"))
    parser.add_argument("--out", type=Path, default=OUT)
    args = parser.parse_args()
    OUT = args.out.resolve()
    globals()[args.action]()
