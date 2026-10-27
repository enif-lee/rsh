#!/usr/bin/env python3
"""Turns ../results/*.json into the markdown tables of ../benchmark.md."""
import glob, json, os, statistics

HERE = os.path.dirname(os.path.abspath(__file__))
RESULTS = os.path.join(HERE, "..", "results")
TOOLS = ["ssh", "mosh", "et", "rsh"]
NETS = ["lan", "wan100", "far300", "lossy", "slow1m"]
SLOW = ["slow256k", "slow64k"]


def load(name):
    path = os.path.join(RESULTS, name + ".json")
    return json.load(open(path)) if os.path.exists(path) else None


def pct(values, p):
    values = sorted(v for v in values if v is not None)
    if not values:
        return None
    k = (len(values) - 1) * p
    lo, hi = int(k), min(int(k) + 1, len(values) - 1)
    return values[lo] + (values[hi] - values[lo]) * (k - lo)


def fmt(v, digits=0, unit=""):
    if v is None:
        return "-"
    return ("%." + str(digits) + "f%s") % (v, unit)


# Processes that belong to a tool's session, on each end.
def server_owned(tool, proc):
    name, cmd = proc["name"], proc["cmd"]
    return {
        "ssh": name == "sshd" and "sshd: dev" in cmd,
        "mosh": name == "mosh-server",
        "et": name in ("etserver", "etterminal"),
        "rsh": name == "rsh",
        "rsh-tcp": name == "rsh",
    }[tool]


def client_owned(tool, proc):
    return proc["name"] == {"ssh": "ssh", "mosh": "mosh-client", "et": "et", "rsh": "rsh", "rsh-tcp": "rsh"}[tool]


def usage(samples, owned, start, end):
    """CPU seconds spent in [start, end] and the largest RSS (KiB) seen."""
    def at(t):
        cpu = {}
        for s in samples:
            if s["t"] > t:
                break
            cpu = {p["pid"]: p["cpu"] for p in s["p"] if owned(p)}
        return cpu
    first, last = at(start), at(end)
    spent = sum(c - first.get(pid, 0.0) for pid, c in last.items())
    rss = max((sum(p["rss"] for p in s["p"] if owned(p)) for s in samples if start <= s["t"] <= end), default=0)
    return spent, rss


def phase(result, name):
    for p in result.get("phases", []):
        if p["name"] == name:
            return p["start"], p["end"]
    return None


def runs(net, tool):
    """The first run and the repetition (`-r2`), when there is one."""
    return [r for r in (load("net-%s-%s" % (net, tool)), load("net-%s-%s-r2" % (net, tool))) if r]


def two(net, tool, get, show):
    """`first · second` of one metric over the runs."""
    values = [get(r) for r in runs(net, tool)]
    return " · ".join(show(v) for v in values) if values else "-"


def echo_runs(net, tool):
    """Echo-only runs (`TESTS=echo`, suffixes -e1 -e2: mosh and rsh taken
    in turn at the same time), else the echo of the full runs."""
    found = [r for r in (load("net-%s-%s-e%d" % (net, tool, i)) for i in (1, 2)) if r]
    return found or runs(net, tool)


def latency_table(tools=TOOLS, nets=NETS):
    rows = ["| 망 | " + " | ".join(tools) + " |", "| --- |" + " --- |" * len(tools)]
    for net in nets:
        cells = []
        for tool in tools:
            def show(echo):
                if not echo:
                    return "-"
                lost = sum(1 for v in echo if v is None)
                return "%s / %s%s" % (fmt(pct(echo, 0.5)), fmt(pct(echo, 1.0)), " (%d 미표시)" % lost if lost else "")
            values = [r.get("echo") or [] for r in echo_runs(net, tool)]
            cells.append(" · ".join(show(v) for v in values) if values else "-")
        rows.append("| %s | %s |" % (net, " | ".join(cells)))
    return "\n".join(rows)


def prediction_table():
    rows = ["| 도구 | 예측 켬 (기본) | 예측 끔 | 예측 켬에서 왕복을 기다린 키 |", "| --- | --- | --- | --- |"]
    for tool in ("mosh", "rsh"):
        on = load("net-wan100-%s-e1" % tool) or load("net-wan100-%s" % tool)
        off = load("net-wan100-%s-np-e1" % tool) or load("net-wan100-%s-np" % tool)
        cell = lambda r: "%s / %s" % (fmt(pct(r["echo"], 0.5)), fmt(pct(r["echo"], 1.0))) if r and r.get("echo") else "-"
        slow = sum(1 for v in (on or {}).get("echo", []) if v and v > 50)
        rows.append("| %s | %s | %s | %d / %d |" % (tool, cell(on), cell(off), slow, len((on or {}).get("echo", []))))
    return "\n".join(rows)


