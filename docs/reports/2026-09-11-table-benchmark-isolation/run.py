#!/usr/bin/env python3
"""Measure isolated table workloads with the previous diagnosis's probe."""
from pathlib import Path
import argparse, hashlib, importlib.util, json, shutil, statistics, subprocess, time
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
BASE = HERE.parent / '2026-09-11-table-profiling'
spec = importlib.util.spec_from_file_location('table_probe_runner', BASE / 'run.py')
probe = importlib.util.module_from_spec(spec)
spec.loader.exec_module(probe)
LANES = ['ferro-owned', 'pull-owned', 'ferro-reuse', 'pull-reuse', 'ferro-retained']
CASES = [
    ('tables-plain', 'fixture', 'tables', 'tables-plain'),
    ('tables-inline', 'fixture', 'overlap', 'tables-inline'),
    ('tables-disabled-syntax', 'fixture', 'tables', 'tables-inline'),
    ('short-cells', 'short', 'tables', None),
    ('long-cells', 'long', 'tables', None),
    ('eight-columns', 'cols8', 'tables', None),
    ('nine-columns', 'cols9', 'tables', None),
]

def input_root(case):
    name, _, _, fixture = case
    if fixture is None:
        return ROOT
    root = ROOT / 'target/table-probe/isolated-inputs' / name
    dest = root / 'benches/fixtures/tables-5k.md'
    dest.parent.mkdir(parents=True, exist_ok=True)
    source = HERE / 'tables-inline.md' if fixture == 'tables-inline' else ROOT / f'benches/fixtures/{fixture}.md'
    shutil.copyfile(source, dest)
    return root

def arguments(binary, mode, case, lane, ms=250):
    _, corpus, config, _ = case
    return list(map(str, [binary, mode, corpus, config, lane, input_root(case), ms]))

def invoke(binary, mode, case, lane):
    result = json.loads(subprocess.check_output(arguments(binary, mode, case, lane), text=True))
    result['case'] = case[0]
    return result

def main():
    ap = argparse.ArgumentParser(); ap.add_argument('mode', choices=['time','counts','sample']); ap.add_argument('--out', type=Path, required=True)
    args = ap.parse_args(); out = args.out
    out.mkdir(parents=True, exist_ok=True)
    binary = probe.build(args.mode == 'counts')
    if args.mode == 'time':
        if (out / 'timings.jsonl').exists():
            raise SystemExit('Use a new directory; refusing to overwrite timings')
        verification = []
        for case in CASES:
            result = invoke(binary, 'verify', case, 'all')
            assert probe.CanonicalHTML(result['ferromark']).tokens == probe.CanonicalHTML(result['pulldown']).tokens, case
            (out / f'{case[0]}.outputs.json').write_text(json.dumps(result, indent=2)+'\n')
            verification.append({'case':case[0], 'config':case[2], 'input_bytes':len(result['input'].encode()), 'input_sha256':hashlib.sha256(result['input'].encode()).hexdigest(), 'normalized_equal':True, 'exact_equal':result['ferromark']==result['pulldown']})
        (out/'verification.json').write_text(json.dumps(verification,indent=2)+'\n')
        metadata = {'revision':probe.command('git','rev-parse','HEAD').strip(), 'rustc':probe.command('rustc','-Vv'), 'flags':(ROOT/'.cargo/config.toml').read_text(), 'allocator':'System', 'rounds':9,'window_ms':250,'warmup_renders':32,'clock_batch':16,'binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'probe_sha256':hashlib.sha256((BASE/'probe.rs').read_bytes()).hexdigest(),'fixture_sha256':{f:hashlib.sha256((HERE/'tables-inline.md' if f == 'tables-inline' else ROOT/f'benches/fixtures/{f}.md').read_bytes()).hexdigest() for f in ['tables-plain','tables-inline']},'production_source_sha256':{str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted((ROOT/'src').rglob('*.rs'))}}
        (out/'metadata.json').write_text(json.dumps(metadata,indent=2)+'\n')
        shutil.copyfile(probe.WORK/'Cargo.lock',out/'Cargo.lock')
        rows=[]
        with (out/'timings.jsonl').open('w') as stream:
            for r in range(9):
                cases = CASES[r%len(CASES):]+CASES[:r%len(CASES)]
                lanes = LANES[r%len(LANES):]+LANES[:r%len(LANES)]
                if r%2: lanes=lanes[::-1]
                for case in cases:
                    for lane in lanes:
                        x=invoke(binary,'time',case,lane);x['round']=r;rows.append(x);stream.write(json.dumps(x)+'\n');stream.flush()
                print(f'round {r+1}/9 complete',flush=True)
        summary=[]
        for case in CASES:
            for lane in LANES:
                values=[x['elapsed_ns']/x['iterations']/1000 for x in rows if x['case']==case[0] and x['lane']==lane]
                summary.append({'case':case[0],'lane':lane,'median_us':statistics.median(values),'min_us':min(values),'max_us':max(values),'windows_us':values})
        (out/'summary.json').write_text(json.dumps(summary,indent=2)+'\n')
    elif args.mode == 'counts':
        rows=[invoke(binary,'counts',case,lane) for case in CASES for lane in LANES]
        (out/'counts.json').write_text(json.dumps(rows,indent=2)+'\n')
    else:
        profiles=[]
        for case in CASES[:3]:
            for lane in ['ferro-reuse','pull-reuse']:
                label=f'{case[0]}-{lane}'
                with (out/f'{label}.process.log').open('w') as log:
                    p=subprocess.Popen(arguments(binary,'forever',case,lane),stdout=log,stderr=log)
                    try:
                        time.sleep(.25);assert p.poll() is None
                        x=subprocess.run(['/usr/bin/sample',str(p.pid),'5','1','-mayDie','-fullPaths','-file',str(out/f'{label}.sample.txt')],capture_output=True,text=True,timeout=20)
                        (out/f'{label}.sample.log').write_text(x.stdout+x.stderr)
                        assert x.returncode==0,x.stderr
                        assert p.poll() is None
                        profiles.append({'case':case[0],'lane':lane,'seconds':5,'interval_ms':1,'file':f'{label}.sample.txt'})
                        print(label,'profile complete',flush=True)
                    finally:
                        p.terminate();p.wait(timeout=5)
        (out/'profiles.json').write_text(json.dumps(profiles,indent=2)+'\n')

if __name__=='__main__':main()
