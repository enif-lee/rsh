"""Portable compact arithmetic/provenance checks; no raw body or terminal replay."""
from pathlib import Path
import collections,csv,gzip,hashlib,json,math,re,statistics
HERE=Path(__file__).resolve().parent
read=lambda n:json.loads((HERE/n).read_text())
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
config=read('archive-config.json');expected=config['expected_jobs'];blocks=config['expected_blocks'];moving=config['nets']==['moving']
if (HERE/'manifest.json').exists():
    manifest=read('manifest.json')
    for name,digest in manifest.items():assert sha(HERE/name)==digest,name
    files={str(p.relative_to(HERE)) for p in HERE.rglob('*') if p.is_file() and p.name!='manifest.json'}
    assert files=={name for name in manifest if not name.startswith('../')},'archive inventory'
data=read('core-results.json');results=data['results'];rows=list(csv.DictReader((HERE/'core-metrics.csv').open()))
key=lambda r:(r['net'],r['scene'],int(r['iteration']),r['variant'])
by_key={key(r):r for r in rows};assert len(rows)==len(by_key)==len(results)==expected
assert {key(r) for r in results}==set(by_key)=={(n,s,i,r) for n in config['nets'] for s in ('compress','bandwidth') for i in (0,1) for r in ('ssh','main-A','main-B','candidate')}
assert data['parallel']==4 and data['iterations']==2 and data['valid'] and data['data_integrity_valid']
assert data['performance_eligible'] is config['performance_eligible'] and data['final_selection'] is None and data['candidate_selected'] is False
assert data['focused_performance_screen'] is (not moving)
percentile=lambda xs,q:xs[math.ceil(len(xs)*q)-1]
def stats(frames,lo,hi):
    assert hi>lo and all(lo<=f[0]<=hi for f in frames)
    assert all(a[0]<=b[0] and a[1]<b[1] for a,b in zip(frames,frames[1:]))
    times=[f[0] for f in frames];gaps=sorted((b-a)*1000 for a,b in zip(times,times[1:]));ages=sorted((f[0]-f[2])*1000 for f in frames);edges=[lo]+times+[hi]
    return dict(frames=len(frames),fps=round(len(frames)/(hi-lo),1),low1_fps=round(1000/max(percentile(gaps,.99),.001),1) if len(gaps)>=10 else None,low5_fps=round(1000/max(percentile(gaps,.95),.001),1) if len(gaps)>=10 else None,age_ms=round(statistics.median(ages)) if ages else None,age_p95_ms=round(percentile(ages,.95)) if len(ages)>=20 else None,max_gap_ms=round(max(b-a for a,b in zip(edges,edges[1:]))*1000),per_s=[sum(lo+k<=f[0]<lo+k+1 for f in frames) for k in range(int(hi-lo))])
