#!/usr/bin/env python3
"""Development-only ablations; never substitutes for the published benchmark.

python3 benchmark/lab/experiment.py build
python3 benchmark/lab/experiment.py run
python3 benchmark/lab/experiment.py analyze

Uses one iteration and eight workers. Experimental Rust changes are made
only in out/hypotheses/source. The installed/release binaries are untouched.
"""
import argparse
import concurrent.futures
import hashlib
import json
import os
from pathlib import Path
import queue
import shlex
import shutil
import subprocess
import sys
import time

import lab

ROOT = Path(__file__).resolve().parents[2]
OUT = Path(lab.HERE) / "out/hypotheses"
BASE = Path.home() / ".rsh/dist/rsh-linux-aarch64"
FLAGS = {"control": "", "gather0": "gather", "client0": "client",
         "tcp-credit": "credit", "tcp-backlog": "backlog",
         "tcp-both": "credit,backlog"}


def save(path, value):
    path.write_text(json.dumps(value, indent=1) + "\n")


def replace(path, old, new):
    text = path.read_text()
    assert text.count(old) == 1, (path, old, text.count(old))
    path.write_text(text.replace(old, new))


def build():
    OUT.mkdir(parents=True, exist_ok=True)
    src = OUT / "source"
    src.mkdir(exist_ok=True)
    for name in ("Cargo.toml", "Cargo.lock", "build.rs"):
        shutil.copy2(ROOT / name, src / name)
    shutil.copytree(ROOT / "src", src / "src", dirs_exist_ok=True)
    (src / "src/labexp.rs").write_text(r'''
use std::sync::{Mutex, OnceLock};
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};
pub fn enabled(bit: u8) -> bool {
    static FLAGS: OnceLock<u8> = OnceLock::new();
    *FLAGS.get_or_init(|| std::env::var("RSH_LAB_FLAGS").unwrap_or_default()
        .split(',').fold(0, |a, x| a | match x { "gather" => 1, "client" => 2,
            "credit" => 4, "backlog" => 8, _ => 0 })) & bit != 0
}
pub fn event(kind: &str, us: u128, value: usize) {
    static FILE: OnceLock<Option<Mutex<std::fs::File>>> = OnceLock::new();
    let file = FILE.get_or_init(|| std::env::var("RSH_LAB_TRACE").ok()
        .and_then(|p| std::fs::OpenOptions::new().create(true).append(true).open(p).ok())
        .map(Mutex::new));
    if let Some(file) = file {
        if let Ok(mut file) = file.lock() {
            let at = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros();
            let _ = writeln!(file, "{at},{kind},{us},{value}");
        }
    }
}
''')
    with (src / "src/lib.rs").open("a") as f:
        f.write("\npub mod labexp;\n")
    server, client, proto = (src / "src" / p for p in ("server.rs", "client.rs", "proto.rs"))
    replace(server, "        let quiet = self.output_at.map_or(since, |at| at + gap);",
            "        let gap = if crate::labexp::enabled(1) && self.animating() && self.output.slow_rate().is_none() { Duration::ZERO } else { gap };\n"
            "        let quiet = self.output_at.map_or(since, |at| at + gap);")
    replace(server, "        let packed = z.push(&raw);",
            "        if let Some(since) = self.gathering { crate::labexp::event(\"gather\", since.elapsed().as_micros(), raw.len()); }\n"
            "        let cpu_start = Instant::now();\n        let packed = z.push(&raw);\n"
            "        crate::labexp::event(\"compress\", cpu_start.elapsed().as_micros(), raw.len());")
    replace(server, "    fn drain_guess(&self, now: Instant) -> Option<f64> {",
            "    fn drain_guess(&self, now: Instant) -> Option<f64> {\n"
            "        if is_tcp_addr(self.addr) && crate::labexp::enabled(8) { return None; }")
    replace(server, "    fn drain_backlog(&self, pending: usize) -> usize {",
            "    fn drain_backlog(&self, pending: usize) -> usize {\n"
            "        if is_tcp_addr(self.addr) && crate::labexp::enabled(8) { return pending.saturating_sub(self.output.in_flight() as usize); }")
    replace(client, "        if !self.pending.is_empty() && !self.catching_up(now) && (self.tokens >= 1.0 || self.pending.len() >= PACED_MAX) {",
            "        if !self.pending.is_empty() && (crate::labexp::enabled(2) || (!self.catching_up(now) && (self.tokens >= 1.0 || self.pending.len() >= PACED_MAX))) {")
    replace(client, "        if self.terminal && self.ground {",
            "        if let Some(since) = self.since { crate::labexp::event(\"client-wait\", since.elapsed().as_micros(), self.pending.len()); }\n"
            "        if self.terminal && self.ground {")
    replace(proto, "        let mut window = self.window.saturating_sub(self.carried).max(CHUNK as u64);",
            "        let bypass = self.lossless && crate::labexp::enabled(4);\n"
            "        let mut window = if bypass { 1 << 20 } else { self.window.saturating_sub(self.carried).max(CHUNK as u64) };")
    replace(proto, "        if let Some(bdp) = self.slow_bdp() {\n            // A slow link:",
            "        if let Some(bdp) = self.slow_bdp().filter(|_| !bypass) {\n            // A slow link:")
    replace(proto, "            if self.flight_wire() >= delivered.max(self.least_flight()).max(open) {",
            "            if !bypass && self.flight_wire() >= delivered.max(self.least_flight()).max(open) {\n"
            "                crate::labexp::event(\"delivery-limit\", 0, self.in_flight() as usize);")
    replace(proto, "        if self.sent - self.base >= window {",
            "        if self.sent - self.base >= window {\n"
            "            crate::labexp::event(\"window-limit\", 0, self.in_flight() as usize);")
    # The exact patch is retained alongside measurements for reproduction.
    patch = subprocess.run(["git", "diff", "--no-index", str(ROOT / "src"), str(src / "src")],
                           capture_output=True, text=True)
    (OUT / "experimental.patch").write_text(patch.stdout.replace(str(src / "src"), "/src").replace(str(ROOT / "src"), "/src"))
    subprocess.run(["docker", "run", "--rm", "--platform", "linux/arm64",
                    "-v", f"{src}:/src:ro", "-v", "rsh-cargo-aarch64:/usr/local/cargo/registry",
                    "-v", "rsh-target-aarch64:/target", "-v", f"{OUT}:/out", "-w", "/src",
                    "-e", "CARGO_TARGET_DIR=/target", "rust:1-alpine", "sh", "-c",
                    "apk add --no-cache musl-dev >/dev/null && cargo build --release --locked && cp /target/release/rsh /out/rsh-exp"], check=True)


