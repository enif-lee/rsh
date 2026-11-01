#!/usr/bin/env python3
"""Replay and summarize the reason prototype without assuming PTY lineage."""
import argparse
import collections
import concurrent.futures
import csv
import json
import math
from pathlib import Path
import statistics


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


def replay_all(root):
    environment = json.loads((root / "environment.json").read_text())
    jobs, batches = [], []
    for path in sorted(root.glob("*/batch.json")):
        data = json.loads(path.read_text())
        batches.append({k:v for k,v in data.items() if k != "results"})
        jobs.extend(dict(job, batch=data["tag"]) for job in data["results"])
    with concurrent.futures.ProcessPoolExecutor(8) as pool:
        results = list(pool.map(one, [(root, job) for job in jobs]))
    statuses = {p.stem: json.loads(p.read_text()) for p in root.glob("*-host-status.json")}
    invalid = []
    schedule = environment["schedule"]
    expected = len(schedule)
    parallel = environment["parallel"]
    if parallel not in (4, 8) or len(jobs) != parallel*expected or len(batches) != expected or any(len(b["window_closed_notifications"]) != parallel or b["parallel"] != parallel for b in batches):
        invalid.append("scheduled public4/development8 block coverage mismatch")
    expected_tags = {s[0] if isinstance(s, list) else s for s in schedule}
    # The initial probe's human-readable schedule labels predate its folder
    # tags. New policy plans always use explicit tags and must match exactly.
    if schedule and isinstance(schedule[0], list) and expected_tags != {b["tag"] for b in batches}:
        invalid.append("schedule tags do not match measured blocks")
    if {p.removesuffix("-host-status") for p in statuses} != {b["tag"] for b in batches} or any(not s.get("accepted") for s in statuses.values()):
        invalid.append("missing or rejected host guard")
    for job in results:
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
    output = dict(environment=environment,
        valid=not invalid, invalid_reasons=invalid, host_statuses=statuses, batches=batches, results=results)
    if (root / "analysis-manifest.json").exists():
        output["analysis_manifest"] = json.loads((root / "analysis-manifest.json").read_text())
    (root / "probe-summary.json").write_text(json.dumps(output, indent=1)+"\n")
    print(json.dumps(dict(jobs=len(results), valid=not invalid, invalid_reasons=invalid)))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("root", type=Path)
    args = parser.parse_args()
    replay_all(args.root)


if __name__ == "__main__":
    main()
