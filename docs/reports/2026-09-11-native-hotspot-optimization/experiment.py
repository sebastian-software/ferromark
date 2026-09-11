#!/usr/bin/env python3
"""Native/shared-mimalloc hotspot experiments; isolated source, frozen baseline."""
from pathlib import Path
import argparse,difflib,gzip,hashlib,importlib.util,json,os,shutil,statistics,subprocess,time
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[2]
WORK=ROOT/'target/native-hotspot'
BASE=WORK/'baseline'
SOURCE=WORK/'source'
BUN=Path('/private/tmp/ferromark-bun-publication-20260911')
PRIMARY=['commonmark/tiny','commonmark/short-100b','gfm_overlap/tiny','gfm_overlap/short-100b','tables/tables-plain','tables/tables-commonmark-inline','tables/tables-links','commonmark/publication-5k']

def inputs():
    catalog=json.loads(gzip.decompress((ROOT/'docs/reports/2026-09-11-table-optimization-refresh/native/catalog.jsonl.gz').read_bytes()))
    texts=dict(catalog)
    result={case:(0 if case.startswith('commonmark/') else 7 if case.startswith('gfm_overlap/') else 1,texts[case.split('/')[1]]) for case in PRIMARY}
    spec=importlib.util.spec_from_file_location('prior',ROOT/'docs/reports/2026-09-11-table-optimization-variants/experiment.py');mod=importlib.util.module_from_spec(spec);spec.loader.exec_module(mod)
    result.update({f'guard/{name}':(1,text) for name,text in mod.fixtures().items()})
    result['guard/references']=(0,'[x]: /url "title"\n\n[x] and **[x]**\n\n'+('[x](/inline) *word*\n\n'*40))
    result['guard/empty']=(0,'')
    result['guard/gfm-document']=(7,texts['publication-5k'])
    result['guard/long-delimiters']=(1,'| a | b | c | d |\n|'+ '|'.join(['-'*4096]*4)+'|\n| 1 | 2 | 3 | 4 |\n')
    result['guard/deep-emphasis']=(0,'*a '*512+'z'+' b*'*512)
    return result

def change(text,old,new):
    assert old in text,old[:100]
    return text.replace(old,new,1)

