import struct,hashlib,json
from pathlib import Path
base=Path('/private/tmp/ferromark-v2-rounds-2')
def text_section(p):
 b=p.read_bytes(); magic,cpu,sub,filetype,ncmds,size,flags,res=struct.unpack_from('<8I',b)
 assert magic==0xfeedfacf
 off=32
 for _ in range(ncmds):
  cmd,size=struct.unpack_from('<2I',b,off)
  if cmd==0x19:
   nsects=struct.unpack_from('<I',b,off+64)[0]
   for n in range(nsects):
    s=off+72+n*80
    if b[s:s+16].rstrip(b'\0')==b'__text':
     addr,length,fileoff=struct.unpack_from('<QQI',b,s+32)
     return b[fileoff:fileoff+length],{'address':addr,'length':length,'file_offset':fileoff}
  off+=size
 raise ValueError(p)
result={}
for name in ['baseline','candidate']:
 paths={'original':Path('/private/tmp/ferromark-v2-simd-round/build-once')/name/'target/release/simd-round-worker','symbols':base/'symbols'/name/'release/simd-round-worker'}
 values={k:text_section(p) for k,p in paths.items()}
 result[name]={k:{**info,'sha256':hashlib.sha256(b).hexdigest()} for k,(b,info) in values.items()}
 result[name]['text_equal']=values['original'][0]==values['symbols'][0]
(base/'artifacts/symbol-audit-corrected.json').write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps(result,indent=2))
