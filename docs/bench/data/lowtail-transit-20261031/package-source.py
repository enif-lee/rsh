"""Original-machine extraction record, not a portable public entry point.

This was executed from the local study directory, beside the original run 20
and raw inputs. The published copy records that procedure; its sibling input
paths do not exist in the public data directory and DEST is machine-specific.
Use the separate portable plot.py/verify-metrics.py for the compact archive.
Execute extraction only outside live measurement windows.
"""
from pathlib import Path
import hashlib
import csv
import json
import shutil

HERE = Path(__file__).resolve().parent
RUN = HERE / "20-transit-core-screen"
DEST = Path("/Users/ed/personal/rsh-parity/docs/bench/data/lowtail-transit-20261031")
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()


def main():
    summary = json.loads((RUN / "probe-summary.json").read_text())
    assert summary["valid"] and summary["data_integrity_valid"]
    assert not summary["invalid_reasons"] and len(summary["results"]) == 48
    assert len(summary["host_statuses"]) == 12
    assert all(s["accepted"] for s in summary["host_statuses"].values())
    DEST.mkdir(parents=True, exist_ok=True)
    results = []
    originals = {}
    for row in summary["results"]:
        assert row["data_integrity"]["valid"]
        assert row["source"]["dropped"] == 0
        assert not row["source"]["missing_oracle_frame_ids"]
        result = {k: row[k] for k in (
            "batch", "net", "scene", "iteration", "variant", "slot", "tool",
            "duration", "scene_fps", "cols", "rows", "binary_sha256",
            "window_start", "window_end", "kbit_s", "rss_client_kb",
            "rss_server_kb", "cpu_client", "cpu_server", "complete_body",
            "complete_frames", "data_integrity", "source", "delivery_outcome")}
        results.append(result)
        for field in ("capture", "source_ledger"):
            filename = row[field]
            originals[filename] = sha(RUN / filename)
    snapshot = dict(
        dataset="20-transit-core-screen", date="2026-10-31",
        parallel=4, iterations=2, requested_fps=60,
        valid=True, data_integrity_valid=True, performance_accepted=False,
        meaning="Development TCP transit core screen, not a public full-suite rerun. Metrics use actual-source body characters; not GPU presentation. No pooling across repetitions.",
        results=results,
        host_statuses=summary["host_statuses"],
        original_summary_sha256=sha(RUN / "probe-summary.json"),
        original_raw_files_sha256=originals,
        raw_input_availability="Raw captures/source bodies remain local; hashes are committed. This compact archive permits independent metric recalculation, not independent body-oracle replay.",
    )
    (DEST / "core-results.json").write_text(json.dumps(snapshot, separators=(",", ":")) + "\n")
    for name in ("acceptance-per-rep.csv", "acceptance-per-rep.json", "runtime-platform.json", "run-status.json", "source-header-join-audit.json",
                 "prepared-inputs.json", "artifact-identities.json"):
        shutil.copyfile(RUN / name, DEST / name)
    for name in ("transit-v2env-release-pair.json", "tcp-transit.patch",
                 "tcp-transit-source-manifest.json", "main-source-manifest.json",
                 "build-transit-v2env.log"):
        shutil.copyfile(HERE / name, DEST / name)
    (DEST / "original-harness-manifest.json").write_bytes((RUN / "harness/manifest.json").read_bytes())
    (DEST / "package-source.py").write_bytes(Path(__file__).read_bytes())
    fields = "net scene iteration variant slot frames fps low1 low5 age age95 kbit rssc rsss cpuc cpus".split()
    with (DEST / "core-metrics.csv").open("w") as handle:
        writer = csv.DictWriter(handle, fieldnames=fields)
        writer.writeheader()
        for row in results:
            body = row["complete_body"]
            value = {k: row[k] for k in ("net", "scene", "iteration", "variant", "slot")}
            value.update(frames=body["frames"], fps=body["fps"], low1=body["low1_fps"], low5=body["low5_fps"],
                         age=body["age_ms"], age95=body["age_p95_ms"], kbit=row["kbit_s"],
                         rssc=row["rss_client_kb"], rsss=row["rss_server_kb"], cpuc=row["cpu_client"], cpus=row["cpu_server"])
            writer.writerow(value)
    print("packaged", len(results), "cases", sum(len(r["complete_frames"]) for r in results), "complete-body frames")


if __name__ == "__main__":
    main()
