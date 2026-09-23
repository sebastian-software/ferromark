"""Aggregate a macOS `sample` call graph: tree.py <sample.txt[.gz]> incl [N] | callers <name> | callees <name>

incl     inclusive samples per symbol (a symbol counts once per stack, recursion folded)
callers  who calls the first frame matching <name>
callees  self samples of <name> and the children it spends time in
"""
import collections
import gzip
import re
import sys


def short(sym):
    sym = re.sub(r'::h[0-9a-f]{16}$', '', sym)
    sym = sym.replace('_$LT$','<').replace('$GT$','>').replace('$u20$',' ').replace('..','::').replace('$u7b$','{').replace('$u7d$','}').replace('$C$',',').replace('$RF$','&')
    sym = re.sub(r'<impl ferromark::(parser|renderer::html::renderer)::(Parser|HtmlRenderer)>::', r'\2::', sym)
    sym = sym.replace('ferromark::parser::','P:').replace('ferromark::renderer::html::','R:')
    return sym


def load(path):
    opener = gzip.open if path.endswith('.gz') else open
    with opener(path, 'rt') as handle:
        lines = handle.read().split('\n')
    start = next(i for i,l in enumerate(lines) if l.startswith('Call graph:'))
    nodes = []  # (depth, count, sym, parent_idx)
    stack = []
    for l in lines[start+1:]:
        if not l.strip(): break
        m = re.match(r'^([\s+!:|]*)(\d+) (.+?)(?:  \(in [^)]*\))?(?: \+ [\d,.]+)?(?:  \[.*)?$', l)
        if not m: continue
        depth = len(m.group(1)); cnt = int(m.group(2)); sym = short(m.group(3).strip())
        while stack and nodes[stack[-1]][0] >= depth: stack.pop()
        parent = stack[-1] if stack else -1
        nodes.append([depth, cnt, sym, parent]); stack.append(len(nodes)-1)
    return nodes


def main():
    nodes = load(sys.argv[1]); cmd = sys.argv[2]
    children = collections.defaultdict(list)
    for i,n in enumerate(nodes): children[n[3]].append(i)
    total = sum(nodes[i][1] for i in children[-1])
    def anc(i):
        p = nodes[i][3]
        while p != -1:
            yield p; p = nodes[p][3]
    if cmd == 'incl':
        inc = collections.Counter()
        for i,n in enumerate(nodes):
            if any(nodes[a][2]==n[2] for a in anc(i)): continue
            inc[n[2]] += n[1]
        for s,c in inc.most_common(int(sys.argv[3]) if len(sys.argv)>3 else 60): print(f"{c:6d} {100*c/total:5.1f}%  {s}")
    elif cmd == 'callers':
        pat = sys.argv[3]; cal = collections.Counter(); tot=0
        for i,n in enumerate(nodes):
            if pat in n[2] and not any(pat in nodes[a][2] for a in anc(i)):
                p = n[3]
                # skip same-name frames (inlined dups)
                while p!=-1 and nodes[p][2]==n[2]: p=nodes[p][3]
                cal[nodes[p][2] if p!=-1 else '<root>'] += n[1]; tot+=n[1]
        print('total', tot, f"{100*tot/total:.1f}%")
        for s,c in cal.most_common(25): print(f"{c:6d}  {s}")
    elif cmd == 'callees':
        pat = sys.argv[3]; cal = collections.Counter(); selfc=0
        for i,n in enumerate(nodes):
            if n[2]==pat or (pat in n[2] and '=' not in pat):
                if any(pat in nodes[a][2] for a in anc(i)): continue
                s = n[1]
                for c in children[i]:
                    cal[nodes[c][2]] += nodes[c][1]; s -= nodes[c][1]
                selfc += s
        print('self', selfc)
        for s,c in cal.most_common(30): print(f"{c:6d}  {s}")


main()
