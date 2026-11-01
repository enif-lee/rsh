"""Recalculate compact per-run metrics; this is not raw-body replay."""
from pathlib import Path
import csv,gzip,hashlib,json,math,statistics

HERE=Path(__file__).resolve().parent
data=json.loads((HERE/'core-results.json').read_text())
rows=list(csv.DictReader((HERE/'core-metrics.csv').open()))
key=lambda r:(r['net'],r['scene'],int(r['iteration']),r['variant'])
by_key={key(r):r for r in rows}
assert len(rows)==len(by_key)==len(data['results'])==48
assert {key(r) for r in data['results']}==set(by_key)
percentile=lambda xs,q:xs[math.ceil(len(xs)*q)-1]
missing=0
for result in data['results']:
    frames=result['complete_frames'];times=[f[0] for f in frames]
    gaps=sorted((b-a)*1000 for a,b in zip(times,times[1:]));ages=sorted((f[0]-f[2])*1000 for f in frames)
    span=result['window_end']-result['window_start'];assert span>0
    calculated=dict(frames=len(frames),fps=round(len(frames)/span,1),
        low1_fps=round(1000/max(percentile(gaps,.99),.001),1) if len(gaps)>=10 else None,
        low5_fps=round(1000/max(percentile(gaps,.95),.001),1) if len(gaps)>=10 else None,
        age_ms=round(statistics.median(ages)) if ages else None,
        age_p95_ms=round(percentile(ages,.95)) if len(ages)>=20 else None)
    assert all(result['complete_body'][k]==v for k,v in calculated.items()),key(result)
    assert all(result['window_start']<=t<=result['window_end'] for t in times)
    assert all(b[1]>a[1] for a,b in zip(frames,frames[1:]))
    csvrow=by_key[key(result)]
    for name,column in [('frames','frames'),('fps','fps'),('low1_fps','low1'),('low5_fps','low5'),('age_ms','age'),('age_p95_ms','age95'),
                        ('kbit_s','kbit'),('rss_client_kb','rssc'),('rss_server_kb','rsss'),('cpu_client','cpuc'),('cpu_server','cpus')]:
        expected=calculated[name] if name in calculated else result[name]
        if expected is None:assert csvrow[column]=='';missing+=1
        else:assert float(csvrow[column])==expected,(key(result),name)
    assert result['source']['dropped']==0 and not result['source']['missing_oracle_frame_ids']
audit=json.loads((HERE/'source-header-join-audit.json').read_text())
assert audit['valid'] and audit['complete_body_headers_checked']==sum(len(r['complete_frames']) for r in data['results'])
assert audit['summary_sha256']==data['original_summary_sha256']
cpu=json.loads((HERE/'cpu30-summary.json').read_text())
assert len(cpu['admission'])==13 and all(r['qualifying_cpu_pct']<30 for r in cpu['admission'])
assert data['performance_accepted'] is False
for name,digest in json.loads((HERE/'raw-build-log-sha256.json').read_text()).items():
    assert hashlib.sha256(gzip.decompress((HERE/(name+'.gz')).read_bytes())).hexdigest()==digest
print(json.dumps(dict(cases=48,complete_frames=sum(len(r['complete_frames']) for r in data['results']),
    metric_and_csv_match=True,missing_values_preserved=missing,archived_build_logs_match=True,raw_body_replay_performed=False)))
