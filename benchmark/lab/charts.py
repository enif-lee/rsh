#!/usr/bin/env python3
"""Charts for the README from a `lab.py run` result: grouped horizontal bars
as SVG, one group per network, one bar per tool, the median of the runs.

    python3 benchmark/lab/charts.py benchmark/lab/out/run-<time>.json[,run-<time>.json...] docs/bench [chart ...]

With chart names (fps-dashboard, fps-random, lows-dashboard, lows-random,
age, flood, traffic, packets, memory, memory-sessions), only those; a chart
the result has no data for is left out. Lows are over every run's frame
gaps together (`gaps_ms`), as `lab.py` sums them up.

Colors are a validated categorical palette (light and dark), assigned to the
tools in a fixed order; every bar carries its value, so identity and value
never rest on color alone.
"""
import json, math, os, statistics, sys

ORDER = ["rsh", "rsh-tcp", "ssh", "et", "mosh"]
NAME = {"rsh": "rsh (UDP)", "rsh-tcp": "rsh (TCP)", "ssh": "OpenSSH", "et": "ET", "mosh": "mosh"}
LIGHT = {"rsh": "#2a78d6", "rsh-tcp": "#eb6834", "ssh": "#1baf7a", "et": "#eda100", "mosh": "#e87ba4"}
DARK = {"rsh": "#3987e5", "rsh-tcp": "#d95926", "ssh": "#199e70", "et": "#c98500", "mosh": "#d55181"}
NETS = [("lan", "LAN"), ("wan", "RTT 100 ms"), ("lowbw", "1 Mbit/s · RTT 100 ms"), ("verylow", "256 kbit/s · RTT 100 ms"),
        ("slow64k", "64 kbit/s · RTT 100 ms"), ("mobile", "512 kbit/s · RTT 200 ms · 1% loss"),
        ("badmobile", "256 kbit/s · RTT 400 ms · 2% loss"), ("highlat", "RTT 300 ms"), ("lossy", "2% loss · RTT 100 ms")]

BAR, GAP, GROUP_GAP, LABEL_W, VALUE_W, WIDTH = 11, 2, 18, 92, 78, 760


def medians(results, test, key, scene=None):
    """{net: {tool: median}} of `key` over the runs."""
    out = {}
    for r in results:
        if r.get("test") != test or (scene and r.get("scene") != scene) or r.get(key) is None:
            continue
        out.setdefault(r["net"], {}).setdefault(r["tool"], []).append(r[key])
    return {net: {tool: statistics.median(v) for tool, v in tools.items()} for net, tools in out.items()}


def lows(results, scene):
    """{net: {tool: (1% low, 5% low)}} over every run's frame gaps."""
    gaps = {}
    for r in results:
        if r.get("test") == "render" and r.get("scene") == scene and r.get("gaps_ms") is not None:
            gaps.setdefault(r["net"], {}).setdefault(r["tool"], []).extend(r["gaps_ms"])
    out = {}
    for net, tools in gaps.items():
        for tool, g in tools.items():
            g.sort()
            if len(g) >= 10:
                at = lambda q: g[min(len(g) - 1, math.ceil(len(g) * q) - 1)]
                out.setdefault(net, {})[tool] = (1000 / max(at(0.99), 1e-3), 1000 / max(at(0.95), 1e-3))
    return out


def packets(results, scene):
    """{net: {tool: (packets in, out)}} for each frame shown, the client's."""
    out = {}
    for r in results:
        if r.get("test") == "render" and r.get("scene") == scene and r.get("pkts_in_s") is not None and r.get("fps"):
            out.setdefault(r["net"], {}).setdefault(r["tool"], []).append((r["pkts_in_s"] / r["fps"], r["pkts_out_s"] / r["fps"]))
    return {net: {tool: (statistics.median(a for a, _ in v), statistics.median(b for _, b in v)) for tool, v in tools.items()} for net, tools in out.items()}


def sessions(results):
    """[(side, {tool: MB})] with `count` sessions at once."""
    sides = []
    count = next((r["count"] for r in results if r.get("test") == "sessions"), None)
    for key, label in (("client_kb", "Clients, all of them"), ("server_kb", "Server side, all of it")):
        values = {}
        for r in results:
            if r.get("test") == "sessions" and r.get(key):
                values.setdefault(r["tool"], []).append(r[key] / 1024)
        if values:
            sides.append((label, {tool: statistics.median(v) for tool, v in values.items()}))
    return sides, count


