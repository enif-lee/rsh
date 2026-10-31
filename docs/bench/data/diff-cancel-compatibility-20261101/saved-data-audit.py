"""Independent saved-evidence arithmetic/raw source audit; no terminal emulation or workloads."""
import base64,collections,csv,datetime,hashlib,json,math,re,statistics,struct
from pathlib import Path
from zoneinfo import ZoneInfo
S=Path('/Users/ed/personal/rsh/benchmark/lab/out/lowtail-research-20261031')
N=Path(__file__).parent
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
MAIN='d45fc85fef7e2bb9b314ad3cdd68f3af24022d344911daa02e43f28860f41a7a'
CAND='b63aeed9fc8af5e758b136ba5dd9c3c6438ac46241f29a66c086962269057d11'
ID=('batch','net','scene','iteration','variant','tool','slot','binary_sha256','artifact_variant','observer_mode','topology','comparison_protocol')
FPS=('fps','low1_fps','low5_fps');RES=('kbit_s','rss_client_kb','rss_server_kb','cpu_client','cpu_server')
for number,name,schema,expected_bodies in [(36,'36-diff-cancel-compatibility',2,774)]:
 R=S/name;s=json.loads((R/'probe-summary.json').read_text());env=s['environment'];jobs=s['results'];errors=[]
 def check(value,label):
  if not value:errors.append(label)
 key=lambda j:tuple(j.get(k) for k in ID)
 check(len(jobs)==8 and len(s['batches'])==2,'8 jobs/2 blocks')
 check(env['parallel']==4 and env['iterations']==2 and collections.Counter(map(key,jobs))==collections.Counter(map(key,env['expected_jobs'])),'exact4x2 planned identity')
 check(s['valid'] and s['data_integrity_valid'] and not s['invalid_reasons'] and s['performance_eligible'] is False and s['compatibility_validation_only'] is True and s['final_selection'] is None,'compatibility inputvalid/fullperformanceexcluded')
 hashes={}
 for filename,base in [('prepared-inputs.json',R),('harness/manifest.json',R/'harness'),('parent-input-identities.json',S)]:
  data=json.loads((R/filename).read_text());check(all(sha(base/n)==v for n,v in data.items()),filename);hashes[filename]=len(data)
 a=json.loads((R/'analysis-manifest.json').read_text());check(all(sha(R/a['source']/n)==v and sha(R/'harness'/n)==v for n,v in a['sha256'].items()),'analysis frozen sources');hashes['analysis']=len(a['sha256'])
 check(env['driver_sha256']==sha(R/'run.py') and env['schedule_sha256']==sha(R/'schedule.json'),'actual driver/schedule')
 platform=json.loads((R/'runtime-platform.json').read_text());check(len(platform['endpoints'])==8 and all(p['image_id']==platform['image_id'] for p in platform['endpoints']),'eight runtime images')
 startup=[json.loads(l) for l in (R/'preflight-host.jsonl').read_text().splitlines()];startcpu=[x['user_pct']+x['system_pct'] for x in startup];check(startcpu[-1]<30 and all(v>=30 for v in startcpu[:-1]),'first startup CPU admission')
 metrics=[];phases=[];source_hashes={};capture_details=[];source_records=payload_bytes=pending=joined_pending=bodies=network_count=endpoint_count=live_count=0;source_details=[]
 for tag,net,scene,iteration in env['schedule']:
  group=[j for j in jobs if j['batch']==tag];batch=json.loads((R/tag/'batch.json').read_text());raw={j['slot']:j for j in batch['results']};ends=batch['window_closed_notifications'];closed=batch['all_windows_closed_epoch']
  check(len(group)==4 and set(ends)=={'0','1','2','3'} and closed==max(ends.values()),'four closure fence '+tag)
  q=[json.loads(l) for l in (R/(tag+'-quiet.jsonl')).read_text().splitlines()];cpus=[x['user_pct']+x['system_pct'] for x in q]
  status=json.loads((R/(tag+'-host-status.json')).read_text());host=[json.loads(l) for l in (R/(tag+'-host.jsonl')).read_text().splitlines()];totals=[x['user_pct']+x['system_pct'] for x in host]
  check(cpus[-1]<30 and all(v>=30 for v in cpus[:-1]) and q[-1]['at']<=status['phase_start']<min(j['window_start'] for j in group),'first qualifying CPU before window '+tag)
  check(status['accepted'] and not status['reasons'] and len(host)>=2,'host metadata accepted '+tag)
  check(status['total_cpu_pct']==dict(max=max(totals),mean=sum(totals)/len(totals),samples_ge30=sum(v>=30 for v in totals)),'runtime CPU context arithmetic '+tag)
  check(status['max_external_process_cpu_pct']==max(x['max_external_process_cpu_pct'] for x in host),'runtime process context '+tag)
  check(all(0<=b['at']-a['at']<=15 for a,b in zip(host,host[1:])) and all(0<=x['at']-x['began']<=15 for x in host) and abs(status['phase_end']-status['phase_start']-status['monotonic_elapsed'])<=2,'runtime clock metadata '+tag)
  phases.append(dict(batch=tag,admission_cpu=cpus[-1],admission_attempts=len(q),runtime_cpu=status['total_cpu_pct'],external_max=status['max_external_process_cpu_pct']))
  network=json.loads((R/tag/'network-setup.json').read_text());check(len(network)==4,'four prepared networks '+tag)
  for setup in network:
   j=next(j for j in group if j['slot']==setup['slot']);check(setup['net']==net and setup['variant']==j['variant'] and setup['binary_sha256']==j['binary_sha256'] and setup['configured_before_agent_launch'],'network job identity '+tag)
   for endpoint in setup['endpoints']:
    qs=[x for x in endpoint['qdiscs'] if x['kind']=='netem'];drop=j['tool']=='rsh-tcp' and endpoint['container'].endswith('-srv')
    check(endpoint['udp_dropped']==drop and ('-A INPUT -p udp -j DROP' in endpoint['iptables'])==drop,'strict UDP forcing '+tag)
    if net=='lan':check(not qs,'LAN no netem '+tag)
    else:
     check(len(qs)==1,'one netem '+tag)
     if qs:
      options=qs[0]['options'];delay=.15 if net=='highlat' else .05;rate={'lowbw':125000,'verylow':32000,'slow64k':8000}.get(net) if endpoint['container'].endswith('-srv') else None
      check(abs(options['delay']['delay']-delay)<1e-6 and (options.get('rate',{}).get('rate')==rate if rate else not options.get('rate')),'exact netem delay/rate '+tag)
    network_count+=1
  before=json.loads((R/tag/'identities-before.json').read_text());after=json.loads((R/tag/'identities-after.json').read_text());deploy=json.loads((R/(tag+'-deployment.json')).read_text())
  check(deploy['old_rsh_stopped_before_copy'] and len(deploy['no_agents'])==8 and all(x['active_agents']==0 for x in deploy['no_agents']),'fresh stop-before-deploy '+tag)
  for ids in (before,after):
   check(len(ids)==8,'eight identities '+tag)
   for p in ids:
    j=next(j for j in group if j['slot']==p['slot']);expected=CAND if j['variant']=='candidate' else MAIN
    check(p['sha256']==p['expected_sha256']==j['binary_sha256']==expected and p['variant']==j['variant'] and p['artifact_variant']==j['artifact_variant'],'actual artifact identity '+tag);endpoint_count+=1
    if p['side']=='srv':check(p['live_server']['exe_sha256']==expected and p['live_server']['listening_2222'],'live executable/listener '+tag);live_count+=1
  for p in before:
   if p['side']=='srv':
    other=next(x for x in after if x['container']==p['container']);check(all(p['live_server'][k]==other['live_server'][k] for k in ('pid','starttime','exe_sha256')),'same server lifetime '+tag)
  for j in group:
   label=tag+'/'+str(j['slot'])
   capture=R/j['capture'];capture_bytes=capture.read_bytes();offset=0;read_count=0;previous_at=None;payload=[]
   while offset<len(capture_bytes):
    check(offset+12<=len(capture_bytes),'capture header complete '+label)
    at,size=struct.unpack('!dI',capture_bytes[offset:offset+12]);offset+=12;block=capture_bytes[offset:offset+size];offset+=size
    check(len(block)==size and math.isfinite(at) and (previous_at is None or at>=previous_at),'capture read/time integrity '+label);previous_at=at;payload.append(block);read_count+=1
   check(read_count==j['data_integrity']['capture_reads'],'raw capture read count '+label)
   wire=b''.join(payload);signature=b'\x18\x1b[0m\x1b(B\x0f\x1b[?6l\x1b[4l\x1b[?7l'
   capture_details.append(dict(batch=tag,slot=j['slot'],variant=j['variant'],capture=j['capture'],sha256=hashlib.sha256(capture_bytes).hexdigest(),bytes=len(capture_bytes),read_records=read_count,tty_bytes=len(wire),can_delta_prefix_count=wire.count(signature),known_old_nested_csi_sgr_count=wire.count(b'\x1b[\x1b[0m'),can_after_partial_csi_count=len(re.findall(rb'\x1b\[[0-9:;?]*\x18',wire))))
   frames=j['complete_frames'];body=j['complete_body'];lo,hi=j['window_start'],j['window_end'];joins=j['source_joined_complete_frames']
   check(j['data_integrity']['valid'] and j['reason_trace'] is False and not j.get('endpoint_trace'),'input valid/untraced '+label)
   check(hi<=ends[str(j['slot'])] and len(frames)==body['frames']==j['source']['joined_complete_count']==len(joins),'window/body/join counts '+label)
   check(all(lo<=f[0]<=hi and f[3]==scene for f in frames) and all(a[1]<b[1] and a[0]<=b[0] for a,b in zip(frames,frames[1:])),'complete order/window '+label)
   gaps=sorted((b[0]-a[0])*1000 for a,b in zip(frames,frames[1:]));low=lambda q:None if len(gaps)<10 else round(1000/max(gaps[math.ceil(len(gaps)*q)-1],1e-3),1)
   ages=sorted((f[0]-f[2])*1000 for f in frames);times=[lo]+[f[0] for f in frames]+[hi]
   calc=dict(frames=len(frames),fps=round(len(frames)/(hi-lo),1),low1_fps=low(.99),low5_fps=low(.95),age_ms=round(statistics.median(ages)) if ages else None,age_p95_ms=round(ages[math.ceil(len(ages)*.95)-1]) if len(ages)>=20 else None,max_gap_ms=round(max(b-a for a,b in zip(times,times[1:]))*1000))
   check(all(body[k]==v for k,v in calc.items()),'body FPS/low/age/gap recalculation '+label)
   per_s=[sum(lo+k<=f[0]<lo+k+1 for f in frames) for k in range(int(hi-lo))]
   check(per_s==body['per_s'],'body persecond bins '+label)
   boundaries=dict(first_ms=(frames[0][0]-lo)*1000 if frames else (hi-lo)*1000,last_ms=(hi-frames[-1][0])*1000 if frames else (hi-lo)*1000,adjacent_max_ms=max(gaps) if gaps else None)
   check(abs(sum(j['visible_state'].values())-100)<1e-7,'visiblestate percentage sum '+label)

   check(all(raw[j['slot']][k]==j[k] for k in RES),'raw resource transfer '+label)
   metrics.append(dict(batch=tag,net=net,scene=scene,iteration=iteration,variant=j['variant'],slot=j['slot'],online_fps=j['fps'],uniform_header_frames=j['uniform_header']['frames'],uniform_header_fps=j['uniform_header']['fps'],visible_state=j['visible_state'],per_s=per_s,boundary_gaps=boundaries,**calc,**{k:j[k] for k in RES}));bodies+=len(frames)
   path=R/j['source_ledger'];data=path.read_bytes();source_hashes[j['source_ledger']]=hashlib.sha256(data).hexdigest();ledger=json.loads(data);source=ledger['frames'];lookup={};headers={};lower=[];upper=[];written=[];bytes_here=pending_here=0;previous=None
   check(ledger['schema']==schema and ledger['dropped']==0 and len(source)==j['source']['generated'] and not j['source']['missing_oracle_frame_ids'],'source schema/count/drop '+label)
   check(ledger['dump_begin_epoch']>=closed,'post-fence source dump '+label)
   if schema==2:
    c=j['collector'];signal=c['signal'];ready=signal['ready'];flag=signal['flush_flag'];expected={k:j.get(k) for k in ('source_token','slot','tool','variant','iteration','binary_sha256','scene','cols','rows','scene_fps','duration')}
    check(c==json.loads((R/j['collector_sidecar']).read_text())==raw[j['slot']]['collector'],'collector sidecar '+label)
    check(c['job']==ready['job']==flag['job']==ledger['job']==expected and ledger['source_token']==j['source_token'],'nonce job identity '+label)
    proc=ready['identity'];check(proc==flag['identity']==ledger['source_identity'] and proc['uid']==1001 and proc['pid']>0 and proc['starttime_ticks']>0 and proc['cmdline']==['python3','/tmp/scene.py',scene,'8','60'],'source process identity '+label)
    check(c['closure_proof']==flag['closure_proof']==ledger['flush_flag']['closure_proof'] and c['closure_proof']['window_closed_notifications']==ends and c['closure_proof']['all_windows_closed_epoch']==closed,'actual four-window proof '+label)
    check(c['signal_sent'] and c['initial_deadline_met'] and c['deadline_seconds']==5 and c['elapsed_wait_seconds']<5 and c['signal_request']['returncode']==0 and signal['pidfd_transaction'] and signal['signal_number']==ledger['dump_signal']==12,'pidfd and original 5s '+label)
    check(closed<=c['request_begin_epoch']<=signal['signal_epoch'] and flag['flag_created_epoch']<=ledger['dump_begin_epoch'],'signal ordering '+label)
   for f in source:
    n,draw,begin,wall=(f[k] for k in ('n','draw_begin_ns','write_begin_ns','wall_ns'));end=f['write_end_ns'];done=f.get('write_completed',True)
    check(all(type(v) is int for v in (n,draw,begin,wall)) and n>=0 and wall>0 and draw<=begin,'source timestamps '+label)
    check(previous is None or n>previous[0] and begin>=previous[1] and wall>=previous[2],'source ordered '+label);previous=(n,begin,wall)
    if schema==2:
     m0,m1=f['clock_mono_before_ns'],f['clock_mono_after_ns'];check(draw<=m0<=m1<=begin,'prewrite bracket '+label);lower.append(wall-m1);upper.append(wall-m0)
    else:lower.append(wall-end);upper.append(wall-begin)
    if done is True:check(type(end) is int and begin<=end,'real completed end '+label);written.append((end-begin)/1e6)
    else:check(done is False and end is None,'pending end null '+label);pending_here+=1
    payload=base64.b64decode(f['payload_b64'],validate=True);bytes_here+=len(payload);h=re.search(rb'FRAME (\d+) T=(\d+) (\w+)',payload)
    check(h is not None and int(h[1])==n and int(h[2])>0 and h[3].decode()==scene,'raw payload header '+label)
    if h:headers[n]=(int(h[1]),int(h[2]),h[3].decode())
    lookup[n]=f
   check(max(lower)<=min(upper) and bytes_here==ledger['bytes']==j['source']['bytes'],'common clock/retained bytes '+label)
   if written:check(sorted(written)[math.ceil(len(written)*.99)-1]==j['source']['write_ms']['p99'],'completed write p99 '+label)
   if schema==2:check(pending_here==j['source_quality']['in_progress_writes'] and len(written)==j['source_quality']['completed_writes'],'source completed/pending counts '+label)
   for observed,joined in zip(frames,joins):
    at,n,made,sc=observed;f=lookup[n];check(headers[n]==(n,int(round(made*1000)),sc) and (joined['n'],joined['complete_read_epoch'])==(n,at),'raw source/body header join '+label)
    if schema==2:
     project=lambda t:[(f['wall_ns']+t-f['clock_mono_after_ns'])/1e9,(f['wall_ns']+t-f['clock_mono_before_ns'])/1e9]
     check(joined['source_header_matches'] and joined['source_timing_valid'] and joined['source_prewrite_epoch_interval']==project(f['write_begin_ns']),'prewrite join '+label)
     interval=project(f['write_end_ns']) if f['write_completed'] else None
     if not f['write_completed']:joined_pending+=1
    else:interval=[f['wall_ns']/1e9,f['wall_ns']/1e9+(f['write_end_ns']-f['write_begin_ns'])/1e9]
    check(joined['source_write_end_epoch_interval']==interval and joined['after_source_write_ms_interval']==([(at-interval[1])*1000,(at-interval[0])*1000] if interval else None),'write end projection '+label)
   source_records+=len(source);payload_bytes+=bytes_here;pending+=pending_here;source_details.append(dict(path=j['source_ledger'],records=len(source),pending=pending_here,payload_bytes=bytes_here,offset_intersection_ns=min(upper)-max(lower)))
 with (R/'acceptance-per-rep.csv').open() as stream:cs=list(csv.DictReader(stream))
 check(len(cs)==8,'8 CSV rows');cm={(j['batch'],int(j['slot'])):j for j in cs}
 for j in metrics:
  for k in (*FPS,*RES,'frames','age_ms','age_p95_ms','max_gap_ms'):
   v=cm[(j['batch'],j['slot'])][k];check((None if v=='' else float(v))==j[k],'CSV independent metric '+j['batch']+'/'+str(j['slot'])+'/'+k)
 comparisons=[]
 for tag,*_ in env['schedule']:
  role={j['variant']:j for j in metrics if j['batch']==tag};c=role['candidate'];deficits=[]
  for control in ('ssh','main-A','main-B'):
   for k in FPS:
    x,y=c[k],role[control][k]
    if x is not None and y is not None and x<y:deficits.append(dict(control=control,metric=k,candidate=x,reference=y))
  comparisons.append(dict(batch=tag,roles=role,observed_fps_deficits=deficits))
 check(bodies==expected_bodies,'expected aggregate actual bodies')
 windows=[v for j in jobs for v in (j['window_start'],j['window_end'])];fmt=lambda at:datetime.datetime.fromtimestamp(at,ZoneInfo('Asia/Seoul')).isoformat()
 out=dict(valid_saved_audit=not errors,errors=errors,study=number,performance_eligible=False,compatibility_validation_only=True,final_selection=None,reference_scope='main-A/main-B are duplicated oldfar d45, notactualmain',geometry_method='new36childfd0 SET/GETassertbeforeexec; sourceflow andsuccessfulcapture only, notretainednativeioctl result orlaterproductgeometry; noabsolute34pool',capture_details=capture_details,jobs=len(jobs),blocks=len(phases),body_frames=bodies,source_records=source_records,retained_payload_bytes=payload_bytes,source_json_bytes=sum((R/n).stat().st_size for n in source_hashes),source_schema=schema,pending_writes_saved=pending if schema==2 else None,pending_body_joins=joined_pending if schema==2 else None,zero_body_jobs=sum(j['frames']==0 for j in metrics),missing_low_jobs=[dict(batch=j['batch'],variant=j['variant'],frames=j['frames']) for j in metrics if j['low1_fps'] is None or j['low5_fps'] is None],hash_scopes=hashes,source_sha256=source_hashes,strict_network_checks=network_count,endpoint_file_checks=endpoint_count,live_server_checks=live_count,source_details=source_details,startup=dict(attempts=len(startup),admission_cpu=startcpu[-1]),phases=phases,comparisons=comparisons,actual_window_kst=[fmt(min(windows)),fmt(max(windows))],resource_units=dict(kbit_s='whole client non-loopback received bytes / scene seconds, decimal kbit/s; not bidirectional sum',rss='midpoint process-name sums in KiB; not peak/session total',cpu='transport process/thread scheduler percent of one core; source/observer Python excluded'),limits=['No independent whole8terminal/emulator replay; residue inference from savedbody/headercoherence and bounded rawsignatures, notGPU orallterminal proof','KnownoldnestedCSIabsence inoldfar controls means prior34failure maynotbereproduced, so donotattributebodycountchange acrossstudies toCAN alone','Raw transport RX/CPU counter endpoints and RSS samples are not separately saved: resource verification covers raw-result-to-summary-to-CSV identity and formula/unit source review, not independent counter remeasurement','No independent terminal emulation/full capture replay','Data validity and artifact isolation do not establish candidate acceptance or causal mechanism','Age is conditional on completed body frames, not input/held-screen/GPU latency','36 has newpreexecgeometry method; cannotabsolutelypoolwith34 orclaimfullperformancepromotion','CPU30 is admission only; runtime CPU/process values are retained context; no P-core affinity proof'],script_sha256=sha(Path(__file__)))
 (N/'compat36-actual-audit.json').write_text(json.dumps(out,indent=2)+'\n')
 print(json.dumps({k:out[k] for k in ('study','valid_saved_audit','errors','jobs','blocks','body_frames','source_records','retained_payload_bytes','pending_writes_saved','pending_body_joins','zero_body_jobs','missing_low_jobs','actual_window_kst','startup')}))