def agent():
    """Keep the original online collector and capture its input for replay."""
    path = OUT / "agent.py"
    shutil.copy2(Path(lab.HERE) / "agent.py", path)
    replace(path, "        self.frames = []  # (shown at, n, made at, scene)",
            "        self.capture = []\n        self.frames = []  # (shown at, n, made at, scene)")
    replace(path, "            now = time.time()\n            self.bytes += len(data)",
            "            now = time.time()\n            self.capture.append((now, data))\n            self.bytes += len(data)")
    replace(path, "    c.finish()\n    return out\n\n\ndef main_watch",
            "    c.finish()\n"
            "    if test in ('render', 'switch'):\n"
            "        out.update(window_start=start, window_end=stop)\n"
            "        if test == 'switch':\n"
            "            out['switch_moved'] = marks['shape ' + job['to']]\n"
            "        with open('/tmp/capture.bin', 'wb') as capture:\n"
            "            for at, data in c.capture:\n"
            "                capture.write(struct.pack('!dI', at, len(data)))\n"
            "                capture.write(data)\n"
            "    return out\n\n\ndef main_watch")
    return path


def configure(variant, trace):
    flags = FLAGS.get(variant, "")
    ks = lab.slots()
    binary = BASE if variant == "release" else OUT / "rsh-exp"
    lab.install_rsh(str(binary), ks)
    path = agent()
    for k in ks:
        lab.sh("docker", "cp", str(path), f"{lab.cli(k)}:/tmp/agent.py")
    def start(k):
        trace_env = "RSH_LAB_TRACE=/tmp/trace.csv" if trace else ""
        command = f"env RSH_LAB_FLAGS={shlex.quote(flags)} {trace_env} rsh serve --bind 0.0.0.0:2222 --authorized /home/dev/.rsh/authorized"
        lab.dexec(lab.srv(k), "sh", "-c", "pkill -x rsh; rm -f /tmp/trace.csv; sleep 0.3; su dev -c " + shlex.quote("nohup " + command + " >/tmp/rsh.log 2>&1 &"))
    with concurrent.futures.ThreadPoolExecutor(8) as ex:
        list(ex.map(start, ks))
    time.sleep(1)
    for tool in ("rsh", "rsh-tcp"):
        lab.CLIENT[tool] = f"env RSH_LAB_FLAGS={shlex.quote(flags)} " + ("RSH_LAB_TRACE=/tmp/trace.csv " if trace else "") + "rsh server demo"
    # agent.py executes COMMAND, not lab.CLIENT.
    text = path.read_text()
    inject = "\nfor _tool in ('rsh', 'rsh-tcp'):\n    COMMAND[_tool] = ['env', 'RSH_LAB_FLAGS=" + flags + "'] + " + ("['RSH_LAB_TRACE=/tmp/trace.csv'] + " if trace else "") + "COMMAND[_tool]\n"
    text = text.replace('FRAME = re.compile', inject + '\nFRAME = re.compile', 1)
    path.write_text(text)
    for k in ks:
        lab.sh("docker", "cp", str(path), f"{lab.cli(k)}:/tmp/agent.py")


