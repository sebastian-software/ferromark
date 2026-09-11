#!/usr/bin/env python3
"""Reproduce the table diagnosis; run from the repository root on macOS."""
from pathlib import Path
import argparse, hashlib, json, os, platform, shutil, statistics, subprocess, sys, time
ROOT = Path(__file__).resolve().parents[3]
HERE = Path(__file__).resolve().parent
WORK = ROOT / 'target/table-probe'
sys.path.insert(0, str(ROOT / 'benchmarks/bun-comparison'))
from run import CanonicalHTML

def command(*args):
    return subprocess.check_output(list(map(str, args)), cwd=ROOT, text=True)

def build(counts=False):
    (WORK / 'src').mkdir(parents=True, exist_ok=True)
    if not (WORK / 'src/main.rs').exists() or (HERE / 'probe.rs').read_bytes() != (WORK / 'src/main.rs').read_bytes():
        shutil.copyfile(HERE / 'probe.rs', WORK / 'src/main.rs')
    (WORK / 'Cargo.toml').write_text('''[package]
name = "ferromark-table-probe"
version = "0.0.0"
edition = "2024"
[workspace]
[dependencies]
ferromark = { path = "../.." }
pulldown-cmark = "=0.13.4"
serde_json = "1.0"
ferromark-pulldown-comparison = { path = "../../benchmarks/pulldown-comparison", optional = true }
[features]
counts = ["dep:ferromark-pulldown-comparison", "ferromark/profiling"]
[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
debug = true
strip = false
panic = "abort"
''')
    lock = HERE / 'Cargo.lock'
    if lock.exists(): shutil.copyfile(lock, WORK / 'Cargo.lock')
    args = ['cargo','build','--offline','--locked','--release','--manifest-path',WORK/'Cargo.toml']
    if counts: args += ['--features','counts']
    subprocess.run(list(map(str,args)),cwd=ROOT,check=True)
    binary = WORK / ('counts' if counts else 'timing')
    shutil.copy2(WORK/'target/release/ferromark-table-probe', binary)
    return binary

CASES = [(c,'tables') for c in ['published','fixture','control','short','long','emphasis','escaped','cols8','cols9','cols16','rows10','rows1000','many','long-prose']] + [('published','overlap')]
LANES = ['ferro-owned','pull-owned','ferro-reuse','pull-reuse','ferro-retained','ferro-block','pull-events']

def invoke(binary, mode, case, lane, ms=250):
    return json.loads(command(binary,mode,*case,lane,ROOT,ms))

def main():
    ap=argparse.ArgumentParser(); ap.add_argument('mode',choices=['time','counts','sample']); ap.add_argument('--out',type=Path,required=True); ap.add_argument('--rounds',type=int,default=9); args=ap.parse_args()
    args.out.mkdir(parents=True,exist_ok=True)
    binary=build(args.mode=='counts')
    if args.mode=='time':
        verification=[]
        for case in CASES:
            o=invoke(binary,'verify',case,'all')
            match=CanonicalHTML(o['ferromark']).tokens==CanonicalHTML(o['pulldown']).tokens
            assert match,case
            label='-'.join(case)
            (args.out/f'{label}.outputs.json').write_text(json.dumps(o,indent=2)+'\n')
            verification.append({'case':case,'normalized_equal':match,'exact_equal':o['ferromark']==o['pulldown'],'input_sha256':hashlib.sha256(o['input'].encode()).hexdigest()})
        (args.out/'verification.json').write_text(json.dumps(verification,indent=2)+'\n')
        metadata={'revision':command('git','rev-parse','HEAD').strip(),'rustc':command('rustc','-Vv'),'platform':platform.platform(),'machine':platform.machine(),'cpu_flags':(ROOT/'.cargo/config.toml').read_text(),'binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'lock_sha256':hashlib.sha256((WORK/'Cargo.lock').read_bytes()).hexdigest(),'rounds':args.rounds,'window_ms':250,'warmup_renders':32,'clock_batch':16,'allocator':'System','features':'none','order':'rotating/reversing lanes and rotating cases','environment_overrides':{k:v for k,v in os.environ.items() if k in ['RUSTFLAGS','CARGO_ENCODED_RUSTFLAGS','CARGO_BUILD_TARGET']}}
        (args.out/'metadata.json').write_text(json.dumps(metadata,indent=2)+'\n')
        rows=[]
        with (args.out/'timings.jsonl').open('w') as file:
            for round_no in range(args.rounds):
                cases=CASES[round_no%len(CASES):]+CASES[:round_no%len(CASES)]
                lanes=LANES[round_no%len(LANES):]+LANES[:round_no%len(LANES)]
                if round_no%2: lanes=lanes[::-1]
                for case in cases:
                    for lane in lanes:
                        x=invoke(binary,'time',case,lane); x['round']=round_no; rows.append(x); file.write(json.dumps(x)+'\n'); file.flush()
                print(f'round {round_no+1}/{args.rounds} complete',flush=True)
        summary=[]
        for case in CASES:
            for lane in LANES:
                values=[x['elapsed_ns']/x['iterations']/1000 for x in rows if (x['corpus'],x['config'])==case and x['lane']==lane]
                summary.append({'corpus':case[0],'config':case[1],'lane':lane,'median_us':statistics.median(values),'min_us':min(values),'max_us':max(values),'windows_us':values})
        (args.out/'summary.json').write_text(json.dumps(summary,indent=2)+'\n')
    elif args.mode=='counts':
        rows=[invoke(binary,'counts',case,lane) for case in CASES for lane in LANES]
        (args.out/'counts.json').write_text(json.dumps(rows,indent=2)+'\n')
    else:
        profiles=[]
        for case,lane in [(('published','tables'),'ferro-reuse'),(('published','tables'),'pull-reuse'),(('fixture','tables'),'ferro-reuse'),(('fixture','tables'),'pull-reuse'),(('short','tables'),'ferro-retained'),(('short','tables'),'pull-reuse'),(('long','tables'),'ferro-retained')]:
            label='-'.join((*case,lane))
            with (args.out/f'{label}.process.log').open('w') as log:
                p=subprocess.Popen(list(map(str,[binary,'forever',*case,lane,ROOT])),stdout=log,stderr=log)
                try:
                    time.sleep(.25); assert p.poll() is None
                    result=subprocess.run(['/usr/bin/sample',str(p.pid),'5','1','-mayDie','-fullPaths','-file',str(args.out/f'{label}.sample.txt')],capture_output=True,text=True,timeout=20)
                    (args.out/f'{label}.sample.log').write_text(result.stdout+result.stderr)
                    assert result.returncode==0,result.stderr
                    assert p.poll() is None
                    profiles.append({'case':case,'lane':lane,'seconds':5,'interval_ms':1,'file':f'{label}.sample.txt'})
                    print(label,'complete',flush=True)
                finally:
                    p.terminate(); p.wait(timeout=5)
        (args.out/'profiles.json').write_text(json.dumps(profiles,indent=2)+'\n')

if __name__=='__main__': main()
