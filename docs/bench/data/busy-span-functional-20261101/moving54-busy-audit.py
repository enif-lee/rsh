"""Saved54 sample intervals vs strict screen-only poll context; no workload/replay."""
from pathlib import Path
import bisect,collections,csv,hashlib,json,math,statistics,struct
S=Path('/Users/ed/personal/rsh/benchmark/lab/out/lowtail-research-20261031/54-busy-sample-trace');N=Path(__file__).parent
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
def rate(bits):return None if bits==-1 else struct.unpack('>d',struct.pack('>q',bits))[0]
def stat(xs):
 xs=sorted(xs);return dict(n=len(xs),min=xs[0],p50=statistics.median(xs),p95=xs[math.ceil(.95*len(xs))-1],max=xs[-1]) if xs else dict(n=0)
def union(intervals):
 out=[]
 for lo,hi in sorted(intervals):
  if hi<=lo:continue
  if out and lo<=out[-1][1]:out[-1]=(out[-1][0],max(hi,out[-1][1]))
  else:out.append((lo,hi))
 return out
X=json.loads((S/'probe-summary.json').read_text());assert X['functional_valid'] is True and X['performance_eligible'] is False and len(X['results'])==16
assert not X['functional_invalid_reasons'];proof=[];output=[]
for job in X['results']:
 if job['tool']=='ssh':continue
 for side in ('cli','srv'):
  folder=S/job['batch']/f"{job['slot']}-{side}";mp=next(folder.glob('meta-*.json'));m=json.loads(mp.read_text());cp=folder/f"trace-{m['pid']}.csv";events=[]
  assert m['dropped']==0 and m['capacity']==131072 and m['record_bytes']==80
  for index,row in enumerate(csv.DictReader(cp.open())):
   events.append(dict(i=index,ns=int(row['ns']),kind=row['kind'],d=[int(row[k]) for k in 'abcdefgh']))
  assert len(events)==m['events'] and all(a['ns']<=b['ns'] for a,b in zip(events,events[1:]))
  proof.append(dict(batch=job['batch'],slot=job['slot'],variant=job['variant'],side=side,pid=m['pid'],events=len(events),dropped=m['dropped'],csv_sha256=sha(cp),meta_sha256=sha(mp),drift_ns=m['dump_realtime_ns']-m['anchor_realtime_ns']-m['dump_monotonic_ns']))
  if side=='cli':continue
  anchor=m['anchor_realtime_ns']/1e9;origin=(job['window_start']-anchor)*1e9;rel=lambda ns:(ns-origin)/1e9
  previous_choice=None;pending_timer=None;strict=[];other=collections.Counter();samples=[];refresh=[]
  for e in events:
   if e['kind']=='screen_choice':previous_choice=e
   elif e['kind']=='server_timer':pending_timer=(e,previous_choice)
   elif e['kind']=='server_poll' and pending_timer:
    timer,choice=pending_timer;pending_timer=None;now,raw,returned,mask,due,*_=timer['d'];enter,end,locked,request,rc,network,ready,nfds=e['d']
    assert enter<=end<=locked<=e['ns']
    if mask!=8:
     other['screen_tie_poll' if mask&8 else 'other_timer_poll']+=1;continue
    if rc!=0 or network or ready:other['screen_only_eventful_or_interrupted_poll']+=1;continue
    if raw<=0 or due<=now:other['screen_only_nonfuture_poll']+=1;continue
    if not choice:other['missing_choice']+=1;continue
    gen,cmask,last,out,P,gap,rawdue,tail=choice['d'];effective=max(rawdue,tail+20_000_000 if tail>=0 and cmask&(1<<17) else rawdue)
    if (cmask&255) not in (20,30,40) or not cmask&(1<<16) or not cmask&(1<<17) or effective!=due or timer['ns']>enter or request!=min(returned//1_000_000,200):
     other['screen_only_unmatched_context']+=1;continue
    hi=min(end,due)
    if hi<=enter:other['screen_only_entered_after_due']+=1;continue
    strict.append(dict(lo=enter,hi=hi,return_ns=end,deadline_ns=due,gen=gen,branch=cmask&255,choice_ns=choice['ns'],timer_ns=timer['ns'],poll_ns=e['ns'],overshoot_ns=max(0,end-due)))
   elif e['kind']=='busy_sample_commit':
    gen,begin,end,wire,sbits,oldbits,newbits,count=e['d'];v=rate(sbits);old=rate(oldbits);new=rate(newbits)
    assert begin<=end<=e['ns'] and end-begin>=30_000_000 and wire>0 and 1<=count<=15
    assert math.isfinite(v) and v>0 and math.isclose(v,wire/((end-begin)/1e9),rel_tol=1e-12,abs_tol=1e-6)
    if count<3:assert oldbits==newbits
    samples.append(dict(event_index=e['i'],row_ns=e['ns'],gen=gen,from_ns=begin,ack_now_ns=end,wire=wire,sample_Bps=v,old_capacity_Bps=old,new_capacity_Bps=new,retained=count,median_lowered=old is not None and new is not None and new<old,begin_s=rel(begin),end_s=rel(end),row_s=rel(e['ns'])))
   elif e['kind']=='screen_refresh':refresh.append(e)
  intervals_by_gen={gen:union([(h['lo'],h['hi']) for h in strict if h['gen']==gen]) for gen in {h['gen'] for h in strict}}
  for sample in samples:
   lo,hi=sample['from_ns'],sample['ack_now_ns'];intervals=intervals_by_gen.get(sample['gen'],[]);ends=[b for a,b in intervals];overlap=[];start=bisect.bisect_left(ends,lo)
   for a,b in intervals[start:]:
    if a>=hi:break
    if min(hi,b)>max(lo,a):overlap.append((max(lo,a),min(hi,b)))
   sample['strict_hold_overlap_ms']=sum(b-a for a,b in overlap)/1e6;sample['strict_hold_overlap_fraction']=sum(b-a for a,b in overlap)/(hi-lo)
   sample['strict_hold_segments']=len(overlap)
   sample['refresh_entries_inside_span']=sum(lo<=r['d'][3]<=hi for r in refresh)
   following=events[sample['event_index']+1:];q=next((e for e in following if e['kind']=='sender_rate'),None)
   if q:
    assert q['d'][0]==sample['gen'] and q['d'][1]==(-1 if sample['new_capacity_Bps'] is None else int(sample['new_capacity_Bps']))
    sample['post_median_rate25_capacity_Bps']=q['d'][1]
    sample['post_rate25_same_ack_probe']=None;sample['post_rate25_refloor_clear']=False
    for e in events[q['i']+1:]:
     if e['kind'] in ('sender_rate','busy_sample_commit'):break
     if e['kind']=='capacity_probe':
      gen,R,took,us,wire,flight,pending,tcp=e['d'];classification='definite_pass' if took>=.6*(R+1) else 'definite_fail' if took+1<.6*R else 'rounding_ambiguous'
      sample['post_rate25_same_ack_probe']=dict(rel_s=rel(e['ns']),rate_Bps=R,observed_Bps=took,ratio=took/R if R>0 else None,classification=classification)
     if e['kind']=='by_size_clear' and e['d'][1]==3:sample['post_rate25_refloor_clear']=True
  def group(ss):
   return dict(samples=len(ss),median_lowerings=sum(s['median_lowered'] for s in ss),with_strict_overlap=sum(s['strict_hold_overlap_ms']>0 for s in ss),lowerings_with_strict_overlap=sum(s['median_lowered'] and s['strict_hold_overlap_ms']>0 for s in ss),sample_Bps=stat([s['sample_Bps'] for s in ss]),strict_hold_overlap_ms=stat([s['strict_hold_overlap_ms'] for s in ss]),strict_hold_overlap_fraction=stat([s['strict_hold_overlap_fraction'] for s in ss]),fully_contained_after23=sum(s['begin_s']>=23 and s['end_s']<=30 for s in ss))
  late=[s for s in samples if 23<=s['end_s']<=30];rf=[r for r in refresh if 23<=rel(r['d'][3])<=30]
  output.append(dict(batch=job['batch'],slot=job['slot'],variant=job['variant'],scene=job['scene'],iteration=job['iteration'],source_build_id=X['environment']['binaries'][job['artifact_variant']]['source_build_id'],body=job['complete_body'],strict_hold_count=len(strict),strict_hold_window_count=sum(0<=rel(h['lo'])<=30 for h in strict),excluded_poll_contexts=dict(other),lifetime=group(samples),window=group([s for s in samples if 0<=s['end_s']<=30]),after23=group(late),strict_hold_examples=[dict(begin_s=rel(h['lo']),end_s=rel(h['hi']),deadline_s=rel(h['deadline_ns']),gen=h['gen'],branch=h['branch'],overshoot_ms=h['overshoot_ns']/1e6) for h in strict if 23<=rel(h['lo'])<=30][:12],after23_refresh_entry_intervals_ms=stat([(b['d'][3]-a['d'][3])/1e6 for a,b in zip(rf,rf[1:])]),samples=samples))
result=dict(study=str(S),functional_valid=X['functional_valid'],performance_eligible=X['performance_eligible'],host_rejected=X['host_rejected'],jobs=16,traces=len(proof),summary_sha256=sha(S/'probe-summary.json'),trace_proofs=proof,results=output,strict_definition='Exact timer mask8 (ties separate), future effective screen due matching preceding36, matching poll38rc0/network0/otherready0; actual poll [enter,min(return,due)] union, no overshoot counted. Single Sender/process provenance remains required.',limits=['Poll-context overlap is not evidence of physical link idle or continuous app demand, nor a sample-discard rule.','Sample interval ends at passed ACK now, row time occurs after median update; later same-ACK probe/refloor are separate.','39 refresh timing uses actual entry d; snapshots/PTY chunks do not assign full producer-frame ownership or GPU presentation.','No emulator replay/raw-source-payload regeneration/new benchmark; functional54 timing is not pooled with untraced45/53.'],script_sha256=sha(Path(__file__)))
(N/'moving54-busy-audit.json').write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps([dict(batch=r['batch'],variant=r['variant'],after23=r['after23'],holds=r['strict_hold_count'],excluded=r['excluded_poll_contexts']) for r in output if r['variant']=='combined-trace']))
