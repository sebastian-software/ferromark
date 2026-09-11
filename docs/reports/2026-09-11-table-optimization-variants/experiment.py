#!/usr/bin/env python3
"""Isolated, paired table optimization experiments; production is never edited here."""
from pathlib import Path
import argparse,difflib,hashlib,json,shutil,statistics,subprocess
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[2]
WORK=ROOT/'target/table-variants'
SOURCE=WORK/'ferromark'
PROBE=WORK/'probe'
BASE=WORK/'baseline'

def edit(s,old,new):
    assert old in s,old[:100]
    return s.replace(old,new,1)

def transform(name,files):
    p=files['src/block/parser.rs']; lib=files['src/lib.rs']
    for variant in name.split('+'):
        if variant=='inline-split':
            p=edit(p,'    fn split_table_cells(', '    #[inline(always)]\n    fn split_table_cells(')
        elif variant=='short-scan':
            p=edit(p,"let Some(offset) = memchr::memchr3(b'|', b'\\\\', b'`', &line[pos..scan_end]) else {", """let tail = &line[pos..scan_end];
            let boundary = if tail.len() <= 32 {
                tail.iter().position(|&b| matches!(b, b'|' | b'\\\\' | b'`'))
            } else {
                memchr::memchr3(b'|', b'\\\\', b'`', tail)
            };
            let Some(offset) = boundary else {""")
        elif variant=='delimiter':
            start=p.index('        let (cells, truncated) = Self::split_table_cells(line);',p.index('    fn is_delimiter_row('))
            end=p.index('        Some((alignments, truncated))',start)
            p=p[:start]+'''        let mut start = 0;
        let mut end = line.len();
        while start < end && matches!(line[start], b' ' | b'\\t') { start += 1; }
        while end > start && matches!(line[end - 1], b' ' | b'\\t') { end -= 1; }
        if start < end && line[start] == b'|' { start += 1; }
        if end > start && line[end - 1] == b'|' { end -= 1; }
        if start >= end { return None; }
        let mut alignments = SmallVec::new();
        let mut truncated = false;
        loop {
            let cell_end = line[start..end].iter().position(|&b| b == b'|').map_or(end, |n| start + n);
            let (s, e) = Self::trim_cell(&line[start..cell_end], start);
            if s == e { return None; }
            let mut i = s;
            let left = line[i] == b':';
            i += usize::from(left);
            let dash_start = i;
            while i < e && line[i] == b'-' { i += 1; }
            if i == dash_start { return None; }
            let right = i < e && line[i] == b':';
            i += usize::from(right);
            if i != e { return None; }
            alignments.push(match (left, right) {
                (true, true) => Alignment::Center,
                (true, false) => Alignment::Left,
                (false, true) => Alignment::Right,
                (false, false) => Alignment::None,
            });
            if cell_end == end { break; }
            start = cell_end + 1;
            if alignments.len() >= limits::MAX_TABLE_COLUMNS {
                truncated = start < end;
                break;
            }
        }
''' +p[end:]
        elif variant=='batch-cells':
            lib=edit(lib,'''        for event in events.iter() {
            context.render_block_event(input, event);
        }''','''        context.render_events(input, events);''')
            marker="impl<R: FencedCodeRenderer + ?Sized> RenderContext<'_, '_, R> {"
            lib=edit(lib,marker,marker+'''
    fn render_events(&mut self, input: &[u8], mut events: &[BlockEvent]) {
        while let Some(event) = events.first() {
            if let [BlockEvent::TableCellStart { alignment, colspan }, BlockEvent::Text(range), BlockEvent::TableCellEnd, ..] = events {
                let text = range.slice(input);
                if !self.para_state.in_paragraph && !self.heading_state.in_heading
                    && !self.cell_state.in_cell && memchr::memchr(b'\\\\', text).is_none() {
                    if *self.in_table_head { self.writer.th_start(*alignment, *colspan); }
                    else { self.writer.td_start(*alignment, *colspan); }
                    let mut end = text.len();
                    while end > 0 && matches!(text[end - 1], b' ' | b'\\t') { end -= 1; }
                    if end > 0 {
                        render_inline_content(&text[..end], self.writer, self.inline_parser,
                            self.inline_events, self.link_refs, self.footnote_store,
                            self.footnote_numbers, self.options);
                    }
                    if *self.in_table_head { self.writer.th_end(); } else { self.writer.td_end(); }
                    events = &events[3..];
                    continue;
                }
            }
            self.render_block_event(input, event);
            events = &events[1..];
        }
    }
''')
        elif variant=='cells16':p=p.replace('[TableCell; 8]','[TableCell; 16]')
        elif variant!='baseline':raise ValueError(variant)
    return {'src/block/parser.rs':p,'src/lib.rs':lib}

