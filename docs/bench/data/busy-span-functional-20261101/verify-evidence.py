"""Bundled hash and derived sample arithmetic only; no CSV/body-oracle replay."""
from pathlib import Path
import json,hashlib,math
P=Path(__file__).resolve().parent
read=lambda n:json.loads((P/n).read_text())
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
if (P/'manifest.json').exists():
 m=read('manifest.json');assert set(m)=={str(f.relative_to(P)) for f in P.rglob('*') if f.is_file() and f.name!='manifest.json'}
 for n,h in m.items():assert sha(P/n)==h,n
for n,item in read('source-imports.json').items():assert sha(P/item['public'])==item['sha256'],n
a=read('moving54-busy-audit.json');p=read('moving54-busy-peer-check.json');e=read('evidence-summary.json')
assert a['functional_valid'] and a['performance_eligible'] is False and a['jobs']==16 and a['traces']==24
assert p['verdict']=='PASS' and not p['errors'] and p['author_audit_sha256']==sha(P/'moving54-busy-audit.json')
assert a['script_sha256']==sha(P/'moving54-busy-audit.py') and e['author_audit_sha256']==sha(P/'moving54-busy-audit.json') and e['peer_sha256']==sha(P/'moving54-busy-peer-check.json')
assert len(a['trace_proofs'])==24 and all(t['dropped']==0 for t in a['trace_proofs'])
assert sum(t['events'] for t in a['trace_proofs'])==643005 and max(t['events'] for t in a['trace_proofs'])==49423 and max(abs(t['drift_ns']) for t in a['trace_proofs'])==6167
assert len(a['results'])==12 and len(e['cohorts'])==12
all_samples=[]
for r,c in zip(a['results'],e['cohorts']):
 assert all(c[k]==r[k] for k in c)
 ss=r['samples'];all_samples.extend(ss)
 for s in ss:
  dt=s['ack_now_ns']-s['from_ns'];assert dt>=30000000 and s['row_ns']>=s['ack_now_ns'] and s['wire']>0 and 1<=s['retained']<=15
  assert math.isclose(s['sample_Bps'],s['wire']/(dt/1e9),rel_tol=1e-12,abs_tol=1e-6)
  overlap=s['strict_hold_overlap_ms'];assert 0<=overlap<=dt/1e6 and math.isclose(s['strict_hold_overlap_fraction'],overlap*1e6/dt,rel_tol=1e-12,abs_tol=1e-12)
  assert s['median_lowered']==(s['old_capacity_Bps'] is not None and s['new_capacity_Bps'] is not None and s['new_capacity_Bps']<s['old_capacity_Bps'])
  if s['retained']<3:assert s['old_capacity_Bps']==s['new_capacity_Bps']
  if 'post_median_rate25_capacity_Bps' in s:assert s['post_median_rate25_capacity_Bps']==(-1 if s['new_capacity_Bps'] is None else int(s['new_capacity_Bps']))
 for key,select in [('lifetime',ss),('window',[s for s in ss if 0<=s['end_s']<=30]),('after23',[s for s in ss if 23<=s['end_s']<=30])]:
  q=r[key];assert q['samples']==len(select) and q['median_lowerings']==sum(s['median_lowered'] for s in select) and q['with_strict_overlap']==sum(s['strict_hold_overlap_ms']>0 for s in select) and q['lowerings_with_strict_overlap']==sum(s['median_lowered'] and s['strict_hold_overlap_ms']>0 for s in select)
 peer=next(t for t in p['results'] if (t['batch'],t['slot'],t['variant'])==(r['batch'],r['slot'],r['variant']))
 assert peer['strict_hold_count']==r['strict_hold_count'] and all(peer['after23'][k]==r['after23'][k] for k in peer['after23'])
assert len(all_samples)==e['lifetime_commits']==2137
assert sum(s.get('post_rate25_same_ack_probe') is not None for s in all_samples)==e['same_ack_probe_records']==0
assert sum(s.get('post_rate25_refloor_clear',False) for s in all_samples)==e['same_ack_refloor_clears']==28
assert sum(s.get('post_rate25_refloor_clear',False) and 23<=s['end_s']<=30 for s in all_samples)==e['same_ack_refloor_clears_after23']==0
for w in e['witnesses']:
 r=next(r for r in a['results'] if r['batch']==w['batch'] and r['variant']==w['variant']);assert w['sample'] in r['samples'] and w['sample']['median_lowered'] and w['sample']['strict_hold_overlap_ms']>0
assert 446*8/64000*1000==55.75
print(json.dumps(dict(functional_jobs=16,trace_endpoints=24,server_commits=2137,median_stage_not_ack_exit=True,witnesses=2,performance_eligible=False,raw_replay=False)))
