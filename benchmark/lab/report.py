#!/usr/bin/env python3
"""Rebuild BENCHMARK.md, README benchmark sections and charts from saved data.

    python3 benchmark/lab/report.py

No containers, network access or third-party Python packages are required.
"""
import json
import statistics
import subprocess
import sys
from pathlib import Path

import charts

ROOT = Path(__file__).resolve().parents[2]
DATA = ROOT / "docs/bench/data"
STATIC = json.loads((DATA / "static-20261028.json").read_text())
MOVING = json.loads((DATA / "moving-20261029.json").read_text())
RESULTS = [r for b in STATIC["batches"] for r in b["results"]]
TOOLS = charts.ORDER
NETS = dict(charts.NETS)


def runs(test, tool, net=None, scene=None, to=None):
    return [r for r in RESULTS if r["test"] == test and r["tool"] == tool
            and (net is None or r["net"] == net)
            and (scene is None or r.get("scene") == scene)
            and (to is None or r.get("to") == to)]


def median(rs, key):
    values = [r[key] for r in rs if r.get(key) is not None]
    return statistics.median(values) if values else None


def fmt(value, digits=1):
    return "—" if value is None else f"{value:.{digits}f}"


def table(headers, rows):
    return "\n".join(["| " + " | ".join(headers) + " |",
                      "| " + " | ".join("---" for _ in headers) + " |"]
                     + ["| " + " | ".join(map(str, row)) + " |" for row in rows])


def low(rs, key="gaps_ms", quantile=.99):
    # Match lab.py pooled_low: missing gaps do not become zero-FPS samples.
    if any(r.get(key) is None for r in rs):
        return None
    gaps = sorted(g for r in rs for g in r[key])
    if len(gaps) < 10:
        return None
    import math
    return 1000 / max(gaps[min(len(gaps) - 1, math.ceil(len(gaps) * quantile) - 1)], .001)


def render_row(net, tool, scene):
    rs = runs("render", tool, net, scene)
    ok = [r for r in rs if r.get("fps") is not None]
    avg = statistics.mean(r["fps"] for r in ok) if ok else None
    return [NETS[net], charts.NAME[tool], len(ok), fmt(avg), fmt(median(rs, "fps")),
            fmt(low(ok)), fmt(low(ok, quantile=.95)), fmt(median(rs, "age_ms"), 0),
            fmt(median(rs, "kbit_s"), 0),
            fmt(median(rs, "rss_client_kb") / 1024, 2),
            fmt(median(rs, "rss_server_kb") / 1024, 2)]


RENDER_HEADERS = ["Link", "Tool", "Runs", "Avg FPS", "Median FPS", "1% low", "5% low",
                  "Age ms", "RX kbit/s", "Client RSS MiB", "Server RSS MiB"]


def image(name, alt):
    return f"![{alt}](docs/bench/{name}.svg)"


def save_chart(name, title, note, groups, formatter, higher, **kwargs):
    text = charts.svg(title, note, groups, formatter, higher, **kwargs)
    assert text is not None
    (ROOT / f"docs/bench/{name}.svg").write_text(text)