def init():
    if BASE.exists():return
    BASE.mkdir(parents=True)
    archive = HERE / 'baseline.tar.gz'
    if archive.exists():
        import tarfile
        with tarfile.open(archive) as source:
            source.extractall(BASE, filter='data')
        return
    shutil.copytree(ROOT/'src',BASE/'src')
    (BASE/'Cargo.toml').write_bytes((ROOT/'Cargo.toml').read_bytes())
    (HERE/'baseline-source.json').write_text(json.dumps({str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest() for p in (ROOT/'src').rglob('*.rs')},indent=2)+'\n')
    (HERE/'environment.json').write_text(json.dumps({'revision':subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),'rustc':subprocess.check_output(['rustc','-Vv'],text=True),'flags':(ROOT/'.cargo/config.toml').read_text(),'allocator':'System','profile':'opt3 fat LTO CGU1 debug=true panic=abort','timing_features':[]},indent=2)+'\n')

def build(name):
    init();SOURCE.mkdir(parents=True,exist_ok=True);PROBE.mkdir(exist_ok=True)
    shutil.copytree(BASE/'src',SOURCE/'src',dirs_exist_ok=True)
    (SOURCE/'Cargo.toml').write_text('''[package]
name="ferromark"
version="0.8.0"
edition="2024"
[workspace]
[dependencies]
memchr="2.7"
smallvec="=1.15.2"
html-escape={version="=0.2.14",default-features=false}
rustc-hash="2.0"
unicode-ident="1.0"
[features]
default=[]
mdx=[]
profiling=[]
''')
    files={f:(BASE/f).read_text() for f in ['src/block/parser.rs','src/lib.rs']}
    changed=transform(name,files)
    for f,s in changed.items():(SOURCE/f).write_text(s)
    subprocess.run(['rustfmt','--edition','2024',str(SOURCE/'src/lib.rs')],check=True)
    patch=''.join(''.join(difflib.unified_diff(files[f].splitlines(True),(SOURCE/f).read_text().splitlines(True),fromfile='a/'+f,tofile='b/'+f)) for f in files)
    (HERE/(name+'.patch')).write_text(patch)
    (PROBE/'src').mkdir(exist_ok=True)
    shutil.copyfile(ROOT/'docs/reports/2026-09-11-table-profiling/probe.rs',PROBE/'src/main.rs')
    (PROBE/'Cargo.toml').write_text('''[package]
name="ferromark-table-probe"
version="0.0.0"
edition="2024"
[workspace]
[dependencies]
ferromark={path="../ferromark"}
pulldown-cmark="=0.13.4"
serde_json="1.0"
[features]
counts=[]
[profile.release]
opt-level=3
lto="fat"
codegen-units=1
debug=true
strip=false
panic="abort"
''')
    lock=HERE/'Cargo.lock'
    if lock.exists():shutil.copyfile(lock,PROBE/'Cargo.lock')
    else:subprocess.run(['cargo','generate-lockfile','--offline','--manifest-path',str(PROBE/'Cargo.toml')],check=True)
    subprocess.run(['cargo','build','--release','--locked','--offline','--manifest-path',str(PROBE/'Cargo.toml')],check=True)
    shutil.copyfile(PROBE/'Cargo.lock',lock)
    dest=WORK/'bin'/name;dest.parent.mkdir(exist_ok=True)
    shutil.copy2(PROBE/'target/release/ferromark-table-probe',dest)
    (HERE/(name+'.binary.json')).write_text(json.dumps({'sha256':hashlib.sha256(dest.read_bytes()).hexdigest(),'source_sha256':{f:hashlib.sha256((SOURCE/f).read_bytes()).hexdigest() for f in files}},indent=2)+'\n')
    return dest

