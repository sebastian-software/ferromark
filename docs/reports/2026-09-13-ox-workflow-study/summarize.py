"""Verify the archived diagnostic observations and regenerate their tables."""
from pathlib import Path
import argparse, gzip, hashlib, json, math, re, statistics, sys

HERE=Path(__file__).resolve().parent
MODES=['ferro-fresh','ferro-reuse','ferro-fresh-ids','ferro-reuse-ids','ox-reuse','ox-fresh']
LABELS={'ferro-fresh':'Ferromark fresh API, IDs off','ferro-reuse':'Ferromark reusable Renderer, IDs off',
        'ferro-fresh-ids':'Ferromark fresh API, IDs on','ferro-reuse-ids':'Ferromark reusable Renderer, IDs on',
        'ox-reuse':'Ox reusable HTML renderer, fresh arena/AST','ox-fresh':'Ox fresh HTML renderer, fresh arena/AST'}

def read(name):
    raw=(HERE/name).read_bytes()
    return json.loads(gzip.decompress(raw) if name.endswith('.gz') else raw)

def require(ok,message):
    if not ok:raise ValueError(message)

def validate():
    hashes=read('checksums.json')
    for name,digest in hashes.items():
        require(hashlib.sha256((HERE/name).read_bytes()).hexdigest()==digest,'Changed evidence: '+name)
    corpus=read('corpus.json');outputs=read('outputs.json.gz');windows=read('windows.json.gz');warmups=read('warmups.json')
    require(set(outputs)==set(MODES),'Missing API mode')
    require(corpus['documentation']==read('../2026-09-13-workflow-engine-field/corpus.json')['documentation'],'Original practical inputs changed')
    sys.path.insert(0,str(HERE.parents[2]/'benchmarks/workflows/field'))
    from common import without_generated_heading_ids
    original=read('../2026-09-13-workflow-engine-field/outputs.json.gz')
    for mode in MODES:
        reference=original['ferromark' if mode.startswith('ferro') else 'ox-content']['stream']['outputs']
        actual=outputs[mode]['outputs']
        require([r['id'] for r in actual]==[r['id'] for r in reference],'Missing original document output')
        for a,b in zip(actual,reference):
            if mode.endswith('-ids'):
                require(without_generated_heading_ids(a['html'])==without_generated_heading_ids(b['html']),'Heading-ID control changed content')
            else:require(a['html']==b['html'],'API control changed original output')
    variants=[(m,'documentation',life) for life in ('stream','retain') for m in MODES]
    variants += [(m,'doc-'+str(i),'stream') for i in range(12) for m in ('ferro-fresh','ferro-reuse','ox-reuse','ox-fresh')]
    require(len(windows)==len(variants)*27,'Incomplete diagnostic timing windows')
    expected={(m,g,l,r,w) for m,g,l in variants for r in range(3) for w in range(9)}
    require({(r['mode'],r['group'],r['lifetime'],r['round'],r['window']) for r in windows}==expected,'Duplicate or missing diagnostic window')
    require(len(warmups)==len(variants)*3 and {(r['mode'],r['group'],r['lifetime'],r['round']) for r in warmups}=={(m,g,l,r) for m,g,l in variants for r in range(3)},'Incomplete warmups')
    for round_id in range(3):
        order=variants[round_id:]+variants[:round_id]
        for group,life in dict.fromkeys((g,l) for _,g,l in order):
            modes=[m for m,g,l in order if (g,l)==(group,life)]
            for window in range(9):
                offset=(round_id+window)%len(modes)
                expected_order=modes[offset:]+modes[:offset]
                if window%2:expected_order.reverse()
                actual=sorted((r for r in windows if (r['group'],r['lifetime'],r['round'],r['window'])==(group,life,round_id,window)),key=lambda r:r['position'])
                require([r['mode'] for r in actual]==expected_order,'Diagnostic engine order changed')
    for r in windows+warmups:
        require(r['iterations']>0 and r['iterations']%4==0,'Invalid workload count')
        minimum=(63 if r['group']=='documentation' else 30) if 'window' in r else (300 if r['group']=='documentation' else 100)
        require(r['elapsed_ns']>=minimum*1_000_000,'Short diagnostic window')
        require(math.isclose(r['ns_per_workload'],r['elapsed_ns']/r['iterations'],rel_tol=1e-15),'Inconsistent duration/count')
        html={d['id']:d['html'] for d in outputs[r['mode']]['outputs']}
        size=sum(len(html[d['id']].encode()) for d in corpus[r['group']])
        require(r['output_units']==r['iterations']*size,'Changed completed output work')
    rows=[]
    for mode,group,life in variants:
        samples=[r for r in windows if (r['mode'],r['group'],r['lifetime'])==(mode,group,life)]
        medians=[statistics.median(r['ns_per_workload'] for r in samples if r['round']==i) for i in range(3)]
        median=statistics.median(medians)
        rows.append(dict(mode=mode,group=group,lifetime=life,median_us=median/1000,round_medians_us=[v/1000 for v in medians],spread_percent=(max(medians)-min(medians))/median*100))
    require(rows==read('summary.json'),'Diagnostic summary does not reproduce')
    inputs=read('cache-inputs.json');refs=read('cache-references.json.gz');sequences=read('cache-sequences.json.gz');validation=read('cache-validation.json')
    require(len(inputs['unique'])==53 and len(inputs['sequence'])==117,'State-control inputs changed')
    require(set(refs)==set(sequences)==set(MODES),'Missing state-control mode')
    checked=0
    for mode in MODES:
        require(len(refs[mode])==len(inputs['unique']),'Missing fresh-process reference')
        expected={hashlib.sha256(value.encode()).hexdigest() for value in inputs['unique']}
        require({r['input_sha256'] for r in refs[mode]}==expected,'Fresh-process input identity changed')
        lookup={r['input_sha256']:r['html'] for r in refs[mode]}
        for storage in ['same-address','distinct-addresses']:
            answers=sequences[mode][storage]
            require(len(answers)==117,'Incomplete state sequence')
            for value,answer in zip(inputs['sequence'],answers):
                require(answer['html']==lookup[hashlib.sha256(value.encode()).hexdigest()],'Stale or state-dependent HTML')
                checked+=1
            if storage=='same-address':require(len({r['address'] for r in answers})==1,'Did not reuse input address')
            else:require(len({r['address'] for r,s in zip(answers,inputs['sequence']) if s})==sum(bool(s) for s in inputs['sequence']),'Input allocations were not distinct')
            require(any(r['html']!=answers[0]['html'] for r in answers),'Constant-output negative control cannot distinguish inputs')
    require(validation['checked_outputs']==checked==1404 and validation['fresh_processes']==318,'Wrong cache-control counts')
    require(validation['negative_controls']==[[m,s,'constant-output cache rejected'] for m in MODES for s in ['same-address','distinct-addresses']],'Missing negative controls')
    cache=read('cache-windows.json.gz');warming=read('cache-warmups.json')
    variants=[(m,p) for m in ['ferro-fresh','ferro-reuse','ox-reuse','ox-fresh'] for p in ['repeated','relocated','changed-content']]
    require(len(cache)==324 and {(r['mode'],r['policy'],r['round'],r['window']) for r in cache}=={(m,p,r,w) for m,p in variants for r in range(3) for w in range(9)},'Incomplete cache sensitivity observations')
    require(len(warming)==36 and {(r['mode'],r['policy'],r['round']) for r in warming}=={(m,p,r) for m,p in variants for r in range(3)},'Missing cache sensitivity warmups')
    for round_id in range(3):
        for window in range(9):
            offset=(round_id+window)%len(variants)
            expected_order=variants[offset:]+variants[:offset]
            if window%2:expected_order.reverse()
            actual=sorted((r for r in cache if (r['round'],r['window'])==(round_id,window)),key=lambda r:r['position'])
            require([(r['mode'],r['policy']) for r in actual]==expected_order,'Cache sensitivity engine order changed')
    for r in cache+warming:
        require(r['iterations']>0 and r['iterations']%4==0 and r['output_bytes']>0,'Missing cache-control work')
        require(r['elapsed_ns']>=(63 if 'window' in r else 300)*1_000_000,'Short cache-control measurement')
        require(math.isclose(r['ns_per_collection'],r['elapsed_ns']/r['iterations'],rel_tol=1e-15),'Inconsistent cache-control time')
        require(r['pool_size']==(1 if r['policy']=='repeated' else 32),'Wrong input pool size')
    cache_rows=[]
    for m,p in variants:
        medians=[statistics.median(r['ns_per_collection'] for r in cache if (r['mode'],r['policy'],r['round'])==(m,p,i))/1000 for i in range(3)]
        cache_rows.append(dict(mode=m,policy=p,median_us=statistics.median(medians),round_medians_us=medians))
    require(cache_rows==read('cache-summary.json'),'Cache summary does not reproduce')
    return rows,cache_rows,validation,corpus

