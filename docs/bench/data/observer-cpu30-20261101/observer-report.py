#!/usr/bin/env python3
"""Replay and summarize the reason prototype without assuming PTY lineage."""
import argparse
import base64
import collections
import concurrent.futures
import csv
import json
import math
from pathlib import Path
import statistics
import struct


def recorded_input_integrity(job,ledger,frames,capture):
    """Check recorded input structure/clocks independently of delivered frames."""
    errors=[]
    lo,hi=job["window_start"],job["window_end"]
    if not math.isfinite(lo) or not math.isfinite(hi) or hi<=lo:errors.append("invalid receive window")
    if ledger.get("dropped",1)!=0:errors.append("source ledger lost records")
    previous=None;lower=[];upper=[];produced=0
    for frame in frames:
        n,draw,begin,end,wall=(frame[k] for k in ("n","draw_begin_ns","write_begin_ns","write_end_ns","wall_ns"))
        if not all(isinstance(x,int) for x in (n,draw,begin,end,wall)):
            errors.append("invalid source timestamp types");continue
        if not draw<=begin<=end:errors.append("invalid source monotonic timestamps")
        if previous and (n<=previous[0] or begin<previous[1] or wall<previous[2]):errors.append("unordered source IDs/clocks")
        previous=(n,begin,wall);lower.append(wall-end);upper.append(wall-begin)
        if lo<=wall/1e9<=hi:produced+=1
        try:base64.b64decode(frame["payload_b64"],validate=True)
        except (ValueError,TypeError):errors.append("invalid source payload encoding")
    if not produced:errors.append("no recorded source writes in receive window")
    # Wall sampling occurs between write_begin/end. A common offset must fit
    # every bracket before source/observer clock joins are treated as valid.
    if lower and max(lower)>min(upper):errors.append("source realtime/monotonic brackets have no stable offset")
    count=0;last=None
    with capture.open("rb") as stream:
        while True:
            header=stream.read(12)
            if not header:break
            if len(header)!=12:errors.append("truncated capture header");break
            at,size=struct.unpack("!dI",header);data=stream.read(size)
            if len(data)!=size:errors.append("truncated capture body");break
            if not math.isfinite(at) or (last is not None and at<last):errors.append("unordered capture realtime")
            last=at;count+=1
    if not count:errors.append("capture contains no recorded reads")
    return dict(valid=not errors,reasons=sorted(set(errors)),source_writes_in_window=produced,capture_reads=count,
                clock_scope="source monotonic order and common realtime-offset brackets; capture realtime ordering; no GPU clock or presentation proof")


def delivery_outcome(job):
    body=job["complete_body"];zero=body["frames"]==0
    return dict(status="zero_complete_body" if zero else "complete_bodies_observed",target_failure=zero,
                complete_frames=body["frames"],fps=body["fps"],low1_fps=body["low1_fps"],low5_fps=body["low5_fps"],
                note="Zero delivery is retained as a failed observation; missing low statistics remain null. This is not a final no-regression or SSH-parity decision.")


def stats(values, scale=1):
    xs = sorted(x / scale for x in values)
    if not xs:
        return dict(count=0)
    pick = lambda q: xs[max(0, math.ceil(len(xs)*q)-1)]
    return dict(count=len(xs), p50=statistics.median(xs), p95=pick(.95), p99=pick(.99), max=xs[-1])


