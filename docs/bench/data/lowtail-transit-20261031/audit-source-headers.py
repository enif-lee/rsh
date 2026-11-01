"""Audit completed-body IDs/headers against actual saved source payloads."""
from pathlib import Path
import base64
import hashlib
import json
import re
import sys
import time

root = Path(sys.argv[1]).resolve()
summary_path = root / "probe-summary.json"
summary_raw = summary_path.read_bytes()
summary = json.loads(summary_raw)
checks, mismatches = [], []
for job in summary["results"]:
    raw = (root / job["source_ledger"]).read_bytes()
    frames = {frame["n"]: frame for frame in json.loads(raw)["frames"]}
    errors = []
    for observed, number, made, scene in job["complete_frames"]:
        frame = frames.get(number)
        match = None if frame is None else re.search(
            rb"FRAME (\d+) T=(\d+) (\w+)", base64.b64decode(frame["payload_b64"][:1024]))
        if match is None or (int(match[1]), int(match[2]), match[3].decode()) != (number, round(made * 1000), scene):
            errors.append(number)
    check = dict(batch=job["batch"], slot=job["slot"], variant=job["variant"],
                 completed_frames_checked=len(job["complete_frames"]), mismatch_count=len(errors),
                 source_sha256=hashlib.sha256(raw).hexdigest())
    checks.append(check)
    if errors:
        mismatches.append(dict(check=check, frame_ids=errors))
result = dict(at=time.time(), summary_sha256=hashlib.sha256(summary_raw).hexdigest(),
              meaning="Each observed complete-body frame ID, actual header timestamp and scene matches its recorded source payload header; this supplements body matching and is not GPU/whole-stream proof.",
              valid=not mismatches, checks=checks, mismatches=mismatches)
(root / "source-header-join-audit.json").write_text(json.dumps(result, indent=2) + "\n")
print(json.dumps(dict(valid=result["valid"], cases=len(checks), complete_frames=sum(c["completed_frames_checked"] for c in checks), mismatches=len(mismatches))))
assert result["valid"]
