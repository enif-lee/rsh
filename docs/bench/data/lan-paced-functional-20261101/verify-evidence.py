#!/usr/bin/env python3
"""Portable bundled-hash/stored-arithmetic check; no raw replay or workload."""
from pathlib import Path
import json,hashlib,collections
D=Path(__file__).resolve().parent
read=lambda f:json.loads((D/f).read_text())
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
manifest=read('manifest.json')
assert set(manifest)=={p.name for p in D.iterdir() if p.is_file() and p.name!='manifest.json'}, 'archive membership'
for f,h in manifest.items():assert sha(D/f)==h, f
for f,v in read('source-imports.json').items():assert sha(D/f)==v['sha256'],f
j=read('lan59-paced-analysis.json');c=read('lan59-catch-edge-analysis.json');e=read('evidence-summary.json')
rows=j['rows'];rsh=[r for r in rows if 'client_gate_count'in r]
assert len(rows)==16 and len(rsh)==12 and j['functional_valid'] is True and j['performance_eligible'] is False
assert j['host_rejected']==['02-lan-bandwidth-i0-host-status'] and e['host_rejected']==j['host_rejected']
assert e['jobs']==16 and e['blocks']==4 and e['parallel']==4 and e['iterations']==2 and e['trace_endpoints']==24
assert sum(r['official']['frames'] for r in rows)==e['body_frames']==7679
for r,m in zip(rows,e['official_metrics']):
 assert m==dict(batch=r['batch'],variant=r['variant'],**{k:r['official'][k] for k in ['frames','fps','low1_fps','low5_fps','age_ms','age_p95_ms','max_gap_ms']})
 assert r['official']['fps'] in (59.9,60.0)
 assert sum(r['top1pct_location'].values())==len(r['top_body_gaps'])
counts={sc:dict(sum((collections.Counter(r['gate_reason_counts']) for r in rsh if r['scene']==sc),collections.Counter())) for sc in ['compress','bandwidth']}
assert counts==e['gate_reasons_by_scene']=={'compress':{'0':2877},'bandwidth':{'0':2882,'2':1418}}
loc=dict(sum((collections.Counter(r['top1pct_location']) for r in rsh),collections.Counter()))
assert loc==e['top_gap_categories']=={'later':57,'first_half_second':1,'crossing_half_second':2}
assert sum(r['row_time_robust_cases'] for r in rsh)==e['row_time_token_robust']==0
assert sum(r['row_time_possible_only_cases'] for r in rsh)==e['row_time_token_possible_only']==0
assert c['cache_sha256']==j['source_cache_sha256']
for key,target in [('catch_only',1418),('row_time_crossing',0),('crossing_then_poll_without_mutation',0),('budget_10ms',1418),('budget_300ms',0)]:
 assert sum(r['counts'].get(key,0) for r in c['rows'])==target
assert c['total_catch_only']==e['catch_only']==1418
assert c['row_time_crossings']==e['catch_row_crossings']==0
assert c['then_poll_crossings']==e['catch_then_poll_crossings']==0
assert e['catch_budget_10ms']==1418 and e['catch_budget_300ms']==0
p=read('lan59-paced-peer-review.json');q=read('lan59-catch-edge-peer-review.json')
assert p['verdict'].startswith('PASS') and not p['adopted_defects'] and not p['remaining_blockers']
assert q['verdict'].startswith('PASS') and not q['adopted_defects'] and not q['errors']
assert q['script_sha256']==sha(D/'lan59-catch-edge-analysis.py') and q['analysis_sha256']==sha(D/'lan59-catch-edge-analysis.json')
ids=read('artifact-identities.json');assert len(ids)==3
assert [a['source_build_id'] for a in ids]==['d742268da4df','d742268da4df','3031e105b1aa']
for a in ids:
 f='main-source-manifest.json' if a['label']=='main' else 'combined-source-manifest.json'
 assert a['source_manifest_sha256']==sha(D/f) and a['source_build_verified'] is True
assert read('run-status.json')['completed_jobs']==16
print('PASS: bundled hashes/imports,16 functional jobs/7679 bodies,0 token candidates,1418 ten-ms catch gates/0 crossings,57+1+2 diagnostic split. No raw trace/source/capture replay or new measurement.')
