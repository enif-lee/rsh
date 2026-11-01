"""Check the eight saved compatibility records; no terminal or workload run."""
from pathlib import Path
import csv,hashlib,json,math,statistics
HERE=Path(__file__).resolve().parent
manifest=json.loads((HERE/'manifest.json').read_text())
assert set(manifest)=={str(p.relative_to(HERE)) for p in HERE.rglob('*') if p.is_file() and p.name!='manifest.json'}
for name,digest in manifest.items():assert hashlib.sha256((HERE/name).read_bytes()).hexdigest()==digest,name
for name,digest in json.loads((HERE/'imported-evidence.json').read_text()).items():assert manifest[name]==digest
data=json.loads((HERE/'records.json').read_text());rows=data['results'];csvrows=list(csv.DictReader((HERE/'metrics.csv').open()))
assert len(rows)==len(csvrows)==8 and data['performance_eligible'] is False and data['final_selection'] is None
assert {(r['variant'],r['iteration']) for r in rows}=={(v,i) for v in ('ssh','main-A','main-B','candidate') for i in (0,1)}
for row in rows:
    f=row['complete_frames'];lo,hi=row['window_start'],row['window_end'];span=hi-lo
    gaps=sorted((b[0]-a[0])*1000 for a,b in zip(f,f[1:]));ages=sorted((x[0]-x[2])*1000 for x in f)
    q=lambda xs,p:xs[math.ceil(len(xs)*p)-1]
    times=[lo]+[x[0] for x in f]+[hi]
    expected=dict(frames=len(f),fps=round(len(f)/span,1),low1_fps=round(1000/max(q(gaps,.99),.001),1) if len(gaps)>=10 else None,
        low5_fps=round(1000/max(q(gaps,.95),.001),1) if len(gaps)>=10 else None,
        age_ms=round(statistics.median(ages)) if ages else None,age_p95_ms=round(q(ages,.95)) if len(ages)>=20 else None,
        max_gap_ms=round(max(b-a for a,b in zip(times,times[1:]))*1000))
    assert all(row['complete_body'][k]==v for k,v in expected.items())
    cr=next(r for r in csvrows if r['variant']==row['variant'] and int(r['iteration'])==row['iteration'])
    for key,col in [('frames','frames'),('fps','fps'),('low1_fps','low1'),('low5_fps','low5'),('age_ms','age'),('age_p95_ms','age95'),('max_gap_ms','maxgap')]:
        assert cr[col]=='' if expected[key] is None else float(cr[col])==expected[key]
    assert int(cr['headerframes'])==row['uniform_header']['frames']==len(f)
    assert float(cr['headerfps'])==row['uniform_header']['fps']
    assert float(cr['tail_s'])==(hi-f[-1][0] if f else span)
    assert float(cr['incomplete_pct'])==row['visible_state']['incomplete_pct']
    for key,col in [('kbit_s','kbit'),('rss_client_kb','rssc'),('rss_server_kb','rsss'),('cpu_client','cpuc'),('cpu_server','cpus')]:assert float(cr[col])==row[key]
audit=json.loads((HERE/'saved-data-audit.json').read_text())
assert audit['valid_saved_audit'] and not audit['errors'] and audit['body_frames']==sum(len(r['complete_frames']) for r in rows)==774
assert audit['source_records']==3319 and audit['pending_writes_saved']==1 and audit['pending_body_joins']==0
assert all(r['known_old_nested_csi_sgr_count']==0 for r in audit['capture_details'])
assert [r['can_delta_prefix_count'] for r in audit['capture_details'] if r['variant']=='candidate']==[120,118]
print(json.dumps(dict(records=8,bodies=774,metric_csv_manifest_match=True,performance_eligible=False,new_terminal_runs=0)))
