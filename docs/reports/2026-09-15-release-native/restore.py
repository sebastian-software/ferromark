from pathlib import Path
import hashlib,json,subprocess
root=Path(__file__).resolve().parent
records={}
def run(args,cwd=None): subprocess.run(args,cwd=cwd,check=True)
for name,url,rev in [
 ('bun','https://github.com/oven-sh/bun.git','76e9dcc6ad272a4fb1ee4a4dbbde4809201b71d1'),
 ('md4c','https://github.com/mity/md4c.git','65c6c9d72cebd9a731aaa5597414ce04d9ea5de3')]:
 dest=root/name
 run(['git','init','-q',str(dest)])
 run(['git','remote','add','origin',url],dest)
 if name=='bun':
  run(['git','config','remote.origin.promisor','true'],dest)
  run(['git','config','remote.origin.partialclonefilter','blob:none'],dest)
 args=['git','fetch','--depth=1']
 if name=='bun':args+=['--filter=blob:none']
 run(args+['origin',rev],dest)
 if name=='bun':
  run(['git','sparse-checkout','init','--no-cone'],dest)
  run(['git','sparse-checkout','set','--no-cone','/src/','/scripts/build/','/Cargo.toml','/Cargo.lock','/rust-toolchain.toml','/.cargo/','/LICENSE*'],dest)
 run(['git','checkout','--detach',rev],dest)
 records[name]={'url':url,'revision':rev}
for name,url,expected in [
 ('ox.tar.gz','https://api.github.com/repos/ubugeeei-prod/ox-content/tarball/a71a58939ffe7f154117cea026f6d6e71a139393','7df34e3e2db30678981f6df838eafe3e7950e966453221ae180acfa7938f1cfd'),
 ('native/mimalloc.tar.gz','https://codeload.github.com/oven-sh/mimalloc/tar.gz/6a64e1ba7f5b2130d4efccb67ec87fd0003f0f6a','f36343416ad823dfcca61bd18ad4bb7f0d8814ab77e0fd5ef169f4a9cddeb8b8'),
 ('native/highway.tar.gz','https://codeload.github.com/google/highway/tar.gz/2607d3b5b0113992fe84d3848859eae13b3b52c1','741d705781e0b3e406beda8f1f994fbae01321237ce8023a1ad90fbaf7940c25')]:
 dest=root/name
 run(['curl','-fL','--retry','2','--max-time','180','-o',str(dest),url])
 actual=hashlib.sha256(dest.read_bytes()).hexdigest()
 assert actual==expected,(name,actual,expected)
 records[name]={'url':url,'sha256':actual}
(root/'restore.json').write_text(json.dumps(records,indent=2)+'\n')
print('Pinned sources and original archive checksums restored.',flush=True)