def trace(root, job, side):
    folder = root / job["batch"] / f"{job['slot']}-{side}"
    metas = list(folder.glob("meta-*.json"))
    if not metas:
        return dict(valid=False, reasons=["no trace metadata"]), [], []
    if len(metas) != 1:
        return dict(valid=False, reasons=["ambiguous process trace"]), [], []
    meta = json.loads(metas[0].read_text())
    path = folder / f"trace-{meta['pid']}.csv"
    with path.open() as f:
        records = [dict(ns=int(row["ns"]), kind=row["kind"], data=[int(row[k]) for k in "abcdefgh"])
                   for row in csv.DictReader(f)]
    epoch = meta["anchor_realtime_ns"] / 1e9
    drift_ns = meta["dump_realtime_ns"] - meta["anchor_realtime_ns"] - meta["dump_monotonic_ns"]
    reasons = []
    if len(records) != meta["events"]:
        reasons.append("CSV row count mismatch")
    if meta["dropped"]:
        reasons.append("recorder overflow")
    if abs(drift_ns) > 2_000_000:
        reasons.append("clock projection drift >2ms")
    window = [r for r in records if job["window_start"] <= epoch + r["ns"]/1e9 <= job["window_end"]]
    by = collections.defaultdict(list)
    for r in window:
        by[r["kind"]].append(r)
    output = dict(valid=not reasons, reasons=reasons, metadata=meta,
        projection_drift_ms=drift_ns/1e6,
        kinds={k: len(v) for k, v in by.items()},
        duration_ms={k: stats([r["data"][0] for r in by[k]], 1e6)
            for k in ("poll_end", "rx_accept", "write_end", "pty_apply", "compress", "tx_batch_end")})
    sleeps = []
    if side == "cli":
        gate, begin = None, None
        for r in records:
            if r["kind"] == "gate":
                gate = r
            elif r["kind"] == "poll_begin":
                begin = (r, gate)
            elif r["kind"] == "poll_end" and begin:
                start, previous = begin
                begin = None
                if previous and start["data"][3] > 0 and previous["data"][2] > 0:
                    mask = previous["data"][3]
                    lo = epoch + start["ns"]/1e9
                    hi = epoch + r["ns"]/1e9
                    lo, hi = max(lo, job["window_start"]), min(hi, job["window_end"])
                    if hi >= lo:
                        sleeps.append(dict(begin=lo, end=hi, mask=mask,
                            reason={0:"ready",1:"token",2:"catch",3:"both"}[mask & 3],
                            request_ms=start["data"][0], decision_staleness_ms=(start["ns"]-start["data"][1])/1e6,
                            due_epoch=epoch+start["data"][2]/1e9 if start["data"][2] >= 0 else None))
        output["pending_poll"] = {reason: dict(count=sum(s["reason"] == reason for s in sleeps),
            total_ms=sum((s["end"]-s["begin"])*1000 for s in sleeps if s["reason"] == reason))
            for reason in ("ready", "token", "catch", "both")}
        output["enqueue_origins"] = {str(origin): dict(
            events=sum(r["data"][0] == origin for r in by["enqueue"]),
            bytes=sum(r["data"][1] for r in by["enqueue"] if r["data"][0] == origin),
            cadence_changes=sum(r["data"][3] != r["data"][4] for r in by["enqueue"] if r["data"][0] == origin))
            for origin in range(4)}
        output["refill_ms"] = stats([r["data"][5] for r in by["gate"]], 1e6)
        output["stream_flags"] = dict(collections.Counter(r["data"][4] for r in by["rx_stream"]))
        output["udp_timestamp_samples"] = dict(collections.Counter(
            f"cmsg={r['data'][1]},projection={r['data'][2]}" for r in by["wire_udp"]))
        output["rx_accept_bytes"] = sum(r["data"][3] for r in by["rx_accept"])
    else:
        output["mode_snapshot_counts"] = dict(collections.Counter(r["data"][5] for r in by["pump"]))
        output["pty_bytes"] = sum(r["data"][0] for r in by["pty_read"])
    return output, records, sleeps


