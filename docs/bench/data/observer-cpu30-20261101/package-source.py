"""Extract saved observer-study evidence; no benchmark or terminal replay.

Usage: python3 package-source.py /path/to/lowtail-research-20261031
Run only outside active measurement/analysis windows. Raw files stay local.
"""
from pathlib import Path
from datetime import datetime
from zoneinfo import ZoneInfo
import argparse, hashlib, json, shutil

HERE=Path(__file__).resolve().parent
CONFIG=json.loads((HERE/'archive-config.json').read_text())
def sha(path):
    digest=hashlib.sha256()
    with path.open('rb') as stream:
        for block in iter(lambda:stream.read(1<<20),b''):digest.update(block)
    return digest.hexdigest()
def save(name,value):
    (HERE/name).write_text(json.dumps(value,separators=(',',':'))+'\n')

def main(study):
    run=study/CONFIG['run'];summary=json.loads((run/'probe-summary.json').read_text())
    assert summary['valid'] and summary['data_integrity_valid'] and not summary['invalid_reasons']
    assert len(summary['results'])==96 and len(summary['host_statuses'])==24
    notes=study.parents[3]/'research_notes'/'LAN 원거리 low 성능 연구'
    audit_path=notes/'observer26-actual-review.json'
    audit=json.loads(audit_path.read_text())
    assert audit['valid_saved_audit'] and not audit['errors'] and audit['jobs']==96
    fields=('batch','net','scene','iteration','variant','slot','tool','duration','scene_fps','cols','rows','binary_sha256',
            'window_start','window_end','kbit_s','rss_client_kb','rss_server_kb','cpu_client','cpu_server',
            'complete_body','complete_frames','data_integrity','source','delivery_outcome','observer_mode',
            'observer_transition_epoch','observer_transition_lag_ms')
    results=[];originals={}
    for row in summary['results']:
        assert row['data_integrity']['valid'] and row['source']['dropped']==0 and not row['source']['missing_oracle_frame_ids']
        result={key:row[key] for key in fields};result['online_fps_placeholder']=row['fps']
        results.append(result)
        for field in ('capture','source_ledger'):originals[row[field]]=sha(run/row[field])
        assert originals[row['source_ledger']]==audit['source_sha256'][row['source_ledger']]
    lo=min(r['window_start'] for r in results);hi=max(r['window_end'] for r in results)
    stamp=lambda at:datetime.fromtimestamp(at,ZoneInfo('Asia/Seoul')).isoformat(timespec='milliseconds')
    assert sum(len(r['complete_frames']) for r in results)==audit['body_frames']==42132
    save('observer-results.json',dict(dataset=CONFIG['run'],date='2026-11-01',environment_created_at=summary['environment']['date'],
        receive_windows_kst=dict(first=stamp(lo),last=stamp(hi)),parallel=4,iterations_per_mode=2,requested_fps=60,
        valid=True,data_integrity_valid=True,performance_accepted=False,product_changed=False,measurement_pipeline=CONFIG['pipeline'],
        meaning='Whole-block ABBA observer treatment. Own legacy anchors; no startup exclusion, pooling, tolerance or product acceptance.',
        results=results,host_statuses=summary['host_statuses'],original_summary_sha256=sha(run/'probe-summary.json'),
        original_raw_files_sha256=originals,
        raw_input_availability='192 original source/capture hashes; raw files remain local. Metric recalculation is supported, terminal-character oracle replay is not.'))
    shutil.copyfile(audit_path,HERE/'saved-data-audit.json')
    admission=[];runtime=[]
    names=['preflight-host.jsonl']+[b['tag']+'-quiet.jsonl' for b in summary['batches']]
    for name in names:
        samples=[json.loads(line) for line in (run/name).read_text().splitlines() if line]
        totals=[s['user_pct']+s['system_pct'] for s in samples]
        assert totals[-1]<30 and all(t>=30 for t in totals[:-1])
        admission.append(dict(log=name,samples=len(samples),qualifying_cpu_pct=totals[-1],at=samples[-1]['at']))
    for tag,status in summary['host_statuses'].items():
        runtime.append(dict(phase=tag.removesuffix('-host-status'),accepted=status['accepted'],reasons=status['reasons'],
                            samples=status['samples'],total_cpu_pct=status['total_cpu_pct'],
                            max_external_process_cpu_pct=status['max_external_process_cpu_pct']))
    save('cpu30-summary.json',dict(policy='One admission interval user+system<30%; runtime CPU/process context only. Timing/metadata failures still reject.',
        admission=admission,runtime=runtime,host_process_names_archived=False,
        descriptive_unweighted_mode_mean_of_phase_cpu_means=audit['mode_mean_phase_cpu_context'],
        max_runtime_cpu_pct=max(r['total_cpu_pct']['max'] for r in runtime)))
    for name in ('observer-per-rep.csv','observer-per-rep.json','observer-per-rep.md','runtime-platform.json',
                 'run-status.json','prepared-inputs.json','artifact-identities.json','analysis-manifest.json','plan.json',
                 'main-build-provenance.json','pyte-runtime-provenance.json','parent-input-identities.json','verification-summary.json'):
        shutil.copyfile(run/name,HERE/name)
    shutil.copyfile(run/'harness/manifest.json',HERE/'original-harness-manifest.json')
    shutil.copyfile(study/'main-source-manifest.json',HERE/'main-source-manifest.json')
    analysis=run/summary_analysis_source(run)
    for path,name in ((run/'agent.py','observer-agent.py'),(analysis/'probe_report.py','observer-report.py'),
                      (analysis/'replay.py','body-replay.py'),(analysis/'ledger_oracle.py','ledger-oracle.py'),
                      (run/'scene-ledger.py','scene-ledger.py')):
        shutil.copyfile(path,HERE/name)
    save('extraction-provenance.json',dict(original_summary_sha256=sha(run/'probe-summary.json'),
        original_audit_sha256=sha(audit_path),raw_files_hashed=len(originals),
        source_hashes_compared_to_independent_audit=len(audit['source_sha256']),
        raw_source_header_audit_reused=True,terminal_replay_performed=False,
        scope='Exact saved-data audit copied; 192 current raw files hashed and 96 source hashes compared to that audit. Bundled helper source identifies methods, not a portable rerun harness.'))
    print(json.dumps(dict(cases=96,bodies=42132,original_raw_hashes=len(originals),performance_accepted=False)))

def summary_analysis_source(run):
    return json.loads((run/'analysis-manifest.json').read_text())['source']

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('study',type=Path);main(parser.parse_args().study)
