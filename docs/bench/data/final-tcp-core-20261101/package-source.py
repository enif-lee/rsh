"""Adapted from the combined38/39 exporter; imports independent saved audits.

python3 package-source.py /path/to/lowtail-research-20261031
No benchmark, terminal replay or second raw-payload parse. Audit scripts and
raw captures/ledgers stay local; imported hashes are not new raw rehashes.
"""
from pathlib import Path
from datetime import datetime
from zoneinfo import ZoneInfo
import argparse,csv,difflib,gzip,hashlib,json,shutil

HERE=Path(__file__).resolve().parent
CONFIG=json.loads((HERE/'archive-config.json').read_text())
def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def save(name,value):(HERE/name).write_text(json.dumps(value,separators=(',',':'))+'\n')

def main(study):
    run=study/CONFIG['run'];notes=study.parents[3]/'research_notes'/'LAN 원거리 low 성능 연구'
    summary=json.loads((run/'probe-summary.json').read_text())
    audit_path=notes/(CONFIG['audit_stem']+'-actual-audit.json')
    audit=json.loads(audit_path.read_text())
    assert summary['valid'] and summary['data_integrity_valid'] and not summary['invalid_reasons']
    assert audit['valid_saved_audit'] and not audit['errors']
    assert audit['saved_summary_sha256']==sha(run/'probe-summary.json')
    assert summary['performance_eligible'] is CONFIG['performance_eligible']
    assert audit['performance_eligible'] is CONFIG['performance_eligible']
    assert len(summary['results'])==CONFIG['expected_jobs'] and len(summary['batches'])==CONFIG['expected_blocks']
    fields=('batch','net','scene','iteration','variant','slot','tool','duration','scene_fps','cols','rows','binary_sha256',
        'window_start','window_end','kbit_s','rss_client_kb','rss_server_kb','cpu_client','cpu_server',
        'complete_body','complete_frames','uniform_header','visible_state','checked_states','incomplete_states',
        'data_integrity','source','delivery_outcome')
    results=[]
    for row in summary['results']:
        assert row['data_integrity']['valid'] and row['source']['dropped']==0 and not row['source']['missing_oracle_frame_ids']
        result={name:row[name] for name in fields}
        for name in ('source_quality','collector','moving','moving_readiness','moving_common_start','moving_common_end'):
            if name in row:result[name]=row[name]
        times=[f[0] for f in row['complete_frames']];lo,hi=row['window_start'],row['window_end']
        result['completion_coverage']=dict(adjacent_gap_samples=max(0,len(times)-1),
            first_complete_offset_s=times[0]-lo if times else None,
            last_complete_offset_s=times[-1]-lo if times else None,
            no_new_complete_tail_s=hi-times[-1] if times else hi-lo,
            tail_scope='No newly detected complete body; not absence of received bytes or headers.')
        results.append(result)
    originals=dict(audit['source_sha256'])
    originals.update({r['capture']:r['sha256'] for r in audit['capture_details']})
    assert len(originals)==2*CONFIG['expected_jobs']
    stamp=lambda t:datetime.fromtimestamp(t,ZoneInfo('Asia/Seoul')).isoformat(timespec='milliseconds')
    save('core-results.json',dict(dataset=CONFIG['run'],date='2026-11-01',
        environment_created_at=summary['environment']['date'],
        receive_windows_kst=dict(first=stamp(min(r['window_start'] for r in results)),last=stamp(max(r['window_end'] for r in results))),
        parallel=4,iterations=2,requested_fps=60,valid=True,data_integrity_valid=True,
        performance_eligible=CONFIG['performance_eligible'],focused_performance_screen=CONFIG['performance_eligible'],
        final_selection=None,candidate_selected=False,measurement_pipeline=CONFIG['pipeline'],results=results,
        host_statuses=summary['host_statuses'],original_summary_sha256=sha(run/'probe-summary.json'),
        original_raw_files_sha256=originals,raw_input_availability='Raw captures/source payloads local only. Hashes imported from independent actual audit; export does not rehash/reparse raw files. Compact records allow metric arithmetic, not body-oracle/GPU replay.'))
    save('source-header-join-audit.json',dict(valid=True,scope='Counts and hashes copied from independent actual audit, not a second publisher raw parse.',
        independent_audit_sha256=sha(audit_path),summary_sha256=audit['saved_summary_sha256'],source_frames=audit['source_records'],
        pending_writes=audit['pending_writes_saved'],pending_body_joins=audit['pending_body_joins'],
        complete_body_headers_checked=audit['body_frames'],source_details=audit['source_details']))
    for ext in ('json','md'):shutil.copyfile(notes/(CONFIG['audit_stem']+'-actual-audit.'+ext),HERE/('saved-data-audit.'+ext))
    shutil.copyfile(notes/(CONFIG['audit_stem']+'-actual-metrics.csv'),HERE/'independent-metrics.csv')
    for name in ('acceptance-per-rep.csv','acceptance-per-rep.json','acceptance-per-rep.md','runtime-platform.json',
                 'run-status.json','prepared-inputs.json','artifact-identities.json','analysis-manifest.json','plan.json'):
        shutil.copyfile(run/name,HERE/name)
    shutil.copyfile(run/'harness/manifest.json',HERE/'original-harness-manifest.json')
    admission=[];runtime=[]
    for name in ['preflight-host.jsonl']+[b['tag']+'-quiet.jsonl' for b in summary['batches']]:
        samples=[json.loads(line) for line in (run/name).read_text().splitlines() if line]
        totals=[r['user_pct']+r['system_pct'] for r in samples];assert totals[-1]<30
        admission.append(dict(log=name,samples=len(samples),qualifying_cpu_pct=totals[-1],at=samples[-1]['at']))
    for tag,status in summary['host_statuses'].items():
        runtime.append(dict(phase=tag.removesuffix('-host-status'),accepted=status['accepted'],reasons=status['reasons'],
            samples=status['samples'],total_cpu_pct=status['total_cpu_pct'],max_external_process_cpu_pct=status['max_external_process_cpu_pct']))
    save('cpu30-summary.json',dict(policy='One user+system<30 interval admits; runtimeCPU/process context only. Timing/metadata failures reject.',
        admission=admission,runtime=runtime,host_process_names_archived=False,max_runtime_cpu_pct=max(r['total_cpu_pct']['max'] for r in runtime)))
    logs={};imports={}
    for name in CONFIG['provenance']:
        if name.endswith('.log'):
            raw=(study/name).read_bytes();public=name+'.gz';(HERE/public).write_bytes(gzip.compress(raw,mtime=0));logs[name]=hashlib.sha256(raw).hexdigest()
        else:public=name;shutil.copyfile(study/name,HERE/public)
        imports[name]=dict(public=public,original_sha256=sha(study/name))
    save('raw-build-log-sha256.json',logs);save('source-imports.json',imports)
    # Exact controller delta against the common safe-PTY reference, not the
    # archived older TCP-batch candidate and not a Git working-tree guess.
    reference=study/'pty-name-account-source';candidate=study/'fps-combined-pty-account-source';patch='';source_hashes={}
    for name in ('src/proto.rs','src/server.rs'):
        old=(reference/name).read_bytes();new=(candidate/name).read_bytes()
        patch+=''.join(difflib.unified_diff(old.decode().splitlines(True),new.decode().splitlines(True),fromfile='a/'+name,tofile='b/'+name))
        source_hashes[name]=dict(reference_sha256=hashlib.sha256(old).hexdigest(),candidate_sha256=hashlib.sha256(new).hexdigest())
    (HERE/'controller.patch').write_text(patch);save('controller-source.json',dict(reference='4f98af080434',candidate='295f81d54ea4',files=source_hashes,scope='Two production/controller files including their cfg(test) fixtures; shared PTY/CAN code not changed by this delta.'))
    if CONFIG['study_number']==45:
        native=[]
        for batch in summary['batches']:
            tag=batch['tag'];dest=HERE/'native'/tag;dest.mkdir(parents=True,exist_ok=True)
            names=['moving-network.json','moving-ready.json','moving-armed.json','native-qdisc-before.json','native-qdisc-after.json','native-cleanup.json','native-plan.txt']
            names += [f'rl-{slot}-{side}.{suffix}' for slot in range(4) for side in ('srv','cli') for suffix in ('stdout','stderr')]
            for name in names:shutil.copyfile(run/tag/name,dest/name)
            native.append(dict(batch=tag,files={name:sha(dest/name) for name in names}))
        save('native-files.json',native)
        shutil.copyfile(study/'42-native-netlink-spin12/native-netem.c',HERE/'native-netem.c')
        shutil.copyfile(study/'42-native-netlink-spin12/controller-built/build-activation.json',HERE/'native-build.json')
        save('moving-derived.json',dict(recovery=audit['moving_recovery_verified'],native_blocks=audit['native_timing_blocks'],loaded_fidelity_judgement=audit['loaded_fidelity_judgement'],performance_eligible=False))
    columns='net scene iteration variant slot frames fps low1 low5 age age95 kbit rssc rsss cpuc cpus headerframes headerfps maxgap adjacentgaps tail_s incomplete_pct'.split()
    with (HERE/'core-metrics.csv').open('w',newline='') as stream:
        writer=csv.DictWriter(stream,fieldnames=columns);writer.writeheader()
        for row in results:
            b=row['complete_body'];value={k:row[k] for k in ('net','scene','iteration','variant','slot')}
            value.update(frames=b['frames'],fps=b['fps'],low1=b['low1_fps'],low5=b['low5_fps'],age=b['age_ms'],age95=b['age_p95_ms'],
                kbit=row['kbit_s'],rssc=row['rss_client_kb'],rsss=row['rss_server_kb'],cpuc=row['cpu_client'],cpus=row['cpu_server'],
                headerframes=row['uniform_header']['frames'],headerfps=row['uniform_header']['fps'],maxgap=b['max_gap_ms'],
                adjacentgaps=row['completion_coverage']['adjacent_gap_samples'],tail_s=row['completion_coverage']['no_new_complete_tail_s'],incomplete_pct=row['visible_state']['incomplete_pct'])
            writer.writerow(value)
    save('extraction-provenance.json',dict(independent_audit_sha256=sha(audit_path),source_and_capture_hashes_imported=len(originals),
        current_raw_files_hashed=0,raw_payloads_reparsed=0,terminal_replays=0,new_benchmarks=0,
        original_summary_sha256=sha(run/'probe-summary.json'),source_scope='Independent audit performed raw checks; this publisher imports exact audit and saved summary.'))
    print(json.dumps(dict(run=CONFIG['run'],jobs=len(results),body_frames=sum(len(r['complete_frames']) for r in results),performance_eligible=CONFIG['performance_eligible'])))

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('study',type=Path);main(parser.parse_args().study)