def one(item):
    import replay
    from ledger_oracle import LedgerSource
    root, job = item
    if job.get("fps") is None:
        return job
    ledger = json.loads((root / job["source_ledger"]).read_text())
    frames = ledger.pop("frames")
    job["data_integrity"]=recorded_input_integrity(job,ledger,frames,root/job["capture"])
    if not job["data_integrity"]["valid"]:return job
    source = LedgerSource(frames, job["cols"], job["rows"])
    job.update(replay.observe(root / job["capture"], job, source, include_frames=True))
    job["oracle"] = "actual source-ledger body characters"
    by_n = {f["n"]: f for f in frames}
    job["source"] = dict(**ledger, generated=len(frames),
        draw_ms=stats([f["write_begin_ns"]-f["draw_begin_ns"] for f in frames], 1e6),
        write_ms=stats([f["write_end_ns"]-f["write_begin_ns"] for f in frames], 1e6),
        release_lateness_ms=stats([f["wall_ns"]/1e6-f["intended_epoch"]*1000 for f in frames]))
    joined = []
    for observed_at, n, _, _ in job["complete_frames"]:
        if n in by_n:
            f = by_n[n]
            # Source wall sample brackets neither GPU presentation nor network
            # transport offsets. This is a frame-id join with a projected
            # source write end, using source monotonic duration only.
            source_end = f["wall_ns"]/1e9 + (f["write_end_ns"]-f["write_begin_ns"])/1e9
            joined.append(dict(n=n, complete_read_epoch=observed_at, source_write_end_epoch=source_end,
                after_source_write_ms=(observed_at-source_end)*1000,
                source_write_end_epoch_interval=[f["wall_ns"]/1e9, source_end],
                after_source_write_ms_interval=[(observed_at-source_end)*1000, (observed_at-f["wall_ns"]/1e9)*1000],
                source_release_lateness_ms=f["wall_ns"]/1e6-f["intended_epoch"]*1000))
    job["source_joined_complete_frames"] = joined
    job["delivery_outcome"]=delivery_outcome(job)
    job["source"]["joined_complete_count"] = len(joined)
    job["source"]["missing_oracle_frame_ids"] = sorted(source.missing)
    job["source"]["projection_note"] = "Wall sampled between write_begin_ns and write_end_ns; projected end is upper endpoint, not exact end. Intervals assume stable guest realtime/monotonic offset. Neither source return nor complete read is GPU presentation."
    if job["reason_trace"] and job["tool"].startswith("rsh"):
        cli, _, sleeps = trace(root, job, "cli")
        srv, _, _ = trace(root, job, "srv")
        job["endpoint_trace"] = dict(cli=cli, srv=srv)
        gaps = sorted(zip(job["complete_frames"], job["complete_frames"][1:]), key=lambda pair: pair[1][0]-pair[0][0], reverse=True)[:5]
        job["late_gap_examples"] = [dict(previous_n=a[1], n=b[1], gap_ms=(b[0]-a[0])*1000,
            source_draws_during_gap=sum(a[0] <= f["wall_ns"]/1e9 < b[0] for f in frames),
            source_draws_between_frame_ids=sum(a[1] < f["n"] <= b[1] for f in frames),
            current_source_join=next((f for f in joined if f["n"] == b[1]), None),
            pending_poll_ms={reason: sum(max(0, min(b[0], s["end"])-max(a[0], s["begin"]))*1000
                for s in sleeps if s["reason"] == reason) for reason in ("ready", "token", "catch", "both")}) for a,b in gaps]
    # Keep frame timestamps/ids for independent audit; no claim of presentation.
    return job


def coverage_errors(environment, batches, results):
    """Require the planned repetitions and exact four-job blocks, not just totals."""
    problems = []
    if environment.get("parallel") != 4 or environment.get("iterations") != 2:
        problems.append("study policy requires four workers and two evaluations")
    fields = ("batch", "comparison_protocol", "net", "scene", "tool", "variant", "observer_mode", "topology", "binary_sha256", "artifact_variant", "iteration", "slot")
    key = lambda row: tuple(row.get(field) for field in fields)
    planned = environment.get("expected_jobs", [])
    if not planned:
        problems.append("missing expected job identities")
        return problems
    expected = collections.Counter(key(row) for row in planned)
    actual = collections.Counter(key(row) for row in results)
    if expected != actual:
        problems.append("planned job identities or iteration coverage mismatch")
    for counter, label in ((expected, "plan"), (actual, "results")):
        repeats = collections.defaultdict(list)
        for identity, count in counter.items():
            repeats[identity[1:-2]].extend([identity[-2]] * count)
        if any(sorted(values) != [0, 1] for values in repeats.values()):
            problems.append(label + " must contain exactly iterations 0 and 1 for every case")
    for batch in batches:
        found = [row for row in results if row.get("batch") == batch.get("tag")]
        if len(found) != 4 or {row.get("slot") for row in found} != {0, 1, 2, 3}:
            problems.append("block requires exactly four distinct slots: " + str(batch.get("tag")))
        if len({(r.get("comparison_protocol"), r.get("net"), r.get("scene"), r.get("iteration"), r.get("observer_mode")) for r in found}) != 1:
            problems.append("block mixes comparison contexts: " + str(batch.get("tag")))
        expected_roles={("ssh","ssh"),("rsh-tcp","main-A"),("rsh-tcp","main-B"),("rsh-tcp","main-C")}
        if collections.Counter((r.get("tool"),r.get("variant")) for r in found)!=collections.Counter(expected_roles):
            problems.append("block requires SSH, main-A, main-B and main-C roles: "+str(batch.get("tag")))
        if any(r.get("comparison_protocol")!="tcp" or r.get("topology")!="endpoint-egress" or r.get("observer_mode") not in ("legacy","raw-only") for r in found):
            problems.append("block changes TCP/egress/observer development context: "+str(batch.get("tag")))
    artifacts=environment.get("binaries",{})
    if set(artifacts)!={"main-A","main-B","main-C"} or any(not row.get("sha256") for row in artifacts.values()) or len({row.get("sha256") for row in artifacts.values()})!=1:
        problems.append("fixed identical main artifacts required")
    if len(results)!=96 or len(batches)!=24:
        problems.append("observer study requires exact96 jobs/24 blocks")
    if {(r.get("net"),r.get("scene"),r.get("observer_mode")) for r in results}!={(n,s,m) for n in ("lan","wan","highlat") for s in ("compress","bandwidth") for m in ("legacy","raw-only")}:
        problems.append("observer context coverage mismatch")
    for row in results:
        role="main-A" if row.get("variant")=="ssh" else row.get("variant")
        if row.get("artifact_variant")!=role or row.get("binary_sha256")!=artifacts.get(role,{}).get("sha256"):
            problems.append("role artifact identity mismatch")
    return sorted(set(problems))


