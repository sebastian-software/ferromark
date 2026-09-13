"""State-isolation controls against fresh-process output and changing input identity."""
from pathlib import Path
import gzip,hashlib,json,statistics,subprocess
from run import HERE,MODES,write,corpus
from support import observation

class Worker:
    def __init__(self,mode):
        name='ferro-cache' if mode.startswith('ferro') else 'ox-cache'
        self.p=subprocess.Popen([str(HERE/name),mode,str(HERE/'corpus.json')],stdin=subprocess.PIPE,stdout=subprocess.PIPE,text=True)
    def call(self,action,**fields):
        self.p.stdin.write(json.dumps(dict(action=action,**fields))+'\n');self.p.stdin.flush()
        value=self.p.stdout.readline();assert value
        return json.loads(value)
    def close(self):
        self.p.stdin.close();self.p.wait();self.p.stdout.close();assert self.p.returncode==0

def check(answers,inputs,expected,storage):
    assert len(answers)==len(inputs)
    for row,value in zip(answers,inputs):assert row['html']==expected[value], 'Stale or changed HTML'
    if storage=='same-address':assert len({r['address'] for r in answers})==1
    else:assert len({r['address'] for r,s in zip(answers,inputs) if s})==sum(bool(s) for s in inputs)

def main():
    inputs=[d['input'] for d in corpus['documentation']]
    inputs += [inputs[i%12]+f'\n\nAudit nonce {i:04}.\n' for i in range(32)]
    a='# Alpha\n\n# Alpha\n\n[link][ref]\n\n[ref]: /one\n'
    b='# Bravo\n\n# Bravo\n\n[link][ref]\n\n[ref]: /two\n'
    inputs += [a,b,'','&copy; &amp; &ngE;\n','- [x] done\n- [ ] pending\n',
               '| A | B |\n|---|---|\n| one | two |\n','A **bold** [link](https://example.com/a)\n',
               '日本語 🦀 äöü\n\n`<&>`\n',inputs[0]*3]
    inputs=list(dict.fromkeys(inputs))
    sequence=[a,a,b,a,'',a,inputs[-1],a]+inputs+inputs[::-1]+[a,b,a]
    references={};results={};negative_controls=[]
    for mode in MODES:
        expected={}
        for value in inputs:
            w=Worker(mode);expected[value]=w.call('render',input=value)['html'];w.close()
        assert expected[a]!=expected[b]
        references[mode]=[{'input_sha256':hashlib.sha256(s.encode()).hexdigest(),'html':expected[s]} for s in inputs]
        results[mode]={}
        for storage in ['same-address','distinct-addresses']:
            w=Worker(mode);answers=w.call('sequence',inputs=sequence,storage=storage)['answers'];w.close()
            check(answers,sequence,expected,storage)
            results[mode][storage]=answers
            wrong=[dict(r,html=answers[0]['html']) for r in answers]
            try:check(wrong,sequence,expected,storage)
            except AssertionError:negative_controls.append([mode,storage,'constant-output cache rejected'])
            else:raise AssertionError('Control failed to detect stale output')
        print('Cache/state controls passed:',mode,flush=True)
    write('cache-inputs.json',dict(unique=inputs,sequence=sequence))
    write('cache-references.json.gz',references);write('cache-sequences.json.gz',results)
    write('cache-validation.json',dict(unique_inputs=len(inputs),sequence_length=len(sequence),
        fresh_processes=len(MODES)*len(inputs),checked_outputs=len(MODES)*len(sequence)*2,
        negative_controls=negative_controls))
    samples=[];warmups=[];observations=[observation()]
    modes=['ferro-fresh','ferro-reuse','ox-reuse','ox-fresh']
    variants=[(m,p) for m in modes for p in ['repeated','relocated','changed-content']]
    for round_id in range(3):
        workers={m:Worker(m) for m in modes}
        for mode,policy in variants:warmups.append(dict(round=round_id,policy=policy,**workers[mode].call('pool',policy=policy,milliseconds=300)))
        for window in range(9):
            offset=(round_id+window)%len(variants)
            order=variants[offset:]+variants[:offset]
            if window%2:order.reverse()
            for position,(mode,policy) in enumerate(order):
                row=workers[mode].call('pool',policy=policy,milliseconds=63)
                samples.append(dict(round=round_id,window=window,position=position,policy=policy,**row))
        for w in workers.values():w.close()
        observations.append(observation())
        print('Cache sensitivity round',round_id+1,'complete',flush=True)
    write('cache-windows.json.gz',samples);write('cache-warmups.json',warmups)
    write('cache-observations.json',observations)
    rows=[]
    for mode,policy in variants:
        medians=[statistics.median(r['ns_per_collection'] for r in samples if (r['mode'],r['policy'],r['round'])==(mode,policy,i))/1000 for i in range(3)]
        rows.append(dict(mode=mode,policy=policy,median_us=statistics.median(medians),round_medians_us=medians))
    write('cache-summary.json',rows)
    for r in rows:print(r['mode'],r['policy'],round(r['median_us'],2),'us',flush=True)

if __name__=='__main__':main()