def fixtures():
    result={name:(ROOT/f'benches/fixtures/{f}.md').read_text() for name,f in [('plain','tables-plain'),('mixed','tables-commonmark-inline'),('links','tables-links'),('mixed-document','commonmark-5k')]}
    def table(cols,rows,cell):
        row='|' + (f' {cell} |'*cols)+'\n'
        return row+'|'+' --- |'*cols+'\n'+row*rows
    result.update({'one-table':table(4,100,'data'),'long-cells':table(4,100,'data'*32),'wide9':table(9,100,'data'),'wide16':table(16,100,'data'),
      'escapes-code':table(4,50,r'`a\|b` and da\|ta **ok** &amp;'),
      'prose':('Ordinary prose stays deliberately uneventful.\n\n'*150),
      'links-prose':('[Guide](https://example.com/guide?a=one&b=two) **label**\n\n'*100)})
    import random
    rng = random.Random(20260911)
    fragments = []
    for _ in range(256):
        cols = rng.choice([1, 2, 3, 8, 9, 16, 127, 128, 129])
        delimiter = '|'.join(rng.choice(['---', ':---:', ' ---: ', ':-', '', ' : : ', '--x']) for _ in range(cols))
        head = '|'.join([' H '] * cols)
        body = '|'.join(rng.choice(['plain', '*em*', '**strong**', '`code`', r'a\|b', '&amp;', '[x](/url)', '', ' café ']) for _ in range(cols))
        fragments.append(rng.choice(['', '|']) + head + rng.choice(['', '|']) + '\n' + delimiter + '\n' + body + '\n\n')
    result['edge-cases'] = ''.join(fragments)
    return result

def invoke(binary,mode,root,lane,ms):
    return json.loads(subprocess.check_output(list(map(str,[binary,mode,'fixture','tables',lane,root,ms])),text=True))

def measure(name,rounds,ms,out,base='baseline',lanes=None,selected=None):
    lanes=lanes or ['ferro-reuse']
    out=HERE/out;out.mkdir(exist_ok=False)
    roots={};verified=[]
    for case,text in fixtures().items():
        if selected and case not in selected:continue
        root=WORK/'inputs'/case;p=root/'benches/fixtures/tables-5k.md';p.parent.mkdir(parents=True,exist_ok=True);p.write_text(text);roots[case]=root
        before=invoke(WORK/'bin'/base,'verify',root,'all',ms)
        after=invoke(WORK/'bin'/name,'verify',root,'all',ms)
        assert before==after,(name,case)
        (out/(case+'.outputs.json')).write_text(json.dumps(after,indent=2)+'\n')
        verified.append({'case':case,'exact_baseline_equal':True,'input_sha256':hashlib.sha256(text.encode()).hexdigest()})
    (out/'verification.json').write_text(json.dumps(verified,indent=2)+'\n')
    (out/'protocol.json').write_text(json.dumps({'baseline':base,'candidate':name,'rounds':rounds,'window_ms':ms,'lanes':lanes,'warmup_renders':32,'clock_batch':16,'order':'rotating cases; alternating baseline/candidate'},indent=2)+'\n')
    rows=[];cases=list(roots)
    with (out/'timings.jsonl').open('w') as stream:
        for r in range(rounds):
            for i,case in enumerate(cases[r%len(cases):]+cases[:r%len(cases)]):
                for lane in lanes:
                    for variant in ([base,name] if (r+i)%2==0 else [name,base]):
                        x=invoke(WORK/'bin'/variant,'time',roots[case],lane,ms);x.update(case=case,variant=variant,round=r)
                        rows.append(x);stream.write(json.dumps(x)+'\n');stream.flush()
            print(f'{name}: round {r+1}/{rounds}',flush=True)
    summary=[]
    for case in cases:
        for lane in lanes:
            paired=[];values={}
            for variant in [base,name]:
                values[variant]=[x['elapsed_ns']/x['iterations']/1000 for x in rows if x['case']==case and x['variant']==variant and x['lane']==lane]
            for a,b in zip(values[base],values[name]):paired.append(100*(b/a-1))
            summary.append({'case':case,'lane':lane,'baseline_us':statistics.median(values[base]),'candidate_us':statistics.median(values[name]),'paired_median_percent':statistics.median(paired),'paired_percent':paired,'windows_us':values})
    (out/'summary.json').write_text(json.dumps(summary,indent=2)+'\n')
    for x in summary:print(x['case'],x['lane'],round(x['paired_median_percent'],2),flush=True)

if __name__=='__main__':
    ap=argparse.ArgumentParser();ap.add_argument('mode',choices=['build','measure']);ap.add_argument('name');ap.add_argument('--rounds',type=int,default=5);ap.add_argument('--ms',type=int,default=120);ap.add_argument('--out');ap.add_argument('--baseline',default='baseline');ap.add_argument('--lane',action='append');ap.add_argument('--case',action='append');a=ap.parse_args()
    if a.mode=='build':build(a.name)
    else:measure(a.name,a.rounds,a.ms,a.out or ('screen-'+a.name),a.baseline,a.lane,a.case)
