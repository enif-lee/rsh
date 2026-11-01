"""Extract compact public evidence from local raw study inputs.

Usage: python3 package-source.py /path/to/lowtail-research-20261031
Raw inputs are not bundled. This extractor is not a body-oracle replay.
Run only outside active measurement/analysis windows.
"""
from pathlib import Path
from datetime import datetime
from zoneinfo import ZoneInfo
import argparse, base64, csv, gzip, hashlib, json, math, re, shutil

HERE=Path(__file__).resolve().parent
CONFIG=json.loads((HERE/'archive-config.json').read_text())
def sha(path):
    value=hashlib.sha256()
    with path.open('rb') as stream:
        for block in iter(lambda:stream.read(1<<20),b''):value.update(block)
    return value.hexdigest()
def save(name,value):
    (HERE/name).write_text(json.dumps(value,separators=(',',':'))+'\n')

def main(study):
    run=study/CONFIG['run'];summary=json.loads((run/'probe-summary.json').read_text())
    assert summary['valid'] and summary['data_integrity_valid'] and not summary['invalid_reasons']
    assert len(summary['results'])==48 and len(summary['host_statuses'])==12
    results=[];originals={};header_rows=[];source_count=completed=pending=pending_joined=body_count=0
    fields=('batch','net','scene','iteration','variant','slot','tool','duration','scene_fps','cols','rows','binary_sha256',
            'window_start','window_end','kbit_s','rss_client_kb','rss_server_kb','cpu_client','cpu_server',
            'complete_body','complete_frames','data_integrity','source','delivery_outcome')
    for row in summary['results']:
        assert row['data_integrity']['valid'] and row['source']['dropped']==0 and not row['source']['missing_oracle_frame_ids']
        result={key:row[key] for key in fields}
        for key in ('source_quality','collector'):
            if key in row:result[key]=row[key]
        results.append(result)
        for field in ('capture','source_ledger'):originals[row[field]]=sha(run/row[field])
        ledger=json.loads((run/row['source_ledger']).read_text());frames=ledger['frames'];by_n={f['n']:f for f in frames}
        assert len(by_n)==len(frames);source_count+=len(frames)
        for frame in frames:
            header=re.search(rb'FRAME (\d+) T=(\d+) (\w+)',base64.b64decode(frame['payload_b64'][:1024]))
            assert header and int(header[1])==frame['n'] and header[3].decode()==row['scene'] and int(header[2])>0
            if frame.get('write_completed',True):
                assert frame['write_end_ns']>=frame['write_begin_ns'];completed+=1
            else:
                assert frame['write_completed'] is False and frame['write_end_ns'] is None;pending+=1
        for at,n,made,scene in row['complete_frames']:
            frame=by_n[n];header=re.search(rb'FRAME (\d+) T=(\d+) (\w+)',base64.b64decode(frame['payload_b64'][:1024]))
            assert (int(header[1]),int(header[2]),header[3].decode())==(n,round(made*1000),scene)
            body_count+=1;pending_joined+=frame.get('write_completed',True) is False
        header_rows.append(dict(batch=row['batch'],slot=row['slot'],variant=row['variant'],source_frames=len(frames),
                                completed_frames_checked=len(row['complete_frames']),source_sha256=originals[row['source_ledger']],mismatch_count=0))
    lo=min(r['window_start'] for r in results);hi=max(r['window_end'] for r in results)
    stamp=lambda at:datetime.fromtimestamp(at,ZoneInfo('Asia/Seoul')).isoformat(timespec='milliseconds')
    save('core-results.json',dict(dataset=CONFIG['run'],date='2026-11-01',environment_created_at=summary['environment']['date'],
        receive_windows_kst=dict(first=stamp(lo),last=stamp(hi)),parallel=4,iterations=2,requested_fps=60,
        valid=True,data_integrity_valid=True,performance_accepted=False,measurement_pipeline=CONFIG['pipeline'],
        meaning='Development screen. Actual-source body metrics; no GPU proof, repeat pooling or automatic acceptance.',
        results=results,host_statuses=summary['host_statuses'],original_summary_sha256=sha(run/'probe-summary.json'),
        original_raw_files_sha256=originals,raw_input_availability='Raw captures/source bodies remain local; all96 original hashes are archived. Compact data permits metric recalculation, not body-oracle replay.'))
    save('source-header-join-audit.json',dict(valid=True,summary_sha256=sha(run/'probe-summary.json'),
        source_frames=source_count,completed_writes=completed,retained_pending_records=pending,
        pending_writes=pending if CONFIG['pipeline']=='source-ram-signal-schema2' else None,
        retained_pending_body_joins=pending_joined,pending_body_joins=pending_joined if CONFIG['pipeline']=='source-ram-signal-schema2' else None,
        complete_body_headers_checked=body_count,checks=header_rows,
        scope='Raw source payload-prefix FRAME n/T/scene check for all source records and observed bodies. No new terminal-character emulation or kernel/GPU proof.'))
    admission=[];runtime=[]
    names=['preflight-host.jsonl']+[b['tag']+'-quiet.jsonl' for b in summary['batches']]
    for name in names:
        samples=[json.loads(line) for line in (run/name).read_text().splitlines() if line]
        totals=[s['user_pct']+s['system_pct'] for s in samples]
        assert totals[-1]<30
        admission.append(dict(log=name,samples=len(samples),qualifying_cpu_pct=totals[-1],at=samples[-1]['at']))
    for tag,status in summary['host_statuses'].items():
        runtime.append(dict(phase=tag.removesuffix('-host-status'),accepted=status['accepted'],reasons=status['reasons'],
                            samples=status['samples'],total_cpu_pct=status['total_cpu_pct'],
                            max_external_process_cpu_pct=status['max_external_process_cpu_pct']))
    save('cpu30-summary.json',dict(policy='Admission: one interval user+system<30%; runtime CPU/process context only. Timing/metadata failures still reject.',
        admission=admission,runtime=runtime,host_process_names_archived=False,
        max_runtime_cpu_pct=max(r['total_cpu_pct']['max'] for r in runtime)))
    for name in ('acceptance-per-rep.csv','acceptance-per-rep.json','acceptance-per-rep.md','runtime-platform.json',
                 'run-status.json','prepared-inputs.json','artifact-identities.json','analysis-manifest.json','plan.json'):
        shutil.copyfile(run/name,HERE/name)
    shutil.copyfile(run/'harness/manifest.json',HERE/'original-harness-manifest.json')
    notes=study.parents[3]/'research_notes'/'LAN 원거리 low 성능 연구'
    for original,public in ((f"probe{CONFIG['study_number']}-actual-audit.json",'saved-data-audit.json'),
                            (f"probe{CONFIG['study_number']}-actual-audit.md",'saved-data-audit.md'),
                            ('host-context-derived-json-read-20261101.json','host-context-derived-read.json')):
        shutil.copyfile(notes/original,HERE/public)
    log_hashes={}
    for name in CONFIG['provenance']:
        if name.endswith('.log'):
            original=(study/name).read_bytes()
            (HERE/(name+'.gz')).write_bytes(gzip.compress(original,mtime=0))
            log_hashes[name]=hashlib.sha256(original).hexdigest()
            if (HERE/name).exists():(HERE/name).unlink()
        else:shutil.copyfile(study/name,HERE/name)
    save('raw-build-log-sha256.json',log_hashes)
    columns='net scene iteration variant slot frames fps low1 low5 age age95 kbit rssc rsss cpuc cpus'.split()
    with (HERE/'core-metrics.csv').open('w',newline='') as output:
        writer=csv.DictWriter(output,fieldnames=columns);writer.writeheader()
        for row in results:
            b=row['complete_body'];value={k:row[k] for k in ('net','scene','iteration','variant','slot')}
            value.update(frames=b['frames'],fps=b['fps'],low1=b['low1_fps'],low5=b['low5_fps'],age=b['age_ms'],age95=b['age_p95_ms'],
                         kbit=row['kbit_s'],rssc=row['rss_client_kb'],rsss=row['rss_server_kb'],cpuc=row['cpu_client'],cpus=row['cpu_server'])
            writer.writerow(value)
    print(json.dumps(dict(dataset=CONFIG['run'],cases=48,bodies=body_count,source_frames=source_count,pending=pending,pending_body_joins=pending_joined)))

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('study',type=Path);main(parser.parse_args().study)
