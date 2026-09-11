#!/usr/bin/env python3
"""Confirm retained production code with the unchanged five-parser native driver."""
from pathlib import Path
import difflib,hashlib,json,os,shutil,statistics,subprocess
from experiment import HERE,ROOT,WORK,BASE,BUN,PRIMARY

def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()

def main():
    out=HERE/'production-confirmation';out.mkdir(exist_ok=False)
    shutil.copyfile(HERE/'original-bun-workspace.toml',BUN/'Cargo.toml')
    shutil.copyfile(HERE/'Cargo.lock',BUN/'Cargo.lock')
    env=os.environ.copy();env.update(BUN_CODEGEN_DIR='/private/tmp/ferromark-bun-build-20260911/codegen',RUSTFLAGS='-C target-cpu=generic')
    with (out/'build.log').open('w') as log:
        subprocess.run(['cargo','+nightly-2026-07-20','build','--offline','--locked','--release','-p','ferromark-bun-comparison'],cwd=BUN,env=env,stdout=log,stderr=subprocess.STDOUT,check=True)
    binary=BUN/'target/release/ferromark-bun-comparison';after=WORK/'production-after';shutil.copy2(binary,after)
    before=WORK/'publication-baseline';paths={'before':before,'after':after}
    stamp_path=BUN/'ferromark-comparison/build-info.json';stamp=json.loads(stamp_path.read_text())
    source={str(p.relative_to(ROOT)):sha(p) for p in sorted((ROOT/'src').rglob('*.rs'))}
    stamp.update(binary_sha256=sha(binary),ferromark_source_sha256=source);stamp_path.write_text(json.dumps(stamp,indent=2)+'\n')
    (out/'build-info.json').write_text(json.dumps(stamp,indent=2)+'\n')
    assert (BUN/'ferromark-comparison/driver.rs').read_bytes()==(ROOT/'benchmarks/bun-comparison/driver.rs').read_bytes()
    (out/'protocol.json').write_text(json.dumps({'rounds':2,'windows_per_process':20,'window_ms':40,'warmup_ms':250,'cases':PRIMARY,'driver_sha256':sha(ROOT/'benchmarks/bun-comparison/driver.rs'),'binaries':{k:sha(p) for k,p in paths.items()},'order':'case-by-case before/after, reversing each round; alternating all five parsers inside every process','statistic':'median of two process medians; paired changes between process medians','scope':'focused diagnostic, not the publication protocol'},indent=2)+'\n')
    patch=''.join(''.join(difflib.unified_diff(p.read_text().splitlines(True),(ROOT/p.relative_to(BASE)).read_text().splitlines(True),fromfile='a/'+str(p.relative_to(BASE)),tofile='b/'+str(p.relative_to(BASE)))) for p in (BASE/'src').rglob('*.rs'))
    (out/'retained.patch').write_text(patch)
    for mode in ['verify','spec']:
        data={}
        for variant,b in paths.items():
            raw=subprocess.check_output([str(b),mode],text=True);(out/f'{variant}-{mode}.jsonl').write_text(raw);data[variant]=[json.loads(line) for line in raw.splitlines()]
        assert data['before']==data['after'],mode
        print('Exact output equality:',mode,len(data['after']),flush=True)
    rows=[];allow=WORK/'production-case.json'
    with (out/'samples.jsonl').open('w') as stream:
        for r in range(2):
            for i,case in enumerate(PRIMARY[r:]+PRIMARY[:r]):
                allow.write_text(json.dumps([case])+'\n')
                order=['before','after'] if (r+i)%2==0 else ['after','before']
                for variant in order:
                    raw=subprocess.check_output([str(paths[variant]),'bench',str(allow),'20','40','250',str(r)],text=True)
                    records=[json.loads(line) for line in raw.splitlines()];assert len(records)==5 and all(x['case']==case and len(x['ns_per_render'])==20 for x in records)
                    for row in records:
                        row.update(variant=variant,round=r);rows.append(row);stream.write(json.dumps(row)+'\n');stream.flush()
                print('Production',r+1,case,flush=True)
    summary=[]
    for case in PRIMARY:
        item={'case':case,'parsers':{}}
        for parser in ['ferromark','pulldown-cmark','bun_md','comrak','md4c']:
            d={}
            for variant in paths:
                medians=[statistics.median(x['ns_per_render']) for x in rows if x['case']==case and x['variant']==variant and x['parser']==parser]
                assert len(medians)==2;d[variant]={'median_ns':statistics.median(medians),'process_medians_ns':medians}
            item['parsers'][parser]=d
        f=item['parsers']['ferromark'];p=item['parsers']['pulldown-cmark']
        item['paired_change_percent']=statistics.median([(b/a-1)*100 for a,b in zip(f['before']['process_medians_ns'],f['after']['process_medians_ns'])])
        item['after_vs_pulldown_percent']=(f['after']['median_ns']/p['after']['median_ns']-1)*100
        summary.append(item)
    (out/'summary.json').write_text(json.dumps(summary,indent=2)+'\n')
    for row in summary:print(row['case'],round(row['paired_change_percent'],2),round(row['after_vs_pulldown_percent'],2),flush=True)

if __name__=='__main__':main()