def batch(tag, variant, tools, nets, scenes, parallel=8, trace=False, duration=8, jobs=None):
    print(tag, flush=True)
    folder = OUT / tag
    folder.mkdir()
    configure(variant, trace)
    pool = queue.Queue()
    for k in lab.slots()[:parallel]:
        pool.put(k)
    if jobs is None:
        jobs = [dict(test="render", tool=t, net=n, scene=s, iteration=0,
                     duration=duration, scene_fps=60, cols=100, rows=30)
                for n in nets for s in scenes for t in tools]
    def one(pair):
        i, job = pair
        k = pool.get()
        try:
            # The server caches an open trace fd for its process lifetime;
            # never unlink that file between jobs. Select by window instead.
            lab.dexec(lab.cli(k), "rm", "-f", "/tmp/trace.csv")
            result = lab.run_job(k, job)
            if result.get("fps") is not None:
                lab.sh("docker", "cp", f"{lab.cli(k)}:/tmp/capture.bin", str(folder / f"{i}.bin"))
                result["capture"] = f"{tag}/{i}.bin"
            if trace:
                for side, container in (("srv", lab.srv(k)), ("cli", lab.cli(k))):
                    lines = lab.dexec(container, "cat", "/tmp/trace.csv", check=False).stdout.splitlines()
                    lo, hi = result.get("window_start", 0), result.get("window_end", 0)
                    result["trace_" + side] = "\n".join(line for line in lines if lo <= int(line.split(",")[0]) / 1e6 <= hi)
            result["variant"] = variant
            save(folder / f"{i}.json", result)
            print(f"  {i+1}/{len(jobs)} {job['tool']} {job['net']} {job['scene']} {result.get('fps', result.get('error'))}", flush=True)
            return result
        finally:
            pool.put(k)
    began = time.time()
    with concurrent.futures.ThreadPoolExecutor(parallel) as ex:
        results = list(ex.map(one, enumerate(jobs)))
    save(folder / "batch.json", dict(tag=tag, variant=variant, parallel=parallel,
                                    iterations=1, trace=trace, elapsed_s=time.time()-began, results=results))


