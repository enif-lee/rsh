#!/usr/bin/env python3
"""Samples CPU time (ticks) and RSS of processes whose name matches, every
0.2 s, as JSON lines: sample.py <out> <regex>."""
import json, os, re, sys, time
out, pattern = sys.argv[1], re.compile(sys.argv[2])
tick = os.sysconf("SC_CLK_TCK")
page = os.sysconf("SC_PAGE_SIZE") // 1024
with open(out, "w") as f:
    while True:
        now = time.time()
        procs = []
        for pid in os.listdir("/proc"):
            if not pid.isdigit():
                continue
            try:
                stat = open("/proc/%s/stat" % pid).read()
                name = stat[stat.index("(") + 1 : stat.rindex(")")]
                cmd = open("/proc/%s/cmdline" % pid, "rb").read().replace(b"\0", b" ").decode(errors="replace").strip()
                if not pattern.search(name) and not pattern.search(cmd.split(" ")[0] if cmd else ""):
                    continue
                fields = stat[stat.rindex(")") + 2 :].split()
                cpu = int(fields[11]) + int(fields[12])
                rss = int(open("/proc/%s/statm" % pid).read().split()[1]) * page
                procs.append({"pid": int(pid), "name": name, "cmd": cmd[:80], "cpu": cpu / tick, "rss": rss})
            except (OSError, ValueError, IndexError):
                continue
        f.write(json.dumps({"t": now, "p": procs}) + "\n")
        f.flush()
        time.sleep(0.2)
