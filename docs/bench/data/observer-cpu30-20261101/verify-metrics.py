"""Recalculate saved per-run body metrics and mode deltas; no raw replay."""
from pathlib import Path
import csv, hashlib, json, math, statistics

HERE=Path(__file__).resolve().parent
data=json.loads((HERE/'observer-results.json').read_text());results=data['results']
rows=list(csv.DictReader((HERE/'observer-per-rep.csv').open()))
key=lambda r:(r['net'],r['scene'],int(r['iteration']),r['variant'],r['observer_mode'])
by_key={key(r):r for r in rows};jobs={key(r):r for r in results}
expected={(n,s,i,v,m) for n in ('lan','wan','highlat') for s in ('compress','bandwidth')
          for i in (0,1) for v in ('ssh','main-A','main-B','main-C') for m in ('legacy','raw-only')}
assert len(rows)==len(by_key)==len(results)==len(jobs)==96 and set(jobs)==set(by_key)==expected
percentile=lambda xs,q:xs[math.ceil(len(xs)*q)-1]
missing=0;online_differences=0;body_count=0
for identity,result in jobs.items():
    frames=result['complete_frames'];times=[f[0] for f in frames]
    gaps=sorted((b-a)*1000 for a,b in zip(times,times[1:]));ages=sorted((f[0]-f[2])*1000 for f in frames)
    span=result['window_end']-result['window_start'];assert math.isfinite(span) and span>0
    calculated=dict(frames=len(frames),fps=round(len(frames)/span,1),
        low1_fps=round(1000/max(percentile(gaps,.99),.001),1) if len(gaps)>=10 else None,
        low5_fps=round(1000/max(percentile(gaps,.95),.001),1) if len(gaps)>=10 else None,
        age_ms=round(statistics.median(ages)) if ages else None,
        age_p95_ms=round(percentile(ages,.95)) if len(ages)>=20 else None)
    assert all(result['complete_body'][k]==v for k,v in calculated.items()),identity
    assert all(result['window_start']<=t<=result['window_end'] for t in times)
    assert all(b[1]>a[1] for a,b in zip(frames,frames[1:]))
    assert result['slot']==({'ssh':0,'main-A':1,'main-B':2,'main-C':3} if result['iteration']==0 else
                            {'main-B':0,'main-C':1,'ssh':2,'main-A':3})[result['variant']]
    at=result['observer_transition_epoch'];lag=result['observer_transition_lag_ms']
    assert result['window_start']<=at<=result['window_end'] and abs((at-result['window_start'])*1000-lag)<1e-6
    if result['variant']!='ssh':assert result['binary_sha256']=='9ab79c22ebd289047ffa616a5e8aef3c7d9f2498756b5974ae5a74b2d0ecb463'
    csvrow=by_key[identity]
    for name in ('frames','fps','low1_fps','low5_fps','age_ms','age_p95_ms','kbit_s','rss_client_kb','rss_server_kb',
                 'cpu_client','cpu_server','observer_transition_epoch','observer_transition_lag_ms'):
        value=calculated[name] if name in calculated else result[name]
        if value is None:assert csvrow[name]=='';missing+=1
        else:assert float(csvrow[name])==value,(identity,name)
    assert result['source']['dropped']==0 and not result['source']['missing_oracle_frame_ids']
    body_count+=len(frames)
    online_differences+=result['observer_mode']=='raw-only' and result['variant']!='ssh' and result['online_fps_placeholder']!=calculated['fps']
pairs=json.loads((HERE/'observer-per-rep.json').read_text());assert len(pairs)==48
pair_ids=set()
for pair in pairs:
    base=(pair['net'],pair['scene'],pair['iteration'],pair['variant']);assert base not in pair_ids;pair_ids.add(base)
    assert pair['paired_identity_valid'] and len(pair['points'])==2
    points={p['observer_mode']:p for p in pair['points']};assert set(points)=={'legacy','raw-only'}
    for mode,point in points.items():
        row=jobs[(*base,mode)]
        for k,v in point.items():
            value=row['complete_body'].get(k,row.get(k));assert v==value,(base,mode,k)
    for metric,value in pair['raw_minus_legacy'].items():
        raw=points['raw-only'][metric];legacy=points['legacy'][metric]
        assert value==(raw-legacy if raw is not None and legacy is not None else None)
audit=json.loads((HERE/'saved-data-audit.json').read_text())
assert audit['valid_saved_audit'] and not audit['errors'] and audit['body_frames']==body_count==42132
assert audit['jobs']==96 and audit['blocks']==24 and audit['source_records']==audit['raw_source_headers']==46080
assert audit['rsh_raw_online_vs_body_different']==online_differences==28
assert len(data['original_raw_files_sha256'])==192
for name,digest in audit['source_sha256'].items():assert data['original_raw_files_sha256'][name]==digest
cpu=json.loads((HERE/'cpu30-summary.json').read_text())
assert len(cpu['admission'])==25 and all(0<=r['qualifying_cpu_pct']<30 for r in cpu['admission'])
assert len(cpu['runtime'])==24 and all(r['accepted'] and not r['reasons'] for r in cpu['runtime'])
assert data['performance_accepted'] is False and data['product_changed'] is False
extraction=json.loads((HERE/'extraction-provenance.json').read_text())
assert extraction['original_summary_sha256']==data['original_summary_sha256']
assert extraction['original_audit_sha256']==hashlib.sha256((HERE/'saved-data-audit.json').read_bytes()).hexdigest()
assert extraction['raw_files_hashed']==192 and extraction['source_hashes_compared_to_independent_audit']==96
assert extraction['terminal_replay_performed'] is False
print(json.dumps(dict(cases=96,pairs=48,complete_frames=body_count,metric_and_csv_match=True,
    missing_values_preserved=missing,online_placeholder_differences=online_differences,raw_body_replay_performed=False)))
