"""Verify archived controls with the standard library; no terminal replay."""
from pathlib import Path
import gzip,hashlib,json,re

HERE=Path(__file__).resolve().parent
sha=lambda data:hashlib.sha256(data).hexdigest()
manifest=json.loads((HERE/'manifest.json').read_text())
assert set(manifest)=={str(p.relative_to(HERE)) for p in HERE.rglob('*') if p.is_file() and p.name!='manifest.json'}
for name,digest in manifest.items():assert sha((HERE/name).read_bytes())==digest,name
imports=json.loads((HERE/'imported-evidence.json').read_text())
for name,record in imports['imports'].items():
    data=(HERE/name).read_bytes()
    if name.endswith('.gz'):data=gzip.decompress(data)
    assert sha(data)==record['original_sha256'],name
inputs=json.loads((HERE/'diff-cancel-interop-inputs.json').read_text())
for variant in inputs['variants'].values():
    data=gzip.decompress((HERE/(variant['source_log']+'.gz')).read_bytes())
    assert sha(data)==variant['source_log_sha256']
    found={}
    for key,value in re.findall(rb'DIFF_CANCEL_INTEROP (\w+)=([0-9a-f]+)',data):
        key,value=key.decode(),value.decode()
        assert key not in found or found[key]==value
        found[key]=value
    assert found==variant['hex']
old,new=(inputs['variants'][role]['hex'] for role in ('old','new'))
for key in ('snapshot','client_snapshot','expected_render'):assert old[key]==new[key]
for key in ('delta','client_delta'):assert bytes.fromhex(new[key])==b'\x18'+bytes.fromhex(old[key])
result=json.loads((HERE/'diff-cancel-frozen-pyte-interop.json').read_text())
assert len(result['expected_display'])==30 and all(len(row)==100 for row in result['expected_display'])
assert {(r['variant'],r['feed']) for r in result['results']}=={(v,m) for v in ('old','new') for m in ('whole','split','bytewise')}
assert len(result['results'])==6
for row in result['results']:
    assert row['cursor']==row['expected_cursor']==[94,24] and row['cursor_equal']
    assert row['different_cells']==(3 if row['variant']=='old' else 0)
    assert row['character_equal']==(row['variant']=='new')
    assert row['printable_continuation_advances_at_cursor'] and not row['public_at_ground_api']
native=json.loads((HERE/'native-tmux/tmux-native-controls.json').read_text())
assert native['version']=='tmux 3.7c' and native['all_matched'] and len(native['cases'])==5
assert {row['case'] for row in native['cases']}=={'literal','ordinary','nested','nested-bytewise','cancel'}
for row in native['cases']:
    assert row['actual']==row['expected']==('[0m' if row['case']=='literal' else '   ')
    assert row['metadata']['cols']==100 and row['metadata']['rows']==30
pair=json.loads((HERE/'diff-cancel-release-pair.json').read_text())
assert pair['tests']==dict(targeted_passed=3,library_passed=99,ignored=2)
assert pair['control']['passed']==2 and pair['control']['expected_failed']==1
integration=json.loads((HERE/'diff-cancel-integration-results.json').read_text())
assert integration['library_prior']==dict(passed=99,ignored=2)
assert integration['integration']==dict(cli_passed=4,session_passed=25,failed=0)
assert integration['fresh_false_integration_artifacts']==2
assert sha(gzip.decompress((HERE/'build-diff-cancel-integration.log.gz').read_bytes()))==integration['log_sha256']
proof=json.loads((HERE/'diff-cancel-integration-proof.json').read_text())
assert proof['production_unchanged'] and sha((HERE/'integration-session-fixture.rs').read_bytes())==proof['fixture_sha256']
print(json.dumps(dict(manifest_files=len(manifest),imported_original_bytes_match=True,pyte_cases=6,native_minimal_cases=5,new_terminal_runs=0)))