missing=0
for result in results:
    frames=result['complete_frames'];lo,hi=result['window_start'],result['window_end'];calculated=stats(frames,lo,hi)
    assert all(result['complete_body'][k]==v for k,v in calculated.items()),key(result)
    assert all(f[3]==result['scene'] and math.isfinite(f[2]) for f in frames)
    csvrow=by_key[key(result)]
    for name,column in [('frames','frames'),('fps','fps'),('low1_fps','low1'),('low5_fps','low5'),('age_ms','age'),('age_p95_ms','age95'),('kbit_s','kbit'),('rss_client_kb','rssc'),('rss_server_kb','rsss'),('cpu_client','cpuc'),('cpu_server','cpus')]:
        want=calculated[name] if name in calculated else result[name]
        if want is None:assert csvrow[column]=='';missing+=1
        else:assert float(csvrow[column])==want,(key(result),name)
    coverage=result['completion_coverage'];tail=hi-frames[-1][0] if frames else hi-lo
    assert coverage['adjacent_gap_samples']==max(0,len(frames)-1) and coverage['no_new_complete_tail_s']==tail
    assert int(csvrow['headerframes'])==result['uniform_header']['frames'] and float(csvrow['headerfps'])==result['uniform_header']['fps']
    assert int(csvrow['maxgap'])==calculated['max_gap_ms'] and int(csvrow['adjacentgaps'])==coverage['adjacent_gap_samples'] and float(csvrow['tail_s'])==tail
    assert float(csvrow['incomplete_pct'])==result['visible_state']['incomplete_pct'] and abs(sum(result['visible_state'].values())-100)<1e-7
    assert result['data_integrity']['valid'] and result['source_quality']['valid'] and result['source']['dropped']==0 and not result['source']['missing_oracle_frame_ids']
    source=result['source'];collector=result['collector'];signal=collector['signal'];ready=signal['ready'];flag=signal['flush_flag']
    assert collector['job']==ready['job']==flag['job']==source['job'] and source['source_token']==source['job']['source_token']
    assert all(source['job'][k]==result[k] for k in ('slot','tool','variant','iteration','binary_sha256','scene','cols','rows','scene_fps','duration'))
    assert source['job']['source_token'] and source['job']['source_token']==source['flush_flag']['job']['source_token']
    assert ready['identity']==flag['identity']==source['source_identity'] and ready['identity']['pid']>0 and ready['identity']['uid']==1001
    assert collector['closure_proof']==flag['closure_proof']==source['flush_flag']['closure_proof']
    proof=collector['closure_proof'];assert set(proof['window_closed_notifications'])=={'0','1','2','3'}
    assert hi<=proof['all_windows_closed_epoch']==max(proof['window_closed_notifications'].values())<=collector['request_begin_epoch']<=signal['signal_epoch']
    assert source['dump_begin_epoch']>=proof['all_windows_closed_epoch'] and collector['initial_deadline_met'] and collector['deadline_seconds']==5 and collector['elapsed_wait_seconds']<5
    assert signal['pidfd_transaction'] and signal['signal_number']==source['dump_signal']==12 and collector['signal_request']['returncode']==0
assert len({r['source']['source_token'] for r in results})==expected
for tag in {r['batch'] for r in results}:
    group=[r for r in results if r['batch']==tag];assert len(group)==4 and {r['slot'] for r in group}=={0,1,2,3}
    proofs=[r['collector']['closure_proof'] for r in group];assert all((p['window_closed_notifications'],p['all_windows_closed_epoch'])==(proofs[0]['window_closed_notifications'],proofs[0]['all_windows_closed_epoch']) for p in proofs)
    assert all(r['window_end']<=proofs[0]['window_closed_notifications'][str(r['slot'])] for r in group)
independent=read('saved-data-audit.json');audit=read('source-header-join-audit.json');extraction=read('extraction-provenance.json')
assert independent['valid_saved_audit'] and not independent['errors'] and independent['jobs']==expected and independent['blocks']==blocks
assert independent['performance_eligible'] is config['performance_eligible'] and independent['final_selection'] is None
assert independent['saved_summary_sha256']==data['original_summary_sha256']==audit['summary_sha256']==extraction['original_summary_sha256']
assert sha(HERE/'saved-data-audit.json')==audit['independent_audit_sha256']==extraction['independent_audit_sha256']
assert audit['valid'] and independent['body_frames']==audit['complete_body_headers_checked']==sum(len(r['complete_frames']) for r in results)
assert independent['source_records']==audit['source_frames']==sum(r['source']['generated'] for r in results)
assert independent['pending_writes_saved']==audit['pending_writes']==sum(r['source_quality']['in_progress_writes'] for r in results)
assert independent['pending_body_joins']==audit['pending_body_joins'] and independent['retained_payload_bytes']==sum(r['source']['bytes'] for r in results)
assert extraction['current_raw_files_hashed']==extraction['raw_payloads_reparsed']==extraction['terminal_replays']==extraction['new_benchmarks']==0
assert len(data['original_raw_files_sha256'])==2*expected and extraction['source_and_capture_hashes_imported']==2*expected
for name,digest in independent['source_sha256'].items():assert data['original_raw_files_sha256'][name]==digest
assert len(independent['capture_details'])==expected
for capture in independent['capture_details']:assert data['original_raw_files_sha256'][capture['capture']]==capture['sha256']
for original,entry in read('source-imports.json').items():
    blob=(HERE/entry['public']).read_bytes();blob=gzip.decompress(blob) if entry['public'].endswith('.gz') else blob
    assert hashlib.sha256(blob).hexdigest()==entry['original_sha256'],original