def topology_errors(root, batches, results):
    problems = []
    for batch in batches:
        tag = batch["tag"]
        path = root / tag / "network-setup.json"
        if not path.is_file():
            problems.append("missing strict topology setup: " + tag)
            continue
        configured = json.loads(path.read_text())
        if len(configured) != 4 or {row.get("slot") for row in configured} != {0, 1, 2, 3}:
            problems.append("topology setup requires four distinct slots: " + tag)
            continue
        jobs = {row["slot"]: row for row in results if row.get("batch") == tag}
        for item in configured:
            job = jobs.get(item["slot"], {})
            if not item.get("configured_before_agent_launch") or not job.get("network_prepared"):
                problems.append("unprepared topology participant: " + tag + "/" + str(item["slot"]))
            if (item.get("topology"), item.get("net")) != (job.get("topology"), job.get("net")) or len(item.get("endpoints", [])) != 2:
                problems.append("topology setup identity mismatch: " + tag + "/" + str(item["slot"]))
            if any(item.get(field)!=job.get(field) for field in ("variant","iteration","binary_sha256")):
                problems.append("topology role identity mismatch: "+tag+"/"+str(item["slot"]))
            tcp = job.get("tool") in ("rsh-tcp", "rsh-b-tcp")
            endpoints = {row.get("container"): row for row in item.get("endpoints", [])}
            expected = {f"rl-{item['slot']}-srv": tcp, f"rl-{item['slot']}-cli": False}
            if item.get("tool") != job.get("tool") or item.get("udp_dropped") is not tcp or set(endpoints) != set(expected):
                problems.append("transport firewall evidence mismatch: " + tag + "/" + str(item["slot"]))
            for container, dropped in expected.items():
                row = endpoints.get(container, {})
                rules = row.get("iptables")
                if not isinstance(rules, list) or row.get("udp_dropped") is not dropped or ("-A INPUT -p udp -j DROP" in rules) != dropped:
                    problems.append("transport firewall evidence mismatch: " + tag + "/" + str(item["slot"]))
    return sorted(set(problems))


