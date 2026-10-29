#!/usr/bin/env python3
"""One lab for OpenSSH, Eternal Terminal, mosh and rsh: the performance
tests, run in parallel and summed up, and the side-by-side demo for screen
recording, on the same networks and scenes.

  lab.py run [options]                 tests, in parallel over the slots
  lab.py demo [options]                four panes in tmux (ssh, et, mosh, rsh)
  lab.py up [--slots 4] [--rsh PATH]   slots kept between runs (iterating)
  lab.py rsh PATH                      put an rsh Linux build in every slot
  lab.py down                          remove the session, slots, networks

`run` and `demo` bring up the slots they need when none are there and
remove them when they end (`--keep` keeps them); slots brought up with
`up` stay until `down`.

Each slot is a server and a client container on a network of their own
(`rl-<k>-srv`, `rl-<k>-cli`, `rshlab-<k>`), shaped with `tc netem` (delay
and loss both ways, the rate from the server), so jobs in different slots
do not share a link; they share the machine's CPUs, so keep `--parallel`
below about half of them. The image is the harness's (`benchmark/harness/
run.py build`); `--rsh` swaps in another rsh build without rebuilding it.

Tests (`run --tests`), each job a fresh connection of one tool:
  render    scene.py (`--scenes`) for `--duration` s on each of `--nets`:
            frames a second, frame age (median, p95), longest gap, the
            client's bytes and packets a second in and out
  switch    a scene while the link changes (`--switches a:b`, after
            `--switch-at` s; `--duration` s after it): fps before and
            after, and `adapt_s`, seconds until a second at 90% of the
            rate the tool settles at
  echo      key to echo on the screen (`--keys`), on each of `--nets`;
            the client's packets in and out for each key
  flood     `seq 1 --lines`: seconds until the line after it shows, and
            the client's packets in and out
  sessions  `--sessions` sessions at once on the LAN, each shell having
            printed 3000 lines: the clients' and the server side's
            memory in all, and the packets a second they cost idle
  outage    the link cut for `--down` s under a ticker: longest freeze,
            seconds from the link's return until the ticker moves (watched
            for `--duration` s after it)
  roam      the client moves to another address under a ticker: the same
  reattach  the client killed and started again: same shell or not
  udpblock  the server drops UDP silently: connect time
The connect time of every job is kept too. Each test runs `--iterations`
times; the summary shows the median (and the range of the main figure).
The events run on `--event-net`. Results go to `out/run-<time>.json`.

The demo (`demo`) runs the tools' clients in tmux panes of one size
(stacked, or side by side in a terminal wider than tall) on slots 0 to n-1
(`tmux -L rshlab`): the render scenes on each network, each until every
client has shown its end (a slow link's backlog; `RENDER_LAG` at most),
then the connection scenarios (`--events`). The top header shows the test
profile; over each pane: program, frames a second, frame age, download rate. At the end the
control window sums it up in bar charts (frames a second, frame age,
client and server memory and data received per frame for each network
and scene, two by two when the window is wide; the clock's longest freeze
in each connection scenario, same shell, connect time), which the mouse
wheel scrolls, and waits for Enter, then closes.

To stop early: Ctrl-C stops a run, Ctrl-b X the demo (in its tmux), and
`lab.py stop` either from another shell. What ended so far is summed up
and saved, and the slots they made go as at the end.
"""
import argparse, concurrent.futures, fcntl, json, math, os, queue, re, shlex, shutil, signal, statistics, subprocess, sys, threading, time

from moving import timeline

HERE = os.path.dirname(os.path.abspath(__file__))
IMAGE = "rsh-bench"
# `rsh-tcp` is rsh over TCP: its slot's server drops UDP, so the client
# goes over TCP as on a network that lets only TCP through.
TOOLS = ["ssh", "et", "mosh", "rsh", "rsh-tcp", "rsh-b"]
LABEL = {"ssh": "OpenSSH", "et": "ET", "mosh": "mosh", "rsh": "rsh (UDP)", "rsh-tcp": "rsh (TCP)", "rsh-b": "rsh (B)"}
PROCESS = {"ssh": "ssh", "et": "et", "mosh": "mosh-client", "rsh": "rsh", "rsh-tcp": "rsh", "rsh-b": "rsh-b"}
# Each tool's server side, as `ps -C` names it (sshd: the listener too).
SERVER_PROCESS = {"ssh": "sshd", "et": "etserver,etterminal", "mosh": "mosh-server", "rsh": "rsh", "rsh-tcp": "rsh", "rsh-b": "rsh-b"}
CLIENT = {"ssh": "ssh -tt dev@server", "et": "et dev@server:2022", "mosh": "mosh dev@server", "rsh": "rsh server demo", "rsh-tcp": "rsh server demo",
          "rsh-b": "env RSH_PEER=server:2223 rsh-b server demo"}
# One-way delay (ms, both ways), loss (%, both ways), server-to-client rate.
NETS = {
    "lan": (0, 0, None, "LAN"),
    "wan": (50, 0, None, "RTT 100 ms"),
    "lowbw": (50, 0, "1mbit", "1 Mbit/s, RTT 100 ms"),
    "mobile": (100, 1, "512kbit", "512 kbit/s, RTT 200 ms, 1% loss"),
    # Twice as bad as `mobile` on each count: a weak cellular signal.
    "badmobile": (200, 2, "256kbit", "256 kbit/s, RTT 400 ms, 2% loss"),
    "verylow": (50, 0, "256kbit", "256 kbit/s, RTT 100 ms"),
    "slow64k": (50, 0, "64kbit", "64 kbit/s, RTT 100 ms"),
    # Above the slow-link line (250 KB/s), below what a full screen at 60
    # fps takes as it is: where compression still decides the frame rate.
    "mid5": (50, 0, "5mbit", "5 Mbit/s, RTT 100 ms"),
    "highlat": (150, 0, None, "RTT 300 ms"),
    "lossy": (50, 2, None, "2% loss each way, RTT 100 ms"),
    "moving": (35, 0.1, "8mbit", "Moving network (30 s cycle)"),
}
# A synthetic drive through congestion, weak signal and a handover, then
# recovery. Seconds, one-way delay/jitter (ms), loss (% each way), downlink
# (kbit/s), label. Uplink is a quarter of downlink. The phase order repeats;
# all slots replay the same time-based jitter and outage trace (moving.py).
MOVING = (
    (6, 35, 5, 0.1, 8000, "clear signal"),
    (4, 70, 20, 0.8, 2000, "congestion"),
    (4, 160, 60, 3, 512, "weak signal"),
    (3, 300, 120, 8, 128, "deep fade"),
    (1, 180, 60, 100, 128, "handover: link down"),
    (5, 90, 30, 1, 1000, "recovering"),
    (7, 40, 8, 0.1, 5000, "clear again"),
)
MOVING_PERIOD = sum(phase[0] for phase in MOVING)
SCENES = {
    "compress": "compressible: full-screen dashboard",
    "bandwidth": "incompressible: random full screen",
}
EVENTS = {
    "outage": "link down",
    "roam": "client moves to another IP address",
    "reattach": "client killed and started again",
    "udpblock": "network drops UDP",
}
SOCKET, SESSION = "rshlab", "demo"
# The longest a demo scene waits, past its length, for every client to show
# its end (a slow link's backlog) before the next one.
RENDER_LAG = 120


def sh(*args, check=True, stdin=None):
    return subprocess.run(args, check=check, capture_output=True, text=True, input=stdin)


def dexec(container, *args, check=True):
    return sh("docker", "exec", container, *args, check=check)


def srv(k):
    return "rl-%d-srv" % k


def cli(k):
    return "rl-%d-cli" % k


def prefix(k):
    return "10.79.%d" % (10 + k)


def slots():
    names = sh("docker", "ps", "--format", "{{.Names}}").stdout.split()
    return sorted(int(n.split("-")[1]) for n in names if n.startswith("rl-") and n.endswith("-cli") and srv(int(n.split("-")[1])) in names)


def guard():
    names = sh("docker", "ps", "-a", "--format", "{{.Names}}").stdout.split()
    busy = [n for n in ("rb-server", "rb-client") if n in names]
    if busy:
        sys.exit("lab: the benchmark harness's containers exist (%s); remove them first: docker rm -f %s" % (", ".join(busy), " ".join(busy)))
    if sh("docker", "image", "inspect", IMAGE, check=False).returncode != 0:
        sys.exit("lab: no image %s; build it with benchmark/harness/run.py build" % IMAGE)


