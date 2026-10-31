"""Offline pyte0.8 control over bytes emitted by actual compiled Rust fixtures."""
import hashlib,importlib.metadata,json
from pathlib import Path
import pyte,pyte.screens,pyte.streams
N=Path('/notes');inputs=json.loads((N/'diff-cancel-interop-inputs.json').read_text());rows=[]

def make(parts,mode):
 screen=pyte.Screen(100,30);stream=pyte.ByteStream(screen)
 if mode=='whole':stream.feed(b''.join(parts))
 elif mode=='split':
  for part in parts:stream.feed(part)
 else:
  for part in parts:
   for byte in part:stream.feed(bytes([byte]))
 return screen,stream
expected,_=make([bytes.fromhex(inputs['variants']['new']['hex']['expected_render'])],'whole')
for role,variant in inputs['variants'].items():
 parts=[bytes.fromhex(variant['hex'][key]) for key in ('client_snapshot','client_delta')]
 for mode in ('whole','split','bytewise'):
  screen,stream=make(parts,mode);differences=[]
  for y,(actual,want) in enumerate(zip(screen.display,expected.display)):
   for x,(a,b) in enumerate(zip(actual,want)):
    if a!=b:differences.append(dict(row=y,col=x,actual=a,expected=b,actual_ord=ord(a),expected_ord=ord(b)))
  target_cells=screen.display[24][94:97];before=(screen.cursor.x,screen.cursor.y);stream.feed(b'Z');sentinel_advances=screen.cursor.x==before[0]+1 and screen.cursor.y==before[1] and screen.buffer[before[1]][before[0]].data=='Z'
  rows.append(dict(variant=role,feed=mode,character_equal=not differences,cursor_equal=before==(expected.cursor.x,expected.cursor.y),cursor=list(before),expected_cursor=[expected.cursor.x,expected.cursor.y],different_cells=len(differences),differences=differences[:12],row24_columns94_96_before_sentinel=target_cells,public_at_ground_api=hasattr(stream,'at_ground') or hasattr(screen,'at_ground'),printable_continuation_advances_at_cursor=sentinel_advances))
assert all(r['character_equal'] and r['cursor_equal'] for r in rows if r['variant']=='new')
assert all(r['different_cells']==3 and r['cursor_equal'] and r['row24_columns94_96_before_sentinel']=='[0m' for r in rows if r['variant']=='old')
assert all(r['printable_continuation_advances_at_cursor'] for r in rows)
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
out=dict(scope='Actual Rust client_snapshot+client_delta through frozen pyte image, characters/cursor/one-printable continuation only',valid=True,pyte_version=importlib.metadata.version('pyte'),pyte_screens_sha256=sha(Path(pyte.screens.__file__)),pyte_streams_sha256=sha(Path(pyte.streams.__file__)),inputs_sha256=sha(N/'diff-cancel-interop-inputs.json'),source_logs={k:dict(file=v['source_log'],sha256=v['source_log_sha256']) for k,v in inputs['variants'].items()},actual_byte_difference='Candidate delta and client_delta are exactly oneCAN followed by control bytes; snapshot/client_snapshot/expected_render equal',expected_display=expected.display,expected_cursor=[expected.cursor.x,expected.cursor.y],results=rows,ground_limit='Pyte has no public at_ground API here. Successful single-printable continuation is observed for both; no general internal parser-ground theorem.',limits=['No new benchmark or product executable was run in this offline control','No GPU/every-terminal/finalPacedreceivepath/performance proof','Original34capture/oracle/framecount remain unchanged'],script_sha256=sha(Path(__file__)))
(N/'diff-cancel-frozen-pyte-interop.json').write_text(json.dumps(out,indent=2)+'\n');print(json.dumps({k:out[k] for k in ('valid','pyte_version','pyte_screens_sha256','pyte_streams_sha256','results','ground_limit')}))
