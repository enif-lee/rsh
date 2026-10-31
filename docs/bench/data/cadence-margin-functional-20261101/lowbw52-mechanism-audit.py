"""Read saved summary and 24 CSVs; no emulator, imports of experiment code or workload."""
from pathlib import Path
import bisect,collections,csv,hashlib,json,math,statistics
S=Path('/Users/ed/personal/rsh/benchmark/lab/out/lowtail-research-20261031/52-lowbw-cadence-trace')
N=Path(__file__).resolve().parent
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
X=json.loads((S/'probe-summary.json').read_text())
assert X['functional_valid'] is True and X['performance_eligible'] is False and len(X['results'])==16
assert X['functional_invalid_reasons']==[]
def stat(xs):
 xs=sorted(xs)
 return dict(count=len(xs),min=xs[0],median=statistics.median(xs),p95=xs[math.ceil(.95*len(xs))-1],max=xs[-1]) if xs else dict(count=0)
def fields(r):return [int(r[k]) for k in 'abcdefgh']
def records(job,side):
 d=S/job['batch']/f"{job['slot']}-{side}";mf=list(d.glob('meta*.json'));cf=list(d.glob('trace*.csv'));assert len(mf)==len(cf)==1
 m=json.loads(mf[0].read_text());assert m['dropped']==0 and m['capacity']==131072 and m['record_bytes']==80
 rr=[];ep=m['anchor_realtime_ns']/1e9
 for row in csv.DictReader(cf[0].open()):
  ns=int(row['ns']);rr.append(dict(ns=ns,t=ep+ns/1e9,rel=ep+ns/1e9-job['window_start'],kind=row['kind'],d=fields(row)))
 assert len(rr)==m['events'] and all(a['ns']<=b['ns'] for a,b in zip(rr,rr[1:]))
 proof=dict(csv=str(cf[0].relative_to(S)),csv_sha256=sha(cf[0]),meta_sha256=sha(mf[0]),events=len(rr),drops=m['dropped'],pid=m['pid'],anchor_bracket_ns=m['anchor_bracket_ns'],dump_clock_drift_ns=m['dump_realtime_ns']-m['anchor_realtime_ns']-m['dump_monotonic_ns'])
 return rr,proof

def count(rr):return dict(collections.Counter(r['kind'] for r in rr))
def probe_class(r):
 _,rate,took,*_=r['d']
 return 'definite_pass' if took>=.6*(rate+1) else 'definite_fail' if took+1<.6*rate else 'rounding_ambiguous'
def screen(r):
 g,mask,at,out,p,gap,due,tail=r['d'];return dict(rel=r['rel'],gen=g,branch=mask&255,animating=bool(mask&(1<<18)),raw_P_ms=p/1e6,actual_gap_ms=gap/1e6,screen_at_rel=(at-r['ns'])/1e9+r['rel'] if at!=-1 else None,due_rel=(due-r['ns'])/1e9+r['rel'] if due!=-1 else None,tail_since_rel=(tail-r['ns'])/1e9+r['rel'] if tail!=-1 else None,pick=(mask>>8)&15)

def pending_sleeps(rr):
 gate=None;begin=None;out=[]
 for r in rr:
  if r['kind']=='gate':gate=r
  elif r['kind']=='poll_begin':begin=(r,gate)
  elif r['kind']=='poll_end' and begin:
   b,g=begin;begin=None
   if g and b['d'][3]>0 and g['d'][2]>0:
    out.append((b['t'],r['t'],g['d'][3]&3))
 return out

def subevents(rr,lo,hi):return [r for r in rr if lo<=r['t']<=hi]