def start_rsh(k):
    dexec(srv(k), "sh", "-c", "pkill -x rsh; sleep 0.3; su dev -c 'nohup rsh serve --bind 0.0.0.0:2222 --authorized /home/dev/.rsh/authorized >/tmp/rsh.log 2>&1 &'", check=False)


def install_rsh_b(path, ks):
    """A second rsh build as the tool `rsh-b` (its server on port 2223), to
    compare two builds in one run: the same slots, networks and moment."""
    def one(k):
        for c in (srv(k), cli(k)):
            sh("docker", "cp", path, "%s:/usr/local/bin/rsh-b" % c)
        dexec(srv(k), "sh", "-c", "pkill -x rsh-b; sleep 0.3; su dev -c 'nohup rsh-b serve --bind 0.0.0.0:2223 --authorized /home/dev/.rsh/authorized >/tmp/rsh-b.log 2>&1 &'", check=False)
    with concurrent.futures.ThreadPoolExecutor(8) as ex:
        list(ex.map(one, ks))
    time.sleep(1)


def install_rsh(path, ks):
    def one(k):
        for c in (srv(k), cli(k)):
            sh("docker", "cp", path, "%s:/usr/local/bin/rsh" % c)
        start_rsh(k)
    with concurrent.futures.ThreadPoolExecutor(8) as ex:
        list(ex.map(one, ks))
    time.sleep(1)


def up(n, rsh):
    guard()
    # The slots at once: each is a few docker calls of its own.
    with concurrent.futures.ThreadPoolExecutor(n) as ex:
        list(ex.map(up_slot, range(n)))
    time.sleep(2)
    if rsh:
        install_rsh(rsh, range(n))
    print("lab: %d slots up" % n)


def up_slot(k):
    net = "rshlab-%d" % k
    sh("docker", "rm", "-f", srv(k), cli(k), check=False)
    sh("docker", "network", "rm", net, check=False)
    sh("docker", "network", "create", "--subnet", "%s.0/24" % prefix(k), net)
    # `--init` reaps what the clients and servers leave behind.
    sh("docker", "run", "-d", "--init", "--name", srv(k), "--hostname", "server", "--network", net, "--network-alias", "server",
       "--cap-add", "NET_ADMIN", IMAGE, "/usr/local/bin/start-server.sh")
    sh("docker", "run", "-d", "--init", "--name", cli(k), "--hostname", "client", "--network", net, "--ip", "%s.10" % prefix(k),
       "--cap-add", "NET_ADMIN", IMAGE)
    sh("docker", "cp", os.path.join(HERE, "scene.py"), "%s:/tmp/scene.py" % srv(k))
    sh("docker", "cp", os.path.join(HERE, "agent.py"), "%s:/tmp/agent.py" % cli(k))


def down():
    sh("tmux", "-L", SOCKET, "kill-server", check=False)
    remove_slots()
    print("lab: down")


def stop():
    """Interrupts the run or demo that holds the lock, as Ctrl-C would: it
    sums up what ended and cleans up as at its end."""
    path = os.path.join(HERE, "out", ".lock")
    try:
        lock = open(path, "r+")
    except FileNotFoundError:
        sys.exit("lab: nothing to stop")
    try:
        fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        sys.exit("lab: nothing to stop")
    except OSError:
        pass
    try:
        lock.seek(0)
        pid = int(lock.read().strip())
        os.kill(pid, signal.SIGINT)
    except (ValueError, ProcessLookupError):
        sys.exit("lab: the run holding the slots is not known; `lab.py down` removes them")
    print("lab: stopping %d" % pid)


def remove_slots():
    names = sh("docker", "ps", "-a", "--format", "{{.Names}}").stdout.split()
    gone = [n for n in names if n.startswith("rl-")]
    if gone:
        sh("docker", "rm", "-f", *gone)
    for net in sh("docker", "network", "ls", "--format", "{{.Name}}").stdout.split():
        if net.startswith("rshlab-"):
            sh("docker", "network", "rm", net, check=False)


def iface(container, k):
    out = dexec(container, "ip", "-o", "-4", "addr", "show").stdout
    return next(line.split()[1] for line in out.splitlines() if " %s." % prefix(k) in line)


def netem(k, c, net, phase=0, change=False):
    """Shell lines that give container `c` of slot `k` the link of `net`.

    A new qdisc each time: what an earlier job left queued (a flood on a
    slow link takes minutes) goes, and `replace` would keep the options
    left out and, changing a rate under a queue, stalled mosh and rsh
    alike for seconds. A static-link switch mid-scene so drops what is
    queued, as a handover may. The moving profile instead changes the
    existing qdisc and supplies every varying option at each step."""
    delay, loss, rate, _ = NETS[net]
    if net == "moving":
        _, delay, jitter, loss, down, _ = MOVING[phase]
        rate = down if c == srv(k) else down // 4
        args = ["delay", "%dms" % delay, "loss", "100%" if loss == 100 else "0%", "rate", "%dkbit" % rate]
        device = "dev=$(ip -o -4 addr show | awk '/ %s\\./ {print $2; exit}')" % prefix(k)
        if change:
            return [device, 'tc qdisc change dev "$dev" root netem ' + " ".join(args)]
        return [device, 'tc qdisc del dev "$dev" root 2>/dev/null', 'tc qdisc add dev "$dev" root netem ' + " ".join(args)]
    args = (["delay", "%dms" % delay] if delay else []) + (["loss", "%g%%" % loss] if loss else []) + (["rate", rate] if rate and c == srv(k) else [])
    lines = ["dev=$(ip -o -4 addr show | awk '/ %s\\./ {print $2; exit}')" % prefix(k), 'tc qdisc del dev "$dev" root 2>/dev/null']
    if args:
        lines.append('tc qdisc add dev "$dev" root netem ' + " ".join(args))
    return lines


def shape(k, net, only=None, phase=0, change=False):
    for c in only or (srv(k), cli(k)):
        dexec(c, "sh", "-c", "; ".join(netem(k, c, net, phase, change)), check=net == "moving")


def prepare(k, net, udp_dropped=False):
    """A slot as a job finds it: no rule, client or ticker left from an
    earlier job, the link of `net`; one docker call per container."""
    # `[a]gent`: the pattern does not match this shell's own command line.
    client = ["iptables -F", "pkill -9 -x ssh; pkill -9 -x et; pkill -9 -x mosh-client; pkill -9 -x rsh; pkill -9 -x rsh-b; pkill -9 -x script; pkill -9 -f '[a]gent.py'"]
    server = ["iptables -F", "pkill -f '[s]cene.py'; pkill -x mosh-server; pkill -x etterminal"] + (["iptables -I INPUT -p udp -j DROP"] if udp_dropped else [])
    for c, lines in ((cli(k), client), (srv(k), server)):
        dexec(c, "sh", "-c", "; ".join(lines + netem(k, c, net)) + "; true", check=False)


def roam(k, net):
    """The client to the other of its two addresses (.10 and .20)."""
    here = iface(cli(k), k)
    addr = dexec(cli(k), "ip", "-o", "-4", "addr", "show", "dev", here).stdout.split()[3].split("/")[0]
    other = "%s.%d" % (prefix(k), 20 if addr.endswith(".10") else 10)
    sh("docker", "network", "disconnect", "rshlab-%d" % k, cli(k))
    sh("docker", "network", "connect", "--ip", other, "rshlab-%d" % k, cli(k))
    shape(k, net, only=[cli(k)])


# ---------------------------------------------------------------- run


