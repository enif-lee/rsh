import os,sys,time,json,struct,fcntl,termios
from pathlib import Path
p=Path(sys.argv[1]);data=bytes.fromhex(sys.argv[2]);mode=sys.argv[3]
size=struct.unpack("HHHH",fcntl.ioctl(1,termios.TIOCGWINSZ,b"\0"*8))
os.write(1,b"\x1b[2J\x1b[H")
if mode=="bytewise":
 for byte in data:
  os.write(1,bytes([byte]));time.sleep(.001)
else:os.write(1,data)
os.write(1,b"\x1b[1;1HDONE")
p.with_suffix(".meta.json").write_text(json.dumps({"rows":size[0],"cols":size[1],"payload_hex":data.hex(),"mode":mode}))
p.with_suffix(".ready").touch()
time.sleep(10)