for name,digest in read('raw-build-log-sha256.json').items():assert hashlib.sha256(gzip.decompress((HERE/(name+'.gz')).read_bytes())).hexdigest()==digest
paired=read('paired-build-provenance.json');all_artifacts={a['variant']:a for a in paired['artifacts']}
assert set(all_artifacts)=={'main-A','main-B','candidate'} and all_artifacts['main-A']['sha256']==all_artifacts['main-B']['sha256']
artifacts={'baseline':all_artifacts['main-A'],'candidate':all_artifacts['candidate']}
expected_artifacts={'baseline':('295f81d54ea4','fb04386fdfe25f25f1bfd0354f11ba7a01c954525a11634ae35caafd46c2eec4',110,140,'fps-combined-pty-account','fps-combined-pty-account'),'candidate':('ea6c0df76a70','e34a2fc46a1b10ec69cebb155a50958a52b70b2aa86205ff5458ad8f02a7aa45',113,143,'48-tcp-cadence-margin','cadence48')}
for role,(bid,digest,lib,total,source_prefix,log_prefix) in expected_artifacts.items():
    artifact=artifacts[role];assert artifact['source_build_id']==bid and artifact['sha256']==digest and artifact['source_build_verified'] and artifact['source_manifest_unchanged']
    assert sha(HERE/(source_prefix+'-source-manifest.json'))==artifact['source_manifest_sha256'] and sha(HERE/(source_prefix+'-source-full-manifest.json'))==artifact['source_full_manifest_sha256']
    assert artifact['functional_total_passed']==total and artifact['cargo_root_artifacts']
    assert all(x['fresh'] is False for group in artifact['cargo_root_artifacts'].values() for x in group)
    for suite,passed,ignored in [('library',lib,2),('binary_cli',4,0),('session',25,0),('pty_concurrency',1,0)]:
        meta=artifact['functional_tests'][suite];log=gzip.decompress((HERE/(log_prefix+'-'+suite+'-tests.log.gz')).read_bytes())
        assert meta['passed']==passed and meta['ignored']==ignored and hashlib.sha256(log).hexdigest()==meta['log_sha256']
        assert f'test result: ok. {passed} passed; 0 failed; {ignored} ignored'.encode() in log
assert paired['same_environment_pair_verified'] is False
assert paired['candidate_actual_build']['flags_asserted_empty_in_build_script'] and paired['candidate_actual_build']['compiler_image']
control=paired['negative_control_results'];assert control['source_build_id']=='72e051a3f7a2' and control['passed']==2 and control['failed']==1 and control['test_exit']==101 and control['expected_failure_observed'] and control['sourceguard_verified']
assert control['failed_test']=='tcp_cadence_margin_scales_raw_step_before_existing_bounds'
log=gzip.decompress((HERE/'cadence48-control-library-tests.log.gz').read_bytes());assert b'test result: FAILED. 2 passed; 1 failed; 0 ignored' in log
for result in results:assert result['binary_sha256']==artifacts['candidate' if result['variant']=='candidate' else 'baseline']['sha256']
identities=read('artifact-identities.json');assert len(identities)==3
for a in identities:assert (a['source_build_id'],a['sha256'])==expected_artifacts['candidate' if a['variant']=='candidate' else 'baseline'][:2]
controller=read('controller-source.json');assert (controller['reference'],controller['candidate'])==('295f81d54ea4','ea6c0df76a70')
ref=read('fps-combined-pty-account-source-manifest.json');candidate=read('48-tcp-cadence-margin-source-manifest.json')
for name,values in controller['files'].items():assert values['reference_sha256']==ref[name] and values['candidate_sha256']==candidate[name]
assert ref['src/proto.rs']==candidate['src/proto.rs'] and ref['src/server.rs']!=candidate['src/server.rs']
cpu=read('cpu30-summary.json');assert len(cpu['admission'])==blocks+1 and all(r['qualifying_cpu_pct']<30 for r in cpu['admission'])
assert len(cpu['runtime'])==blocks and all(r['accepted'] and not r['reasons'] for r in cpu['runtime'])
assert cpu['max_runtime_cpu_pct']==max(r['total_cpu_pct']['max'] for r in cpu['runtime'])
for r in cpu['runtime']:
    saved=data['host_statuses'][r['phase']+'-host-status'];assert r['total_cpu_pct']==saved['total_cpu_pct'] and r['max_external_process_cpu_pct']==saved['max_external_process_cpu_pct']
    assert saved['accepted'] and not saved['reasons']
