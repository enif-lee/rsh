"""Independent bounded saved-CSV check of strict hold/sample intersections."""
from pathlib import Path
import json,csv,hashlib,struct,bisect,collections,math,statistics
R=Path('/Users/ed/personal/rsh/benchmark/lab/out/lowtail-research-20261031/54-busy-sample-trace');N=Path(__file__).parent;A=json.loads((N/'moving54-busy-audit.json').read_text());errors=[];sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
def check(v,s):
 if not v:errors.append(s)
def rate(i):return None if i==-1 else struct.unpack('>d',struct.pack('>q',i))[0]
def sweep(intervals):
 changes=collections.Counter()
 for a,b in intervals:
  if b>a:changes[a]+=1;changes[b]-=1
 out=[];level=0;start=None
 for t in sorted(changes):
  previous=level;level+=changes[t]
  if previous==0 and level>0:start=t
  elif previous>0 and level==0:out.append((start,t))
 return out
results=[]
for proof in A['trace_proofs']:
 folder=R/proof['batch']/f"{proof['slot']}-{proof['side']}";meta=folder/f"meta-{proof['pid']}.json";file=folder/f"trace-{proof['pid']}.csv";m=json.loads(meta.read_text());check(sha(meta)==proof['meta_sha256'] and sha(file)==proof['csv_sha256'],'raw hash')
 check(m['events']==proof['events'] and m['dropped']==0 and m['capacity']==131072 and m['record_bytes']==80,'meta count/drop/shape')
 if proof['side']=='cli':continue
 rows=[]
 with file.open() as stream:
  for i,r in enumerate(csv.DictReader(stream)):rows.append((i,int(r['ns']),r['kind'],[int(r[c]) for c in 'abcdefgh']))
 choices=[r for r in rows if r[2]=='screen_choice'];timers=[r for r in rows if r[2]=='server_timer'];ci=[r[0] for r in choices];ti=[r[0] for r in timers];holds=[];previous_poll=-1
 for r in [r for r in rows if r[2]=='server_poll']:
  position=bisect.bisect_left(ti,r[0])-1;check(position>=0 and timers[position][0]>previous_poll,'timer/poll one-loop association');previous_poll=r[0]
  if position<0:continue
  t=timers[position];a=t[3];p=r[3];k=bisect.bisect_left(ci,t[0])-1
  if a[3]!=8 or p[4]!=0 or p[5]!=0 or p[6]!=0 or a[1]<=0 or a[4]<=a[0] or k<0:continue
  c=choices[k];d=c[3];effective=max(d[6],d[7]+20000000 if d[7]>=0 and d[1]&(1<<17) else d[6])
  if d[1]&255 not in (20,30,40) or not d[1]&(1<<16) or not d[1]&(1<<17) or effective!=a[4] or t[1]>p[0] or p[3]!=min(a[2]//1000000,200):continue
  check(a[1]==a[4]-a[0],'strict single-screen raw wait/due consistency')
  end=min(p[1],a[4])
  if end>p[0]:holds.append((d[0],p[0],end))
 unions={g:sweep([(a,b) for gen,a,b in holds if gen==g]) for g in {g for g,_,_ in holds}}
 original=next(x for x in A['results'] if (x['batch'],x['slot'])==(proof['batch'],proof['slot']));check(len(holds)==original['strict_hold_count'],'strict hold count')
 commits=[r for r in rows if r[2]=='busy_sample_commit'];check(len(commits)==len(original['samples']),'commit count');samples=[]
 for r,saved in zip(commits,original['samples']):
  g,a,b,wire,bit,old,new,count=r[3];check((saved['event_index'],saved['row_ns'],saved['gen'],saved['from_ns'],saved['ack_now_ns'])==(r[0],r[1],g,a,b),'sample identity/time')
  check(b-a>=30000000 and a<=b<=r[1] and wire>0 and 1<=count<=15 and math.isclose(rate(bit),wire/((b-a)/1e9),rel_tol=1e-12,abs_tol=1e-6),'sample exact arithmetic')
  overlap=sum(max(0,min(b,v)-max(a,u)) for u,v in unions.get(g,[]));check(saved['strict_hold_overlap_ms']==overlap/1e6 and saved['strict_hold_overlap_fraction']==overlap/(b-a),'union overlap amount/denominator')
  check(0<=overlap<=b-a,'bounded overlap');samples.append(saved)
  next25=next((e for e in rows[r[0]+1:] if e[2]=='sender_rate'),None)
  if next25:check(next25[3][0]==g and next25[3][1]==(-1 if rate(new) is None else int(rate(new))),'postmedian25 before later probe/refloor')
 late=[s for s in samples if 23<=s['end_s']<=30];counts=dict(samples=len(late),median_lowerings=sum(s['median_lowered'] for s in late),with_strict_overlap=sum(s['strict_hold_overlap_ms']>0 for s in late),lowerings_with_strict_overlap=sum(s['median_lowered'] and s['strict_hold_overlap_ms']>0 for s in late));check(all(original['after23'][k]==v for k,v in counts.items()),'after23 counts')
 results.append(dict(batch=proof['batch'],slot=proof['slot'],variant=proof['variant'],strict_hold_count=len(holds),after23=counts))
output=dict(verdict='PASS' if not errors else 'FAIL',errors=errors,traces_hashed=len(A['trace_proofs']),server_csvs_reparsed=len(results),results=results,author_audit_sha256=sha(N/'moving54-busy-audit.json'),scope='Independent closest-predecessor association and sweep-line union/intersection arithmetic; source predicates/tuple read; no original sourcepayload/bodyoracle replay/workload/build',limits=['Strict screen-poll context is not physical idle, continuous demand or a sample discard rule','Passed ACK interval endpoint is distinct from actual sample commit row timestamp','All-generation samples overlap only same-generation holds; full span denominator retained, not clipped to23s'])
(N/'moving54-busy-peer-check.json').write_text(json.dumps(output,indent=2)+'\n');print(json.dumps(dict(verdict=output['verdict'],errors=errors,traces=output['traces_hashed'],server_csvs=len(results),candidate=[r for r in results if r['variant']=='combined-trace'])))