def jobs_of(opts):
    tools = opts.tools.split(",")
    nets = [n for n in opts.nets.split(",") if n]
    base = {"duration": opts.duration, "scene_fps": opts.fps, "cols": opts.cols, "rows": opts.rows}
    scenes = [s for s in opts.scenes.split(",") if s]
    specs = []
    for test in opts.tests.split(","):
        if test == "render":
            specs += [dict(test=test, net=net, scene=scene) for net in nets for scene in scenes]
        elif test == "switch":
            for pair in opts.switches.split(","):
                a, b = pair.replace(">", ":").split(":")
                specs += [dict(test=test, net=a, to=b, scene=scene, switch_at=opts.switch_at) for scene in scenes]
        elif test in ("echo", "flood"):
            specs += [dict(test=test, net=net, keys=opts.keys, lines=opts.lines) for net in nets]
        elif test == "udpblock":
            # A tool that connects does within a few seconds; mosh never.
            specs.append(dict(test=test, net=opts.event_net, connect_timeout=15))
        elif test in ("outage", "roam", "reattach"):
            specs.append(dict(test=test, net=opts.event_net, down=opts.down))
        elif test == "sessions":
            specs.append(dict(test=test, net="lan", count=opts.sessions))
        else:
            sys.exit("lab: no test %s" % test)
    jobs = []
    for i in range(opts.iterations):
        for spec in specs:
            for tool in tools:
                jobs.append(dict(base, **spec, tool=tool, iteration=i))
    # The longest first (slow links, tests that wait out a failure), so the
    # last ones running are short and the slots finish together.
    cost = {"sessions": 5, "flood": 4, "udpblock": 3, "outage": 3, "switch": 2, "reattach": 2, "echo": 1, "roam": 1, "render": 0}
    slow = {"slow64k": 2, "verylow": 1, "mobile": 1, "badmobile": 2}
    jobs.sort(key=lambda j: (cost[j["test"]], slow.get(j["net"], 0) + slow.get(j.get("to"), 0)), reverse=True)
    return jobs


