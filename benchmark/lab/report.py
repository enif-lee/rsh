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
SNAPSHOTS = DATA / "v0.3.0"
FIGURES = ROOT / "docs/bench/v0.3.0"
STATIC = json.loads((SNAPSHOTS / "static.json").read_text())
MOVING = json.loads((SNAPSHOTS / "moving-udp.json").read_text())
MOVING_TCP = json.loads((SNAPSHOTS / "moving-tcp.json").read_text())
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


def mib(value):
    return value / 1024 if value is not None else None


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
    return [NETS[net], charts.NAME[tool], f"{len(ok)}/{len(rs)}", fmt(avg), fmt(median(rs, "fps")),
            fmt(low(ok)), fmt(low(ok, quantile=.95)), fmt(median(rs, "age_ms"), 0),
            fmt(median(rs, "kbit_s"), 0),
            fmt(mib(median(rs, "rss_client_kb")), 2),
            fmt(mib(median(rs, "rss_server_kb")), 2)]


RENDER_HEADERS = ["Link", "Tool", "Completed", "Avg FPS", "Median FPS", "1% low", "5% low",
                  "Age ms", "RX kbit/s", "Client RSS MiB", "Server RSS MiB"]


def image(name, alt):
    return f"![{alt}](docs/bench/v0.3.0/{name}.svg)"


def save_chart(name, title, note, groups, formatter, higher, **kwargs):
    text = charts.svg(title, note, groups, formatter, higher, **kwargs)
    assert text is not None
    FIGURES.mkdir(parents=True, exist_ok=True)
    (FIGURES / f"{name}.svg").write_text(text)


def moving_blocks():
    blocks = ['## 10. Moving network', '''Each scene runs for 60 seconds with two shared 30-second delay/rate/outage
cycles. There are two evaluations per cohort. Four sessions run concurrently
in 100×30 workload PTYs; the403×32 virtual viewer is continuously drained.
One cohort uses rsh UDP and another rsh TCP alongside SSH, ET and mosh.
All moving tools use the same 400×200 pyte header observer, distinct from the
100×30 workload PTYs.
Shared tools are kept separate across cohorts. Moving is descriptive and is not pooled with static results. Replay application
skew is recorded; exact global timing fidelity is not assumed.
Loss fractions here describe intervals of complete outage, not independent
per-packet random loss. No GPU presentation or radio-network claim follows.''']
    for cohort, data, figure in [('UDP', MOVING, 'moving'), ('TCP', MOVING_TCP, 'moving-tcp')]:
        rows, groups = [], []
        for run in data['evaluations']:
            for scene, label in [('compress', 'Dashboard'), ('bandwidth', 'Random screen')]:
                w = next(w for w in run['windows'] if w.get('scene') == scene)
                values = {}
                for r in run['results']:
                    if r['scene'] != scene: continue
                    tool = r['tool'];index = data['tools'].index(tool)
                    values[tool] = (r['fps'], r['low1_fps'], r['low5_fps'])
                    rows.append([cohort, run['iteration'], label, charts.NAME[tool],
                                 fmt(r['fps'], 2), fmt(r['low1_fps'], 2), fmt(r['low5_fps'], 2),
                                 fmt(r['age_ms'], 0), fmt(r['max_gap_ms'], 0),
                                 fmt(w['rx_bytes'][index] / 1024**2, 2),
                                 fmt(mib(w['rss_kb'][index][0]), 2), fmt(mib(w['rss_kb'][index][1]), 2),
                                 'seen' if w['done'][index] else 'not seen'])
                groups.append((f"{label} · evaluation {run['iteration']}", values))
        save_chart(figure, 'Moving link: two 30-second cycles',
                   f'{cohort} cohort; evaluations separate; strongest: FPS, softer: 5% low, softest: 1% low', groups,
                   lambda v: ' / '.join(fmt(x, 2) for x in v), True, pair='lows', value_w=175)
        blocks += [image(figure, f'Moving {cohort} cohort FPS and lows'),
                   table(['Cohort','Evaluation','Scene','Tool','Avg FPS','1% low','5% low','Age ms','Maxgap ms','Scene RX MiB','Client RSS MiB','Server RSS MiB','End marker'],rows)]
        skews=[max(p['applied_at'].values())-min(p['applied_at'].values()) for run in data['evaluations'] for w in run['windows'] if w.get('kind')=='render' for p in w['network_trace']]
        blocks.append(f'{cohort} cohort maximum replay application skew: {max(skews)*1000:.2f}ms.')
    blocks += ['''Scene RX includes warmup/drain outside the common FPS window, so it is
not a 60-second RX rate. Moving RSS is the larger of midpoint/end samples,
not peak RSS; static RSS uses the midpoint sample. FPS is the observed frame
arrival rate, including backlog catch-up. Age and the full frame timelines
help distinguish fresh delivery from replay. Sparse lows remain missing.
The end-marker column preserves clients that did not show the scene end within
the unchanged 120-second drain limit; that drain is outside the FPS window.''']
    return blocks


