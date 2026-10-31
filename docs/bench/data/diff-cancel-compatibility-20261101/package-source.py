"""Extract eight saved compatibility records, not a new performance/replay run."""
from pathlib import Path
import argparse,csv,hashlib,json,shutil

HERE=Path(__file__).resolve().parent
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()

def main(study):
    run=study/'36-diff-cancel-compatibility';notes=study.parents[3]/'research_notes'/'LAN 원거리 low 성능 연구'
    summary=json.loads((run/'probe-summary.json').read_text());audit=json.loads((notes/'compat36-actual-audit.json').read_text())
    assert summary['valid'] and summary['data_integrity_valid'] and not summary['invalid_reasons']
    assert summary['performance_eligible'] is False and summary['final_selection'] is None
    assert audit['valid_saved_audit'] and audit['jobs']==8 and audit['body_frames']==774
    fields=('batch','net','scene','iteration','variant','slot','tool','binary_sha256','window_start','window_end',
        'complete_body','complete_frames','uniform_header','visible_state','kbit_s','rss_client_kb','rss_server_kb',
        'cpu_client','cpu_server','source','source_quality','data_integrity','delivery_outcome')
    rows=[{k:j[k] for k in fields} for j in summary['results']]
    assert len(rows)==8
    raw_hashes=dict(audit['source_sha256'])
    for row in audit['capture_details']:raw_hashes[row['capture']]=row['sha256']
    (HERE/'records.json').write_text(json.dumps(dict(study=36,parallel=4,iterations=2,
        data_integrity_valid=True,performance_eligible=False,compatibility_validation_only=True,final_selection=None,
        reference_scope='main-A/main-B are duplicate oldfar, not actualmain',results=rows,
        original_summary_sha256=sha(run/'probe-summary.json'),original_raw_files_sha256=raw_hashes,
        host_statuses=summary['host_statuses'],raw_scope='Imported independent audit hashes; full raw source/captures remain local. No new header/oracle replay in this extractor.'),separators=(',',':'))+'\n')
    names=('prepared-inputs.json','artifact-identities.json','paired-build-provenance.json','schedule.json',
        'plan.json','runtime-platform.json','run-status.json','analysis-manifest.json','harness-adaptation.patch')
    imports={}
    pairs=[(run/name,name) for name in names]+[(run/'harness/manifest.json','original-harness-manifest.json')]
    pairs += [(notes/('compat36-actual-audit.'+ext),'saved-data-audit.'+ext) for ext in ('json','md','py')]
    for source,name in pairs:
        shutil.copyfile(source,HERE/name);imports[name]=sha(source)
    (HERE/'imported-evidence.json').write_text(json.dumps(imports,indent=2)+'\n')
    cols='iteration variant slot frames fps low1 low5 age age95 maxgap tail_s headerframes headerfps incomplete_pct kbit rssc rsss cpuc cpus'.split()
    with (HERE/'metrics.csv').open('w',newline='') as f:
        writer=csv.DictWriter(f,fieldnames=cols);writer.writeheader()
        for row in rows:
            b=row['complete_body'];t=row['complete_frames'];h=row['uniform_header']
            writer.writerow(dict(iteration=row['iteration'],variant=row['variant'],slot=row['slot'],frames=b['frames'],fps=b['fps'],
                low1=b['low1_fps'],low5=b['low5_fps'],age=b['age_ms'],age95=b['age_p95_ms'],maxgap=b['max_gap_ms'],
                tail_s=row['window_end']-t[-1][0] if t else row['window_end']-row['window_start'],
                headerframes=h['frames'],headerfps=h['fps'],incomplete_pct=row['visible_state']['incomplete_pct'],
                kbit=row['kbit_s'],rssc=row['rss_client_kb'],rsss=row['rss_server_kb'],cpuc=row['cpu_client'],cpus=row['cpu_server']))
    print(json.dumps(dict(records=8,bodies=sum(len(r['complete_frames']) for r in rows),performance_eligible=False)))

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('study',type=Path);main(parser.parse_args().study)
