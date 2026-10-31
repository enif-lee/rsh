"""Copy saved diagnostic evidence; no terminal execution or replay.

Usage: python3 package-controls.py /path/to/lowtail-research-20261031
Run only after the parent releases measurement/build resources.
"""
from pathlib import Path
import argparse,gzip,hashlib,json,shutil

HERE=Path(__file__).resolve().parent
STUDY_FILES=(
    'diff-cancel-release-pair.json','diff-cancel.patch','diff-cancel-tests.rs',
    'diff-cancel-hypothesis.md','diff-cancel-source-manifest.json','diff-cancel-proof.json',
    'build-diff-cancel.log','diff-cancel-control-tests.log','diff-cancel-main-guard.log',
    'diff-cancel-candidate-guard.log','diff-cancel-control-guard.log',
    'diff-cancel-integration-results.json','diff-cancel-integration-proof.json',
    'diff-cancel-integration-source-manifest.json','diff-cancel-integration-build.jsonl',
    'build-diff-cancel-integration.log','build-diff-cancel-integration.sh')
NOTE_FILES=(
    'far34-body-coherence-investigation.md','far34-body-mismatch-diagnostic.json',
    'far34-pyte-stream-controls.json','far34-all-header-source-joins.json',
    'far34-transition-raw-snippets.json','far34-raw-escape-scan.json',
    'far34-offline-replay-provenance.json','far34-body-header-arithmetic-review.md',
    'far34-body-header-arithmetic-review.json','far34-terminal-recovery-source-note.md',
    'diff-cancel-interop-inputs.json','diff-cancel-frozen-pyte-interop.py',
    'diff-cancel-frozen-pyte-interop.json','diff-cancel-frozen-pyte-interop.log',
    'diff-cancel-frozen-pyte-interop.md','diff-cancel-frozen-pyte-provenance.json',
    'diff-cancel-offline-tool-transcript.txt','diff-cancel-pyte-independent-review.md',
    'diff-cancel-pyte-independent-review.json')
NATIVE_FILES=('tmux-native-controls.json','tmux-control-native.py','tmux-parser-child-native.py',
    'parser-tmux-native.conf','tmux-control-native.log','tmux-control.log','tmux-control.py',
    'literal.screen.txt','ordinary.screen.txt','nested.screen.txt','nested-bytewise.screen.txt','cancel.screen.txt')

def main(study):
    notes=study.parents[3]/'research_notes'/'LAN 원거리 low 성능 연구'
    imports={};logs={}
    pairs=[(study/name,name) for name in STUDY_FILES]
    pairs += [(study/'diff-cancel-integration-source/tests/session.rs','integration-session-fixture.rs')]
    pairs += [(notes/name,name) for name in NOTE_FILES]
    pairs += [(study/'terminal-parser-control'/name,'native-tmux/'+name) for name in NATIVE_FILES]
    for source,name in pairs:
        raw=source.read_bytes();digest=hashlib.sha256(raw).hexdigest()
        dest=HERE/(name+'.gz' if name.endswith('.log') else name);dest.parent.mkdir(parents=True,exist_ok=True)
        if name.endswith('.log'):
            dest.write_bytes(gzip.compress(raw,mtime=0));logs[str(dest.relative_to(HERE))]=digest
        else:shutil.copyfile(source,dest)
        imports[str(dest.relative_to(HERE))]=dict(original_name=name,original_sha256=digest)
    (HERE/'imported-evidence.json').write_text(json.dumps(dict(imports=imports,
        log_uncompressed_sha256=logs,scope='Partial diagnostic archive; no raw full capture/source-ledger bundle and no new replay.'),indent=2)+'\n')
    print(json.dumps(dict(imported_files=len(imports),new_terminal_or_benchmark_runs=0)))

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('study',type=Path);main(parser.parse_args().study)