def per_frame(results, scene):
    """{net: {tool: median KB received per frame shown}}."""
    out = {}
    for r in results:
        if r.get("test") == "render" and r.get("scene") == scene and r.get("kbit_s") and r.get("fps"):
            out.setdefault(r["net"], {}).setdefault(r["tool"], []).append(r["kbit_s"] * 1000 / 8 / r["fps"] / 1024)
    return {net: {tool: statistics.median(v) for tool, v in tools.items()} for net, tools in out.items()}


def memory(results):
    """[(side, {tool: median MB})] over every render run."""
    sides = []
    for key, label in (("rss_client_kb", "Client"), ("rss_server_kb", "Server (one session; sshd counts its listener)")):
        values = {}
        for r in results:
            if r.get("test") == "render" and r.get(key):
                values.setdefault(r["tool"], []).append(r[key] / 1024)
        if values:
            sides.append((label, {tool: statistics.median(v) for tool, v in values.items()}))
    return sides


def svg(title, note, groups, fmt, higher, log=False, missing="n/a", pair=None):
    """`groups`: [(label, {tool: value})]; `missing` stands where a tool
    has no value (e.g. it did not finish). With `pair` ("overlay" or
    "stack") each value is two: the first drawn solid, the second light,
    over it (to its length) or after it."""
    tools = [t for t in ORDER if any(t in values for _, values in groups)]
    if not tools:
        return None
    size = (lambda v: v[1] if pair == "overlay" else v[0] + v[1]) if pair else (lambda v: v)
    known = [size(v) for _, values in groups for v in values.values() if v is not None]
    # Log scale from a little under the least value, so the least bar shows.
    least = min(known) if known else 1.0
    scale = (lambda v: math.log10(v / (least * 0.6))) if log else (lambda v: v)
    most = max(scale(v) for v in known) if known else 1.0
    plot = WIDTH - LABEL_W - VALUE_W - 16
    legend_h, top = 22, 58
    group_h = len(tools) * (BAR + GAP) + 18
    height = top + legend_h + len(groups) * (group_h + GROUP_GAP) + 18
    css = ["svg{font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Helvetica,Arial,sans-serif}",
           ".bg{fill:#fcfcfb}.t1{fill:#1d1d1b}.t2{fill:#5f5e58}.axis{stroke:#e2e1dc}"]
    css += [".s-%s{fill:%s}" % (t.replace("-", "_"), LIGHT[t]) for t in tools]
    dark = [".bg{fill:#1a1a19}.t1{fill:#f0efec}.t2{fill:#c3c2b7}.axis{stroke:#3a3a37}"] + [".s-%s{fill:%s}" % (t.replace("-", "_"), DARK[t]) for t in tools]
    css.append("@media (prefers-color-scheme:dark){%s}" % "".join(dark))
    out = ['<svg xmlns="http://www.w3.org/2000/svg" width="%d" height="%d" viewBox="0 0 %d %d" role="img" aria-label="%s">' % (WIDTH, height, WIDTH, height, title),
           "<style>%s</style>" % "".join(css), '<rect class="bg" width="100%" height="100%" rx="8"/>',
           '<text class="t1" x="16" y="26" font-size="15" font-weight="600">%s</text>' % title,
           '<text class="t2" x="16" y="44" font-size="11.5">%s (%s is better%s)</text>' % (note, "higher" if higher else "lower", ", log scale" if log else "")]
    x = 16
    for t in tools:
        out.append('<rect class="s-%s" x="%d" y="%d" width="10" height="10" rx="2"/>' % (t.replace("-", "_"), x, top - 2))
        out.append('<text class="t2" x="%d" y="%d" font-size="11.5">%s</text>' % (x + 14, top + 7, NAME[t]))
        x += 14 + 7.2 * len(NAME[t]) + 18
    y = top + legend_h
    for label, values in groups:
        out.append('<text class="t1" x="16" y="%d" font-size="12" font-weight="600">%s</text>' % (y + 10, label))
        y += 18
        best = (max if higher else min)((v if not pair else v[0] for v in values.values() if v is not None), default=None)
        out.append('<line class="axis" x1="%d" y1="%d" x2="%d" y2="%d" stroke-width="1"/>' % (LABEL_W + 16, y - 2, LABEL_W + 16, y + len(tools) * (BAR + GAP)))
        for t in tools:
            v = values.get(t)
            out.append('<text class="t2" x="%d" y="%d" font-size="11" text-anchor="end">%s</text>' % (LABEL_W + 10, y + BAR - 2, NAME[t]))
            if v is None:
                out.append('<text class="t2" x="%d" y="%d" font-size="11">%s</text>' % (LABEL_W + 20, y + BAR - 2, missing))
            elif pair:
                cls = t.replace("-", "_")
                first = max(2.0, plot * scale(v[0]) / most if most else 0)
                total = max(first, plot * scale(size(v)) / most if most else 0)
                out.append('<rect class="s-%s" x="%d" y="%d" width="%.1f" height="%d" rx="2" opacity="0.38"/>' % (cls, LABEL_W + 16, y, total, BAR))
                out.append('<rect class="s-%s" x="%d" y="%d" width="%.1f" height="%d" rx="2"/>' % (cls, LABEL_W + 16, y, first, BAR))
                weight = ' font-weight="600"' if v[0] == best else ""
                out.append('<text class="t1" x="%.1f" y="%d" font-size="11"%s>%s</text>' % (LABEL_W + 22 + total, y + BAR - 2, weight, fmt(v)))
            else:
                w = max(2.0, plot * scale(v) / most if most else 0)
                out.append('<rect class="s-%s" x="%d" y="%d" width="%.1f" height="%d" rx="2"/>' % (t.replace("-", "_"), LABEL_W + 16, y, w, BAR))
                weight = ' font-weight="600"' if v == best else ""
                out.append('<text class="t1" x="%.1f" y="%d" font-size="11"%s>%s</text>' % (LABEL_W + 22 + w, y + BAR - 2, weight, fmt(v)))
            y += BAR + GAP
        y += GROUP_GAP
    out.append("</svg>")
    return "\n".join(out)


