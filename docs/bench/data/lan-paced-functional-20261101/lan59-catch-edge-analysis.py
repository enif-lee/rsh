"""Only retained59 scalar cache; no raw trace/capture replay or workload."""
from pathlib import Path
import json,hashlib,collections
n=Path(__file__).parent;p=n/'lan59-extracted.json';x=json.loads(p.read_text());rows=[]
for job in x['jobs']:
 if not job['trace']:continue
 c=job['trace']['cli'];rr=c['records'];origin=(job['window_start']-c['metadata']['anchor_realtime_ns']/1e9)*1e9;end=origin+(job['window_end']-job['window_start'])*1e9
 counts=collections.Counter();cases=[]
 for i,(ns,k,d) in enumerate(rr):
  if k!='gate' or not origin<=ns<=end:continue
  now,due,length,mask,tokens,every,since,arrived=d
  if not length>0 or mask&3!=2:continue
  budget=300_000_000 if mask&24 else 10_000_000
  counts['catch_only']+=1;counts['budget_'+str(budget//1_000_000)+'ms']+=1
  assert since>=0 and now<since+budget
  if ns<since+budget:continue
  counts['row_time_crossing']+=1
  mutations=[];boundary=None
  for later in rr[i+1:]:
   if later[1] in ['enqueue','gate']:mutations.append(later[1])
   if later[1] in ['write_begin','poll_begin']:
    boundary=later;break
  eligible=boundary is not None and boundary[1]=='poll_begin' and not mutations and boundary[2][3]>0
  counts['crossing_then_poll_without_mutation']+=eligible
  cases.append(dict(ns=ns,relative_s=(ns-origin)/1e9,gate_data=d,budget_ms=budget/1e6,row_after_deadline_ns=ns-since-budget,next_boundary=None if boundary is None else boundary[1],eligible_upper_time_only=eligible))
 rows.append(dict(batch=job['batch'],variant=job['variant'],counts=dict(counts),cases=cases))
out=dict(status='derived_observation',cache_sha256=hashlib.sha256(p.read_bytes()).hexdigest(),rows=rows,total_catch_only=sum(r['counts'].get('catch_only',0) for r in rows),row_time_crossings=sum(r['counts'].get('row_time_crossing',0) for r in rows),then_poll_crossings=sum(r['counts'].get('crossing_then_poll_without_mutation',0) for r in rows),limits=['Row timestamp is after mark call begins; an upper-time counterfactual, not actual hypothetical flush call Instant.','Existing scalar cache only; no new raw reparse, capture, build, tests or Docker.','No strict poll timeout/sole limiter, unknown source-frame ownership or GPU claim.','Mask bit catch-only is a state observation; deadline crossing would be hypothetical immediate flush eligibility at row time, not causal FPS attribution.'],script_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest())
(n/'lan59-catch-edge-analysis.json').write_text(json.dumps(out,indent=2)+'\n')
print(json.dumps({k:v for k,v in out.items() if k not in ['rows','limits']}))