def write_report():
    blocks = ["""# Benchmark report

Recorded comparisons of rsh (UDP), rsh with UDP blocked (TCP fallback),
OpenSSH, Eternal Terminal (ET), and mosh. All tables and charts below are
generated from the [committed measurement snapshots](docs/bench/data).

## Measurement scope

| Dataset | Conditions | Samples | Version provenance |
| --- | --- | --- | --- |
| Static links, 2026-10-28 | Five transports; 100×30 terminal; requested 60 FPS; 8 s render window | Six render/switch runs per case; three other tests; two ten-session runs | Predates 0.2.0. Original run JSON does not embed an rsh build ID; exact binary cannot be certified. |
| Moving link, 2026-10-29 | SSH/mosh/rsh; 158×43 terminal; requested 60 FPS; 60 s per scene | One run, two 30 s cycles per scene | Development build `7a1d9454d5b1`, with the transport implementation subsequently shipped in 0.2.0. |

The static snapshot restores the original README comparison. It is not a
new benchmark of release build `6729d398d4f1`. The moving measurement is
reported separately because its window, screen size and impairment
semantics differ. No measurements were rerun to produce this report.

### Environment and workloads

The recorded environment was an Apple M5 (10 cores, 32 GB), macOS 27.0.1,
OrbStack/Docker, and Ubuntu 24.04 Linux arm64 containers. The lab image
installs OpenSSH, mosh and ET from Ubuntu/PPA packages
([Dockerfile](benchmark/harness/Dockerfile)). The contemporaneous environment
record lists OpenSSH 9.6p1, mosh 1.4.0 and ET 7.0.0; the saved run JSON does
not independently pin the image digest or package versions.

Each job has a server/client container pair and an isolated shaped link;
jobs share host CPUs. Static batches used ten parallel jobs. This can
affect frame timing. `compress` is a compressible full-screen dashboard;
`bandwidth` changes every cell with random content. Both request 60 FPS.
The client reconstructs terminal output and observes frame headers; this
does not measure GPU presentation or prove that every cell was repainted.

### Metrics and aggregation

- **Avg FPS** is the arithmetic mean of each run's displayed frames divided
  by its observation duration. **Median FPS** is the median of those run
  averages; the original opening table and FPS charts use this median.
- **1% / 5% low** are `1000 / p99(gap_ms)` and `1000 / p95(gap_ms)` using
  nearest-rank percentiles over all six runs' inter-frame gaps pooled
  together. They are percentile reciprocals, not averages of the slowest
  one/five percent. Startup/end boundary gaps are excluded from lows.
- **Age** is the median of each run's median generation-to-observation
  time; it includes processing and queueing and is not RTT. Static windows
  begin at the first observed frame; late backlog is not drained into FPS.
- **Network** is received interface traffic at the client, in decimal
  kbit/s, including protocol overhead. KiB/frame is computed per run and
  then medianed. Packet counts include inbound and outbound interface
  traffic. These are not payload-only byte counts.
- **RSS** is a halfway-through-render sample, in MiB (KiB / 1024), medianed
  across runs; not peak RSS or PSS. Client and server are separate. Server
  process selection includes sshd's listener, ET's shared daemon and
  terminal, mosh-server, or rsh's shared serve process; shells are excluded.
- Other scalar values are medians of available results. Successful/total
  counts, timeouts and collection failures are preserved below. A missing
  observation is never a zero or proof of successful recovery.
- CPU is percent of one core over a render window, not total machine load.
  The original collector's `torn` counter can double-count observations;
  it is not used as a count of visible defects in this report.

Metric implementations: [agent.py](benchmark/lab/agent.py),
[pooled lows and summaries](benchmark/lab/lab.py),
[original charts](benchmark/lab/charts.py), and
[report generator](benchmark/lab/report.py).

### Static network profiles

`tc netem` applies delay and independent random loss in both directions;
rate limits below apply server → client. RTT is the nominal delay total,
before queueing. No bandwidth cap means no configured rate limit.
""", table(["Profile", "Conditions"], [(n, label) for n, label in NETS.items()]),
              """## 1. Dashboard rendering

Light bars show median FPS, medium bars 5% low and solid bars 1% low.
All five transports, all nine network profiles and all six runs are included.
""", image("fps-dashboard", "Dashboard FPS, 1% low and 5% low by link and transport"),
              table(RENDER_HEADERS, [render_row(n, t, "compress") for n in NETS for t in TOOLS]),
              """## 2. Random full-screen rendering

Every cell changes each frame. This workload compresses less effectively
and exposes a different bandwidth and frame-gap tradeoff than the dashboard.
""", image("fps-random", "Random-screen FPS, 1% low and 5% low by link and transport"),
              table(RENDER_HEADERS, [render_row(n, t, "bandwidth") for n in NETS for t in TOOLS]),
              "## 3. Frame age", image("age", "Dashboard median frame age by link and transport"),
              "The rendering tables above include age for both scenes. The chart uses a log scale.",
              "## 4. Network traffic and CPU", image("network", "Dashboard received KiB per displayed frame"),
              image("packets", "Dashboard inbound and outbound packets per displayed frame"),
              "The render tables give RX kbit/s. Below: median per-run bytes/frame, packets/frame and CPU for each scene."]
    for scene, label in [("compress", "Dashboard"), ("bandwidth", "Random screen")]:
        rows = []
        for n in NETS:
            for t in TOOLS:
                rs = runs("render", t, n, scene)
                derived = [{"kib": r["kbit_s"] * 1000 / 8 / r["fps"] / 1024,
                            "in": r["pkts_in_s"] / r["fps"], "out": r["pkts_out_s"] / r["fps"]}
                           for r in rs if r.get("fps")]
                rows.append([n, charts.NAME[t], fmt(median(derived, "kib"), 2),
                             fmt(median(derived, "in"), 2), fmt(median(derived, "out"), 2),
                             fmt(median(rs, "cpu_client"), 2), fmt(median(rs, "cpu_server"), 2)])
        blocks += [f"### {label}", table(["Link", "Tool", "RX KiB/frame", "Packets in/frame", "Packets out/frame", "Client CPU %", "Server CPU %"], rows)]
    blocks += ["## 5. Resident memory", image("memory", "Median client and server RSS while rendering"),
               "Per-link RSS for both scenes is in sections 1–2. The chart pools the render samples across links and scenes.",
               "### Ten simultaneous sessions", image("memory-sessions", "Combined client and server RSS for ten idle sessions")]
    rows = []
    for t in TOOLS:
        rs = runs("sessions", t)
        if not rs:
            continue
        rows.append([charts.NAME[t], len(rs), fmt(median(rs, "client_kb") / 1024, 2),
                     fmt(median(rs, "server_kb") / 1024, 2),
                     fmt(median(rs, "idle_pkts_in_s"), 2), fmt(median(rs, "idle_pkts_out_s"), 2)])
    blocks += [table(["Tool", "Runs", "All clients MiB", "Server MiB", "Idle packets in/s", "Idle packets out/s"], rows),
               "Each shell prints 3,000 lines, then becomes quiet. Packet rates are totals for the container's ten sessions. No rsh-TCP ten-session result was recorded.",
               "## 6. Connection and key echo"]
    rows, connect_groups, echo_groups = [], [], []
    for n in NETS:
        cs, es = {}, {}
        for t in TOOLS:
            rs = runs("echo", t, n)
            ok = [r for r in rs if r.get("echo_ms") is not None]
            cs[t], es[t] = median(rs, "connect_s"), median(ok, "echo_ms")
            rows.append([n, charts.NAME[t], f"{len(ok)}/{len(rs)}", fmt(cs[t], 3), fmt(es[t]),
                         fmt(median(ok, "echo_max_ms")), fmt(median(ok, "pkts_in_key")), fmt(median(ok, "pkts_out_key"))])
        connect_groups.append((NETS[n], cs)); echo_groups.append((NETS[n], es))
    save_chart("connect", "Connection to a shell prompt", "echo jobs only; median seconds", connect_groups, lambda v: f"{v:.3f} s", False)
    save_chart("echo", "Key to observed echo", "median of per-run key medians; milliseconds", echo_groups, lambda v: f"{v:.1f} ms", False)
    blocks += [image("connect", "Connection time in echo jobs"), image("echo", "Key echo latency"),
               table(["Link", "Tool", "Echo runs", "Connect s", "Echo ms", "Echo max ms", "Packets in/key", "Packets out/key"], rows),
               "Ten keys per run. Echo includes predictive local output for mosh/rsh. Connection times come from these same echo jobs, including their startup path; they are not a new dedicated cold-install test.",
               "## 7. Bulk output", image("flood", "Time until the prompt returns after seq 1 100000")]
    rows = []
    for n in NETS:
        for t in TOOLS:
            rs = runs("flood", t, n)
            completed = sum(r.get("flood_s") is not None for r in rs)
            timeout = sum(not r.get("error") and r.get("flood_s") is None for r in rs)
            rows.append([n, charts.NAME[t], f"{completed}/{len(rs)}", fmt(median(rs, "flood_s"), 2), timeout,
                         sum(bool(r.get("error")) for r in rs), fmt(median(rs, "bytes_to_terminal"), 0)])
    blocks += [table(["Link", "Tool", "Completed", "Completion s", "30 s timeouts", "Errors", "Terminal bytes"], rows),
               "Completion is the marker after `seq 1 100000`, watched for 30 s. When a case is mixed, completion time covers only completed runs; the counts remain explicit. mosh and rsh may skip intermediate output, so prompt recovery does not imply delivery of the complete byte stream.",
               "## 8. Link changes", "Each scene runs two seconds on the first link and eight seconds after the change. FPS before/after is medianed; post-change lows pool only post-change inter-frame gaps. Adaptation is the first whole-second bucket reaching 90% of that tool's own settled FPS, not time to recover 60 FPS."]
    switches = list(dict.fromkeys((r["net"], r["to"]) for r in RESULTS if r["test"] == "switch"))
    for scene, label in [("compress", "Dashboard"), ("bandwidth", "Random screen")]:
        groups, rows = [], []
        for n, to in switches:
            values = {}
            for t in TOOLS:
                rs = runs("switch", t, n, scene, to)
                ok = [r for r in rs if r.get("fps_after") is not None]
                values[t] = (median(ok, "fps_after"), low(ok, "gaps_after_ms"), low(ok, "gaps_after_ms", .95))
                rows.append([f"{n} → {to}", charts.NAME[t], f"{len(ok)}/{len(rs)}", fmt(median(ok, "fps_before")),
                             fmt(values[t][0]), fmt(values[t][1]), fmt(values[t][2]), fmt(median(ok, "age_after_ms"), 0), fmt(median(ok, "adapt_s"))])
            groups.append((f"{n} → {to}", values))
        name = "switch-dashboard" if scene == "compress" else "switch-random"
        save_chart(name, f"Link changes: {label}", "after change; light: median FPS, mid: 5% low, solid: 1% low", groups,
                   lambda v: f"{v[0]:.1f} / {fmt(v[1])} / {fmt(v[2])}", True, pair="lows", value_w=160)
        blocks += [f"### {label}", image(name, f"{label} FPS and lows after each link change"),
                   table(["Change", "Tool", "Runs", "FPS before", "FPS after", "1% low after", "5% low after", "Age after ms", "Adapt s"], rows)]
    blocks += ["## 9. Outage, roaming, reattachment and UDP block",
               "Three runs each on RTT 100 ms. Outage is five seconds; roaming is watched for eight seconds. Reattachment restarts the client. UDP-block connection is watched for 15 seconds."]
    rows, groups = [], []
    for test, key, label in [("outage", "resumed_s", "After 5 s outage"), ("roam", "resumed_s", "After IP change"),
                             ("reattach", "reconnect_s", "Client restart"), ("udpblock", "connect_s", "UDP silently blocked")]:
        values = {}
        for t in TOOLS:
            rs = runs(test, t)
            values[t] = median(rs, key)
            observed = sum(r.get(key) is not None for r in rs)
            same = f"{sum(r.get('same_shell') is True for r in rs)}/{len(rs)}" if test == "reattach" else "—"
            rows.append([label, charts.NAME[t], f"{observed}/{len(rs)}", fmt(values[t], 2), same, sum(bool(r.get("error")) for r in rs)])
        groups.append((label, values))
    save_chart("events", "Connectivity events", "median seconds; n/a: no observation within the test", groups, lambda v: f"{v:.2f} s", False)
    blocks += [image("events", "Recovery and connection times for connectivity events"),
               table(["Event", "Tool", "Observed", "Time s", "Same shell", "Errors"], rows),
               "Restart time alone does not establish session continuity: see Same shell. A missing roaming response within eight seconds is a censored observation, not a measured infinite recovery time. All three UDP-block mosh jobs reported no prompt, consistent with its UDP-only transport.",
               "## 10. Moving network", "This is the later development-build dataset, not pooled into the static comparison. Three independent links replay a common delay/rate/outage schedule on the Docker VM clock. Each 60 s scene contains two 30 s cycles. Nominal loss fractions are shared intervals of complete outage, unlike static independent per-packet random loss."]
    phases = [p for p in MOVING["windows"][1]["network_changes"]
              if not p.get("after_measurement") and p["scheduled_s"] < 30]
    phase_rows = []
    for i, p in enumerate(phases):
        end = phases[i + 1]["scheduled_s"] if i + 1 < len(phases) else 30
        phase_rows.append([f"{p['scheduled_s']:g}–{end:g}", p["phase"],
                           f"{p['delay_ms']} ± {p['jitter_ms']}", p["outage_time_pct"],
                           p["down_kbit"], p["up_kbit"]])
    blocks += [table(["Cycle seconds", "Phase", "One-way delay / jitter ms", "Outage time %", "Down kbit/s", "Up kbit/s"], phase_rows),
               "Delay jitter is sampled every 250 ms; the ± column gives the configured jitter parameter, not a hard bound. The committed trace includes exact scheduled values and actual application timestamps."]
    rows, groups = [], []
    for scene, label in [("compress", "Dashboard"), ("bandwidth", "Random screen")]:
        w = next(w for w in MOVING["windows"] if w.get("scene") == scene)
        values = {}
        for r in MOVING["results"]:
            if r["scene"] != scene:
                continue
            t = r["tool"]; i = MOVING["tools"].index(t)
            values[t] = (r["fps"], r["low1_fps"], r["low5_fps"])
            rows.append([label, charts.NAME[t], fmt(r["fps"], 2), fmt(r["low1_fps"], 2), fmt(r["low5_fps"], 2),
                         fmt(r["age_ms"], 0), fmt(w["rx_bytes"][i] / 1024**2, 2), fmt(r["kb_per_frame"], 2),
                         fmt(w["rss_kb"][i][0] / 1024, 2), fmt(w["rss_kb"][i][1] / 1024, 2)])
        groups.append((label, values))
    save_chart("moving", "Moving link: two 30-second cycles", "one 60 s run per scene; light: FPS, mid: 5% low, solid: 1% low", groups,
               lambda v: f"{v[0]:.2f} / {v[1]:.2f} / {v[2]:.2f}", True, pair="lows", value_w=175)
    blocks += [image("moving", "Moving-link displayed FPS and frame-gap lows"),
               table(["Scene", "Tool", "Avg FPS", "1% low", "5% low", "Age ms", "Scene RX MiB", "Reported KiB/frame", "Client RSS MiB", "Server RSS MiB"], rows),
               "The demo's scene RX counters include warmup/drain outside the common FPS window; reported KiB/frame also uses the collector's broader frame count. They are preserved as recorded and must not be read as 60-second RX rates or compared directly with static KiB/frame. RSS uses the demo's recorded samples rather than a peak-memory trace.",
               "Dashboard rsh averaged 58.0 / 57.6 FPS in the final five seconds of each clear phase. Its first full one-second window at ≥54 frames completed 1.2 / 2.45 s after those phases began. Random screens averaged 56.2 / 56.4 FPS in those final windows. Maximum application skew between containers was 26.75 / 32.91 ms (dashboard/random). See the [measurement record](docs/bench/moving-recovery.md#measurement) for timing limits.",
               "## Collection failures and limitations"]
    rows = []
    for batch in STATIC["batches"]:
        for r in batch["results"]:
            if r.get("error"):
                rows.append([Path(batch["source"]).stem, r["test"], r["net"] + (" → " + r["to"] if r.get("to") else ""),
                             r.get("scene", "—"), charts.NAME[r["tool"]], r["iteration"], r["error"]])
    blocks += [table(["Source", "Test", "Link", "Scene", "Tool", "Iteration", "Error"], rows),
               "All 540 static render jobs completed. Two of 300 switch jobs lacked a prompt; their groups contain five measured runs rather than six. Other tests use the first main batch only. Of seven collection failures, three are the expected mosh UDP-block no-prompt cases. No failed job was silently converted to a numeric result.",
               "Results cover synthetic isolated paths on one shared host, not fairness with cross traffic, real radio traces, independent hardware replications or statistical confidence intervals. Static nominal loss percentages do not guarantee equal realized drops across protocols. High average FPS can coexist with worse tails: for example, the static random-screen `badmobile` case favors mosh over rsh in both average delivery and 1% low. No universal win or non-regression claim is made.",
               "## Data and reproduction",
               table(["Source run", "Jobs", "Used for", "Original SHA-256"],
                     [(Path(b["source"]).name, len(b["results"]),
                       "render, switch + other tests" if i == 0 else "render, switch" if i == 1 else "ten sessions", b["sha256"])
                      for i, b in enumerate(STATIC["batches"])]),
               "The [static snapshot](docs/bench/data/static-20261028.json) contains all three batches, original options, per-job results, frame gaps and errors. The [moving snapshot](docs/bench/data/moving-20261029.json) includes all six results, window timestamps, RSS/byte counters and the complete replay trace. SHA-256 fields identify the original local files; JSON is repackaged without changing measurement values.",
               r"""Regenerate this report, the README tables and all charts from committed data:

```sh
python3 benchmark/lab/report.py
```

Run a new static comparison with the original workload settings:

```sh
scripts/dist.sh
python3 benchmark/harness/run.py build
python3 benchmark/lab/lab.py run --tools ssh,et,mosh,rsh,rsh-tcp \
  --tests render,switch,echo,flood,outage,roam,reattach,udpblock \
  --nets lan,wan,lowbw,verylow,slow64k,highlat,lossy,mobile,badmobile \
  --switches lan:highlat,lan:lowbw,lowbw:highlat,highlat:lossy,wan:verylow \
  --scenes compress,bandwidth --fps 60 --duration 8 --iterations 3 \
  --cols 100 --rows 30 --parallel 10
# Repeat render,switch only with the same options for six screen samples.
python3 benchmark/lab/lab.py run --tools ssh,et,mosh,rsh \
  --tests sessions --sessions 10 --iterations 2 --nets lan
```

These commands run the current installed lab image, not the unidentified
historical binary. For cleaner host-timing experiments reduce `--parallel`;
that changes the original conditions. Moving demos need a 158×43 terminal
to reproduce the recorded viewer size:

```sh
python3 benchmark/lab/lab.py demo --tools ssh,mosh,rsh --events '' \
  --nets moving --scenes compress,bandwidth --fps 60 --duration 60
```

## References

The [README references](README.md#references) identify the papers and
protocol specifications considered for rsh's sender/receiver design,
with links to the corresponding implementation. Those publications do
not supply the measurements in this report. The benchmark uses its own
Docker/netem/terminal collector described above.
"""]
    (ROOT / "BENCHMARK.md").write_text("\n\n".join(blocks).rstrip() + "\n")


