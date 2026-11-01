"""Verify saved statistics, imported bytes and manifest; no terminal replay."""
from pathlib import Path
import csv,gzip,hashlib,json,math,statistics
HERE=Path(__file__).resolve().parent
sha=lambda raw:hashlib.sha256(raw).hexdigest()
manifest=json.loads((HERE/'manifest.json').read_text())
assert {n for n in manifest if not n.startswith('../')}=={str(p.relative_to(HERE)) for p in HERE.rglob('*') if p.is_file() and p.name!='manifest.json'}
for name,digest in manifest.items():assert sha((HERE/name).read_bytes())==digest,name
for name,record in json.loads((HERE/'imported-evidence.json').read_text()).items():
    raw=(HERE/name).read_bytes();assert sha(gzip.decompress(raw) if name.endswith('.gz') else raw)==record['original_sha256']
data=json.loads((HERE/'records.json').read_text());rows=data['results'];csvrows=list(csv.DictReader((HERE/'metrics.csv').open()))
assert len(rows)==len(csvrows)==16 and len(data['original_raw_files_sha256'])==32
assert data['performance_eligible'] is True and data['final_selection'] is None
assert {(r['scene'],r['iteration'],r['variant']) for r in rows}=={(s,i,v) for s in ('compress','bandwidth') for i in (0,1) for v in ('ssh','main-A','main-B','candidate')}
for row in rows:
    f=row['complete_frames'];lo,hi=row['window_start'],row['window_end'];g=sorted((b[0]-a[0])*1000 for a,b in zip(f,f[1:]));a=sorted((x[0]-x[2])*1000 for x in f)
    q=lambda xs,p:xs[math.ceil(len(xs)*p)-1];ts=[lo]+[x[0] for x in f]+[hi]
    expected=dict(frames=len(f),fps=round(len(f)/(hi-lo),1),low1_fps=round(1000/max(q(g,.99),.001),1),low5_fps=round(1000/max(q(g,.95),.001),1),
        age_ms=round(statistics.median(a)),age_p95_ms=round(q(a,.95)),max_gap_ms=round(max(b-a for a,b in zip(ts,ts[1:]))*1000))
    assert all(row['complete_body'][k]==v for k,v in expected.items())
    assert len(f)==row['uniform_header']['frames']==480 and expected['fps']==60.0
    cr=next(r for r in csvrows if (r['scene'],int(r['iteration']),r['variant'])==(row['scene'],row['iteration'],row['variant']))
    for key,col in [('frames','frames'),('fps','fps'),('low1_fps','low1'),('low5_fps','low5'),('age_ms','age'),('age_p95_ms','age95'),('max_gap_ms','maxgap')]:assert float(cr[col])==expected[key]
    assert float(cr['firstgap'])==(f[0][0]-lo)*1000 and float(cr['tailgap'])==(hi-f[-1][0])*1000
    for key,col in [('kbit_s','kbit'),('rss_client_kb','rssc'),('rss_server_kb','rsss'),('cpu_client','cpuc'),('cpu_server','cpus')]:assert float(cr[col])==row[key]
audit=json.loads((HERE/'saved-data-audit.json').read_text())
assert audit['valid_saved_audit'] and not audit['errors'] and audit['body_frames']==audit['source_records']==7680
assert audit['pending_writes_saved']==audit['pending_body_joins']==0 and audit['endpoint_file_checks']==64 and audit['live_server_checks']==32
assert audit['saved_summary_sha256']==data['original_summary_sha256']
for name,digest in audit['source_sha256'].items():assert data['original_raw_files_sha256'][name]==digest
pair=json.loads((HERE/'client-subms-wait-release-pair.json').read_text())
assert pair['artifacts'][0]['source_build_id']=='93533574fffa' and pair['artifacts'][1]['source_build_id']=='ca0abbd5fc40'
assert sha(gzip.decompress((HERE/'build-client-subms-wait.log.gz').read_bytes()))==pair['build_log_sha256']
print(json.dumps(dict(records=16,bodies=7680,metric_csv_manifest_match=True,raw_hashes=32,new_terminal_runs=0)))