def transform(name,files):
    if name in ('combined','combined-delimiter','combined-lean','final','final-linear','final-singlepass'):
        suffix={ 'combined':'', 'combined-delimiter':'+delimiter', 'combined-lean':'+lazy-ref-label+lazy-unused-state', 'final':'+lazy-ref-label+lazy-unused-state+delimiter', 'final-linear':'+lazy-ref-label+lazy-unused-state+linear-delimiter', 'final-singlepass':'+lazy-ref-label+lazy-unused-state+single-pass-delimiter' }[name]
        name='lazy-emphasis+scalar-neon-tail+single-paragraph+output-min64-nonempty+lazy-paragraph-buffer'+suffix
    for variant in name.split('+'):
        if variant=='baseline':continue
        if variant=='small-buffers':
            f='src/lib.rs';s=files[f]
            s=s.replace('(input.len() / 16).max(64)','(input.len() / 16).max(8)')
            s=change(s,'let mut inline_events = Vec::with_capacity(64);','let mut inline_events = Vec::with_capacity(16);');files[f]=s
        elif variant=='lazy-emphasis':
            f='src/inline/emphasis.rs';s=files[f];start=s.index('    pub fn reserve_for_marks(');end=s.index('\n}\n',start)
            s=s[:start]+s[end:];s=change(s,'    stacks.reserve_for_marks(marks.len());\n','');files[f]=s
        elif variant=='inline-content':
            f='src/lib.rs';files[f]=change(files[f],'fn render_inline_content(', '#[inline(always)]\nfn render_inline_content(')
        elif variant=='lazy-ref-label':
            f='src/block/parser.rs';files[f]=change(files[f],'link_ref_label_buf: String::with_capacity(64),','link_ref_label_buf: String::new(),')
        elif variant=='lazy-render-state':
            f='src/lib.rs';s=files[f]
            s=s.replace('content: Vec::with_capacity(256),','content: Vec::new(),').replace('content: Vec::with_capacity(64),','content: Vec::new(),')
            s=s.replace('arena: Vec::with_capacity(256),','arena: Vec::new(),').replace('slug_buf: Vec::with_capacity(64),','slug_buf: Vec::new(),')
            s=change(s,'used: std::collections::HashMap::with_capacity_and_hasher(\n                32,','used: std::collections::HashMap::with_capacity_and_hasher(\n                0,');files[f]=s
        elif variant=='dense-events':
            f='src/lib.rs';files[f]=files[f].replace('(input.len() / 16).max(64)','(input.len() / if options.tables { 4 } else { 16 }).max(64)')
        elif variant=='scalar-neon-tail':
            f='src/byte_search.rs';files[f]=change(files[f],'if N > 5 {','if N > 5 || cfg!(target_arch = "aarch64") {')
        elif variant=='output-min64':
            f='src/render.rs';files[f]=change(files[f],'let capacity = input_len + input_len / 4;','let capacity = (input_len + input_len / 4).max(64);')
        elif variant=='output-min64-nonempty':
            f='src/render.rs';files[f]=change(files[f],'let capacity = input_len + input_len / 4;','let capacity = if input_len == 0 { 0 } else { (input_len + input_len / 4).max(64) };')
        elif variant=='lazy-paragraph-buffer':
            f='src/lib.rs';files[f]=change(files[f],'content: Vec::with_capacity(256),','content: Vec::new(),')
        elif variant=='lazy-unused-state':
            f='src/lib.rs';files[f]=files[f].replace('content: Vec::with_capacity(64),','content: Vec::new(),')
        elif variant=='inline-emphasis':
            files=transform('lazy-emphasis',files)
            f='src/inline/emphasis.rs';files[f]=files[f].replace('use super::{','use smallvec::SmallVec;\n\nuse super::{',1).replace('[Vec<OpenerEntry>; 6]','[SmallVec<[OpenerEntry; 1]>; 6]')
        elif variant=='single-paragraph':
            f='src/lib.rs';marker='    fn render_events(&mut self, input: &[u8], mut events: &[BlockEvent]) {\n'
            files[f]=change(files[f],marker,marker+'''        if let [BlockEvent::ParagraphStart, BlockEvent::Text(range), BlockEvent::ParagraphEnd] = events {
            let text = range.slice(input);
            let mut end = text.len();
            while end > 0 && matches!(text[end - 1], b' ' | b'\\t') { end -= 1; }
            self.writer.paragraph_start();
            if end > 0 {
                render_inline_content(&text[..end], self.writer, self.inline_parser,
                    self.inline_events, self.link_refs, self.footnote_store,
                    self.footnote_numbers, self.options);
            }
            self.writer.paragraph_end();
            return;
        }
''')
        elif variant=='delimiter':
            spec=importlib.util.spec_from_file_location('prior',ROOT/'docs/reports/2026-09-11-table-optimization-variants/experiment.py');mod=importlib.util.module_from_spec(spec);spec.loader.exec_module(mod)
            files.update(mod.transform('delimiter',files))
        elif variant=='linear-delimiter':
            f='src/block/parser.rs';s=files[f]
            start=s.index('        let (cells, truncated) = Self::split_table_cells(line);',s.index('    fn is_delimiter_row('))
            end=s.index('        Some((alignments, truncated))',start)
            s=s[:start]+'''        let mut start = 0;
        let mut end = line.len();
        while start < end && matches!(line[start], b' ' | b'\\t') { start += 1; }
        while end > start && matches!(line[end - 1], b' ' | b'\\t') { end -= 1; }
        if start < end && line[start] == b'|' { start += 1; }
        if end > start && line[end - 1] == b'|' { end -= 1; }
        if start >= end { return None; }
        let mut alignments = SmallVec::new();
        let mut truncated = false;
        loop {
            while start < end && matches!(line[start], b' ' | b'\\t') { start += 1; }
            let left_colon = start < end && line[start] == b':';
            start += usize::from(left_colon);
            let dash_start = start;
            while start < end && line[start] == b'-' { start += 1; }
            if start == dash_start { return None; }
            let right_colon = start < end && line[start] == b':';
            start += usize::from(right_colon);
            while start < end && matches!(line[start], b' ' | b'\\t') { start += 1; }
            if start < end && line[start] != b'|' { return None; }
            alignments.push(match (left_colon, right_colon) {
                (true, true) => Alignment::Center,
                (true, false) => Alignment::Left,
                (false, true) => Alignment::Right,
                (false, false) => Alignment::None,
            });
            if start == end { break; }
            start += 1;
            if alignments.len() >= limits::MAX_TABLE_COLUMNS {
                truncated = start < end;
                break;
            }
        }
''' +s[end:];files[f]=s
        elif variant=='single-pass-delimiter':
            files=transform('linear-delimiter',files);f='src/block/parser.rs';s=files[f]
            start=s.index('    /// Quick pre-filter:');end=s.index('    /// Check if a line is a valid GFM table delimiter row.',start)
            s=s[:start]+s[end:]
            s=change(s,'        if !Self::could_be_delimiter_row(line) {\n            return None;\n        }\n','')
            start=s.index('    fn is_delimiter_row(');end=s.index('        Some((alignments, truncated))',start)
            part=s[start:end]
            part=change(part,'            if alignments.len() >= limits::MAX_TABLE_COLUMNS {\n','''            if alignments.len() >= limits::MAX_TABLE_COLUMNS {
                if line[start..end].iter().any(|&b| !matches!(b, b'-' | b':' | b'|' | b' ' | b'\\t')) {
                    return None;
                }
''')
            files[f]=s[:start]+part+s[end:]
        else:raise ValueError(variant)
    return files