def write_report():
    blocks = ["""# Benchmark report

OpenSSH, Eternal Terminal (ET), mosh and rsh over UDP/TCP are compared on
isolated Docker server/client links. All tables and charts use the
[committed measurement snapshots](docs/bench/data/v0.3.0).

## Measurement scope

Static tests use **four parallel jobs and two evaluations per case**.
Ten network profiles and two full-screen workloads request 60 FPS in 100×30
terminals. The 550 jobs comprise render 200, link-switch 100, echo 100,
bulk-output 100, connectivity 40 and ten-session memory 10 observations.
Every attempted job remains in the data, including failures and missing values.

Moving observations are separate: two evaluations per scene/cohort, each 60 s
with two shared 30-second cycles. Four sessions run concurrently. One cohort
compares SSH/ET/mosh/rsh UDP, the other SSH/ET/mosh/rsh TCP; their shared
reference tools are not pooled across cohorts. A 403×32 virtual viewer is
drained continuously, with actual 100×30 workload PTYs checked separately.

**1% low, 5% low and average FPS are primary; CPU/RSS are secondary.**
No aggregate score or universal advantage follows from one metric.

### Environment and workloads

The shared host has an Apple M5 with four performance cores, six efficiency
cores and 32 GiB RAM. Four jobs do not establish physical core affinity. The
[environment manifest](docs/bench/data/v0.3.0/environment.json) records the
host/VM, package versions, executable identities and actual test commands.
Other local services remain running; host scheduling can vary.

Each job has its own shaped link. `compress` is a compressible full-screen
dashboard; `bandwidth` changes every cell with random content. SSH/ET frame
headers in static tests are observed in the raw-output reader; mosh/rsh headers are observed
after pyte reconstructs terminal differences. Header arrival can precede
remaining cells, and emulator scheduling contributes to these measurements.
These are terminal-observer results, not GPU presentation or a complete-body
oracle. No historical dataset supplies values to the tables below.

### Metrics and aggregation

- **Avg FPS** is the arithmetic mean of each run's displayed frames divided
  by its observation duration. **Median FPS** is the median of those run
  averages. The opening table shows both; FPS charts use the median.
- **1% / 5% low** are `1000 / p99(gap_ms)` and `1000 / p95(gap_ms)` using
  nearest-rank percentiles over available runs' inter-frame gaps pooled
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
  The collector's `torn` counter can double-count observations;
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

Full-strength color shows median FPS; 5% and 1% lows use progressively softer shades.
All five transports and all ten static profiles are included; two runs per case were attempted.
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
        rows.append([charts.NAME[t], len(rs), fmt(mib(median(rs, "client_kb")), 2),
                     fmt(mib(median(rs, "server_kb")), 2),
                     fmt(median(rs, "idle_pkts_in_s"), 2), fmt(median(rs, "idle_pkts_out_s"), 2)])
    blocks += [table(["Tool", "Runs", "All clients MiB", "Server MiB", "Idle packets in/s", "Idle packets out/s"], rows),
               "Each shell prints 3,000 lines, then becomes quiet. Packet rates are totals for the container's ten sessions. Both rsh transports have ten-session observations.",
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
        save_chart(name, f"Link changes: {label}", "after change; strongest: median FPS, softer: 5% low, softest: 1% low", groups,
                   lambda v: f"{v[0]:.1f} / {fmt(v[1])} / {fmt(v[2])}", True, pair="lows", value_w=160)
        blocks += [f"### {label}", image(name, f"{label} FPS and lows after each link change"),
                   table(["Change", "Tool", "Runs", "FPS before", "FPS after", "1% low after", "5% low after", "Age after ms", "Adapt s"], rows)]
    blocks += ["## 9. Outage, roaming, reattachment and UDP block",
               "Two runs each on RTT 100 ms. Outage is five seconds; roaming is watched for eight seconds. Reattachment restarts the client. UDP-block connection is watched for 15 seconds."]
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
               "Restart time alone does not establish session continuity: see Same shell. A missing roaming response within eight seconds is a censored observation, not a measured infinite recovery time. UDP-block mosh failures are retained, consistent with its UDP-only transport.",
    ]
    blocks += moving_blocks()
    blocks += ["## Collection failures and limitations"]
    rows = []
    for batch in STATIC["batches"]:
        for r in batch["results"]:
            if r.get("error"):
                rows.append([Path(batch["source"]).stem, r["test"], r["net"] + (" → " + r["to"] if r.get("to") else ""),
                             r.get("scene", "—"), charts.NAME[r["tool"]], r["iteration"], r["error"]])
    blocks += [table(["Source", "Test", "Link", "Scene", "Tool", "Iteration", "Error"], rows),
               f"Static render jobs: {sum(r.get('fps') is not None for r in RESULTS if r['test'] == 'render')}/{sum(r['test'] == 'render' for r in RESULTS)} measured; switch jobs: {sum(r.get('fps_after') is not None for r in RESULTS if r['test'] == 'switch')}/{sum(r['test'] == 'switch' for r in RESULTS)} measured. All tests use the same full batch. Collection failures: {len(rows)}; expected mosh UDP-block no-prompt cases: {sum(r.get('error') is not None and r['tool'] == 'mosh' and r['test'] == 'udpblock' for r in RESULTS)}. No failed job was silently converted to a numeric result.",
               "Results cover synthetic isolated paths on one shared host, not fairness with cross traffic, real radio traces, independent hardware replications or statistical confidence intervals. Static nominal loss percentages do not guarantee equal realized drops across protocols. High average FPS can coexist with worse tails; compare the average and percentile columns separately. No universal win or non-regression claim is made.",
               "## Data and reproduction",
               table(["Source run", "Jobs", "Used for", "Original SHA-256"],
                     [(Path(b["source"]).name, len(b["results"]),
                       ", ".join(sorted({r['test'] for r in b['results']})), b["sha256"])
                      for b in STATIC["batches"]]),
               """The [static snapshot](docs/bench/data/v0.3.0/static.json) contains the complete
batch, options, frame gaps, resource samples and errors. Separate moving
[UDP](docs/bench/data/v0.3.0/moving-udp.json) and
[TCP](docs/bench/data/v0.3.0/moving-tcp.json) snapshots retain both evaluations,
raw frame observations, windows, scene counters and network replay traces.
The [environment manifest](docs/bench/data/v0.3.0/environment.json) records
host/VM configuration, package and executable identities, source/harness hashes
and actual test commands. No older comparison supplies values to this report.

Regenerate this report, the README tables and every linked chart from the
committed snapshots:

```sh
python3 benchmark/lab/report.py
```

Collect the full static workload with four jobs and two evaluations:

```sh
scripts/dist.sh
python3 benchmark/harness/run.py build
python3 benchmark/lab/lab.py run --tools ssh,et,mosh,rsh,rsh-tcp \\
  --tests render,switch,echo,flood,outage,roam,reattach,udpblock,sessions \\
  --nets lan,wan,lowbw,mid5,verylow,slow64k,highlat,lossy,mobile,badmobile \\
  --switches lan:highlat,lan:lowbw,lowbw:highlat,highlat:lossy,wan:verylow \\
  --scenes compress,bandwidth --fps 60 --duration 8 --iterations 2 \\
  --sessions 10 --session-iterations 2 --cols 100 --rows 30 --parallel 4
```

For moving, run each transport cohort twice, with a 403×32 viewer producing
four 100×30 workload panes. The actual pane/source dimensions are recorded;
`--cols`/`--rows` alone do not set demo geometry.

```sh
python3 benchmark/lab/lab.py demo --tools ssh,et,mosh,rsh --events '' \\
  --nets moving --scenes compress,bandwidth --fps 60 --duration 60
python3 benchmark/lab/lab.py demo --tools ssh,et,mosh,rsh-tcp --events '' \\
  --nets moving --scenes compress,bandwidth --fps 60 --duration 60
```

## References

The [README references](README.md#references) connect sender/receiver mechanisms
to their research sources. These measurements come from the repository's
Docker/netem/terminal collectors.
"""]
    (ROOT / "BENCHMARK.md").write_text("\n\n".join(blocks).rstrip() + "\n")


def glance_table():
    order = ['rsh','mosh','ssh','et']
    rs = {t:runs('render',t,'verylow','compress') for t in order}
    def average(tool):
        values=[r['fps'] for r in rs[tool] if r.get('fps') is not None]
        return statistics.mean(values) if values else None
    def packets(tool):
        values=[(r['pkts_in_s']+r['pkts_out_s'])/r['fps']
                for r in rs[tool] if r.get('fps')]
        return statistics.median(values) if values else None
    def memory(tool):
        values=[median(runs('sessions',tool),k) for k in ['client_kb','server_kb']]
        return sum(values)/1024 if all(v is not None for v in values) else None
    specs=[('Dashboard observed FPS (average)',average,True,1),
           ('Dashboard observed FPS (median)',lambda t:median(rs[t],'fps'),True,1),
           ('1% / 5% low FPS',lambda t:(low(rs[t]),low(rs[t],quantile=.95)),True,1),
           ('Frame age, ms',lambda t:median(rs[t],'age_ms'),False,0),
           ('Received network, kbit/s',lambda t:median(rs[t],'kbit_s'),False,0),
           ('Packets per frame (in + out)',packets,False,2),
           ('Key echo, ms',lambda t:median(runs('echo',t,'verylow'),'echo_ms'),False,0),
           ('Bulk output to prompt, s',lambda t:median(runs('flood',t,'verylow'),'flood_s'),False,2),
           ('Ten idle LAN sessions, client + server MiB',memory,False,1)]
    display=lambda v,d:' / '.join(fmt(x,d) for x in v) if isinstance(v,tuple) else fmt(v,d)
    def factor(a,b,higher):
        if isinstance(a,tuple):return ' / '.join(factor(x,y,higher) for x,y in zip(a,b))
        if a is None or b is None or a<=0 or b<=0:return '—'
        return f'{(a/b if higher else b/a):.2f}×'
    rows=[]
    for label,get,higher,digits in specs:
        values={t:get(t) for t in order}
        rows.append([label,'**'+display(values['rsh'],digits)+'**',
                     '**'+factor(values['rsh'],values['ssh'],higher)+'**']+
                    [display(values[t],digits) for t in order[1:]])
    return table(['Measurement','**rsh (UDP)**','**vs SSH**','mosh','OpenSSH','Eternal Terminal'],rows)


def readme_sections():
    opening='''### Performance at a glance

A 256 kbit/s link at RTT 100 ms, requesting 60 FPS in a 100×30 terminal.
Four jobs run in parallel; each case has two evaluations. The memory row
uses ten simultaneous idle LAN sessions.

'''+glance_table()+'''

For FPS, **vs SSH** is rsh/SSH. For age, RX, packets, echo, bulk time and RSS it is
SSH/rsh: a larger factor favors rsh, while a factor below 1 favors SSH.
The 1%/5% factors are separate. Resource ratios describe usage, not speed;
ratios use unrounded statistics. Packets per frame is the median of each
run's client inbound plus outbound packet rate divided by observed FPS.
[Full results and methodology](BENCHMARK.md)
include both rsh transports, every profile, CPU/RSS and collection failures.
'''
    compact=lambda row:row[:2]+row[3:4]+row[5:]
    headers=['Link','Tool','Avg FPS','1% low','5% low','Age ms','RX kbit/s','Client RSS MiB','Server RSS MiB']
    dash=[compact(render_row(n,t,'compress')) for n in ['lowbw','verylow','highlat','lossy','badmobile'] for t in ['rsh','ssh','mosh']]
    random=[compact(render_row(n,t,'bandwidth')) for n in ['lowbw','highlat','lossy','badmobile'] for t in ['rsh','ssh','mosh']]
    detail='''## Performance

Average FPS is the mean of the two per-run rates;1%/5% lows are reciprocals
of pooled p99/p95 frame gaps. Network is client RX including protocol overhead;
RSS is a sample, not peak memory. Frame observers differ across transports.
See the [measurement scope](BENCHMARK.md#measurement-scope).

### Dashboard on slow, distant and lossy links

'''+image('fps-dashboard','Dashboard FPS and lows for all five transports')+'\n\n'+table(headers,dash)+'''

### Random full screen

Every cell changes each frame, exposing bandwidth and frame-gap limits.

'''+image('fps-random','Random-screen FPS and lows')+'\n\n'+table(headers,random)
    detail+='''

### Network and memory

'''+image('network','Received KiB per displayed dashboard frame')+'\n\n'+image('memory-sessions','Client and server RSS for ten idle sessions')+'''

The [full benchmark report](BENCHMARK.md) includes CPU/RSS, packets, connection,
echo, bulk output, link changes and recovery, with all attempts retained.

### Moving network

Each evaluation contains two common 30-second impairment cycles over 60 seconds.
Four sessions use 100×30 workload PTYs. These observations are separate from
static tests and from the other rsh-transport cohort.

'''+image('moving','Moving-link FPS and lows, each evaluation separate')
    for scene,label in [('compress','Dashboard'),('bandwidth','Random screen')]:
        values=[r['fps'] for run in MOVING['evaluations'] for r in run['results'] if r['tool']=='rsh' and r['scene']==scene]
        detail+=f"\n\n{label} rsh averages: **{' / '.join(fmt(v,2) for v in values)} FPS** across the two evaluations."
    detail+='''

High post-outage FPS can include backlog catch-up. Frame age, gaps and replay
application timestamps remain in the [moving comparison](BENCHMARK.md#10-moving-network).
Moving RSS uses the larger of midpoint/end samples; scene traffic includes
warmup/drain. Synthetic links do not establish performance on every real radio
network or fairness with competing traffic.
'''
    path=ROOT/'README.md';text=path.read_text()
    for name,content in [('summary',opening),('details',detail)]:
        start,end=f'<!-- benchmark-{name}:start -->',f'<!-- benchmark-{name}:end -->'
        before,tail=text.split(start,1);_,after=tail.split(end,1)
        text=before+start+'\n'+content.strip()+'\n'+end+after
    path.write_text(text)


if __name__ == "__main__":
    # charts.py consumes a flat results array; the snapshot also keeps batches.
    # Use its existing functions directly so the committed snapshot stays whole.
    import tempfile
    with tempfile.TemporaryDirectory() as temp:
        flat = Path(temp) / "results.json"
        flat.write_text(json.dumps({"results": RESULTS}))
        subprocess.run([sys.executable, str(ROOT / "benchmark/lab/charts.py"), str(flat), str(FIGURES)], check=True)
    write_report()
    readme_sections()
    print("Rebuilt BENCHMARK.md, README benchmark sections and comparison charts")