def flood_table(tools=TOOLS, nets=NETS):
    rows = ["| 망 | " + " | ".join(tools) + " |", "| --- |" + " --- |" * len(tools)]
    for net in nets:
        cells = []
        for tool in tools:
            def show(f):
                if not f:
                    return "-"
                secs = f.get("seconds")
                return "%s (%s)" % (fmt(secs, 2, "s") if secs is not None else "끝나지 않음", fmt(f["bytes_to_terminal"] / 1e6, 2, "MB"))
            # Run alone (`TESTS=flood`, -f1 -f2) where a slow link would keep
            # a run's earlier tests' bytes in front of it.
            alone = [r for r in (load("net-%s-%s-f%d" % (net, tool, i)) for i in (1, 2)) if r]
            values = [r.get("flood") or {} for r in (alone or runs(net, tool))]
            cells.append(" · ".join(show(v) for v in values) if values else "-")
        rows.append("| %s | %s |" % (net, " | ".join(cells)))
    return "\n".join(rows)


def render_table(tools=TOOLS, nets=NETS):
    rows = ["| 망 | " + " | ".join(tools) + " |", "| --- |" + " --- |" * len(tools)]
    for net in nets:
        cells = []
        for tool in tools:
            def show(r):
                if not r:
                    return "-"
                return "%d/%d · %s · %s" % (r["shown"], r["produced"], fmt(max(r["gaps_ms"], default=None), 0), fmt(pct(r["ages_ms"], 0.5), 0))
            cells.append("<br>".join(show((x or {}).get("render") or {}) for x in runs(net, tool)) or "-")
        rows.append("| %s | %s |" % (net, " | ".join(cells)))
    return "\n".join(rows)


def resource_table(net="lan", tools=TOOLS):
    head = "| 도구 | 서버 RSS 최대 | 서버 CPU 출력 폭주 | 서버 CPU 렌더 8초 | 서버 CPU 유휴 10초 | 클라 RSS 최대 | 클라 CPU 출력 폭주 | 클라 CPU 렌더 8초 | 클라 CPU 유휴 10초 |"
    rows = [head, "| --- |" + " --- |" * 8]
    for tool in tools:
        r = load("net-%s-%s" % (net, tool))
        if not r or "samples" not in r:
            continue
        s_server, s_client = r["samples"]["rb-server"], r["samples"]["rb-client"]
        start = r["phases"][0]["start"] if r.get("phases") else 0
        end = r["phases"][-1]["end"] if r.get("phases") else 0
        cells = []
        _, srss = usage(s_server, lambda p: server_owned(tool, p), start, end)
        _, crss = usage(s_client, lambda p: client_owned(tool, p), start, end)
        scpu = [usage(s_server, lambda p: server_owned(tool, p), *phase(r, n))[0] for n in ("flood", "render", "idle")]
        ccpu = [usage(s_client, lambda p: client_owned(tool, p), *phase(r, n))[0] for n in ("flood", "render", "idle")]
        cells = [fmt(srss / 1024, 1, "MB")] + [fmt(c, 2, "s") for c in scpu] + [fmt(crss / 1024, 1, "MB")] + [fmt(c, 2, "s") for c in ccpu]
        rows.append("| %s | %s |" % (tool, " | ".join(cells)))
    return "\n".join(rows)


def load_table(tools=TOOLS):
    head = "| 도구 | 출력 시간 | 터미널에 쓴 양 | 서버 CPU | 클라 CPU | 세션 10개 서버 RSS | 세션 하나 더할 때 서버 | 클라이언트 10개 RSS 합 |"
    rows = [head, "| --- |" + " --- |" * 7]
    for tool in tools:
        big, many = load("load-bigflood-%s" % tool), load("load-many-%s" % tool)
        cells = []
        if big and big.get("bigflood"):
            f = big["bigflood"]
            st, en = phase(big, "bigflood")
            scpu, _ = usage(big["samples"]["rb-server"], lambda p: server_owned(tool, p), st, en)
            ccpu, _ = usage(big["samples"]["rb-client"], lambda p: client_owned(tool, p), st, en)
            cells += [fmt(f["seconds"], 2, "s"), fmt(f["bytes_to_terminal"] / 1e6, 2, "MB"), fmt(scpu, 2, "s"), fmt(ccpu, 2, "s")]
        else:
            cells += ["-"] * 4
        if many and many.get("many"):
            st, en = phase(many, "many")
            _, srss = usage(many["samples"]["rb-server"], lambda p: server_owned(tool, p), st, en)
            _, crss = usage(many["samples"]["rb-client"], lambda p: client_owned(tool, p), st, en)
            one = load("net-lan-%s" % tool)
            _, single = usage(one["samples"]["rb-server"], lambda p: server_owned(tool, p), one["phases"][0]["start"], one["phases"][-1]["end"])
            cells += [fmt(srss / 1024, 1, "MB"), "+" + fmt((srss - single) / 1024 / 9, 1, "MB"), fmt(crss / 1024, 1, "MB")]
        else:
            cells += ["-"] * 3
        rows.append("| %s | %s |" % (tool, " | ".join(cells)))
    return "\n".join(rows)


