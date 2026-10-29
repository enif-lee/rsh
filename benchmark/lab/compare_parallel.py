#!/usr/bin/env python3
"""Rebuild the recorded four-versus-twelve-worker comparison (no containers).

    python3 benchmark/lab/compare_parallel.py
"""
import json
import math
import statistics
from pathlib import Path

import charts

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / "docs/bench"
BASE = json.loads((OUT / "data/static-20261029.json").read_text())
NEW = json.loads((OUT / "data/parallel12-20261029.json").read_text())
BASE_MOVING = json.loads((OUT / "data/moving-20261029-release.json").read_text())
NEW_MOVING = json.loads((OUT / "data/moving-20261029-followup.json").read_text())
OLD = BASE["batches"][0]["results"]  # Matched three attempts, not the six-run aggregate.
CURRENT = NEW["batches"][0]["results"]
TOOLS = list(charts.ORDER)
NAMES = dict(charts.NAME)
NETS = dict(charts.NETS)
LOSSLESS = {"lan", "wan", "highlat", "lowbw", "verylow", "slow64k"}


def fmt(v, digits=1):
    return "—" if v is None else f"{v:.{digits}f}"


def median(rs, key):
    v = [r[key] for r in rs if r.get(key) is not None]
    return statistics.median(v) if v else None


def rss_mib(rs, key):
    value = median(rs, key)
    return value / 1024 if value is not None else None


def low(rs, key, q):
    good = [r for r in rs if not r.get("error")]
    if any(r.get(key) is None for r in good):
        return None
    gaps = sorted(g for r in good for g in r[key])
    return 1000 / max(gaps[math.ceil(len(gaps) * q) - 1], .001) if len(gaps) >= 10 else None


def select(results, test, tool, net=None, scene=None, to=None):
    return [r for r in results if r["test"] == test and r["tool"] == tool
            and (net is None or r["net"] == net) and (scene is None or r.get("scene") == scene)
            and (to is None or r.get("to") == to)]


def fps(rs):
    values = [r["fps"] for r in rs if r.get("fps") is not None]
    return (statistics.mean(values) if values else None,
            low(rs, "gaps_ms", .99), low(rs, "gaps_ms", .95))


def change(a, b):
    return None if a in (None, 0) or b is None else 100 * (b / a - 1)


def pair(a, b, digits=1):
    return f"{fmt(a, digits)} → {fmt(b, digits)}"


def table(headers, rows):
    return "\n".join(["| " + " | ".join(headers) + " |", "| " + " | ".join("---" for _ in headers) + " |"]
                     + ["| " + " | ".join(map(str, r)) + " |" for r in rows])


def image(name):
    return f"![{name} comparison]({name}.svg)"


