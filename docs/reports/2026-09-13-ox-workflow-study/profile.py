from pathlib import Path
import gzip,json,subprocess
from run import HERE,Worker,write

(HERE/'profiles').mkdir(exist_ok=True)
for mode in ['ferro-fresh','ferro-reuse','ox-reuse']:
    for repeat in range(2):
        w=Worker(mode,profile='sample')
        normal=Worker(mode)
        assert w.call('verify')==normal.call('verify')
        normal.close()
        w.call('time',milliseconds=300)
        w.p.stdin.write(json.dumps(dict(action='time',milliseconds=7500))+'\n'); w.p.stdin.flush()
        dest=HERE/'profiles'/f'{mode}-{repeat}.txt'
        with dest.with_suffix('.log').open('w') as log:
            result=subprocess.run(['/usr/bin/sample',str(w.p.pid),'5','1','-file',str(dest)],stdout=log,stderr=subprocess.STDOUT)
        window=json.loads(w.p.stdout.readline()); w.close()
        if result.returncode: raise RuntimeError(dest.with_suffix('.log'))
        raw=dest.read_bytes(); dest.with_suffix('.txt.gz').write_bytes(gzip.compress(raw,mtime=0)); dest.unlink()
        write('profiles/'+f'{mode}-{repeat}.json',dict(mode=mode,pid=w.p.pid,repeat=repeat,seconds=5,interval_ms=1,window=window))
        print('Profiled',mode,repeat,flush=True)