def deployment_errors(root,batches,results):
    problems=[]
    for batch in batches:
        tag=batch["tag"];jobs={r["slot"]:r for r in results if r.get("batch")==tag}
        expected={f"rl-{slot}-{side}":(job,side) for slot,job in jobs.items() for side in ("srv","cli")}
        states={}
        for stage in ("before","after"):
            path=root/tag/("identities-"+stage+".json")
            if not path.is_file():problems.append("missing deployment identity: "+tag+"/"+stage);continue
            rows=json.loads(path.read_text());seen={r.get("container"):r for r in rows}
            states[stage]=seen
            if len(rows)!=8 or set(seen)!=set(expected):problems.append("deployment requires eight distinct endpoints: "+tag+"/"+stage)
            for container,(job,side) in expected.items():
                row=seen.get(container,{})
                if any(row.get(k)!=job.get(k) for k in ("slot","variant","iteration","tool")) or row.get("side")!=side or row.get("sha256")!=job.get("binary_sha256") or row.get("expected_sha256")!=job.get("binary_sha256") or row.get("path")!="/usr/local/bin/rsh" or row.get("artifact_variant")!=job.get("artifact_variant"):
                    problems.append("per-slot artifact identity mismatch: "+tag+"/"+container+"/"+stage)
                if side=="srv":
                    live=row.get("live_server",{});args=live.get("args",[])
                    bound="--bind" in args and len(args)>args.index("--bind")+1 and args[args.index("--bind")+1]=="0.0.0.0:2222"
                    if live.get("state") in (None,"Z") or not isinstance(live.get("pid"),int) or live.get("pid",0)<=0 or not live.get("starttime") or not live.get("listening_2222") or len(args)<2 or args[1]!="serve" or not bound or live.get("exe_sha256")!=job.get("binary_sha256") or live.get("inspection_user")!="dev":
                        problems.append("live server identity mismatch: "+tag+"/"+container+"/"+stage)
        for container,(job,side) in expected.items():
            if side=="srv" and "before" in states and "after" in states:
                a=states["before"].get(container,{}).get("live_server",{});b=states["after"].get(container,{}).get("live_server",{})
                if any(a.get(k)!=b.get(k) for k in ("pid","starttime","exe_sha256")):
                    problems.append("server changed during measured phase: "+tag+"/"+container)
        path=root/(tag+"-deployment.json")
        if not path.is_file():problems.append("missing stopped-before-copy proof: "+tag)
        else:
            proof=json.loads(path.read_text());absent=proof.get("no_agents",[])
            if proof.get("old_rsh_stopped_before_copy") is not True or len(absent)!=8 or {r.get("container") for r in absent}!=set(expected) or any(r.get("active_agents")!=0 for r in absent):
                problems.append("invalid stopped-before-copy proof: "+tag)
    return sorted(set(problems))


def platform_errors(root,results):
    path=root/'runtime-platform.json'
    if not path.is_file():return ['missing premeasurement runtime provenance']
    data=json.loads(path.read_text());rows=data.get('endpoints',[])
    expected={f'rl-{slot}-{side}' for slot in range(4) for side in ('srv','cli')}
    errors=[]
    if len(rows)!=8 or {r.get('container') for r in rows}!=expected:errors.append('runtime provenance requires eight distinct endpoints')
    if not data.get('image_id','').startswith('sha256:') or not data.get('engine',{}).get('Version'):errors.append('missing runtime image/engine identity')
    for row in rows:
        software=row.get('software',{});tools=software.get('tools',{})
        if row.get('image_id')!=data.get('image_id') or not all(software.get(k) for k in ('python','kernel','machine','os_release')) or not all(tools.get(k) for k in ('openssh','iproute','tc')):
            errors.append('runtime image/software identity mismatch')
    started,finished=data.get('started_epoch'),data.get('finished_epoch')
    windows=[r.get('window_start') for r in results]
    finite=lambda t:type(t) in (int,float) and math.isfinite(t)
    valid_times=finite(started) and finite(finished) and bool(windows) and all(finite(t) for t in windows)
    if not valid_times or not started<=finished<min(windows):errors.append('invalid or overlapping runtime provenance timestamps')
    return sorted(set(errors))


ROLE_ORDER=("ssh","main-A","main-B","main-C")
FPS_METRICS=("fps","low1_fps","low5_fps")
RESOURCE_METRICS=("kbit_s","rss_client_kb","rss_server_kb","cpu_client","cpu_server")


def observer_transition_errors(job):
    start,end,at,lag=(job.get(k) for k in ("window_start","window_end","observer_transition_epoch","observer_transition_lag_ms"))
    finite=lambda value:type(value) in (int,float) and math.isfinite(value)
    if not all(map(finite,(start,end,at,lag))) or not start<=at<=end or abs(lag-(at-start)*1000)>1e-6:
        return ["invalid observer transition evidence: "+str(job.get("batch"))+"/"+str(job.get("slot"))]
    return []


