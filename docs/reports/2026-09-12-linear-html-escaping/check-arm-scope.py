"""Rebuild the recorded and scoped ARM candidates and compare machine code."""
from pathlib import Path
import argparse,gzip,hashlib,platform,struct,subprocess,sys
p=argparse.ArgumentParser();p.add_argument('destination',type=Path);a=p.parse_args()
assert platform.machine().lower() in ['arm64','aarch64'],'Run on ARM64'
D=Path(__file__).resolve().parent;W=a.destination.resolve()
assert not W.exists(),'Use a fresh destination'
subprocess.run([sys.executable,str(D/'setup.py'),str(W)],check=True)
subprocess.run(['cargo','fetch','--locked','--manifest-path',str(W/'driver/Cargo.toml')],cwd=W,check=True)
def section(binary):
 b=binary.read_bytes();assert struct.unpack_from('<I',b)[0]==0xfeedfacf,'Recorded comparison uses macOS Mach-O'
 n=struct.unpack_from('<I',b,16)[0];pos=32
 for _ in range(n):
  cmd,size=struct.unpack_from('<II',b,pos)
  if cmd==0x19:
   for i in range(struct.unpack_from('<I',b,pos+64)[0]):
    at=pos+72+i*80
    if b[at:at+16].rstrip(b'\0')==b'__text' and b[at+16:at+32].rstrip(b'\0')==b'__TEXT':
     length=struct.unpack_from('<Q',b,at+40)[0];offset=struct.unpack_from('<I',b,at+48)[0]
     return b[offset:offset+length]
  pos+=size
 raise ValueError('Machine-code section missing')
sys.path.insert(0,str(W));from experiment import build
assert build('linear')
before=section(W/'bin/linear')
(W/'linear-source/src/escape.rs').write_bytes(gzip.decompress((D/'snapshots/arm-scoped-escape.rs.gz').read_bytes()))
assert build('linear')
after=section(W/'bin/linear');assert before==after,'ARM machine code differs'
print('Identical ARM machine code:',len(after),'bytes; SHA-256',hashlib.sha256(after).hexdigest())
