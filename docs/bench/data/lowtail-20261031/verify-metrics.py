"""Recalculate public compact-archive metrics, without claiming raw-body replay."""
from pathlib import Path
import csv
import json
import math
import statistics

HERE = Path(__file__).resolve().parent
data = json.loads((HERE / "core-results.json").read_text())
csv_rows = list(csv.DictReader((HERE / "core-metrics.csv").open()))
key = lambda r: (r["net"], r["scene"], int(r["iteration"]), r["variant"])
csv_by_key = {key(r): r for r in csv_rows}
assert len(csv_rows) == len(csv_by_key) == len(data["results"]) == 48


def percentile(values, quantile):
    return values[max(0, math.ceil(len(values) * quantile) - 1)]


for result in data["results"]:
    frames = result["complete_frames"]
    times = [f[0] for f in frames]
    gaps = sorted((b - a) * 1000 for a, b in zip(times, times[1:]))
    ages = sorted((f[0] - f[2]) * 1000 for f in frames)
    duration = result["window_end"] - result["window_start"]
    calculated = dict(frames=len(frames), fps=round(len(frames) / duration, 1),
                      low1_fps=round(1000 / max(percentile(gaps, .99), .001), 1) if len(gaps) >= 10 else None,
                      low5_fps=round(1000 / max(percentile(gaps, .95), .001), 1) if len(gaps) >= 10 else None,
                      age_ms=round(statistics.median(ages)) if ages else None,
                      age_p95_ms=round(percentile(ages, .95)) if len(ages) >= 20 else None)
    assert all(result["complete_body"][name] == value for name, value in calculated.items()), key(result)
    assert all(result["window_start"] <= t <= result["window_end"] for t in times)
    assert all(b[1] > a[1] for a, b in zip(frames, frames[1:]))
    csv_row = csv_by_key[key(result)]
    for name, col in (("frames", "frames"), ("fps", "fps"), ("low1_fps", "low1"),
                      ("low5_fps", "low5"), ("age_ms", "age"), ("age_p95_ms", "age95")):
        assert float(csv_row[col]) == calculated[name], (key(result), name)
    for name, col in (("kbit_s", "kbit"), ("rss_client_kb", "rssc"),
                      ("rss_server_kb", "rsss"), ("cpu_client", "cpuc"), ("cpu_server", "cpus")):
        assert float(csv_row[col]) == result[name], (key(result), name)

print(json.dumps(dict(cases=48, complete_frames=sum(len(r["complete_frames"]) for r in data["results"]),
                      metric_and_csv_match=True, raw_body_replay_performed=False)))
