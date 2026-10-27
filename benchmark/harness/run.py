#!/usr/bin/env python3
"""Runs the comparison on this machine's docker: a server and a client
container on one bridge network, `tc netem` on both ends for each network
condition, one client driver run per tool, and /proc samplers on both ends.
Results go to ../results/<name>.json.

  run.py build            build the image (rsh from ~/.rsh/dist)
  run.py net              the network-condition runs
  run.py events           outage, roaming, reattach, TCP-only network
"""
import json, os, shutil, subprocess, sys, time

HERE = os.path.dirname(os.path.abspath(__file__))
RESULTS = os.path.join(HERE, "..", "results")
TOOLS = ["ssh", "mosh", "et", "rsh"]
SERVER_NAMES = r"^(sshd|mosh-server|etserver|etterminal|rsh)$"
CLIENT_NAMES = r"^(ssh|mosh-client|et|rsh)$"
# name: (one-way delay ms each direction, loss % each direction, server->client rate)
NETS = {
    "lan": (0, 0, None),
    "wan100": (50, 0, None),
    "far300": (150, 0, None),
    "lossy": (50, 2, None),
    "slow1m": (50, 0, "1mbit"),
    # Slower links, run on request (`run.py net slow256k,slow64k`).
    "slow256k": (50, 0, "256kbit"),
    "slow64k": (50, 0, "64kbit"),
}


def sh(*args, check=True, capture=True):
    return subprocess.run(args, check=check, capture_output=capture, text=True)


def dexec(container, *args, check=True):
    return sh("docker", "exec", container, *args, check=check)


def build():
    arch = "aarch64" if os.uname().machine in ("arm64", "aarch64") else "x86_64"
    shutil.copy(os.path.expanduser("~/.rsh/dist/rsh-linux-%s" % arch), os.path.join(HERE, "rsh"))
    try:
        subprocess.run(["docker", "build", "-q", "-t", "rsh-bench", HERE], check=True)
    finally:
        os.remove(os.path.join(HERE, "rsh"))


def up():
    sh("docker", "rm", "-f", "rb-server", "rb-client", check=False)
    sh("docker", "network", "rm", "rshbench", check=False)
    sh("docker", "network", "create", "--subnet", "10.77.0.0/24", "rshbench")
    sh("docker", "run", "-d", "--name", "rb-server", "--hostname", "server", "--network", "rshbench",
       "--network-alias", "server", "--cap-add", "NET_ADMIN", "rsh-bench", "/usr/local/bin/start-server.sh")
    sh("docker", "run", "-d", "--name", "rb-client", "--hostname", "client", "--network", "rshbench",
       "--ip", "10.77.0.10", "--cap-add", "NET_ADMIN", "rsh-bench")
    time.sleep(2)


def iface(container):
    """The container's interface on the bench network (eth0, or another
    name after a move)."""
    out = dexec(container, "ip", "-o", "-4", "addr", "show").stdout
    return next(line.split()[1] for line in out.splitlines() if "10.77.0." in line)


def shape(net, only=None):
    delay, loss, rate = NETS[net]
    for c in only or ("rb-server", "rb-client"):
        dev = iface(c)
        dexec(c, "tc", "qdisc", "del", "dev", dev, "root", check=False)
        args = []
        if delay:
            args += ["delay", "%dms" % delay]
        if loss:
            args += ["loss", "%g%%" % loss]
        if rate and c == "rb-server":
            args += ["rate", rate]
        if args:
            dexec(c, "tc", "qdisc", "add", "dev", dev, "root", "netem", *args)