def run_job(k, job):
    began = time.time()
    # What the job leaves behind the next one's `prepare` (or the end of
    # the run) takes away.
    prepare(k, job["net"], job["test"] == "udpblock" or job["tool"] == "rsh-tcp")
    proc = subprocess.Popen(["docker", "exec", "-i", cli(k), "python3", "/tmp/agent.py", "run", json.dumps(job)],
                            stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
    other = []
    timer = threading.Timer(job["duration"] + job.get("switch_at", 0) + 150 + 10 * job.get("count", 0), proc.kill)
    timer.start()
    result = None
    try:
        for line in proc.stdout:
            if line.startswith("REQ "):
                what = line.split()[1:]
                if what[0] == "shape":
                    shape(k, what[1])
                elif what[0] == "roam":
                    roam(k, job["net"])
                answer = ""
                if what[0] == "rss":
                    # The server side's memory, which only the host sees.
                    answer = " %s" % rss_kb(srv(k), SERVER_PROCESS[job["tool"]])
                elif what[0] == "cpu":
                    answer = " %s" % cpu_ns(srv(k), SERVER_PROCESS[job["tool"]])
                proc.stdin.write("OK%s\n" % answer)
                proc.stdin.flush()
            elif line.startswith("{"):
                result = json.loads(line)
            else:
                other.append(line)
        proc.wait()
    finally:
        timer.cancel()
    if result is None:
        result = {"tool": job["tool"], "test": job["test"], "error": "agent failed: " + "".join(other)[-300:]}
    return dict(job, **result, slot=k, job_s=round(time.time() - began, 1))


def run(opts):
    if not opts.parallel:
        # A job is a few busy processes at most: one per CPU keeps them
        # from waiting on each other.
        opts.parallel = int(sh("docker", "info", "--format", "{{.NCPU}}").stdout.strip() or 4)
    # Slots made for this run go when it ends (unless `--keep`); slots
    # brought up beforehand (`up`) stay.
    made = not slots()
    if made:
        up(opts.parallel, None)
    try:
        run_on(opts)
    finally:
        if made and not opts.keep:
            remove_slots()
            print("lab: slots removed", file=sys.stderr)


def run_on(opts):
    free = slots()
    if opts.rsh:
        install_rsh(opts.rsh, free)
    if opts.rsh_b:
        install_rsh_b(opts.rsh_b, free)
    parallel = min(opts.parallel, len(free))
    jobs = jobs_of(opts)
    pool = queue.Queue()
    for k in free[:parallel]:
        pool.put(k)
    results, lock = [], threading.Lock()
    began = time.time()

    def one(job):
        k = pool.get()
        try:
            r = run_job(k, job)
        except Exception as err:  # one job's failure is that job's result
            r = dict(job, error="%s: %s" % (type(err).__name__, err), slot=k)
        finally:
            pool.put(k)
        with lock:
            results.append(r)
            try:
                print("\r  %d/%d jobs, %.0f s" % (len(results), len(jobs), time.time() - began), end="", file=sys.stderr, flush=True)
            except OSError:
                pass  # progress only; a closed stderr must not stop the run
        return r

    ex = concurrent.futures.ThreadPoolExecutor(parallel)
    try:
        list(ex.map(one, jobs))
    except KeyboardInterrupt:
        # What finished is still summed up and saved.
        print("\nlab: interrupted; %d of %d jobs done" % (len(results), len(jobs)), file=sys.stderr)
        ex.shutdown(wait=False, cancel_futures=True)
    ex.shutdown(wait=False)
    print(file=sys.stderr)
    summary(results)
    took = {}
    for r in results:
        took.setdefault(r["test"], []).append(r.get("job_s", 0))
    print("\nseconds a job: " + ", ".join("%s %.1f" % (t, statistics.mean(v)) for t, v in took.items()))
    os.makedirs(os.path.join(HERE, "out"), exist_ok=True)
    path = os.path.join(HERE, "out", "run-%s.json" % time.strftime("%Y%m%d-%H%M%S"))
    with open(path, "w") as f:
        json.dump({"options": vars(opts), "results": results}, f, indent=1)
    print("%d jobs in %.0f s on %d slots; saved %s" % (len(jobs), time.time() - began, parallel, path))


# Figures each test reports: (key, column, format).
COLUMNS = {
    "render": [("fps", "fps", "%.1f"), ("low1_fps", "1% low", "%.1f"), ("low5_fps", "5% low", "%.1f"), ("age_ms", "age ms", "%.0f"), ("max_gap_ms", "gap ms", "%.0f"), ("stalls", "stalls", "%.0f"),
               ("torn", "torn", "%.0f"), ("blank", "blank", "%.0f"), ("kbit_s", "kbit/s in", "%.0f"), ("pkts_in_s", "pkt/s in", "%.0f"), ("pkts_out_s", "pkt/s out", "%.0f"),
               ("cpu_client", "cpu% client", "%.2f"), ("cpu_server", "cpu% server", "%.2f"), ("first_s_fps", "1st s fps", "%.0f")],
    "switch": [("fps_before", "fps before", "%.1f"), ("fps_after", "fps after", "%.1f"), ("low1_after", "1% low", "%.1f"), ("low5_after", "5% low", "%.1f"), ("age_after_ms", "age after", "%.0f"), ("max_gap_ms", "gap ms", "%.0f"),
               ("stalls", "stalls", "%.0f"), ("torn", "torn", "%.0f"), ("blank", "blank", "%.0f")],
    "echo": [("echo_ms", "echo ms", "%.0f"), ("echo_max_ms", "max ms", "%.0f"), ("lost", "lost", "%.0f"), ("pkts_in_key", "pkt in/key", "%.1f"), ("pkts_out_key", "pkt out/key", "%.1f")],
    "flood": [("flood_s", "seconds", "%.2f"), ("bytes_to_terminal", "bytes", "%.0f"), ("pkts_in", "pkts in", "%.0f"), ("pkts_out", "pkts out", "%.0f"),
              ("cpu_client", "cpu% client", "%.2f"), ("cpu_server", "cpu% server", "%.2f")],
    "sessions": [("connected", "sessions", "%.0f"), ("client_kb", "client KB", "%.0f"), ("server_kb", "server KB", "%.0f"),
                 ("idle_pkts_in_s", "idle pkt/s in", "%.1f"), ("idle_pkts_out_s", "idle pkt/s out", "%.1f")],
    "outage": [("freeze_s", "freeze s", "%.1f"), ("resumed_s", "resumed s", "%.1f")],
    "roam": [("freeze_s", "freeze s", "%.1f"), ("resumed_s", "resumed s", "%.1f")],
    "reattach": [("reconnect_s", "reconnect s", "%.1f"), ("same_shell", "same shell", "%s")],
    "udpblock": [("connect_s", "connect s", "%.1f")],
}


# Lows over all the runs' frame gaps together (`gaps_ms`), not the median
# of each run's: one run of a few seconds has about a hundred gaps, so its
# 1% low is its longest gap alone, and one stall moves it far.
POOLED = {"low1_fps": ("gaps_ms", 0.99), "low5_fps": ("gaps_ms", 0.95), "low1_after": ("gaps_after_ms", 0.99), "low5_after": ("gaps_after_ms", 0.95)}


def pooled_low(runs, name):
    """`name` over the gaps of all `runs`, or None without them."""
    field, q = POOLED[name]
    gaps = sorted(g for r in runs for g in r.get(field) or [])
    if len(gaps) < 10 or any(r.get(field) is None for r in runs):
        return None
    return 1000 / max(gaps[min(len(gaps) - 1, math.ceil(len(gaps) * q) - 1)], 1e-3)


def summary(results):
    groups = {}
    for r in results:
        what = r.get("scene", "")
        where = r["net"] + (">" + r["to"] if r.get("to") else "")
        groups.setdefault((r["test"], where, what), {}).setdefault(r["tool"], []).append(r)
    for test in COLUMNS:
        keys = [g for g in groups if g[0] == test]
        if not keys:
            continue
        cols = COLUMNS[test]
        print("\n%s" % test)
        print("  %-16s %-10s %-7s %4s %s" % ("network", "scene", "tool", "runs", " ".join("%12s" % c[1] for c in cols)))
        for key in keys:
            for tool in TOOLS:
                runs = groups[key].get(tool)
                if not runs:
                    continue
                cells = []
                for i, (name, _, fmt) in enumerate(cols):
                    vals = [r.get(name) for r in runs if r.get(name) is not None]
                    if not vals:
                        cells.append("%12s" % ("err" if any("error" in r for r in runs) else "-"))
                    elif isinstance(vals[0], bool):
                        cells.append("%12s" % "/".join("y" if v else "n" for v in vals))
                    else:
                        pooled = pooled_low(runs, name) if name in POOLED else None
                        mid = fmt % (pooled if pooled is not None else statistics.median(vals))
                        # The range of the first figure, over the runs.
                        if i == 0 and len(vals) > 1:
                            mid = "%s(%s-%s)" % (mid, fmt % min(vals), fmt % max(vals))
                        cells.append("%12s" % mid)
                print("  %-16s %-10s %-7s %4d %s" % (key[1], key[2], tool, len(runs), " ".join(cells)))


# ---------------------------------------------------------------- demo


def tmux(*args, check=True):
    return sh("tmux", "-L", SOCKET, *args, check=check)


def pane(i):
    return "%s:panes.%d" % (SESSION, i)


def client_line(tool, k, log):
    env = ["-e", "TERM=xterm-256color", "-e", "LANG=C.UTF-8", "-e", "RSH_PEER=server:2222"]
    command = ["docker", "exec", "-it", *env, cli(k), "script", "-qfc", CLIENT[tool], log]
    return " ".join(shlex.quote(a) for a in command) + "; echo; echo '[client ended]'; sleep 3600"


def rss_kb(container, names):
    """Resident memory of the processes so named, KB (None: none runs)."""
    out = dexec(container, "ps", "-o", "rss=", "-C", names, check=False).stdout
    sizes = [int(x) for x in out.split() if x.isdigit()]
    return sum(sizes) if sizes else None


def cpu_ns(container, names):
    """Time the processes so named have run on a CPU, all their threads, ns
    (None: none runs)."""
    out = dexec(container, "sh", "-c", 'for p in $(ps -o pid= -C "$1"); do cat /proc/$p/task/*/schedstat; done 2>/dev/null', "_", names, check=False).stdout
    ns = [int(line.split()[0]) for line in out.splitlines() if line.split()]
    return sum(ns) if ns else None


def rx_total(k):
    """Bytes slot `k`'s client has taken in, loopback aside."""
    out = dexec(cli(k), "sh", "-c", "for d in /sys/class/net/*; do [ ${d##*/} = lo ] || cat $d/statistics/rx_bytes; done", check=False).stdout
    return sum(int(x) for x in out.split() if x.isdigit())


class Demo:
    def __init__(self, tools):
        self.tools = tools
        # Frames are timed by the containers' clock (the Docker VM's), so
        # are the windows: the host's offset from it, taken once.
        sent = time.time()
        theirs = float(dexec(cli(0), "python3", "-c", "import time; print(time.time())").stdout)
        self.offset = theirs - (sent + time.time()) / 2
        self.label = ""
        self.ticking = False
        self.attempt = 0
        self.stop = threading.Event()

    def now(self):
        return time.time() + self.offset

    def status(self, text):
        # tmux reads the status line as a strftime format.
        tmux("set", "-g", "status-left", " " + text.replace("%", "%%"))

    def everyone(self, text):
        for i in range(len(self.tools)):
            # Out of copy mode first (a viewer scrolled the pane with the
            # wheel): keys there would not reach the shell.
            tmux("copy-mode", "-q", "-t", pane(i), check=False)
            tmux("send-keys", "-t", pane(i), text, "Enter")

    def interrupt(self):
        for i in range(len(self.tools)):
            tmux("copy-mode", "-q", "-t", pane(i), check=False)
            tmux("send-keys", "-t", pane(i), "C-c", check=False)

    def shape(self, net, phase=0, change=False):
        self.each(lambda k: shape(k, net, phase=phase, change=change))

    def moving(self, seconds, scene, launch=None, warmup=0):
        """Preload every link, then replay against the Docker VM's clock."""
        trace = timeline(MOVING, seconds)
        restore = dict(delay_ms=MOVING[0][1], loss_pct=0,
                       down_kbit=MOVING[0][4], up_kbit=MOVING[0][4] // 4)
        workers, changes, rss = [], [], None
        self.network_trace = trace
        try:
            with open(os.path.join(HERE, "moving.py")) as source:
                code = source.read()
            for k in range(len(self.tools)):
                for container, direction in ((srv(k), "down"), (cli(k), "up")):
                    device = netem(k, container, "moving")[0]
                    command = device + '; exec python3 -u -c ' + shlex.quote(code) + ' "$dev" ' + direction
                    proc = subprocess.Popen(["docker", "exec", "-i", container, "sh", "-c", command],
                                            stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
                    workers.append((container, proc))
                    if proc.stdout.readline().strip() != "ready":
                        raise RuntimeError("moving worker failed: " + container + " " + proc.stderr.read())
            # Every worker is ready before the common start is chosen.
            start = self.now() + 0.25 + (warmup + DEMO_SKIP if launch else 0)
            if launch:
                launch(start - DEMO_SKIP)
            job = json.dumps(dict(start=start, seconds=seconds, trace=trace, restore=restore)) + "\n"
            for _, proc in workers:
                proc.stdin.write(job)
                proc.stdin.flush()
            began = time.monotonic() + start - self.now()
            until, sample = began + seconds, began + seconds / 2
            previous = None
            for point in trace:
                due = began + point["offset_s"]
                time.sleep(max(0, due - time.monotonic()))
                if any(proc.poll() is not None for _, proc in workers):
                    raise RuntimeError("moving worker ended before the trace finished")
                phase = point["phase"]
                _, delay, jitter, loss, down, label = MOVING[phase]
                self.label = "moving: %s · RTT ~%d ms · ↓%gM ↑%gM · %s" % (
                    label, 2 * point["delay_ms"], down / 1000, down / 4000, SCENES[scene])
                if point["loss_pct"]:
                    self.label = "moving: %s · LINK DOWN · %s" % (label, SCENES[scene])
                self.status(self.label)
                if phase != previous:
                    changes.append(dict(at=start + point["offset_s"], scheduled_s=point["offset_s"], phase=label,
                                        delay_ms=delay, jitter_ms=jitter, outage_time_pct=loss,
                                        down_kbit=down, up_kbit=down // 4))
                previous = phase
                if rss is None and time.monotonic() >= sample:
                    rss = self.each(self.rss)
            time.sleep(max(0, until - time.monotonic()))
        finally:
            errors = []
            for _, proc in workers:
                try:
                    proc.stdin.close()
                except BrokenPipeError:
                    pass
                proc.stdin = None
            for container, proc in workers:
                try:
                    output, error = proc.communicate(timeout=5)
                    if proc.returncode:
                        errors.append(container + " " + error)
                    for point, applied in zip(trace, json.loads(output or "[]")):
                        point.setdefault("applied_at", {})[container] = applied
                except (subprocess.TimeoutExpired, ValueError) as error:
                    errors.append(container + " " + str(error))
                finally:
                    if proc.poll() is None:
                        proc.kill()
                        proc.communicate()
            self.shape("moving", 0, change=True)
            changes.append(dict(at=self.now(), phase="drain: clear signal", after_measurement=True,
                                **restore))
            self.label = "moving: clear signal (draining) · %s" % SCENES[scene]
            if errors and sys.exc_info()[0] is None:
                raise RuntimeError("moving worker failed: " + "; ".join(errors))
        return rss, changes

    def watch(self, k):
        log = "/tmp/demo-%d.out" % self.attempt
        dexec(cli(k), "sh", "-c", "pkill -f '[a]gent.py watch'; rm -f /tmp/demo.status; true", check=False)
        sh("docker", "exec", "-d", cli(k), "python3", "/tmp/agent.py", "watch", log, "/tmp/demo.status", "/tmp/demo.frames")

    def restart(self):
        """New clients in every pane; when each showed a prompt (s after
        its start, or None within 30 s)."""
        self.attempt += 1
        started = {}
        for k, tool in enumerate(self.tools):
            dexec(cli(k), "sh", "-c", "pkill -9 -x %s; pkill -9 -x script; rm -f /tmp/demo-%d.out; true" % (PROCESS[tool], self.attempt), check=False)
            tmux("respawn-pane", "-k", "-t", pane(k), client_line(tool, k, "/tmp/demo-%d.out" % self.attempt))
            started[k] = time.time()
            self.watch(k)
        return self.prompts(started)

    def prompts(self, started):
        """When each pane showed a prompt (s after `started[k]`, or None
        within 30 s)."""
        shown = {}
        while len(shown) < len(self.tools) and time.time() - min(started.values()) < 30:
            for k in range(len(self.tools)):
                if k not in shown and "$" in tmux("capture-pane", "-p", "-t", pane(k)).stdout:
                    shown[k] = round(time.time() - started[k], 1)
            time.sleep(0.2)
        return [shown.get(k) for k in range(len(self.tools))]

    def layout(self):
        """Panes of one size, split along the viewer's terminal's longer
        side: stacked when it is taller than wide (a character cell is about
        twice as tall as wide), side by side when wider (and each pane 30
        columns at least). The panes window is
        cut to what a whole number of panes fills. Again when the terminal's
        size changes."""
        size = tmux("list-clients", "-t", SESSION, "-F", "#{client_width} #{client_height}", check=False).stdout.split()
        if len(size) < 2 or (size[0], size[1]) == getattr(self, "laid_out", None):
            return
        self.laid_out = (size[0], size[1])
        n = len(self.tools)
        # Less the status line.
        width, height = int(size[0]), int(size[1]) - 1
        # Side by side down to 30-column panes: narrower, the scenes'
        # frame headers (`scene.py`) are cut and nothing can be measured.
        if width >= 2 * height and (width + 1) // n - 1 >= 30:
            # A border column between panes.
            width, how = (width + 1) // n * n - 1, "even-horizontal"
            self.pane_cols = (width + 1) // n - 1
        else:
            # Each pane has its header line above it.
            height, how = height // n * n, "even-vertical"
            self.pane_cols = width
        target = "%s:panes" % SESSION
        tmux("set", "-w", "-t", target, "window-size", "manual", check=False)
        tmux("resize-window", "-t", target, "-x", str(width), "-y", str(height), check=False)
        tmux("select-layout", "-t", target, how, check=False)

    def status_loop(self):
        name = max(len(LABEL[tool]) for tool in self.tools)
        while not self.stop.is_set():
            self.layout()
            for k, tool in enumerate(self.tools):
                try:
                    s = json.loads(dexec(cli(k), "cat", "/tmp/demo.status", check=False).stdout)
                except ValueError:
                    continue
                age = "%4.0f ms" % s["age_ms"] if s.get("age_ms") is not None else "   - ms"
                text = "%-*s │ %4.1f fps │ age %s │ ↓ %6.0f kbit/s" % (name, LABEL[tool], s["fps"], age, s["kbit_s"])
                if len(text) > getattr(self, "pane_cols", 1000) - 4:
                    # Keep the same four fields in a narrow pane.
                    age = "%.0fms" % s["age_ms"] if s.get("age_ms") is not None else "-ms"
                    text = "%s %.1ffps %s ↓%.0fk" % (LABEL[tool], s["fps"], age, s["kbit_s"])
                tmux("set", "-p", "-t", pane(k), "@status", text.replace("%", "%%"), check=False)
            time.sleep(0.25)

    def shown(self, text, limit):
        """Whether each pane showed `text` within `limit` seconds. A pane
        whose client ended is not waited for (it never will)."""
        seen, gone = set(), set()
        deadline = time.time() + limit
        while len(seen) + len(gone) < len(self.tools) and time.time() < deadline:
            for k in set(range(len(self.tools))) - seen - gone:
                screen = tmux("capture-pane", "-p", "-t", pane(k), check=False).stdout
                if text in screen:
                    seen.add(k)
                elif "[client ended]" in screen:
                    gone.add(k)
            time.sleep(0.5)
        return [k in seen for k in range(len(self.tools))]

    def render(self, windows, nets, scenes, seconds, fps, warmup):
        """The render scenes, each window added to `windows` as it ends;
        each after `warmup` seconds of its label and a count down, and
        measured from its second second (`DEMO_SKIP`): the links warm up on
        the scene itself."""
        for net in nets:
            self.shape(net)
            for index, scene in enumerate(scenes):
                duration = max(seconds, MOVING_PERIOD) if net == "moving" else seconds
                if net == "moving" and index:
                    self.shape(net)
                self.label = "%s · %s" % (NETS[net][3], SCENES[scene])
                self.status("network: %s │ render: %s (%g s, %g fps)" % (NETS[net][3], SCENES[scene], duration, fps))
                # The next scene (and network) waits until every client has
                # shown this one to its end: over a slow link a client can be
                # seconds behind the program. The mark is typed as a sum, so
                # only the shell's output, not the echoed command, matches.
                mark = len(windows) + 1
                rx = self.each(rx_total)
                command = "clear; python3 /tmp/scene.py %s %g %g %g %s" % (scene, duration + DEMO_SKIP, fps, warmup, shlex.quote(self.label))
                done_command = "; echo DEMO-$((%d+1))-DONE" % (mark - 1)
                # Memory twice, halfway and at the end, the larger kept.
                changes = None
                if net == "moving":
                    rss, changes = self.moving(duration, scene,
                                               launch=lambda at: self.everyone(command + " %.6f" % at + done_command), warmup=warmup)
                    start = changes[0]["at"]
                else:
                    self.everyone(command + done_command)
                    time.sleep(warmup + DEMO_SKIP)
                    start = self.now()
                    time.sleep(duration / 2)
                    rss = self.each(self.rss)
                    time.sleep(duration / 2)
                rss = [tuple(max(x, y, key=lambda v: -1 if v is None else v) for x, y in zip(a, b)) for a, b in zip(rss, self.each(self.rss))]
                self.status("network: %s │ render: %s │ waiting for every client to show the end" % (NETS[net][3], SCENES[scene]))
                done = self.shown("DEMO-%d-DONE" % mark, RENDER_LAG)
                if not all(done):
                    self.interrupt()
                    time.sleep(1)
                rx = [b - a for a, b in zip(rx, self.each(rx_total))]
                window = {"kind": "render", "net": net, "scene": scene, "start": start, "end": start + duration, "seconds": duration,
                          "until": self.now(), "done": done, "rx_bytes": rx, "rss_kb": rss}
                if changes is not None:
                    window["network_changes"] = changes
                    window["network_trace"] = self.network_trace
                    window["start"] = changes[0]["at"]
                    window["end"] = window["start"] + duration
                windows.append(window)

    def each(self, measure):
        """`measure(k)` for every slot at once."""
        with concurrent.futures.ThreadPoolExecutor(len(self.tools)) as ex:
            return list(ex.map(measure, range(len(self.tools))))

    def rss(self, k):
        """The client's and the server's resident memory, KB."""
        tool = self.tools[k]
        return rss_kb(cli(k), PROCESS[tool]), rss_kb(srv(k), SERVER_PROCESS[tool])

    def event(self, name, net, down):
        self.label = "%s · %s" % (NETS[net][3], EVENTS[name])
        self.status("connection: %s │ network: %s │ starting clients" % (EVENTS[name], NETS[net][3]))
        self.shape(net)
        n = len(self.tools)
        w = {"kind": "event", "event": name, "net": net}
        if name == "udpblock":
            for k in range(n):
                dexec(srv(k), "iptables", "-I", "INPUT", "-p", "udp", "-j", "DROP")
            try:
                self.status("connection: the network drops UDP │ clients start again")
                w["connected_s"] = self.restart()
                time.sleep(3)
            finally:
                for k in range(n):
                    dexec(srv(k), "iptables", "-D", "INPUT", "-p", "udp", "-j", "DROP", check=False)
            return w
        self.restart()
        if name == "reattach":
            self.everyone("clear; export KEEP=$RANDOM; echo \"this shell: KEEP=$KEEP\"")
            time.sleep(3)
            keep = []
            for k in range(n):
                text = tmux("capture-pane", "-p", "-t", pane(k)).stdout
                # The output line, not the typed command (which has `$KEEP`).
                # The value alone (rsh's badge may share the top row).
                keep.append(next((l.split("KEEP=")[1].split()[0] for l in text.splitlines() if l.startswith("this shell: KEEP=")), None))
            self.status("connection: clients killed │ started again")
            for k, tool in enumerate(self.tools):
                dexec(cli(k), "pkill", "-9", "-x", PROCESS[tool], check=False)
            time.sleep(3)
            self.restart()
            self.everyone("echo \"after restart: KEEP=$KEEP\"")
            time.sleep(3)
            w["same_shell"] = [None if keep[k] is None else ("after restart: KEEP=%s" % keep[k]) in tmux("capture-pane", "-p", "-t", pane(k)).stdout for k in range(n)]
            time.sleep(5)
            return w
        self.everyone("clear; python3 /tmp/scene.py ticker 120 10")
        self.ticking = True
        w["start"] = self.now()
        time.sleep(5)
        try:
            w["back"] = [None] * n
            if name == "outage":
                self.status("connection: link DOWN (%d s) │ the program keeps running on the server" % down)
                for k in range(n):
                    dexec(cli(k), "iptables", "-I", "OUTPUT", "-d", "%s.0/24" % prefix(k), "-j", "DROP")
                    dexec(cli(k), "iptables", "-I", "INPUT", "-s", "%s.0/24" % prefix(k), "-j", "DROP")
                time.sleep(down)
                for k in range(n):
                    dexec(cli(k), "iptables", "-F", check=False)
                    w["back"][k] = self.now()
            else:
                self.status("connection: client moved to a new IP address │ who keeps going")
                for k in range(n):
                    roam(k, net)
                    w["back"][k] = self.now()
        finally:
            for k in range(n):
                dexec(cli(k), "iptables", "-F", check=False)
        self.status("connection: link back │ who comes back, and when")
        time.sleep(30)
        w["end"] = self.now()
        self.ticking = False
        self.interrupt()
        return w


def demo(opts):
    tools = opts.tools.split(",")
    # Slots made for the demo go when it closes (unless `--keep`); slots
    # brought up with `up` are used as they are.
    have = slots()
    made = not have
    if made:
        up(len(tools), None)
    elif have[: len(tools)] != list(range(len(tools))):
        sys.exit("lab: the demo needs slots 0-%d; `lab.py down`, or `lab.py up --slots %d`" % (len(tools) - 1, len(tools)))
    if opts.rsh:
        install_rsh(opts.rsh, range(len(tools)))
    # Retained slots may predate the scheduled-start argument used below.
    for k in range(len(tools)):
        sh("docker", "cp", os.path.join(HERE, "scene.py"), "%s:/tmp/scene.py" % srv(k))
    tmux("kill-server", check=False)
    for k, tool in enumerate(tools):
        dexec(cli(k), "sh", "-c", "pkill -9 -x %s; pkill -9 -x script; pkill -f '[a]gent.py'; rm -f /tmp/demo*; true" % PROCESS[tool], check=False)
        dexec(srv(k), "sh", "-c", "pkill -f '[s]cene.py'; true", check=False)
        dexec(cli(k), "iptables", "-F", check=False)
        dexec(srv(k), "iptables", "-F", check=False)
        if tool == "rsh-tcp":
            dexec(srv(k), "iptables", "-I", "INPUT", "-p", "udp", "-j", "DROP")
        # A slot kept from an earlier run may still have its last network
        # (over RTT 300 ms ET takes 5 s to connect): every client starts
        # over the LAN.
        shape(k, "lan")
        line = client_line(tool, k, "/tmp/demo-0.out")
        if k == 0:
            tmux("new-session", "-d", "-s", SESSION, "-n", "panes", "-x", "240", "-y", "66", line)
        else:
            tmux("split-window", "-t", "%s:panes" % SESSION, line)
        # All panes the same size, every tool the same terminal (a tiled
        # grid is uneven for 3 or 5); `Demo.layout` picks the direction.
        tmux("select-layout", "-t", "%s:panes" % SESSION, "even-vertical")
    for option, value in (("pane-border-status", "top"), ("pane-border-format", " #[bold]#{@status} "),
                          ("status-position", "top"), ("status-format[0]", "#{T:status-left}#[align=right]#{T:status-right}"),
                          ("status-style", "bg=colour236,fg=colour250"), ("status-left-length", "200"),
                          ("status-right", " Ctrl-b X: stop "), ("status-left", " demo: waiting for a viewer to attach"),
                          # The summary is longer than a screen: the wheel
                          # scrolls it (tmux's copy mode; q leaves it).
                          ("mouse", "on"), ("history-limit", "100000")):
        tmux("set", "-g", option, value)
    # Stops the demo from any window: Ctrl-C to the controller.
    tmux("bind-key", "X", "send-keys", "-t", "%s:control" % SESSION, "C-c")
    for k, tool in enumerate(tools):
        tmux("set", "-p", "-t", pane(k), "@status", LABEL[tool])
    argv = [sys.executable, os.path.abspath(__file__), "control"] + sys.argv[2:] + (["--teardown"] if made and not opts.keep else [])
    # When the controller ends, the session ends with it.
    tmux("new-window", "-d", "-t", SESSION, "-n", "control", " ".join(shlex.quote(a) for a in argv) + "; tmux -L %s kill-server" % SOCKET)
    print("lab: demo ready; attach with: tmux -L %s attach -t %s" % (SOCKET, SESSION))
    if not opts.no_attach and sys.stdout.isatty():
        os.execvp("tmux", ["tmux", "-L", SOCKET, "attach", "-t", "%s:panes" % SESSION])


def control(opts):
    tools = opts.tools.split(",")
    d = Demo(tools)
    windows = []
    # Ctrl-b X (or `lab.py stop`) interrupts the controller: what ended so
    # far is summed up, and the demo closes as it would at the end.
    stopped = False
    try:
        try:
            while not tmux("list-clients", "-t", SESSION, check=False).stdout.strip():
                time.sleep(0.5)
            d.layout()
            # Keys typed before a client is connected are lost (ET, mosh):
            # the scenes wait for every prompt, 30 s at most from when a
            # viewer came (the clients started before; a viewer may come
            # much later).
            since = time.time()
            d.status("demo: waiting for the clients to connect")
            windows.append({"kind": "start", "connected_s": d.prompts({k: since for k in range(len(tools))})})
            for n in (3, 2, 1):
                d.status("demo: starting in %d" % n)
                time.sleep(1)
            for k in range(len(tools)):
                d.watch(k)
            threading.Thread(target=d.status_loop, daemon=True).start()
            if opts.scenes:
                d.render(windows, [n for n in opts.nets.split(",") if n], [s for s in opts.scenes.split(",") if s], opts.duration, opts.fps, opts.warmup)
            for name in [e for e in opts.events.split(",") if e]:
                windows.append(d.event(name, opts.event_net, opts.down))
        except KeyboardInterrupt:
            stopped = True
            d.interrupt()
        d.stop.set()
        tmux("select-window", "-t", "%s:control" % SESSION, check=False)
        d.status("demo: %s │ scroll: mouse wheel or Ctrl-b [ (q to stop) │ Enter in this window closes it" % ("stopped" if stopped else "done"))
        if stopped:
            print("\nlab: demo stopped; %d scene(s) ended before" % sum(1 for w in windows if w["kind"] != "start"))
        demo_summary(tools, windows)
        # Stopped with nobody looking (`lab.py stop`), no one would press
        # Enter: it closes at once.
        if tmux("list-clients", "-t", SESSION, check=False).stdout.strip():
            input("\nEnter to close the demo ")
    except (KeyboardInterrupt, EOFError):
        pass
    finally:
        # Whatever ended it (a second Ctrl-C, a failed docker call), the
        # clients stop and the slots it made go.
        for k, tool in enumerate(tools):
            dexec(cli(k), "sh", "-c", "pkill -9 -x %s; pkill -9 -x script; pkill -f '[a]gent.py'; true" % PROCESS[tool], check=False)
            dexec(srv(k), "sh", "-c", "pkill -f '[s]cene.py'; true", check=False)
        if opts.teardown:
            # The slots the demo made (the tmux session ends right after).
            remove_slots()


# The demo's results leave out each scene's first second.
DEMO_SKIP = 1.0
COLOR = {"ssh": 245, "et": 214, "mosh": 39, "rsh": 42, "rsh-tcp": 141, "rsh-b": 208}


def bar(value, most, width):
    """A bar `value/most` of `width` cells, in eighths."""
    eighths = int(round(max(0.0, min(1.0, value / most if most else 0.0)) * width * 8))
    return ("█" * (eighths // 8) + ("", "▏", "▎", "▍", "▌", "▋", "▊", "▉")[eighths % 8]).ljust(width)


def chart(title, rows, fmt, higher, log=False):
    """One bar group per row: (label, {tool: value or None}, {tool: note})."""
    print("\n".join(chart_lines(title, rows, fmt, higher, log, shutil.get_terminal_size((120, 40)).columns)))


def chart_lines(title, rows, fmt, higher, log, columns):
    """`chart` as lines `columns` wide, the bars short of the longest note.
    A row may carry each tool's (1% low, 5% low) too: shades of its bar
    (░ to the 1% low, ▒ to the 5% low, █ to the value) and numbers."""
    import math
    longest = max((len(note) for row in rows for note in row[2].values()), default=0)
    # The names as wide as the longest of them, the bars the rest.
    name = max((len(LABEL[tool]) for row in rows for tool in row[1]), default=0)
    shaded = any(len(row) > 3 for row in rows)
    width = max(10, columns - name - 22 - longest - (16 if shaded else 0))
    head = "%s (%s is better%s)" % (title, "higher" if higher else "lower", ", log" if log else "")
    lines = ["", " \x1b[1m%s\x1b[0m%s" % (title, head[len(title):columns - 1])]
    if shaded:
        lines.append("  \x1b[2m░ 1% low  ▒ 5% low  █ average\x1b[0m")
    for row in rows:
        label, values, notes = row[:3]
        lows = row[3] if len(row) > 3 else {}
        lines.append("  \x1b[2m%s\x1b[0m" % label[:columns - 2])
        known = [v for v in values.values() if v is not None]
        best = (max if higher else min)(known) if known else None
        scale = (lambda v: math.log10(max(v, 10.0)) - 1) if log else (lambda v: v)
        most = max((scale(v) for v in known), default=0.0)
        for tool, value in values.items():
            mark = " ★" if value is not None and value == best and len(known) > 1 else ""
            text = "-" if value is None else fmt % value
            low1, low5 = lows.get(tool, (None, None))
            if value is not None and low1 is not None:
                cells = lambda v: int(round(max(0.0, min(1.0, scale(v) / most if most else 0.0)) * width))
                body = ("░" * cells(low1) + "▒" * (cells(low5) - cells(low1)) + "█" * (cells(value) - cells(low5))).ljust(width)
                text += " \x1b[2m1%% %4.1f 5%% %4.1f\x1b[22m" % (low1, low5)
            else:
                body = bar(scale(value) if value is not None else 0, most, width)
            lines.append("    \x1b[38;5;%dm%-*s %s\x1b[0m %8s%s %s" % (COLOR[tool], name, LABEL[tool], body, text, mark, notes.get(tool, "")))
    return lines


def side_by_side(left, right, columns):
    """Two blocks of lines next to each other, the left `columns` wide."""
    visible = lambda line: len(re.sub(r"\x1b\[[0-9;]*m", "", line))
    out = []
    for i in range(max(len(left), len(right))):
        a = left[i] if i < len(left) else ""
        b = right[i] if i < len(right) else ""
        out.append(a + " " * max(1, columns - visible(a)) + b)
    return out


def demo_summary(tools, windows):
    """The demo's results as charts (and saved under out/)."""
    frames = {}
    for k, tool in enumerate(tools):
        raw = dexec(cli(k), "cat", "/tmp/demo.frames", check=False).stdout
        frames[tool] = [(float(a), int(b), float(c), d) for a, b, c, d in (l.split() for l in raw.splitlines() if len(l.split()) == 4)]
    fps_rows, age_rows, rss_rows, data_rows, results = [], [], [], [], []
    for w in windows:
        if w["kind"] != "render":
            continue
        fps, age, notes, rss, rss_notes, data, lows = {}, {}, {}, {}, {}, {}, {}
        for k, tool in enumerate(tools):
            if not w.get("done", [True] * len(tools))[k]:
                notes[tool] = "(no end)"
            # Made in this window, too: a client still behind after an
            # earlier one timed out shows its frames in this one.
            # A pane narrower than the frame's header (or rsh's badge over
            # it) cuts its scene name.
            # (Made since the window's start: the scene's first second,
            # left out, may show late within it.)
            seen = [f for f in frames[tool] if w["scene"].startswith(f[3]) and w["start"] <= f[0] <= w.get("until", w["end"] + 5) and f[2] >= w["start"]]
            # What came in for each frame shown, all of the scene's.
            data[tool] = w["rx_bytes"][k] / len(seen) / 1024 if seen and "rx_bytes" in w else None
            if "rss_kb" in w:
                client, server = w["rss_kb"][k]
                if client is not None and server is not None:
                    rss[tool] = (client + server) / 1024
                    rss_notes[tool] = "c %.1f s %.1f" % (client / 1024, server / 1024)
                else:
                    rss[tool] = None
                    rss_notes[tool] = "(no %s)" % ("client" if client is None else "server")
            # A changing link must use the same observation interval for
            # every tool; moving a stalled tool's window includes recovery
            # or drain time in place of the fade. Static links retain the
            # original first-displayed-frame convention.
            first = w["start"] if w["net"] == "moving" else seen[0][0] if seen else w["start"]
            seen = [f for f in seen if f[0] <= first + w["seconds"]]
            times = [first] + [f[0] for f in seen] + [first + w["seconds"]]
            fps[tool] = len(seen) / w["seconds"]
            # As games count them: the rate the slowest 1% and 5% of frame
            # times would give.
            gaps = sorted(b[0] - a[0] for a, b in zip(seen, seen[1:]))
            at = lambda q: gaps[min(len(gaps) - 1, math.ceil(len(gaps) * q) - 1)]
            lows[tool] = (1 / max(at(0.99), 1e-3), 1 / max(at(0.95), 1e-3)) if len(gaps) >= 10 else (None, None)
            age[tool] = statistics.median((f[0] - f[2]) * 1000 for f in seen) if seen else None
            results.append(dict(kind="render", net=w["net"], scene=w["scene"], tool=tool, fps=fps[tool], low1_fps=lows[tool][0], low5_fps=lows[tool][1], age_ms=age[tool],
                                max_gap_ms=max(b - a for a, b in zip(times, times[1:])) * 1000, to_end=tool not in notes,
                                rss_mb=rss.get(tool), kb_per_frame=data[tool]))
        label = "%s · %s" % (NETS[w["net"]][3], SCENES[w["scene"]])
        fps_rows.append((label, fps, notes, lows))
        age_rows.append((label, age, notes))
        rss_rows.append((label, rss, rss_notes))
        data_rows.append((label, data, {}))
    freeze_rows, other = [], []
    for w in windows:
        if w["kind"] == "start" and None in w["connected_s"]:
            other.append("  %-34s %s" % ("demo start: connected", "   ".join(
                "\x1b[38;5;%dm%s %s\x1b[0m" % (COLOR[t], LABEL[t], "✗ not connected in 30 s" if w["connected_s"][k] is None else "✓") for k, t in enumerate(tools))))
        if w["kind"] != "event":
            continue
        if "start" in w:
            freeze, notes = {}, {}
            for k, tool in enumerate(tools):
                seen = [f for f in frames[tool] if "ticker".startswith(f[3]) and w["start"] <= f[0] <= w["end"]]
                if not seen or seen[0][0] > w["start"] + 3:
                    freeze[tool], notes[tool] = None, "(no data)"
                    continue
                times = [w["start"]] + [f[0] for f in seen] + [w["end"]]
                freeze[tool] = max(b - a for a, b in zip(times, times[1:]))
                after = [f[0] for f in seen if f[0] > w["back"][k]]
                notes[tool] = "back %.1f s after" % (after[0] - w["back"][k]) if after else "never came back"
                results.append(dict(kind="event", event=w["event"], tool=tool, freeze_s=freeze[tool], back_s=after[0] - w["back"][k] if after else None))
            freeze_rows.append((EVENTS[w["event"]], freeze, notes))
        if "same_shell" in w:
            other.append("  %-34s %s" % (EVENTS[w["event"]] + ": same shell", "   ".join(
                "\x1b[38;5;%dm%s %s\x1b[0m" % (COLOR[t], LABEL[t], {True: "✓", False: "✗", None: "?"}[w["same_shell"][k]]) for k, t in enumerate(tools))))
        if "connected_s" in w:
            other.append("  %-34s %s" % (EVENTS[w["event"]] + ": connected in", "   ".join(
                "\x1b[38;5;%dm%s %s\x1b[0m" % (COLOR[t], LABEL[t], "✗" if w["connected_s"][k] is None else "%.1f s" % w["connected_s"][k]) for k, t in enumerate(tools))))
    # Saved before the charts: stopping while they print loses nothing.
    os.makedirs(os.path.join(HERE, "out"), exist_ok=True)
    path = os.path.join(HERE, "out", "demo-%s.json" % time.strftime("%Y%m%d-%H%M%S"))
    with open(path, "w") as f:
        json.dump({"windows": windows, "results": results}, f, indent=1)
    legend = "  ".join("\x1b[38;5;%dm■ %s\x1b[0m" % (COLOR[t], LABEL[t]) for t in tools)
    print("\n\x1b[1m━━ results ━━\x1b[0m   %s" % legend)
    if fps_rows:
        # Four charts: frames a second and age above, memory and data per
        # frame below; side by side when the window is wide enough.
        columns = shutil.get_terminal_size((120, 40)).columns
        two = columns >= 120
        half = columns // 2 if two else columns
        blocks = [chart_lines("Rendering: frames a second, 1% and 5% low", fps_rows, "%.1f fps", True, False, half - 2),
                  chart_lines("Rendering: age of what is on screen", age_rows, "%.0f ms", False, True, half - 2),
                  chart_lines("Memory: RSS, c(lient) s(erver) MB", rss_rows, "%.1f MB", False, False, half - 2),
                  chart_lines("Network: received per frame shown", data_rows, "%.1f KB", False, False, half - 2)]
        if two:
            print("\n".join(side_by_side(blocks[0], blocks[1], half) + side_by_side(blocks[2], blocks[3], half)))
        else:
            print("\n".join(line for block in blocks for line in block))
    if freeze_rows:
        chart("Connection: longest freeze of a running clock", freeze_rows, "%.1f s", False)
    if other:
        print("\n \x1b[1mConnection\x1b[0m")
        print("\n".join(other))
    print("\nsaved %s" % path)


def main():
    p = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    p.add_argument("action", choices=["up", "rsh", "run", "demo", "down", "control", "stop"])
    p.add_argument("path", nargs="?", help="rsh build (action rsh)")
    p.add_argument("--slots", type=int, default=4, help="up: container pairs")
    p.add_argument("--rsh", help="an rsh Linux build to test instead of the image's")
    p.add_argument("--rsh-b", help="a second rsh Linux build, as the tool rsh-b (compare two builds in one run)")
    p.add_argument("--tools", default=",".join(t for t in TOOLS if t != "rsh-b"))
    p.add_argument("--tests", default="render,switch", help="run: " + ",".join(COLUMNS))
    p.add_argument("--nets", default="lowbw,highlat,lossy", help=",".join(NETS))
    p.add_argument("--scenes", default="compress,bandwidth", help=",".join(SCENES) + ' ("" for none in a demo)')
    p.add_argument("--switches", default="lan:lowbw,lowbw:verylow,verylow:lowbw,lowbw:highlat", help="run switch: from:to,...")
    p.add_argument("--switch-at", type=float, default=2, help="run switch: seconds before the change")
    p.add_argument("--events", default="outage,roam,reattach,udpblock", help="demo: " + ",".join(EVENTS) + ' ("" for none)')
    p.add_argument("--event-net", default="wan")
    p.add_argument("--iterations", type=int, default=2)
    p.add_argument("--duration", type=float, default=4, help="seconds of each scene (after the change, for switch); moving demo: at least 30 s")
    p.add_argument("--warmup", type=float, default=3, help="demo: seconds each scene runs unmeasured first, under its label and a count down")
    p.add_argument("--fps", type=float, default=30)
    p.add_argument("--keys", type=int, default=10, help="echo: keys typed")
    p.add_argument("--lines", type=int, default=100000, help="flood: seq length")
    p.add_argument("--sessions", type=int, default=10, help="sessions: how many at once")
    p.add_argument("--down", type=float, default=5, help="outage: seconds the link is down")
    p.add_argument("--parallel", type=int, help="run: jobs at once (default: docker's CPUs)")
    p.add_argument("--cols", type=int, default=100)
    p.add_argument("--rows", type=int, default=30)
    p.add_argument("--no-attach", action="store_true")
    p.add_argument("--keep", action="store_true", help="run, demo: keep the slots they made (for the next run)")
    p.add_argument("--teardown", action="store_true", help=argparse.SUPPRESS)
    opts = p.parse_args()
    if ("moving" in opts.nets.split(",") and opts.action not in ("demo", "control")) or opts.event_net == "moving" or any(
        "moving" in pair.replace(">", ":").split(":") for pair in opts.switches.split(",")
    ):
        p.error("moving is supported only by demo --nets (not run, --event-net or --switches)")
    for net in [n for n in opts.nets.split(",") if n] + [opts.event_net] + [x for s in opts.switches.split(",") for x in s.replace(">", ":").split(":")]:
        if net not in NETS:
            sys.exit("lab: no network %s (%s)" % (net, ", ".join(NETS)))
    for scene in [s for s in opts.scenes.split(",") if s]:
        if scene not in SCENES:
            sys.exit("lab: no scene %s (%s)" % (scene, ", ".join(SCENES)))
    if opts.action == "up":
        up(opts.slots, opts.rsh)
    elif opts.action == "rsh":
        if not opts.path:
            sys.exit("lab: rsh needs the path of an rsh Linux build")
        install_rsh(opts.path, slots())
        print("lab: rsh installed in slots %s" % slots())
    elif opts.action == "stop":
        stop()
    elif opts.action in ("run", "demo", "control"):
        # Ctrl-C and `stop` interrupt it even when it was started with
        # SIGINT ignored (in the background of a script, under nohup).
        signal.signal(signal.SIGINT, signal.default_int_handler)
        # One run or demo at a time: each cleans the slots it uses. The
        # demo's controller holds the lock; `demo` only checks it is free.
        os.makedirs(os.path.join(HERE, "out"), exist_ok=True)
        # Not "w": that would empty it, the holder's PID too, before the
        # lock is tried.
        lock = open(os.path.join(HERE, "out", ".lock"), "a+")
        # The controller waits for its `demo` to let go.
        tries = 100 if opts.action == "control" else 1
        for _ in range(tries):
            try:
                fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
                break
            except OSError:
                time.sleep(0.1)
        else:
            sys.exit("lab: another run or demo is using the slots (`lab.py stop` stops it)")
        # Who holds it, for `stop`.
        lock.truncate(0)
        lock.write(str(os.getpid()))
        lock.flush()
        if opts.action == "run":
            guard()
            run(opts)
        elif opts.action == "demo":
            # Held through the setup; let go when tmux takes over (exec
            # closes it), and the controller takes it.
            guard()
            demo(opts)
        else:
            control(opts)
    else:
        down()


if __name__ == "__main__":
    main()
