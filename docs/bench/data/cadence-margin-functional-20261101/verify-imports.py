"""Verify bundled imports and stored negative-activation arithmetic; no raw replay."""
from pathlib import Path
import json,hashlib,csv
P=Path(__file__).resolve().parent
read=lambda n:json.loads((P/n).read_text())
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
if (P/'manifest.json').exists():
 m=read('manifest.json')
 assert set(m)=={str(x.relative_to(P)) for x in P.rglob('*') if x.is_file() and x.name!='manifest.json'}
 for n,h in m.items():assert sha(P/n)==h
for source,item in read('source-imports.json').items():assert sha(P/item['public'])==item['sha256'],source
x=read('lowbw52-mechanism-audit.json');assert x['functional_valid'] and x['performance_eligible'] is False and x['valid'] is False and x['host_rejected']
assert (x['jobs'],x['trace_endpoints'],x['body_total'],x['header_total'])==(16,24,4522,4533)
assert (x['branch30_window_total'],x['branch30_lifetime_total'],x['branch30_actual_vs_baseline_changes'])==(0,40,0)
assert x['script_sha256']==sha(P/'lowbw52-mechanism-audit.py') and all(r['drops']==0 for r in x['trace_proofs'])
s=[r for job in x['rsh_details'] for r in job['branch30_lifetime_snapshots']]
assert len(s)==40 and all(r['actual_gap_ms']==20 and r['raw_P_ms']*.9>20 for r in s)
rows=list(csv.DictReader((P/'lowbw52-metrics.csv').open()));assert len(rows)==16
for row,stored in zip(rows,x['metrics']):
 assert (row['batch'],row['variant'],int(row['slot']))==(stored['batch'],stored['variant'],stored['slot'])
 for key in ('fps','low1','low5'):assert float(row[key])==stored[key]
print(json.dumps(dict(functional_jobs=16,performance_eligible=False,branch30_window=0,lifetime_clamped_equal=40,raw_replay=False)))
