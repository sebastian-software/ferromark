#!/usr/bin/env python3
"""Aggregate an xctrace time-profile XML export into self/inclusive sample shares per symbol."""
import re, sys, xml.etree.ElementTree as ET
from collections import Counter
path = sys.argv[1]
top = int(sys.argv[2]) if len(sys.argv) > 2 else 40
root = ET.parse(path).getroot()
by_id = {}
for el in root.iter():
    i = el.get('id')
    if i is not None:
        by_id[i] = el
def resolve(el):
    r = el.get('ref')
    return by_id[r] if r is not None else el
HASH = re.compile(r'::h[0-9a-f]{16}$')
def demangle(name):
    if name is None: return '?'
    if name.startswith('_RNv') or name.startswith('_RINv'):
        # crude v0: split length-prefixed idents
        parts = re.findall(r'(\d+)([A-Za-z_][A-Za-z0-9_]*)', name)
        out=[]; s=name
        # walk: find sequences like 4core9core_arch...
        i=0; toks=[]
        m=re.finditer(r'(\d+)', name)
        pos=0; res=[]
        while True:
            mm=re.search(r'(\d+)', name[pos:])
            if not mm: break
            n=int(mm.group(1)); start=pos+mm.end()
            ident=name[start:start+n]
            if len(ident)==n and re.fullmatch(r'[A-Za-z0-9_]+', ident or 'x'):
                res.append(ident); pos=start+n
            else:
                pos=start
        return '::'.join(res) if res else name
    n = name.replace('$LT$','<').replace('$GT$','>').replace('$u20$',' ').replace('$RF$','&').replace('$u27$',"'").replace('$u5b$','[').replace('$u5d$',']').replace('$C$',',').replace('$BP$','*')
    n = re.sub(r'\.\.', '::', n)
    n = HASH.sub('', n)
    return n
def frame_info(fr):
    fr = resolve(fr)
    return demangle(fr.get('name')), fr.get('inlined') == 'true'
leaf = Counter(); func = Counter(); incl = Counter(); total = 0
for row in root.iter('row'):
    bt = row.find('tagged-backtrace')
    if bt is None: bt = row.find('backtrace')
    if bt is None: continue
    bt = resolve(bt)
    frames = [frame_info(f) for f in bt.findall('frame')]
    if not frames: continue
    total += 1
    leaf[frames[0][0]] += 1
    fn = next((n for n, inl in frames if not inl), frames[0][0])
    func[fn] += 1
    for name in {n for n, _ in frames}:
        incl[name] += 1
print(f"total samples: {total}")
def show(title, c, k):
    print(f"\n== {title} ==")
    for name, n in c.most_common(k):
        print(f"{100*n/total:6.2f}%  {name[:160]}")
show("self (innermost, inlined-aware)", leaf, top)
show("self (containing out-of-line function)", func, top)
incl_f = Counter({k:v for k,v in incl.items() if any(t in k for t in ("ferromark","memchr","bumpalo","memmove","memcpy","alloc::","hashbrown","compact_str","core::fmt"))})
show("inclusive (ferromark/deps only)", incl_f, top+20)