def main():
    if len(sys.argv) < 3:
        sys.exit(__doc__)
    results = [r for path in sys.argv[1].split(",") for r in json.load(open(path))["results"]]
    os.makedirs(sys.argv[2], exist_ok=True)
    only = set(sys.argv[3:])
    def groups(table):
        return [(label, table[net]) for net, label in NETS if net in table]
    charts = {
        "fps-dashboard.svg": svg("Frames shown per second: a full-screen dashboard at 30 fps", "median of runs",
                                 groups(medians(results, "render", "fps", "compress")), lambda v: "%.1f" % v, True),
        "fps-random.svg": svg("Frames shown per second: a random full screen at 30 fps", "every cell changes each frame",
                              groups(medians(results, "render", "fps", "bandwidth")), lambda v: "%.1f" % v, True),
        "age.svg": svg("How old the screen is: a full-screen dashboard at 30 fps", "median age of what is on screen",
                       groups(medians(results, "render", "age_ms", "compress")), lambda v: "%.0f ms" % v if v < 1000 else "%.1f s" % (v / 1000), False, log=True),
        "flood.svg": svg("Time to finish `seq 1 100000`", "until the prompt is back",
                         groups(medians(results, "flood", "flood_s")), lambda v: "%.2f s" % v, False, log=True, missing="> 30 s (not done)"),
        "traffic.svg": svg("Data received per frame shown: a full-screen dashboard at 30 fps", "network bytes into the client, median of runs",
                           groups(per_frame(results, "compress")), lambda v: "%.2f KB" % v, False, log=True),
        "memory.svg": svg("Memory while drawing at 30 fps", "resident memory halfway through each render, median of all of them",
                          memory(results), lambda v: "%.1f MB" % v, False),
        "lows-dashboard.svg": svg("1% and 5% lows: a full-screen dashboard at 30 fps", "solid: 1% low, light: 5% low, fps the slowest frames would give",
                                  groups(lows(results, "compress")), lambda v: "%.1f / %.1f" % v, True, pair="overlay"),
        "lows-random.svg": svg("1% and 5% lows: a random full screen at 30 fps", "solid: 1% low, light: 5% low, fps the slowest frames would give",
                               groups(lows(results, "bandwidth")), lambda v: "%.1f / %.1f" % v, True, pair="overlay"),
        "packets.svg": svg("Packets for each frame shown: a full-screen dashboard at 30 fps", "the client's, solid: in, light: out",
                           groups(packets(results, "compress")), lambda v: "%.1f in / %.1f out" % v, False, pair="stack"),
    }
    sides, count = sessions(results)
    charts["memory-sessions.svg"] = svg("Memory with %s sessions at once" % count, "each shell having printed 3000 lines, then quiet; resident memory, all processes",
                                        sides, lambda v: "%.1f MB" % v, False) if sides else None
    unknown = only - {name[:-4] for name in charts}
    if unknown:
        sys.exit("charts.py: no chart %s; there are %s" % (", ".join(sorted(unknown)), ", ".join(name[:-4] for name in charts)))
    for name, text in charts.items():
        if only and name[:-4] not in only or text is None:
            continue
        with open(os.path.join(sys.argv[2], name), "w") as f:
            f.write(text)
        print(os.path.join(sys.argv[2], name))


if __name__ == "__main__":
    main()