def readme_sections():
    order = ["rsh", "mosh", "ssh", "et"]
    rs = {t: runs("render", t, "verylow", "compress") for t in order}
    rows = [
        ["Dashboard displayed FPS (median)"] + [fmt(median(rs[t], "fps")) for t in order],
        ["1% / 5% low FPS"] + [f"{fmt(low(rs[t]))} / {fmt(low(rs[t], quantile=.95))}" for t in order],
        ["Frame age, ms"] + [fmt(median(rs[t], "age_ms"), 0) for t in order],
        ["Received network, kbit/s"] + [fmt(median(rs[t], "kbit_s"), 0) for t in order],
        ["Key echo, ms"] + [fmt(median(runs("echo", t, "verylow"), "echo_ms"), 0) for t in order],
        ["Bulk output to prompt, s"] + [fmt(median(runs("flood", t, "verylow"), "flood_s"), 2) for t in order],
        ["Ten idle LAN sessions, client + server MiB"] + [fmt(sum(median(runs("sessions", t), k) for k in ["client_kb", "server_kb"]) / 1024, 1) for t in order],
    ]
    opening = """### Performance at a glance

Recorded five-tool lab snapshot, **2026-10-28, before 0.2.0**. The table
uses a 256 kbit/s link at RTT 100 ms, except the ten-session memory row.
Rendering requests 60 FPS in a 100×30 terminal. Exact historical rsh build
ID was not saved; these are observations, not release-wide guarantees.

""" + table(["Measurement", "rsh (UDP)", "mosh", "OpenSSH", "Eternal Terminal"], rows) + """

Six render runs; pooled frame-gap lows; three echo/bulk runs; two memory
runs. The [full report](BENCHMARK.md) includes rsh TCP fallback, all links,
raw measurements, aggregation rules and collection failures.
"""
    rows = [render_row(n, t, "compress") for n in ["lowbw", "verylow", "highlat", "lossy", "badmobile"] for t in ["rsh", "ssh", "mosh"]]
    # Keep both RSS sides while omitting repeat counts and median FPS here.
    compact = lambda row: row[:2] + row[3:4] + row[5:]
    headers = ["Link", "Tool", "Avg FPS", "1% low", "5% low", "Age ms", "RX kbit/s", "Client RSS MiB", "Server RSS MiB"]
    detail = """## Performance

The following static-link comparisons restore the **2026-10-28 snapshot**.
Avg FPS is the mean of six per-run frame rates; 1%/5% lows are reciprocals
of pooled p99/p95 frame gaps. Network is client RX including protocol
overhead; RSS is a sampled resident size, not a peak. See
[methodology and version scope](BENCHMARK.md#measurement-scope).

### Dashboard on slow, distant and lossy links

""" + image("fps-dashboard", "Dashboard frame rates with 1% and 5% lows for all five transports") + "\n\n" + table(headers, [compact(r) for r in rows]) + """

### Random full screen

All cells change each frame. These results expose frame-gap and bandwidth
limits that a compressible dashboard can hide.

""" + image("fps-random", "Random-screen frame rates with 1% and 5% lows") + "\n\n" + table(headers, [compact(render_row(n, t, "bandwidth")) for n in ["lowbw", "highlat", "lossy", "badmobile"] for t in ["rsh", "ssh", "mosh"]]) + """

### Network and memory

""" + image("network", "Received KiB per displayed dashboard frame") + "\n\n" + image("memory-sessions", "Client and server RSS for ten idle sessions") + """

The [full benchmark report](BENCHMARK.md) includes client/server RSS and CPU
for both workloads on every link, packet counts, all five transports,
connection/echo, bulk output, link changes and session recovery, each with
comparison tables and charts. Raw job results are committed with the report.

### Moving network

The later `7a1d9454d5b1` development build, with the transport implementation
shipped in 0.2.0, was measured at 158×43 for 60 s per scene, two shared
30 s impairment cycles. These results are separate from the static snapshot.

""" + image("moving", "Moving-link average FPS with 1% and 5% lows") + """

rsh displayed **37.72 FPS** across the dashboard trace and **30.22 FPS**
across the random-screen trace. The dashboard reached **58.0 / 57.6 FPS**
in the final five seconds of the two clear phases. The
[moving comparison](BENCHMARK.md#10-moving-network) includes SSH/mosh,
lows, scene traffic and RSS. Shared scheduling still recorded up to
32.91 ms of application skew between containers.

Performance varies by workload and loss sequence. In the static snapshot,
mosh has better random-screen tails on some slow/lossy links, while SSH
has steadier near-link frame timing. No uniform FPS or 1%/5% low advantage
is claimed. These synthetic tests do not establish fairness with competing
flows or performance on every real radio network.
"""
    path = ROOT / "README.md"
    text = path.read_text()
    for name, content in [("summary", opening), ("details", detail)]:
        start, end = f"<!-- benchmark-{name}:start -->", f"<!-- benchmark-{name}:end -->"
        before, tail = text.split(start, 1)
        _, after = tail.split(end, 1)
        text = before + start + "\n" + content.strip() + "\n" + end + after
    path.write_text(text)


if __name__ == "__main__":
    # charts.py consumes a flat results array; the snapshot also keeps batches.
    # Use its existing functions directly so the committed snapshot stays whole.
    import tempfile
    with tempfile.TemporaryDirectory() as temp:
        flat = Path(temp) / "results.json"
        flat.write_text(json.dumps({"results": RESULTS}))
        subprocess.run([sys.executable, str(ROOT / "benchmark/lab/charts.py"), str(flat), str(ROOT / "docs/bench")], check=True)
    write_report()
    readme_sections()
    print("Rebuilt BENCHMARK.md, README benchmark sections and comparison charts")