def sampled(label, argv):
    """Runs the driver with samplers on both ends; returns its JSON plus the samples."""
    for c, names in (("rb-server", SERVER_NAMES), ("rb-client", CLIENT_NAMES)):
        dexec(c, "sh", "-c", "pkill -f sample.py; true", check=False)
        sh("docker", "exec", "-d", c, "python3", "/usr/local/bin/sample.py", "/tmp/samples.jsonl", names)
    time.sleep(1)
    started = time.time()
    proc = subprocess.Popen(["docker", "exec", "rb-client", "python3", "/usr/local/bin/drive.py", *argv],
                            stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    if argv[0] == "roam":
        # Move the client to another address once the driver asks.
        while proc.poll() is None:
            if dexec("rb-client", "test", "-e", "/tmp/roam-request", check=False).returncode == 0:
                sh("docker", "network", "disconnect", "rshbench", "rb-client")
                sh("docker", "network", "connect", "--ip", "10.77.0.20", "rshbench", "rb-client")
                shape(NET_OF_EVENTS, only=["rb-client"])
                dexec("rb-client", "touch", "/tmp/roam-done")
                break
            time.sleep(0.2)
    stdout, stderr = proc.communicate()
    proc = subprocess.CompletedProcess(argv, proc.returncode, stdout, stderr)
    time.sleep(1)
    samples = {}
    for c in ("rb-server", "rb-client"):
        dexec(c, "sh", "-c", "pkill -f sample.py; true", check=False)
        text = dexec(c, "cat", "/tmp/samples.jsonl", check=False).stdout
        samples[c] = [json.loads(line) for line in text.splitlines() if line.strip()]
    try:
        result = json.loads(proc.stdout.strip().splitlines()[-1])
    except (IndexError, json.JSONDecodeError):
        result = {"error": "driver failed", "stdout": proc.stdout[-2000:], "stderr": proc.stderr[-2000:]}
    result["label"] = label
    result["wall_s"] = round(time.time() - started, 1)
    result["samples"] = samples
    return result


def save(name, data):
    os.makedirs(RESULTS, exist_ok=True)
    with open(os.path.join(RESULTS, name + ".json"), "w") as f:
        json.dump(data, f)


def net_runs(nets, tools, suffix=""):
    for net in nets:
        up()
        shape(net)
        for tool in tools:
            tests = os.environ.get("TESTS", "echo,flood,render,idle").split(",")
            print("==", net, tool, flush=True)
            r = sampled("%s/%s" % (net, tool), ["run", tool] + tests)
            r["net"] = net
            save("net-%s-%s%s" % (net, tool, suffix), r)
            print("   connect", r.get("connect_s"), "flood", (r.get("flood") or {}).get("seconds"),
                  "render", (r.get("render") or {}).get("shown"), r.get("error", ""), flush=True)
        if net == "wan100":
            for tool in [t + "-np" for t in tools if t in ("mosh", "rsh")]:
                print("==", net, tool, flush=True)
                r = sampled("%s/%s" % (net, tool), ["run", tool, "echo"])
                r["net"] = net
                save("net-%s-%s%s" % (net, tool, suffix), r)


NET_OF_EVENTS = "wan100"


def event_runs(tools, modes=("outage", "roam", "reattach", "udp")):
    for mode in modes:
        for tool in tools:
            print("==", mode, tool, flush=True)
            # A fresh pair each time: a run may leave the network changed.
            up()
            shape(NET_OF_EVENTS)
            if mode == "udpsilent":
                # UDP dropped on the way, with no error to the sender (a
                # firewall that drops rather than refuses).
                dexec("rb-server", "iptables", "-I", "INPUT", "-p", "udp", "-j", "DROP")
                r = sampled("%s/%s" % (mode, tool), ["connect", tool])
            else:
                r = sampled("%s/%s" % (mode, tool), [mode if mode != "udp" else "udp", tool] + (["echo"] if mode == "udp" else []))
            save("event-%s-%s" % (mode, tool), r)
            print("  ", {k: v for k, v in r.items() if k in (mode, "connect_s", "error", "reattach")}, flush=True)


def connect_runs(tools, nets, reps=3):
    """Connect time alone, with the prompt-based measure, per network."""
    for net in nets:
        up()
        shape(net)
        for tool in tools:
            times = []
            for _ in range(reps):
                r = sampled("connect/%s/%s" % (net, tool), ["connect", tool])
                times.append(r.get("connect_s"))
            save("connect-%s-%s" % (net, tool), {"net": net, "tool": tool, "connect_s": times})
            print("==", net, tool, times, flush=True)


def osc_runs(tools):
    """Which OSC sequences each client passes on to its terminal."""
    up()
    shape("lan")
    for tool in tools:
        r = sampled("osc/%s" % tool, ["osc", tool])
        save("osc-%s" % tool, {k: r.get(k) for k in ("tool", "osc", "osc_exact", "error")})
        print("==", tool, r.get("osc"), flush=True)


def load_runs(tools):
    """Heavier runs on the plain network: a 21 MB flood, and ten sessions."""
    up()
    shape("lan")
    for tool in tools:
        for mode, argv in (("bigflood", ["run", tool, "bigflood"]), ("many", ["many", tool])):
            print("==", mode, tool, flush=True)
            r = sampled("%s/%s" % (mode, tool), argv)
            save("load-%s-%s" % (mode, tool), r)
            print("  ", {k: v for k, v in r.items() if k in ("bigflood", "many", "error")}, flush=True)


if __name__ == "__main__":
    what = sys.argv[1]
    tools = sys.argv[3].split(",") if len(sys.argv) > 3 else TOOLS
    if what == "build":
        build()
    elif what == "net":
        nets = sys.argv[2].split(",") if len(sys.argv) > 2 and sys.argv[2] != "all" else [n for n in NETS if n not in ("slow256k", "slow64k")]
        net_runs(nets, tools, os.environ.get("SUFFIX", ""))
    elif what == "load":
        load_runs(tools)
    elif what == "connect":
        nets = sys.argv[2].split(",") if len(sys.argv) > 2 and sys.argv[2] not in ("all", "x") else list(NETS)
        connect_runs(tools, nets)
    elif what == "osc":
        osc_runs(tools)
    elif what == "events":
        modes = sys.argv[2].split(",") if len(sys.argv) > 2 and sys.argv[2] != "all" else ("outage", "roam", "reattach", "udp")
        event_runs(tools, modes)