def main():
    rows, stats = [], []
    for scene in ("compress", "bandwidth"):
        for net in NETS:
            for tool in TOOLS:
                a, b = select(OLD, "render", tool, net, scene), select(CURRENT, "render", tool, net, scene)
                fa, fb = fps(a), fps(b)
                na = sum(r.get("fps") is not None for r in a)
                nb = sum(r.get("fps") is not None for r in b)
                stats.append(dict(scene=scene, net=net, tool=tool, baseline_runs=na, current_runs=nb,
                                  baseline_fps=fa[0], current_fps=fb[0], mean_change_pct=change(fa[0], fb[0]),
                                  baseline_low1=fa[1], current_low1=fb[1], baseline_low5=fa[2], current_low5=fb[2],
                                  close_mean=na == nb == 3 and abs(fa[0] - fb[0]) <= max(1, .05 * fa[0])))
                rows.append([scene, net, NAMES[tool], f"{na}/3 → {nb}/3", pair(fa[0], fb[0]),
                             fmt(change(fa[0], fb[0])), pair(fa[1], fb[1]), pair(fa[2], fb[2]),
                             pair(median(a, "age_ms"), median(b, "age_ms"), 0)])
    # A descriptive screen, not an equivalence test or confidence interval.
    close_rows = []
    for label, tools in [("rsh UDP", {"rsh"}), ("SSH control", {"ssh"}), ("All five transports", set(TOOLS))]:
        for condition, nets in [("Lossless", LOSSLESS), ("Random loss", set(NETS) - LOSSLESS)]:
            group = [r for r in stats if r["tool"] in tools and r["net"] in nets]
            full = [r for r in group if r["baseline_runs"] == r["current_runs"] == 3]
            close_rows.append([label, condition, len(full), sum(r["close_mean"] for r in full),
                               fmt(statistics.median(abs(r["mean_change_pct"]) for r in full)) if full else "—"])
    # Dedicated colors represent concurrency, while each chart names its tool.
    charts.ORDER = ["baseline", "parallel12"]
    charts.NAME = {"baseline": "4 workers", "parallel12": "12 workers"}
    charts.LIGHT = {"baseline": "#2a78d6", "parallel12": "#eb6834"}
    charts.DARK = {"baseline": "#3987e5", "parallel12": "#d95926"}
    images = []
    for tool in ("rsh", "ssh"):
        for scene in ("compress", "bandwidth"):
            groups = [(label, {"baseline": fps(select(OLD, "render", tool, net, scene)),
                               "parallel12": fps(select(CURRENT, "render", tool, net, scene))}) for net, label in NETS.items()]
            name = f"parallel12-{tool}-{scene}"
            svg = charts.svg(f"{NAMES[tool]}: {scene}, 4 versus 12 workers",
                             "3 attempts each; light: average FPS, mid: 5% low, solid: 1% low", groups,
                             lambda v: f"{fmt(v[0])} / {fmt(v[1])} / {fmt(v[2])}", True, pair="lows", value_w=165)
            (OUT / f"{name}.svg").write_text(svg)
            images.append(image(name))
    times = NEW["timings"]
    case = lambda scene, net: next(r for r in stats if r["tool"] == "rsh" and r["scene"] == scene and r["net"] == net)
    wan_random, lan_random, bad_dashboard = case("bandwidth", "wan"), case("bandwidth", "lan"), case("compress", "badmobile")
    blocks = ["""# Four versus twelve benchmark workers

Same published rsh 0.2.0 Linux arm64 binary (`6729d398d4f1`), image,
packages, scenes and collector; requested 60 FPS, 100×30, eight-second
render windows. The primary baseline is the earlier **first 750-job batch**:
three attempts per case. Its second screen batch is excluded from this
comparison so both sides have equal repeat counts.

Static tests compare 4 versus 12 active workers. Sessions compare the
first attempt per tool, with at most four active jobs on either side.
Moving is a separate three-link retest after all static jobs stop; it is
not a twelve-worker load test. All timings and measurements below were
recorded on 2026-10-29; no application source was changed.

## Actual elapsed time
""", table(["Scope", "Earlier run", "New run", "Interpretation"], [
        ["Matched 750 static jobs", f"{times['baseline_main_s']/60:.2f} min", f"{times['main_s']/60:.2f} min", "Same cases and three repeats; concurrency differs"],
        ["Whole suite, start through moving postflight", f"{times['baseline_wall_s']/60:.2f} min", f"{times['wall_s']/60:.2f} min", "1,178 → 754 static/memory jobs; includes fewer screen/memory repeats and stage gaps"],
        ["Memory batch", "8 jobs (two repeats)", f"{times['memory_s']/60:.2f} min / 4 jobs", "One repeat per tool now"],
    ]), """The earlier whole-suite time includes its second screen batch and manual
stage gaps. Compare the matched 750-job row when assessing concurrency
speedup. The earlier 14–15 minute prediction assumed unchanged per-job
durations and excluded preparation/review; it was an estimate.

## Findings

The faster configuration did not produce identical measurements. rsh UDP
mean FPS is close in all 12 complete lossless scene/link pairs under the
descriptive rule below. Tail frame gaps vary in both directions, and the
lossy-path mean can differ substantially. Resource samples also differ;
these differences are observations, not demonstrated memory optimizations
or application regressions: the application binary was unchanged.
""", f"RTT 100 ms random-screen 1% low: **{pair(wan_random['baseline_low1'], wan_random['current_low1'])} FPS**; LAN random-screen 1% low: **{pair(lan_random['baseline_low1'], lan_random['current_low1'])} FPS**. At 256 kbit/s, RTT 400 ms and nominal 2% loss, dashboard mean FPS was **{pair(bad_dashboard['baseline_fps'], bad_dashboard['current_fps'])}**, a **{fmt(bad_dashboard['mean_change_pct'])}%** change.",
              """The runs differ in concurrency, time and realized random losses. This
experiment cannot isolate the cause of each difference or establish
statistical equivalence from three attempts. Twelve workers are faster for
routine mean-FPS checks; percentile and resource comparisons should retain
the same concurrency and enough independent repeats.

## How close are the results?

Exact equality is not expected across separate host runs and independent
random packet losses. The table below is a descriptive check of **mean FPS**:
difference no greater than the larger of 1 FPS or 5% of the baseline.
This rule was recorded before the new results were inspected. Only pairs
with all three observations are counted. It is not a statistical test of
equivalence; percentile lows and individual cases must also be inspected.
""", table(["Tool", "Link class", "Complete pairs", "Mean within threshold", "Median absolute mean change %"], close_rows),
              *images, "## Render results: 4 workers → 12 workers",
              "Avg FPS is the mean of available per-run FPS. Lows pool inter-frame gaps separately on each side. Age is the median of per-run medians. Counts show missing attempts explicitly. Percent changes are relative to the four-worker baseline.",
              table(["Scene", "Link", "Tool", "Measured runs", "Avg FPS", "Mean Δ %", "1% low", "5% low", "Age ms"], rows),
              "## Render traffic and resources", "All cells are baseline → new medians. RSS uses MiB; CPU is percent of one core. RSS is a sample, not a peak."]
    rows = []
    for scene in ("compress", "bandwidth"):
        for net in NETS:
            for tool in TOOLS:
                a, b = select(OLD, "render", tool, net, scene), select(CURRENT, "render", tool, net, scene)
                values = []
                for key, factor in [("kbit_s", 1), ("rss_client_kb", 1/1024), ("rss_server_kb", 1/1024), ("cpu_client", 1), ("cpu_server", 1)]:
                    va, vb = median(a, key), median(b, key)
                    values.append(pair(va * factor if va is not None else None, vb * factor if vb is not None else None, 2))
                rows.append([scene, net, NAMES[tool], *values])
    blocks += [table(["Scene", "Link", "Tool", "RX kbit/s", "Client RSS MiB", "Server RSS MiB", "Client CPU %", "Server CPU %"], rows),
               "## Link changes", "All cells are baseline → new. Post-change lows pool post-change gaps only."]
    rows = []
    for net, to in dict.fromkeys((r["net"], r["to"]) for r in OLD if r["test"] == "switch"):
        for scene in ("compress", "bandwidth"):
            for tool in TOOLS:
                a, b = select(OLD, "switch", tool, net, scene, to), select(CURRENT, "switch", tool, net, scene, to)
                rows.append([f"{net} → {to}", scene, NAMES[tool],
                             pair(median(a, "fps_after"), median(b, "fps_after")),
                             pair(low(a, "gaps_after_ms", .99), low(b, "gaps_after_ms", .99)),
                             pair(low(a, "gaps_after_ms", .95), low(b, "gaps_after_ms", .95)),
                             pair(median(a, "age_after_ms"), median(b, "age_after_ms"), 0)])
    blocks += [table(["Change", "Scene", "Tool", "FPS after", "1% low after", "5% low after", "Age after ms"], rows), "## Connection, key echo and bulk output"]
    rows = []
    for net in NETS:
        for tool in TOOLS:
            a, b = select(OLD, "echo", tool, net), select(CURRENT, "echo", tool, net)
            fa, fb = select(OLD, "flood", tool, net), select(CURRENT, "flood", tool, net)
            rows.append([net, NAMES[tool], pair(median(a, "connect_s"), median(b, "connect_s"), 3),
                         pair(median(a, "echo_ms"), median(b, "echo_ms")), pair(median(fa, "flood_s"), median(fb, "flood_s"), 2),
                         f"{sum(r.get('flood_s') is not None for r in fa)}/3 → {sum(r.get('flood_s') is not None for r in fb)}/3"])
    blocks += [table(["Link", "Tool", "Connect s", "Echo ms", "Flood s", "Flood completed"], rows), "Missing flood completion is a 30-second timeout or recorded startup failure, never zero seconds.", "## Connectivity events"]
    rows = []
    for test, key in [("outage", "resumed_s"), ("roam", "resumed_s"), ("reattach", "reconnect_s"), ("udpblock", "connect_s")]:
        for tool in TOOLS:
            a, b = select(OLD, test, tool), select(CURRENT, test, tool)
            same = f"{sum(r.get('same_shell') is True for r in a)}/3 → {sum(r.get('same_shell') is True for r in b)}/3" if test == "reattach" else "—"
            rows.append([test, NAMES[tool], pair(median(a, key), median(b, key), 2),
                         f"{sum(r.get(key) is not None for r in a)}/3 → {sum(r.get(key) is not None for r in b)}/3", same])
    blocks += [table(["Event", "Tool", "Recovery/connect s", "Observed", "Same shell"], rows),
               "Outage lasts five seconds; roaming is watched for eight seconds after the change. Missing responses are censored, not infinite measured recovery times. A restarted client reaching a prompt does not imply the same shell survived.", "## Sessions memory: one attempt per tool"]
    old_memory = [r for r in BASE["batches"][2]["results"] if r["iteration"] == 0]
    new_memory = NEW["batches"][1]["results"]
    rows = []
    for tool in ("rsh", "ssh", "mosh", "et"):
        a, b = select(old_memory, "sessions", tool), select(new_memory, "sessions", tool)
        rows.append([NAMES[tool], pair(median(a, "connected"), median(b, "connected"), 0),
                     pair(rss_mib(a, "client_kb"), rss_mib(b, "client_kb"), 2),
                     pair(rss_mib(a, "server_kb"), rss_mib(b, "server_kb"), 2),
                     pair(median(a, "idle_pkts_in_s"), median(b, "idle_pkts_in_s"), 2),
                     pair(median(a, "idle_pkts_out_s"), median(b, "idle_pkts_out_s"), 2)])
    blocks += [table(["Tool", "Connected", "Client RSS MiB", "Server RSS MiB", "Idle packets in/s", "Idle packets out/s"], rows), "## Moving: separate three-link retest"]
    rows = []
    for a in BASE_MOVING["results"]:
        b = next(r for r in NEW_MOVING["results"] if r["scene"] == a["scene"] and r["tool"] == a["tool"])
        rows.append([a["scene"], NAMES[a["tool"]], pair(a["fps"], b["fps"], 2), pair(a["low1_fps"], b["low1_fps"], 2),
                     pair(a["low5_fps"], b["low5_fps"], 2), pair(a["age_ms"], b["age_ms"], 0), pair(a["rss_mb"], b["rss_mb"], 2),
                     f"{a['to_end']} → {b['to_end']}"])
    blocks += [table(["Scene", "Tool", "Avg FPS", "1% low", "5% low", "Age ms", "Total RSS MiB", "End marker"], rows),
               "Both runs use a 158×43 controlled viewer, 52×41 workload panes, and two 30-second cycles per scene. Their impairment schedules are common within a run; actual application times and packet outcomes differ between runs. Scene traffic counters include warmup/drain and are retained in the raw snapshots, not compared as exact common-window rates.",
               "## Failed attempts", "All failures are retained. The baseline here uses its first three-repeat batch only."]
    rows = []
    for label, rs in [("4 workers", OLD), ("12 workers", CURRENT), ("4 workers memory", old_memory), ("12 workers memory", new_memory)]:
        for r in rs:
            if r.get("error"):
                rows.append([label, r["test"], r["net"], r.get("scene", "—"), NAMES[r["tool"]], r["iteration"], r["error"]])
    blocks += [table(["Run", "Test", "Link", "Scene", "Tool", "Iteration", "Error"], rows),
               "## Data and reproducibility",
               "[Four-worker baseline](data/static-20261029.json), [twelve-worker results](data/parallel12-20261029.json), [environment/preflight/postflight](data/environment-parallel12-20261029.json), [earlier moving frames](data/moving-20261029-release.json), [new moving frames](data/moving-20261029-followup.json). Original-file hashes, complete job results and raw frame observations are included. Rebuild with `python3 benchmark/lab/compare_parallel.py`."]
    (OUT / "parallel12-20261029.md").write_text("\n\n".join(blocks).rstrip() + "\n")
    (OUT / "data/parallel12-comparison-summary.json").write_text(json.dumps(dict(timings=times, render=stats), indent=2) + "\n")
    print("Rebuilt parallel12 comparison report, summary and four charts")


if __name__ == "__main__":
    main()
