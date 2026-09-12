from pathlib import Path
import shutil,subprocess,hashlib,struct,json,os
w=Path('/private/tmp/ferromark-escape-round9');rows=[]
def section(binary):
 b=binary.read_bytes();assert struct.unpack_from('<I',b)[0]==0xfeedfacf
 n=struct.unpack_from('<I',b,16)[0];p=32
 for _ in range(n):
  cmd,size=struct.unpack_from('<II',b,p)
  if cmd==0x19:
   count=struct.unpack_from('<I',b,p+64)[0]
   for i in range(count):
    at=p+72+i*80;name=b[at:at+16].rstrip(b'\0');seg=b[at+16:at+32].rstrip(b'\0')
    if name==b'__text' and seg==b'__TEXT':
     length=struct.unpack_from('<Q',b,at+40)[0];offset=struct.unpack_from('<I',b,at+48)[0]
     return b[offset:offset+length]
  p+=size
 raise ValueError('text section missing')
original=section(w/'bin/linear'); print('Recorded ARM code',hashlib.sha256(original).hexdigest(),flush=True)
for name in ['current','masked-direct','masked-twin']:
 shutil.copyfile(Path('/private/tmp/ferromark-linux-prepare-round3b/escape-linux/native-sources')/f'{name}.rs',w/'source/src/escape.rs')
 env=os.environ.copy();env.pop('RUSTFLAGS',None);env.pop('CARGO_ENCODED_RUSTFLAGS',None)
 with (w/(name+'-arm-build.log')).open('w') as log:
  r=subprocess.run(['cargo','build','--release','--offline','--locked','--manifest-path',str(w/'driver/Cargo.toml')],cwd=w,env=env,stdout=log,stderr=subprocess.STDOUT)
 assert r.returncode==0,name
 code=section(w/'driver/target/release/ox-experiment-driver');row=dict(candidate=name,text_bytes=len(code),text_sha256=hashlib.sha256(code).hexdigest());rows.append(row)
 print(row,flush=True); assert code==original, name
 (w/'native-arm-code.json').write_text(json.dumps(rows,indent=2)+'\n')