native_outages=0
if moving:
    derived=read('moving-derived.json');assert derived['performance_eligible'] is False and derived['recovery']==independent['moving_recovery_verified'] and derived['native_blocks']==independent['native_timing_blocks']
    nativebuild=read('native-build.json');worker_sha='c5e4c3d1ca5f8d347be6bb1edffedec1b3873e453508007ed538910b6ef61f87'
    assert nativebuild['worker_sha256']==worker_sha and sha(HERE/'native-netem.c')==nativebuild['source_sha256']
    imports=read('native-files.json');assert len(imports)==blocks
    for entry in imports:
        tag=entry['batch'];folder=HERE/'native'/tag
        for name,digest in entry['files'].items():assert sha(folder/name)==digest
        network=json.loads((folder/'moving-network.json').read_text());ready=json.loads((folder/'moving-ready.json').read_text());armed=json.loads((folder/'moving-armed.json').read_text())
        lo,hi,epoch=network['start'],network['end'],network['epoch_ns'];assert lo==epoch/1e9 and hi==lo+30 and ready['epoch_ns']==epoch
        assert network['coverage_valid'] and not network['errors'] and network['timing_fidelity_accepted'] is False and network['native_worker_sha256']==ready['native_worker_sha256']==worker_sha
        payload=(folder/'native-plan.txt').read_text();lines=payload.splitlines();assert lines[0]==f'MOVING35 {epoch} 30000000000 131' and lines[-1]=='RESTORE 35 0 8000 2000'
        assert hashlib.sha256(payload.encode()).hexdigest()==ready['native_plan_sha256']
        points=[[int(v) for v in line.split()] for line in lines[1:132]];assert len(points)==131
        assert points==[[round(r['offset_s']*1e9),r['delay_ms'],r['loss_pct'],r['down_kbit'],r['up_kbit']] for r in network['intended_trace']]
        workers=network['workers'];assert len(workers)==8 and {w['container'] for w in workers}=={f'rl-{i}-{side}' for i in range(4) for side in ('srv','cli')}
        for worker in workers:
            native=worker['native'];owner=worker['owner'];proof=worker['live_before'];raw=(folder/(worker['container']+'.stdout')).read_text().splitlines()
            assert worker['returncode']==0 and len(raw)==3 and raw[1]=='ready' and json.loads(raw[0])==owner and json.loads(raw[2])==native
            assert proof['identity']==owner and proof['sha256']==worker_sha and proof['exe']==worker['path'] and owner['uid']==0
            assert proof['cmdline']==[worker['path'],worker['device'],worker['direction']]
            assert native['coverage_valid'] and native['completed']==131 and native['error']==0 and native['stopped']==0 and native['mtu_before']==native['mtu_after']
            assert native['epoch_ns']==epoch and native['duration_ns']==30000000000 and len(native['records'])==131
            for seq,(point,row) in enumerate(zip(points,native['records']),1):
                assert [row[k] for k in ('offset_ns','delay_ms','loss_pct','down_kbit','up_kbit')]==point and row['seq']==seq and row['ack_error']==0
                assert row['wall_before_ns']<=row['wall_after_ns'] and row['mono_before_ns']<=row['mono_after_ns']
            restore=native['restore'];assert restore['seq']==132 and restore['ack_error']==0 and [restore[k] for k in ('offset_ns','delay_ms','loss_pct','down_kbit','up_kbit')]==[30000000000,35,0,8000,2000]
            assert worker['applied_at']==[r['wall_after_ns']/1e9 for r in native['records']]
        series=[w['native']['records'] for w in workers];skews=[(max(v[i]['wall_after_ns'] for v in series)-min(v[i]['wall_after_ns'] for v in series))/1e6 for i in range(131)]
        bounds=[];on=None
        for i,point in enumerate(points):
            if point[2]==100 and (i==0 or points[i-1][2]==0):on=i
            if on is not None and point[2]==0:
                individual=[dict(container=w['container'],lower_ms=(w['native']['records'][i]['wall_before_ns']-w['native']['records'][on]['wall_after_ns'])/1e6,upper_ms=(w['native']['records'][i]['wall_after_ns']-w['native']['records'][on]['wall_before_ns'])/1e6) for w in workers]
                bounds.append(dict(on_index=on,off_index=i,intended_ms=(point[0]-points[on][0])/1e6,workers=individual,common_overlap_lower_raw_ms=(min(v[i]['wall_before_ns'] for v in series)-max(v[on]['wall_after_ns'] for v in series))/1e6,common_overlap_upper_raw_ms=(min(v[i]['wall_after_ns'] for v in series)-max(v[on]['wall_before_ns'] for v in series))/1e6));on=None
        assert len(bounds)==7 and [b['intended_ms'] for b in bounds]==[6.,32.,120.,240.,1000.,50.,7.] and skews==network['point_skew_ms'] and bounds==network['outage_bounds']
        saved=next(b for b in derived['native_blocks'] if b['batch']==tag);assert saved['point_skew_ms']==skews and saved['outage_bounds']==bounds;native_outages+=len(bounds)
        assert ready['start']==armed['start']==lo and ready['end']==armed['end']==hi and armed['all_armed_confirmed_epoch']<lo
        assert set(ready['readiness'])==set(armed['sources'])=={'0','1','2','3'}
        cleanup=json.loads((folder/'native-cleanup.json').read_text());assert len(cleanup)==8 and all(r['controller_restore_returncode']==0 for r in cleanup)
        for result in [r for r in results if r['batch']==tag]:
            assert result['window_start']==result['moving_common_start']==lo and result['window_end']==result['moving_common_end']==hi
            assert result['moving_readiness']==ready['readiness'][str(result['slot'])]
            source_ready=armed['sources'][str(result['slot'])];assert source_ready==result['collector']['signal']['ready'] and source_ready['identity']==result['source']['source_identity'] and source_ready['ready_epoch']<lo
    for row in derived['recovery']:
        result=next(r for r in results if r['batch']==row['batch'] and r['slot']==row['slot']);frames=result['complete_frames'];lo,hi=result['window_start'],result['window_end'];edges=(0,6,10,14,17,18,23,30)
        intervals=[(lo+a,lo+b) for a,b in zip(edges,edges[1:])]+[(lo+i,lo+i+1) for i in range(30)];views=[p['complete_body'] for p in row['phases']]+row['per_second']
        assert len(views)==37
        for (begin,end),view in zip(intervals,views):
            selected=[f for f in frames if begin<=f[0] and (f[0]<end or end==hi and f[0]<=end)]
            assert all(view[k]==v for k,v in stats(selected,begin,end).items())
        for offset in (18,23):
            edge=lo+offset;found=next((f for f in frames if f[0]>=edge and f[2]>=edge),None);want=None if found is None else dict(frame=found,delay_ms=(found[0]-edge)*1000)
            assert row['first_new_body_after_edge'][str(offset)]==want
        assert row['final23to30_complete_bodies']==sum(f[0]>=lo+23 for f in frames) and row['final23to30_fresh_source_bodies']==sum(f[0]>=lo+23 and f[2]>=lo+23 for f in frames)
print(json.dumps(dict(cases=expected,complete_frames=sum(len(r['complete_frames']) for r in results),metric_and_csv_match=True,missing_values_preserved=missing,archived_build_logs_match=True,native_outage_bounds_checked=native_outages,raw_body_replay_performed=False,performance_eligible=config['performance_eligible'],final_selection=None)))