def render():
    rows,cache,validation,corpus=validate()
    table=['# Diagnostic results: Ox implementation and benchmark audit','',
        'Generated by `summarize.py` from the archived observations. These are separate diagnostic runs, not replacements for the published full-field measurements.','',
        '## API lifecycle control on the complete collection','',
        '| API / lifecycle | Release each (µs) | Keep all (µs) | Release round spread |',
        '| --- | ---: | ---: | ---: |']
    lookup={(r['mode'],r['group'],r['lifetime']):r for r in rows}
    for mode in MODES:
        a,b=lookup[mode,'documentation','stream'],lookup[mode,'documentation','retain']
        table.append(f"| {LABELS[mode]} | {a['median_us']:.2f} | {b['median_us']:.2f} | {a['spread_percent']:.2f}% |")
    table += ['', 'All modes create owned HTML and release it inside each workload. Ox always creates a fresh growing arena and AST. Its generated heading IDs remain enabled; Ferromark has both ID settings as explicit controls.','',
        '## Input identity and content controls','',
        '| API | Repeated collection (µs) | Same text, different addresses (µs) | Changing nonce text (µs) |',
        '| --- | ---: | ---: | ---: |']
    c={(r['mode'],r['policy']):r['median_us'] for r in cache}
    for mode in ['ferro-fresh','ferro-reuse','ox-reuse','ox-fresh']:
        values=' | '.join(f"{c[mode,p]:.2f}" for p in ['repeated','relocated','changed-content'])
        table.append(f'| {LABELS[mode]} | {values} |')
    table += ['', 'Every version appends a fixed-width nonce paragraph. The relocated and changing-content controls cycle through 32 preloaded collections. Pool construction and output verification are outside the timer; completed HTML is owned and dropped inside it. These changed-input diagnostics have their own compiler unit and must not be numerically substituted into the original collection run.','',
        f"State checks: {validation['checked_outputs']:,} exact HTML comparisons across {validation['fresh_processes']} fresh-process references, {validation['unique_inputs']} unique inputs, and {validation['sequence_length']}-document sequences. All passed; all twelve constant-output negative controls were rejected.",'',
        '## Individual document localization','',
        '| Document | Ferromark fresh (µs) | Ferromark reused (µs) | Ox reused (µs) | Ox fresh (µs) |',
        '| --- | ---: | ---: | ---: | ---: |']
    for i,doc in enumerate(corpus['documentation']):
        values=' | '.join(f"{lookup[m,'doc-'+str(i),'stream']['median_us']:.2f}" for m in ['ferro-fresh','ferro-reuse','ox-reuse','ox-fresh'])
        table.append(f"| {doc['id']} | {values} |")
    table += ['', 'Single-document measurements warm that document repeatedly. Their instruction/data working set differs from cycling the complete collection; these medians cannot be summed to reconstruct collection time.','',
        '## Pipeline work counters','', '| Counter over twelve documents | Count |','| --- | ---: |']
    counters=read('counters.json')
    for key in ['block_events','inline_parses','inline_fast_paths','inline_marks','inline_emit_points','inline_events','paragraph_copied_bytes']:
        table.append(f'| {key} | {sum(row[key] for row in counters.values()):,} |')
    profiles=[]
    for file in sorted((HERE/'profiles').glob('*.txt.gz')):
        raw=gzip.decompress(file.read_bytes()).decode()
        total=int(re.search(r'Call graph:\n\s+(\d+) Thread',raw)[1])
        section=raw.split('Sort by top of stack, same collapsed (when >= 5):')[1].split('Binary Images:')[0]
        leaves=[]
        for line in section.splitlines():
            match=re.match(r'\s+(.+?)\s+(\d+)\s*$',line)
            if match:leaves.append((match[1],int(match[2])))
        counts={'ferro_inline_parser':sum(n for label,n in leaves if 'InlineParser30parse_with_options_in_document' in label),
                'ferro_emitpoint_sort':sum(n for label,n in leaves if 'sort' in label and 'EmitPoint' in label),
                'ferro_inline_render':sum(n for label,n in leaves if '21render_inline_content' in label),
                'ox_inline_parser':sum(n for label,n in leaves if 'Parser12parse_inline' in label or 'InlineMarkerScan4next' in label),
                'ox_parse':sum(n for label,n in leaves if 'Parser5parse' in label)}
        profiles.append(dict(profile=file.name,samples=total,counts=counts,percent={k:round(v/total*100,2) for k,v in counts.items()}))
    require(profiles==read('profile-summary.json'),'Profile summary does not reproduce')
    table += ['', '## CPU profile localization', '',
              '| Capture | Thread samples | Ferromark inline parser | Ferromark inline rendering wrapper | Visible EmitPoint sort |',
              '| --- | ---: | ---: | ---: | ---: |']
    for row in profiles:
        if not row['profile'].startswith('ferro'):continue
        p=row['percent']
        table.append(f"| {row['profile']} | {row['samples']} | {p['ferro_inline_parser']:.2f}% | {p['ferro_inline_render']:.2f}% | {p['ferro_emitpoint_sort']:.2f}% |")
    table += ['', 'These are self/leaf sample shares from the collapsed listing, which omits symbols with fewer than five samples. Inlined functions may be attributed to their caller; the visible sort share is not a complete phase timer. Ox captures show parsing and rendering under repeated input as well. Cross-engine inlining and profile sample percentages do not establish exact causal speedup shares.']
    return '\n'.join(table)+'\n'

if __name__=='__main__':
    parser=argparse.ArgumentParser();parser.add_argument('--check',action='store_true');args=parser.parse_args()
    result=render();path=HERE/'RESULTS.md'
    if args.check:require(path.read_text()==result,'Generated audit results drift')
    else:path.write_text(result)
    print('Ox workflow audit evidence and generated tables verified')