def run():
    assert not lab.slots(), "Existing lab slots must not be overwritten"
    assert not list(OUT.glob("*/batch.json")), "Use a new --out directory for another experiment"
    OUT.mkdir(parents=True, exist_ok=True)
    save(OUT / "environment.json", dict(date=time.strftime("%Y-%m-%dT%H:%M:%S%z"),
         commit=lab.sh("git", "rev-parse", "HEAD").stdout.strip(),
         host=lab.sh("sysctl", "hw.perflevel0.name", "hw.perflevel0.logicalcpu", "hw.perflevel1.name", "hw.perflevel1.logicalcpu").stdout,
         docker=json.loads(lab.sh("docker", "info", "--format", "{{json .}}").stdout),
         binaries={p.name: hashlib.sha256(p.read_bytes()).hexdigest() for p in (BASE, OUT / "rsh-exp")}))
    try:
        lab.up(8, str(BASE))
        nets = ["lan", "wan", "highlat", "lowbw", "mobile"]
        scenes = ["compress", "bandwidth"]
        tools = ["ssh", "rsh", "rsh-tcp"]
        batch("00-release", "release", tools, nets, scenes)
        batch("01-control", "control", tools, nets, scenes)
        batch("02-gather0", "gather0", tools, nets, scenes)
        batch("03-client0", "client0", tools, nets, scenes)
        for i, variant in enumerate(("tcp-credit", "tcp-backlog", "tcp-both"), 4):
            batch(f"{i:02}-{variant}", variant, tools, nets, scenes)
        batch("07-control-trace", "control", tools, ["wan", "highlat"], scenes, trace=True)
        batch("08-release-post", "release", tools, nets, scenes)
    finally:
        lab.remove_slots()


def analyze():
    # Offline parsing runs only after all observation windows have finished.
    name = "rsh-hypothesis-replay"
    subprocess.run(["docker", "run", "--rm", "--name", name,
                    "-v", f"{OUT}:/data", "-v", f"{Path(lab.HERE)}:/lab:ro",
                    "rsh-bench", "python3", "/lab/replay.py", "/data"], check=True)


def host_trace():
    """Separate, instrumented eight-worker probe of the VM's host threads."""
    assert not lab.slots(), "Existing lab slots must not be overwritten"
    processes = lab.sh("ps", "-Ao", "pid=,comm=").stdout.splitlines()
    pid = next(line.split()[0] for line in processes if line.rstrip().endswith(
        "/OrbStack Helper.app/Contents/MacOS/OrbStack Helper"))
    command = ["xcrun", "xctrace", "record", "--template", "System Trace",
               "--attach", pid, "--time-limit", "45s", "--no-prompt",
               "--output", str(OUT / "host.trace")]
    proc = None
    try:
        lab.up(8, str(BASE))
        with (OUT / "host-trace.log").open("w") as log:
            proc = subprocess.Popen(command, stdout=log, stderr=subprocess.STDOUT)
            batch("09-host-trace", "release", ["ssh", "rsh", "rsh-tcp"],
                  ["wan", "highlat"], ["compress", "bandwidth"])
            try:
                status = proc.wait(timeout=60)
                save(OUT / "host-trace-status.json", dict(command=command, returncode=status))
            except subprocess.TimeoutExpired:
                save(OUT / "host-trace-status.json", dict(command=command,
                     error="Recorder did not finish postprocessing within 60s after the batch; trace is inconclusive"))
    finally:
        if proc and proc.poll() is None:
            proc.terminate()
            proc.wait(timeout=10)
        lab.remove_slots()


if __name__ == "__main__":
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("action", choices=["build", "run", "analyze", "host-trace"])
    p.add_argument("--baseline", type=Path, default=BASE, help="verified Linux arm64 release binary")
    p.add_argument("--out", type=Path, default=OUT, help="fresh output directory (same for build/run/analyze)")
    args = p.parse_args()
    BASE = args.baseline.resolve()
    OUT = args.out.resolve()
    {"host-trace": host_trace}.get(args.action, globals().get(args.action))()