def observer_pairs(results):
    groups=collections.defaultdict(list)
    for job in results:groups[(job['net'],job['scene'],job['iteration'],job['variant'])].append(job)
    output=[]
    for (net,scene,iteration,role),jobs in sorted(groups.items()):
        points=[]
        for job in sorted(jobs,key=lambda j:j['observer_mode']):
            body=job.get('complete_body',{})
            points.append(dict(batch=job['batch'],slot=job['slot'],observer_mode=job['observer_mode'],
                frames=body.get('frames'),**{k:body.get(k) for k in FPS_METRICS+('age_ms','age_p95_ms','max_gap_ms')},
                **{k:job.get(k) for k in RESOURCE_METRICS+('observer_transition_epoch','observer_transition_lag_ms')}))
        by={row['observer_mode']:row for row in points};exact=len(points)==2 and set(by)=={'legacy','raw-only'}
        deltas={k:by['raw-only'][k]-by['legacy'][k] if exact and by['raw-only'][k] is not None and by['legacy'][k] is not None else None for k in FPS_METRICS+RESOURCE_METRICS}
        output.append(dict(net=net,scene=scene,iteration=iteration,variant=role,paired_identity_valid=exact,
            points=points,raw_minus_legacy=deltas,interpretation='Whole-block method effect with fixed product only. No pooling, automatic tolerance, product acceptance or startup-seeding exclusion.'))
    return output


def save_observer_tables(root,pairs):
    (root/'observer-per-rep.json').write_text(json.dumps(pairs,indent=2)+'\n')
    fields=['net','scene','iteration','variant','observer_mode','batch','slot','frames',*FPS_METRICS,*RESOURCE_METRICS,'age_ms','age_p95_ms','max_gap_ms','observer_transition_epoch','observer_transition_lag_ms']
    with (root/'observer-per-rep.csv').open('w',newline='') as stream:
        writer=csv.DictWriter(stream,fieldnames=fields);writer.writeheader()
        for group in pairs:
            for point in group['points']:writer.writerow(dict(**{k:group[k] for k in ('net','scene','iteration','variant')},**point))
    value=lambda row:' / '.join('—' if row.get(k) is None else str(row[k]) for k in FPS_METRICS)
    lines=['# Per-repetition observer method evidence','','Offline actual-body avg/1%/5% only; no pooling or automatic product acceptance. Online raw-mode header rates are placeholders. CPU30 is admission only; runtime load is contextual. Startup tool-own firstheader observation remains.','','| Net | Scene | Rep | Role | Legacy avg / 1% / 5% | Raw-mode avg / 1% / 5% |','|---|---|---:|---|---|---|']
    for group in pairs:
        by={p['observer_mode']:p for p in group['points']}
        lines.append('| '+ ' | '.join([group['net'],group['scene'],str(group['iteration']),group['variant'],value(by.get('legacy',{})),value(by.get('raw-only',{}))])+' |')
    (root/'observer-per-rep.md').write_text('\n'.join(lines)+'\n')