def events_table():
    rows = ["| 상황 | " + " | ".join(TOOLS) + " |", "| --- |" + " --- |" * len(TOOLS)]
    def cell(mode, tool):
        r = load("event-%s-%s" % (mode, tool))
        if not r:
            return "-"
        if mode in ("udp", "udpsilent"):
            return "접속 %s" % fmt(r["connect_s"], 1, "s") if r.get("connect_s") is not None else "접속 실패"
        if r.get("error"):
            return r["error"]
        if mode == "outage":
            o = r["outage"]
            if not o["typed_while_down_ran"]:
                return "복구 안 됨" + ("" if o["client_alive"] else " (클라이언트 종료)")
            return "복구, 끊긴 동안 친 키 실행 %s 뒤" % fmt(o["seconds_to_show_it"], 1, "s")
        if mode == "roam":
            o = r["roam"]
            return ("이어짐 %s" % fmt(o["seconds"], 1, "s")) if o["usable_after"] else ("끊김" + ("" if o["client_alive"] else " (클라이언트 종료)"))
        if mode == "reattach":
            o = r["reattach"]
            return "같은 셸" if o["same_shell"] else ("새 셸" if o["reconnected"] else "실패")
    for mode, label in (("outage", "20초 단절 후 복구"), ("roam", "클라이언트 IP 변경"), ("reattach", "클라이언트 강제 종료 후 재실행"), ("udp", "UDP 차단(TCP만 통과)"), ("udpsilent", "UDP 차단, 응답 없이 버림")):
        rows.append("| %s | %s |" % (label, " | ".join(cell(mode, t) for t in TOOLS)))
    return "\n".join(rows)


def connect_column(tools):
    """Connect time of the network runs (one per run: first · second)."""
    rows = ["| 망 | " + " | ".join(tools) + " |", "| --- |" + " --- |" * len(tools)]
    for net in NETS + SLOW:
        rows.append("| %s | %s |" % (net, " | ".join(two(net, t, lambda r: r.get("connect_s"), lambda v: fmt(v, 2, "s")) for t in tools)))
    return "\n".join(rows)


def connect_table():
    """Median of the prompt-timed connects (`run.py connect`), min-max after it."""
    rows = ["| 망 | " + " | ".join(TOOLS) + " |", "| --- |" + " --- |" * len(TOOLS)]
    for net in NETS + SLOW:
        cells = []
        for t in TOOLS:
            times = [v for v in (load("connect-%s-%s" % (net, t)) or {}).get("connect_s", []) if v is not None]
            cells.append("%s (%s–%s)" % (fmt(pct(times, 0.5), 2, "s"), fmt(min(times), 2), fmt(max(times), 2)) if times else "-")
        rows.append("| %s | %s |" % (net, " | ".join(cells)))
    return "\n".join(rows)


if __name__ == "__main__":
    for title, fn in (("connect", connect_table), ("echo", latency_table), ("prediction", prediction_table), ("flood", flood_table),
                      ("render", render_table), ("resources lan", resource_table), ("load", load_table), ("events", events_table)):
        print("##", title)
        print(fn())
        print()
    # Slower links (`run.py net slow256k,slow64k`).
    for title, fn in (("slow echo", lambda: latency_table(TOOLS, SLOW)), ("slow flood", lambda: flood_table(TOOLS, SLOW)),
                      ("slow render", lambda: render_table(TOOLS, SLOW))):
        print("##", title)
        print(fn())
        print()
    # rsh over TCP (UDP blocked), beside rsh over UDP and the TCP tools.
    TCP = ["ssh", "et", "rsh", "rsh-tcp"]
    for title, fn in (("tcp connect", lambda: connect_column(TCP)), ("tcp echo", lambda: latency_table(TCP, NETS + SLOW)), ("tcp flood", lambda: flood_table(TCP, NETS + SLOW)),
                      ("tcp render", lambda: render_table(TCP, NETS + SLOW)), ("tcp resources lan", lambda: resource_table("lan", TCP)),
                      ("tcp load", lambda: load_table(TCP))):
        print("##", title)
        print(fn())
        print()