def build(name):
    shutil.copytree(BASE/'src',SOURCE/'src',dirs_exist_ok=True)
    files={str(p.relative_to(BASE)):p.read_text() for p in (BASE/'src').rglob('*.rs')}
    changed=transform(name,files.copy())
    for f,s in changed.items():(SOURCE/f).write_text(s)
    subprocess.run(['rustfmt','--edition','2024',str(SOURCE/'src/lib.rs')],check=True)
    patch=''.join(''.join(difflib.unified_diff(s.splitlines(True),(SOURCE/f).read_text().splitlines(True),fromfile='a/'+f,tofile='b/'+f)) for f,s in files.items())
    (HERE/(name+'.patch')).write_text(patch)
    env=os.environ.copy();env.update(BUN_CODEGEN_DIR='/private/tmp/ferromark-bun-build-20260911/codegen',RUSTFLAGS='-C target-cpu=generic')
    with (HERE/(name+'.build.log')).open('w') as log:
        subprocess.run(['cargo','+nightly-2026-07-20','build','--offline','--locked','--release','-p','ferromark-native-probe'],cwd=BUN,env=env,stdout=log,stderr=subprocess.STDOUT,check=True)
    bins=WORK/'bin';bins.mkdir(exist_ok=True);shutil.copy2(BUN/'target/release/ferromark-native-probe',bins/name)
    (HERE/(name+'.binary.json')).write_text(json.dumps({'sha256':hashlib.sha256((bins/name).read_bytes()).hexdigest(),'flags':env['RUSTFLAGS'],'profile':'Bun release, fat LTO, 1 CGU, panic abort, line tables','allocator':'Bun pinned mimalloc','source':{f:hashlib.sha256((SOURCE/f).read_bytes()).hexdigest() for f in files}},indent=2)+'\n')
    print('Built',name,flush=True)

def invoke(name,flags,text,parser,ms):
    return json.loads(subprocess.check_output([str(WORK/'bin'/name),'probe',str(flags),str(parser),str(ms)],input=text,text=True))

def verify(base,name):
    rows=[]
    for case,(flags,text) in inputs().items():
        outputs=[]
        for variant in [base,name]:
            outputs.append(json.loads(subprocess.check_output([str(WORK/'bin'/variant),'render',str(flags)],input=text,text=True)))
        assert outputs[0]==outputs[1],case
        rows.append({'case':case,'input_sha256':hashlib.sha256(text.encode()).hexdigest(),'exact_baseline_equal':True,'output_sha256':hashlib.sha256(outputs[1]['outputs']['ferromark'].encode()).hexdigest()})
    return rows