rows=[];details=[];proofs=[]
for j in X['results']:
 lo,hi=j['window_start'],j['window_end'];frames=j['complete_frames'];times=[f[0] for f in frames];gaps=[(b[0]-a[0])*1000 for a,b in zip(frames,frames[1:])];ordered=sorted(gaps);body=j['complete_body']
 lows={f'low{int((1-q)*100+.01)}_fps':round(1000/max(ordered[math.ceil(len(ordered)*q)-1],.001),1) if len(ordered)>=10 else None for q in (.99,.95)}
 assert len(frames)==body['frames'] and round(len(frames)/(hi-lo),1)==body['fps']
 assert all(lows[k]==body[k] for k in lows)
 assert round(max(b-a for a,b in zip([lo]+times,[*times,hi]))*1000)==body['max_gap_ms']
 assert all(lo<=f[0]<=hi for f in frames) and len(j['source_joined_complete_frames'])==len(frames)
 row=dict(batch=j['batch'],scene=j['scene'],iteration=j['iteration'],variant=j['variant'],slot=j['slot'],body=body['frames'],fps=body['fps'],low1=body['low1_fps'],low5=body['low5_fps'],header=j['uniform_header']['frames'],incomplete_states=j['incomplete_states'],incomplete_pct=j['visible_state']['incomplete_pct'],maxgap_ms=body['max_gap_ms'],head_ms=(times[0]-lo)*1000 if times else None,tail_ms=(hi-times[-1])*1000 if times else None)
 if j['tool']=='ssh':rows.append(row);continue
 srv,sp=records(j,'srv');cli,cp=records(j,'cli');proofs.extend([sp,cp]);sw=subevents(srv,lo,hi);cw=subevents(cli,lo,hi);by=lambda name:[r for r in sw if r['kind']==name]
 allchoice=[r for r in srv if r['kind']=='screen_choice'];choice=by('screen_choice');b30=[r for r in allchoice if r['d'][1]&255==30];b30w=[r for r in choice if r['d'][1]&255==30];rfs=by('screen_refresh');probes=by('capacity_probe');rates=by('sender_rate');papp=by('pty_apply');polls=by('server_poll');timers=by('server_timer');sleeps=pending_sleeps(cli)
 actual_diff=[r for r in b30 if r['d'][5]!=(max(8_000_000,min(20_000_000,r['d'][4])) if r['d'][1]&(1<<18) else 20_000_000)]
 first20=next((screen(r) for r in allchoice if r['d'][1]&255==20),None)
 branches=dict(collections.Counter(r['d'][1]&255 for r in choice));pstat=stat([r['d'][6]/1e6 for r in papp]);refinterval=stat([(b['d'][3]-a['d'][3])/1e6 for a,b in zip(rfs,rfs[1:])]);probehist=dict(collections.Counter(probe_class(r) for r in probes));gatehist=dict(collections.Counter(r['d'][3]&3 for r in cw if r['kind']=='gate' and r['d'][2]>0))
 row.update(branch30_window=len(b30w),branch30_lifetime=len(b30),branch30_actual_vs_baseline_differences=len(actual_diff),refreshes=len(rfs),full_resets=sum(r['d'][7]==2 for r in rfs),capacity_probes=len(probes),probe_definite_pass=probehist.get('definite_pass',0),probe_definite_fail=probehist.get('definite_fail',0),server_window_blocks=sum(r['kind']=='window_block' for r in sw),server_delivery_blocks=sum(r['kind']=='delivery_block' for r in sw))
 top=[]
 for a,b in sorted(zip(frames,frames[1:]),key=lambda q:q[1][0]-q[0][0],reverse=True)[:5]:
  rs=subevents(srv,a[0],b[0]);near=subevents(srv,a[0]-.15,b[0]);cs=subevents(cli,a[0],b[0]);over={str(mask):sum(max(0,min(b[0],e)-max(a[0],s))*1000 for s,e,m in sleeps if m==mask) for mask in (0,1,2,3)}
  saved=next((z for z in j['late_gap_examples'] if z['previous_n']==a[1] and z['n']==b[1]),None)
  if saved:
   for mask,name in [(0,'ready'),(1,'token'),(2,'catch'),(3,'both')]:assert abs(over[str(mask)]-saved['pending_poll_ms'][name])<1e-6
  top.append(dict(start_rel=a[0]-lo,end_rel=b[0]-lo,gap_ms=(b[0]-a[0])*1000,previous_source_n=a[1],next_source_n=b[1],previous_made_rel=a[2]-lo,next_made_rel=b[2]-lo,pending_poll_overlap_ms=over,client_event_counts=count(cs),server_same_epoch_interval_events=count(rs),server_same_epoch_interval_branches=dict(collections.Counter(r['d'][1]&255 for r in rs if r['kind']=='screen_choice')),nearby_pty_apply=[dict(rel=r['rel'],bytes=r['d'][1],ground=bool(r['d'][3]&1),tail=r['d'][3]>>1,gen=r['d'][4],P_ms=r['d'][6]/1e6,animation_count=r['d'][7]) for r in near if r['kind']=='pty_apply'][-20:],server_nearby_refreshes=[dict(rel=r['rel'],oldgen=r['d'][0],newgen=r['d'][1],entry_rel=(r['d'][3]-r['ns'])/1e9+r['rel'],kind=r['d'][7],update_len=r['d'][5]) for r in near if r['kind']=='screen_refresh']))
 bins=[]
 for sec in range(8):
  rr=[r for r in sw if sec<=r['rel']<sec+1];rs=[r for r in rr if r['kind']=='sender_rate'];ps=[r for r in rr if r['kind']=='capacity_probe']
  bins.append(dict(second=sec,complete_bodies=sum(sec<=f[0]-lo<sec+1 for f in frames),branches=dict(collections.Counter(r['d'][1]&255 for r in rr if r['kind']=='screen_choice')),refreshes=sum(r['kind']=='screen_refresh' for r in rr),rates=stat([r['d'][3] for r in rs if r['d'][3]>=0]),probe_util=stat([r['d'][2]/r['d'][1] for r in ps if r['d'][1]>0]),probes=dict(collections.Counter(probe_class(r) for r in ps))))
 details.append(dict(**row,branches_window=branches,branch30_lifetime_snapshots=[screen(r) for r in b30],first_branch20_lifetime=first20,pty_output_P_ms=pstat,pty_tail_nonempty=sum((r['d'][3]>>1)>0 for r in papp),pty_parser_not_ground=sum(not(r['d'][3]&1) for r in papp),pty_apply_count=len(papp),animation_count=stat([r['d'][7] for r in papp]),refresh_interval_ms=refinterval,refresh_update_len=stat([r['d'][5] for r in rfs]),refresh_gen_ids=sorted(set(r['d'][1] for r in rfs)),refresh_kinds=dict(collections.Counter(r['d'][7] for r in rfs)),slow_rate_Bps=stat([r['d'][3] for r in rates if r['d'][3]>=0]),capacity_Bps=stat([r['d'][1] for r in rates if r['d'][1]>=0]),floor_ms=stat([r['d'][4]/1000 for r in rates if r['d'][4]>=0]),probe_utilization=stat([r['d'][2]/r['d'][1] for r in probes]),probe_status=probehist,poll_requested_ms=stat([r['d'][3] for r in polls]),poll_overshoot_ms=stat([max(0,(r['d'][1]-r['d'][0])/1e6-r['d'][3]) for r in polls]),post_poll_lock_ms=stat([(r['d'][2]-r['d'][1])/1e6 for r in polls]),timer_screen_binding=sum(bool(r['d'][3]&8) for r in timers),timer_masks=dict(collections.Counter(r['d'][3] for r in timers)),client_pending_gate_counts=gatehist,per_second=bins,largest_body_gaps=top))
 rows.append(row)
