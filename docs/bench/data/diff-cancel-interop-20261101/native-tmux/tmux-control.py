from pathlib import Path
import subprocess,json,time,shlex,sys
root=Path('/evidence');child=Path('/tmp/parser-child.py')
child.write_text('''import os,sys,time,json,struct,fcntl,termios\nfrom pathlib import Path\np=Path(sys.argv[1]);data=bytes.fromhex(sys.argv[2]);mode=sys.argv[3]\nsize=struct.unpack("HHHH",fcntl.ioctl(1,termios.TIOCGWINSZ,b"\\0"*8))\nos.write(1,b"\\x1b[2J\\x1b[H")\nif mode=="bytewise":\n for byte in data:\n  os.write(1,bytes([byte]));time.sleep(.001)\nelse:os.write(1,data)\nos.write(1,b"\\x1b[1;1HDONE")\np.with_suffix(".meta.json").write_text(json.dumps({"rows":size[0],"cols":size[1],"payload_hex":data.hex(),"mode":mode}))\np.with_suffix(".ready").touch()\ntime.sleep(10)\n''')
conf=Path('/tmp/parser-tmux.conf');conf.write_text('set -g status off\n')
prefix=['tmux','-L','rsh-parser-control','-f',str(conf)]
run=lambda args:subprocess.run(prefix+args,capture_output=True,text=True,check=True).stdout
cursor=b'\x1b[25;95H';cases=[('literal',cursor+b'[0m','whole','[0m'),('ordinary',cursor+b'\x1b[0m','whole','   '),('nested',cursor+b'\x1b[\x1b[0m','whole','   '),('nested-bytewise',cursor+b'\x1b[\x1b[0m','bytewise','   '),('cancel',cursor+b'\x1b[\x18\x1b[0m','whole','   ')]
results=[]
for name,data,mode,expected in cases:
 p=root/name;cmd=shlex.join(['python3',str(child),str(p),data.hex(),mode])
 try:
  run(['new-session','-d','-s',name,'-x','100','-y','30',cmd]);deadline=time.monotonic()+5;screen=''
  while time.monotonic()<deadline:
   if p.with_suffix('.ready').exists():
    screen=run(['capture-pane','-p','-t',name+':0.0'])
    if screen.startswith('DONE'):break
   time.sleep(.01)
  else:raise AssertionError('Parser/control readiness timeout: '+name)
  lines=screen.splitlines();line=lines[24] if len(lines)>24 else '';actual=(line+' '*100)[94:97]
  meta=json.loads(p.with_suffix('.meta.json').read_text());assert (meta['rows'],meta['cols'])==(30,100),meta
  p.with_suffix('.screen.txt').write_text(screen);results.append(dict(case=name,expected=expected,actual=actual,matched=actual==expected,metadata=meta))
 finally:
  subprocess.run(prefix+['kill-session','-t',name],capture_output=True)
version=subprocess.check_output(['tmux','-V'],text=True).strip()
out=dict(terminal_model='tmux',version=version,cases=results,all_matched=all(r['matched'] for r in results),scope='Native isolated tmux terminal-parser control, not GPU/presentation or all terminals; no rsh product/benchmark/source processes. Literal negative control proves target cells are observable.')
(root/'tmux-controls.json').write_text(json.dumps(out,indent=2)+'\n');print(json.dumps(out));assert out['all_matched']
