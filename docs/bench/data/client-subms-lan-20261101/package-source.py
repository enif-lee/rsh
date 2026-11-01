"""Copy 16 saved study37 records; no benchmark or terminal replay.

Usage: python3 package-source.py /path/to/lowtail-research-20261031
"""
from pathlib import Path
import argparse,csv,gzip,hashlib,json,shutil
HERE=Path(__file__).resolve().parent
sha=lambda raw:hashlib.sha256(raw).hexdigest()

def main(study):
    run=study/'37-client-subms-lan';notes=study.parents[3]/'research_notes'/'LAN 원거리 low 성능 연구'
    summary=json.loads((run/'probe-summary.json').read_text());audit=json.loads((notes/'subms37-actual-audit.json').read_text())
    assert summary['valid'] and summary['data_integrity_valid'] and not summary['invalid_reasons']
    assert audit['valid_saved_audit'] and audit['jobs']==16 and audit['body_frames']==7680
    fields=('batch','net','scene','iteration','variant','slot','binary_sha256','window_start','window_end','complete_body',
        'complete_frames','uniform_header','visible_state','kbit_s','rss_client_kb','rss_server_kb','cpu_client','cpu_server','source','data_integrity')
    rows=[{key:job[key] for key in fields} for job in summary['results']]
    raw_hashes=dict(audit['source_sha256'])
    for row in audit['capture_details']:raw_hashes[row['capture']]=row['sha256']
    value=dict(study=37,parallel=4,iterations=2,results=rows,data_integrity_valid=True,
        performance_eligible=summary['performance_eligible'],focused_performance_screen=True,final_selection=None,
        reference_scope='Internal main-A/B are identical CAN-fixed935/c68, not unfixed main6729/9ab.',
        original_summary_sha256=sha((run/'probe-summary.json').read_bytes()),original_raw_files_sha256=raw_hashes,
        host_statuses=summary['host_statuses'],raw_scope='32 raw hashes imported from the independent audit; full captures/payload ledgers stay local. No new body replay in this extractor.')
    (HERE/'records.json').write_text(json.dumps(value,separators=(',',':'))+'\n')
    pairs=[(run/name,name) for name in ('prepared-inputs.json','artifact-identities.json','plan.json','runtime-platform.json','run-status.json','analysis-manifest.json')]
    pairs.append((run/'harness/manifest.json','original-harness-manifest.json'))
    pairs += [(notes/('subms37-actual-audit.'+ext),'saved-data-audit.'+ext) for ext in ('json','md','py')]
    pairs += [(study/name,name) for name in ('client-subms-wait-release-pair.json','client-subms-wait.patch',
        'client-subms-wait-proof.json','client-subms-wait-source-manifest.json','client-subms-wait-hypothesis.md',
        'build-client-subms-wait.log','client-subms-wait-baseline-guard.log','client-subms-wait-candidate-guard.log')]
    imports={}
    for source,name in pairs:
        raw=source.read_bytes();dest=HERE/(name+'.gz' if name.endswith('.log') else name)
        dest.write_bytes(gzip.compress(raw,mtime=0)) if name.endswith('.log') else shutil.copyfile(source,dest)
        imports[dest.name]=dict(original_name=name,original_sha256=sha(raw))
    (HERE/'imported-evidence.json').write_text(json.dumps(imports,indent=2)+'\n')
    columns='scene iteration variant slot frames fps low1 low5 age age95 maxgap firstgap tailgap kbit rssc rsss cpuc cpus'.split()
    with (HERE/'metrics.csv').open('w',newline='') as output:
        writer=csv.DictWriter(output,fieldnames=columns);writer.writeheader()
        for row in rows:
            b=row['complete_body'];f=row['complete_frames']
            writer.writerow(dict(scene=row['scene'],iteration=row['iteration'],variant=row['variant'],slot=row['slot'],frames=b['frames'],fps=b['fps'],
                low1=b['low1_fps'],low5=b['low5_fps'],age=b['age_ms'],age95=b['age_p95_ms'],maxgap=b['max_gap_ms'],
                firstgap=(f[0][0]-row['window_start'])*1000,tailgap=(row['window_end']-f[-1][0])*1000,
                kbit=row['kbit_s'],rssc=row['rss_client_kb'],rsss=row['rss_server_kb'],cpuc=row['cpu_client'],cpus=row['cpu_server']))
    print(json.dumps(dict(records=len(rows),bodies=sum(len(r['complete_frames']) for r in rows),raw_hashes=len(raw_hashes))))

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('study',type=Path);main(parser.parse_args().study)