assert len(proofs)==24 and len(details)==12 and sum(r['branch30_window'] for r in details)==0 and sum(r['branch30_actual_vs_baseline_differences'] for r in details)==0
result=dict(study=str(S),summary_sha256=sha(S/'probe-summary.json'),functional_valid=X['functional_valid'],performance_eligible=X['performance_eligible'],valid=X['valid'],host_rejected=X['host_rejected'],jobs=len(rows),trace_endpoints=len(proofs),body_total=sum(r['body'] for r in rows),header_total=sum(r['header'] for r in rows),branch30_lifetime_total=sum(r['branch30_lifetime'] for r in rows if 'branch30_lifetime' in r),branch30_window_total=0,branch30_actual_vs_baseline_changes=0,trace_proofs=proofs,metrics=rows,rsh_details=details,limitations=['No emulator or capture replay; existing incomplete-state counts have no retained per-state timestamps, so exact partial-state timeline is not reconstructed. Body gaps and matched frame IDs are preserved; client and server event neighborhoods are temporal coincidence, not a causal join.','PTY chunks/scalar event columns cannot identify the first unknown complete producer frame or prove full-frame ownership.','Nearest screen_choice snapshots have no caller identity; branch counts are recorded helper outcomes, not unique selected deadlines.','Recorded capacity/slow-rate/probes are controller estimates, not proof of physical spare capacity or continuous demand.','Integer rate/took truncation is handled by sufficient pass/fail intervals; no inferred gain or kernel send is claimed from a marker alone.','This new functional trace cannot retrospectively establish untraced51 cause or performance/no-regression acceptance.','Resource counters are not remeasured; host rejection remains context and functional performance eligibility stays false.'])
result['script_sha256']=sha(Path(__file__))
(N/'lowbw52-mechanism-audit.json').write_text(json.dumps(result,indent=2)+'\n')
fields0=list(rows[0])+sorted(set().union(*(set(r) for r in rows))-set(rows[0]))
with (N/'lowbw52-metrics.csv').open('w',newline='') as f:
 w=csv.DictWriter(f,fieldnames=fields0);w.writeheader();w.writerows(rows)
print(json.dumps({k:result[k] for k in ['jobs','trace_endpoints','body_total','header_total','branch30_lifetime_total','branch30_window_total','branch30_actual_vs_baseline_changes','functional_valid','performance_eligible','host_rejected']}))
for r in details:
 if r['scene']=='bandwidth':print(r['iteration'],r['variant'],'branch',r['branches_window'],'Pms',r['pty_output_P_ms'],'refint',r['refresh_interval_ms'],'probe',r['probe_status'],'client',r['client_pending_gate_counts'])