def measure(name,out,base='baseline',rounds=5,ms=150,guards=False,case_filter=None):
    out=HERE/out;out.mkdir(exist_ok=False)
    (out/'verification.json').write_text(json.dumps(verify(base,name),indent=2)+'\n')
    selected=inputs();selected={k:v for k,v in selected.items() if guards or k in PRIMARY}
    if case_filter:selected={k:v for k,v in inputs().items() if k in case_filter}
    lanes=[(base,0),(name,0),(base,2)]
    (out/'protocol.json').write_text(json.dumps({'baseline':base,'candidate':name,'rounds':rounds,'window_ms':ms,'warmup_renders_per_window':32,'initial_warmup_ms':300,'clock_batch':16,'lanes':lanes,'order':'rotating cases and rotating/reversing baseline, candidate, pulldown'},indent=2)+'\n')
    for flags,text in selected.values():
        for variant,parser in lanes:invoke(variant,flags,text,parser,300)
    rows=[];cases=list(selected)
    with (out/'samples.jsonl').open('w') as stream:
        for r in range(rounds):
            for i,case in enumerate(cases[r%len(cases):]+cases[:r%len(cases)]):
                order=lanes[(r+i)%3:]+lanes[:(r+i)%3]
                if r%2:order=order[::-1]
                flags,text=selected[case]
                for variant,parser in order:
                    row=invoke(variant,flags,text,parser,ms);row.update(case=case,variant=variant,parser=parser,round=r);rows.append(row);stream.write(json.dumps(row)+'\n');stream.flush()
            print(out.name,'round',r+1,flush=True)
    summary=[]
    for case in cases:
        def values(variant,parser):return [x['elapsed_ns']/x['iterations']/1000 for x in rows if x['case']==case and x['variant']==variant and x['parser']==parser]
        before=values(base,0);after=values(name,0);pull=values(base,2)
        summary.append({'case':case,'before_us':statistics.median(before),'after_us':statistics.median(after),'pulldown_us':statistics.median(pull),'paired_change_percent':statistics.median([(b/a-1)*100 for a,b in zip(before,after)]),'paired_vs_pulldown_percent':statistics.median([(b/a-1)*100 for a,b in zip(pull,after)]),'windows_us':{'before':before,'after':after,'pulldown':pull}})
    (out/'summary.json').write_text(json.dumps(summary,indent=2)+'\n')
    for row in summary:print(row['case'],round(row['paired_change_percent'],2),round(row['paired_vs_pulldown_percent'],2),flush=True)

def profile(name):
    out=HERE/('profiles-'+name);out.mkdir(exist_ok=False)
    for case in ['commonmark/tiny','commonmark/short-100b','tables/tables-plain','tables/tables-commonmark-inline']:
        flags,text=inputs()[case]
        for parser in [0,2]:
            label=case.replace('/','-')+('-ferromark' if parser==0 else '-pulldown')
            with (out/(label+'.process.log')).open('w') as log:
                p=subprocess.Popen([str(WORK/'bin'/name),'probe',str(flags),str(parser),'15000'],stdin=subprocess.PIPE,stdout=log,stderr=log,text=True)
                p.stdin.write(text);p.stdin.close()
                try:
                    time.sleep(.25)
                    result=subprocess.run(['/usr/bin/sample',str(p.pid),'4','1','-mayDie','-fullPaths','-file',str(out/(label+'.sample.txt'))],capture_output=True,text=True,timeout=15)
                    (out/(label+'.sample.log')).write_text(result.stdout+result.stderr)
                    assert result.returncode==0,result.stderr
                    print('Profiled',label,flush=True)
                finally:p.terminate();p.wait(timeout=5)

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('mode',choices=['build','measure','profile']);p.add_argument('name');p.add_argument('--base',default='baseline');p.add_argument('--out');p.add_argument('--rounds',type=int,default=5);p.add_argument('--ms',type=int,default=150);p.add_argument('--guards',action='store_true');p.add_argument('--case',action='append');a=p.parse_args()
    if a.mode=='build':build(a.name)
    elif a.mode=='profile':profile(a.name)
    else:measure(a.name,a.out or 'screen-'+a.name,a.base,a.rounds,a.ms,a.guards,a.case)
