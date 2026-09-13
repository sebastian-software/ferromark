"""Diagnostic, alternating timing windows; does not update public benchmark figures."""
from pathlib import Path
import gzip, hashlib, json, os, re, statistics, subprocess, sys, time

HERE=Path(__file__).resolve().parent
ROOT=Path(os.environ.get('FERROMARK_ROOT', '/Users/sebastian/Workspace/.codex/15d0/ferromark')).resolve()
sys.path.insert(0,str(ROOT/'benchmarks/workflows/field'))
from common import without_generated_heading_ids
from support import observation

MODES=['ferro-fresh','ferro-reuse','ferro-fresh-ids','ferro-reuse-ids','ox-reuse','ox-fresh']
corpus=json.loads((HERE/'corpus.json').read_text())

class Worker:
    def __init__(self,mode,group='documentation',lifetime='stream',profile='timing'):
        name=('ferro' if mode.startswith('ferro') else 'ox')+'-'+profile
        self.p=subprocess.Popen([str(HERE/name),mode,str(HERE/'corpus.json'),group,lifetime],stdin=subprocess.PIPE,stdout=subprocess.PIPE,text=True)
    def call(self,action,**fields):
        self.p.stdin.write(json.dumps(dict(action=action,**fields))+'\n'); self.p.stdin.flush()
        value=self.p.stdout.readline()
        if not value: raise RuntimeError('Worker exited unexpectedly')
        return json.loads(value)
    def close(self):
        self.p.stdin.close(); self.p.wait(); self.p.stdout.close()
        assert self.p.returncode==0

def write(name,data):
    value=(json.dumps(data,ensure_ascii=False,indent=2)+'\n').encode()
    path=HERE/name
    path.write_bytes(gzip.compress(value,mtime=0) if name.endswith('.gz') else value)

def main():
    assert not (HERE/'windows.json.gz').exists(), 'Use a fresh diagnostic output directory'
    original=json.loads(gzip.decompress((ROOT/'docs/reports/2026-09-13-workflow-engine-field/outputs.json.gz').read_bytes()))
    verified={}
    for mode in MODES:
        w=Worker(mode); result=w.call('verify'); again=w.call('verify'); w.close()
        assert result==again, 'Document state changed output'
        ref='ferromark' if mode.startswith('ferro') else 'ox-content'
        old=original[ref]['stream']['outputs']
        assert len(result['outputs'])==len(old)==12
        for a,b in zip(old,result['outputs']):
            assert a['id']==b['id']
            if mode.endswith('-ids'):
                assert without_generated_heading_ids(a['html'])==without_generated_heading_ids(b['html'])
            else: assert a['html']==b['html'], (mode,a['id'])
        verified[mode]=result
    write('outputs.json.gz',verified)
    counts={}
    for i,doc in enumerate(corpus['documentation']):
        w=Worker('ferro-fresh','doc-'+str(i),profile='counters')
        raw=w.call('counts'); w.close()
        counts[doc['id']]={k:int(v) for k,v in re.findall(r'(\w+): (\d+)',raw['counters'])}
    write('counters.json',counts)
    windows=[]; warmups=[]; observations=[observation()]
    variants=[(m,'documentation',life) for life in ('stream','retain') for m in MODES]
    variants += [(m,'doc-'+str(i),'stream') for i in range(12) for m in ('ferro-fresh','ferro-reuse','ox-reuse','ox-fresh')]
    for round_id in range(3):
        order=variants[round_id:]+variants[:round_id]
        # Rotate engines within each input/lifetime; fresh process for each round.
        cases=list(dict.fromkeys((group,life) for _,group,life in order))
        for group,life in cases:
            modes=[m for m,g,l in order if (g,l)==(group,life)]
            workers={m:Worker(m,group,life) for m in modes}
            for mode,w in workers.items():
                warmups.append(dict(round=round_id,mode=mode,group=group,lifetime=life,**w.call('time',milliseconds=300 if group=='documentation' else 100)))
            for window in range(9):
                offset=(round_id+window)%len(modes)
                rotated=modes[offset:]+modes[:offset]
                if window%2: rotated.reverse()
                for position,mode in enumerate(rotated):
                    result=workers[mode].call('time',milliseconds=63 if group=='documentation' else 30)
                    documents=corpus[group]
                    lookup={d['id']:d['html'] for d in verified[mode]['outputs']}
                    size=sum(len(lookup[d['id']].encode()) for d in documents)
                    assert result['output_units']==result['iterations']*size
                    windows.append(dict(round=round_id,window=window,position=position,mode=mode,group=group,lifetime=life,**result))
            for w in workers.values(): w.close()
        observations.append(observation())
        print('Diagnostic round',round_id+1,'complete',flush=True)
    write('windows.json.gz',windows); write('warmups.json',warmups); write('observations.json',observations)
    rows=[]
    for mode,group,life in variants:
        samples=[r for r in windows if (r['mode'],r['group'],r['lifetime'])==(mode,group,life)]
        medians=[statistics.median(r['ns_per_workload'] for r in samples if r['round']==i) for i in range(3)]
        median=statistics.median(medians)
        rows.append(dict(mode=mode,group=group,lifetime=life,median_us=median/1000,round_medians_us=[n/1000 for n in medians],spread_percent=(max(medians)-min(medians))/median*100))
    write('summary.json',rows)
    for r in rows[:12]:print(r['mode'],r['lifetime'],round(r['median_us'],2),'us',flush=True)

if __name__=='__main__': main()