def replay_all(root):
    environment = json.loads((root / "environment.json").read_text())
    jobs, batches = [], []
    for path in sorted(root.glob("*/batch.json")):
        data = json.loads(path.read_text())
        batches.append({k:v for k,v in data.items() if k != "results"})
        jobs.extend(dict(job, batch=data["tag"]) for job in data["results"])
    with concurrent.futures.ProcessPoolExecutor(4) as pool:
        results = list(pool.map(one, [(root, job) for job in jobs]))
    statuses = {p.stem: json.loads(p.read_text()) for p in root.glob("*-host-status.json")}
    invalid = []
    schedule = environment["schedule"]
    expected = len(schedule)
    parallel = environment["parallel"]
    if parallel != 4 or len(jobs) != parallel*expected or len(batches) != expected or any(len(b["window_closed_notifications"]) != parallel or b["parallel"] != parallel for b in batches):
        invalid.append("scheduled four-worker block coverage mismatch")
    expected_tags = {s[0] if isinstance(s, list) else s for s in schedule}
    # The initial probe's human-readable schedule labels predate its folder
    # tags. New policy plans always use explicit tags and must match exactly.
    if schedule and isinstance(schedule[0], list) and expected_tags != {b["tag"] for b in batches}:
        invalid.append("schedule tags do not match measured blocks")
    if {p.removesuffix("-host-status") for p in statuses} != {b["tag"] for b in batches} or any(not s.get("accepted") for s in statuses.values()):
        invalid.append("missing or rejected host guard")
    invalid.extend(coverage_errors(environment, batches, results))
    invalid.extend(topology_errors(root, batches, results))
    invalid.extend(deployment_errors(root, batches, results))
    invalid.extend(platform_errors(root, results))
    if environment.get("observer_experiment") is not True:
        invalid.append("missing fixed-product observer experiment context")
    for job in results:
        invalid.extend(observer_transition_errors(job))
        if job.get("reason_trace") is not False:invalid.append("observer experiment requires trace-off product")
        if "data_integrity" in job and not job["data_integrity"]["valid"]:
            invalid.append(f"invalid recorded inputs {job['batch']}/{job['slot']}: {job['data_integrity']['reasons']}")
        if job.get("error"):
            invalid.append(f"job failed {job['batch']}/{job['slot']}")
        if job["reason_trace"] and job["tool"].startswith("rsh") and not job.get("trace_flush_confirmed"):
            invalid.append(f"client flush not confirmed {job['batch']}/{job['slot']}")
        source = job.get("source", {})
        frame_job = job["test"] in ("render", "switch")
        if frame_job and source.get("dropped", 1):
            invalid.append(f"source ledger lost records {job['batch']}/{job['slot']}")
        if frame_job and source.get("missing_oracle_frame_ids"):
            invalid.append(f"source oracle missing IDs {job['batch']}/{job['slot']}")
        block = next(b for b in batches if b["tag"] == job["batch"])
        if frame_job and source.get("dump_begin_epoch", 0) < block["all_windows_closed_epoch"]:
            invalid.append(f"source dump overlapped window {job['batch']}/{job['slot']}")
        for side, data in job.get("endpoint_trace", {}).items():
            if not data["valid"]:
                invalid.append(f"invalid {side} trace {job['batch']}/{job['slot']}: {data['reasons']}")
            meta = data.get("metadata", {})
            # CSV is serialized before dump_realtime_ns. A completion after
            # the barrier alone is NOT proof that serialization began after it;
            # driver control flow and SIGUSR1 request fencing establish that.
            if meta.get("dump_realtime_ns", 0)/1e9 < block["all_windows_closed_epoch"]:
                invalid.append(f"trace completion before barrier {job['batch']}/{job['slot']}/{side}")
    pairs=observer_pairs(results)
    if len(pairs)!=48 or any(not row['paired_identity_valid'] for row in pairs):invalid.append("observer paired evaluation coverage mismatch")
    output = dict(environment=environment,valid=not invalid,data_integrity_valid=not invalid,
        invalid_reasons=invalid,host_statuses=statuses,batches=batches,results=results,
        observer_comparison=dict(evaluated=False,scope="fixed product whole-block runtime observer method effect only",
            pairs=pairs,tables=['observer-per-rep.json','observer-per-rep.csv','observer-per-rep.md'],
            observed_delivery_failures=[dict(batch=j['batch'],slot=j['slot'],variant=j.get('variant'),observer_mode=j['observer_mode'],iteration=j['iteration']) for j in results if j.get('delivery_outcome',{}).get('target_failure')],
            note="No product change/parity/automatic pass. Preserve both modes' tool-own legacy anchor, startup seeding remains; raw-mode online header stats are placeholders. Valid zero delivery and missing lows stay visible."))
    if (root / "analysis-manifest.json").exists():output["analysis_manifest"] = json.loads((root / "analysis-manifest.json").read_text())
    save_observer_tables(root,pairs)
    (root / "probe-summary.json").write_text(json.dumps(output, indent=1)+"\n")
    print(json.dumps(dict(jobs=len(results), valid=not invalid, invalid_reasons=invalid)))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("root", type=Path)
    args = parser.parse_args()
    replay_all(args.root)


if __name__ == "__main__":
    main()
