from experiment import W,build,Worker
import subprocess,os,shutil,json,gzip,hashlib
cases=json.loads((W/'cases.json').read_text());selected=json.loads((W/'selected.json').read_text());expected=None
for name in ['baseline','unstable-packed','stable-packed','stable-tuple']:
    assert build(name)
    env=os.environ.copy();env.pop('RUSTFLAGS',None);env.pop('CARGO_ENCODED_RUSTFLAGS',None)
    with (W/name/'memory-build.log').open('w') as log:
        subprocess.run(['cargo','build','--release','--offline','--locked','--manifest-path',str(W/'memory-driver/Cargo.toml')],env=env,stdout=log,stderr=subprocess.STDOUT,check=True)
    binary=W/'bin'/('memory-'+name);shutil.copy2(W/'memory-driver/target/release/ox-experiment-driver',binary)
    worker=Worker(binary,W/'cases.json');rows=[]
    for i in selected:
        if cases[i]['reuse'] or cases[i]['flags']&256:continue
        rows.append(worker.call('heap',i))
    worker.close()
    actual=[r['html'] for r in rows]
    if expected is None:expected=actual
    assert actual==expected,name
    for row in rows:
        data=row.pop('html').encode();row['html_sha256']=hashlib.sha256(data).hexdigest();row['html_bytes']=len(data)
    (W/name/'memory.json.gz').write_bytes(gzip.compress(json.dumps(rows).encode(),mtime=0))
    print(name,len(rows),'allocation samples checked',flush=True)
