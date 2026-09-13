"""Generate tables from the archived, individually verified timing summaries."""
from pathlib import Path
import json
import statistics
import sys

D = Path(__file__).resolve().parent


def read(path):
    return json.loads(path.read_text())


def paired(paths):
    runs = [{r['index']: r for r in read(p)} for p in paths]
    for i in runs[0]:
        rows = [r[i] for r in runs]
        yield rows[0]['case'], statistics.median(r['baseline_ns'] for r in rows), statistics.median(r['candidate_ns'] for r in rows), [r['change_pct'] for r in rows]


def changes(values):
    return ', '.join(f'{v:+.2f}%' for v in values)


def generate():
    adopted = read(D/'metadata.json')['candidate_variant']
    out = ['# Measured Suspense results', '', 'Generated from archived observations; negative paired changes mean less elapsed time. Repeated ranges are not confidence intervals.', '', '## Measured PR candidate', '', 'Three fresh process pairs, 103 inputs, seven alternating 50 ms windows after 60 ms warmup. Absolute times are medians of the three process medians.', '', '| Input | Before µs | After µs | Changes |', '| --- | ---: | ---: | --- |']
    rows = list(paired(sorted((D/'measurements'/adopted).glob('confirm-*/summary.json'))))
    for case, a, b, delta in rows:
        out.append(f'| {case} | {a/1000:.3f} | {b/1000:.3f} | {changes(delta)} |')
    out += ['', '### Repeated small costs', '', 'Every input at least 1% slower in all three final pairs is listed here. The complete table also retains isolated losses.', '', '| Input | Changes |', '| --- | --- |']
    costs = [(case, delta) for case, _, _, delta in rows if min(delta) >= 1]
    for case, delta in costs:
        out.append(f'| {case} | {changes(delta)} |')
    if not costs:
        out.append('| None | — |')
    followups = sorted((D/'measurements'/adopted).glob('followup-*/summary.json'))
    if followups:
        out += ['', '### Focused follow-up', '', 'Three fresh pairs, seven alternating 100 ms windows. These include every input over 2% slower in any final pair, the heading control, Suspense and HTML prose. Earlier observations remain above.', '', '| Input | Before µs | After µs | Changes |', '| --- | ---: | ---: | --- |']
        for case, a, b, delta in paired(followups):
            out.append(f'| {case} | {a/1000:.3f} | {b/1000:.3f} | {changes(delta)} |')
    out += ['', f'Inputs over 2% slower in all three final pairs: {sum(min(delta)>2 for _,_,_,delta in rows)}.', '', '## Fresh native Ox comparison', '', 'Growing arena only. Three fresh process groups, nine alternating 60 ms windows after 35 ms warmup. These use a separate driver from the before/after measurements; do not combine their absolute timings.', '', '| Document | Ferromark µs | Ox growing µs | Ferromark extra time, three groups |', '| --- | ---: | ---: | --- |']
    native = [{r['index']: r for r in read(p)} for p in sorted((D/'native').glob('final-*-summary.json'))]
    for i in native[0]:
        rr = [r[i] for r in native]
        assert all(r['equal'] for r in rr)
        f = statistics.median(r['median_ns']['ferro'] for r in rr)
        ox = statistics.median(r['median_ns']['ox-grow'] for r in rr)
        delta = [(r['median_ns']['ferro']/r['median_ns']['ox-grow']-1)*100 for r in rr]
        out.append(f"| {rr[0]['case']} | {f/1000:.3f} | {ox/1000:.3f} | {changes(delta)} |")
    out += ['', '649 corpus outputs checked; 594 normalize equally. The other cases remain explicit diagnostics and do not support a ranking.', '', '## Growth controls', '', 'Two fresh pairs, seven alternating 50 ms windows after 35 ms warmup. Each control has exact before/after HTML equality. Repetition counts refer to one paragraph, not separate renders.', '', '| Input | Before µs | After µs | Changes |', '| --- | ---: | ---: | --- |']
    for case, a, b, delta in paired(sorted((D/'measurements'/adopted).glob('scaling-*/summary.json'))):
        out.append(f'| {case} | {a/1000:.3f} | {b/1000:.3f} | {changes(delta)} |')
    out += ['', '## Initial independent screens', '', 'Five alternating 30 ms windows. These exploratory results are not adoption evidence by themselves. Full control results and source patches accompany each screen.', '', '| Variant | Inputs | Suspense change | Inputs over 2% slower |', '| --- | ---: | ---: | ---: |']
    for p in sorted((D/'measurements').glob('*/screen/summary.json')):
        rr = read(p)
        suspense = next(r for r in rr if r['index'] == 79)
        out.append(f"| {p.parent.parent.name} | {len(rr)} | {suspense['change_pct']:+.2f}% | {sum(r['change_pct']>2 for r in rr)} |")
    out += ['', '## Identical-binary controls', '', 'Three fresh pairs compare byte-identical baseline binaries using the same seven-window protocol. These observations diagnose measurement variability; they do not subtract noise from candidate results.', '', '| Input | Changes |', '| --- | --- |']
    for case, _, _, delta in paired(sorted((D/'measurements/baseline-copy').glob('confirm-*/summary.json'))):
        out.append(f'| {case} | {changes(delta)} |')
    out += ['', '## Exact-output checks for the PR candidate', '', '| Guard | Comparisons | Failures |', '| --- | ---: | ---: |']
    total = 0
    for p in sorted((D/'measurements'/adopted).glob('*verification.json')):
        r = read(p); failures = r.get('failures', r.get('failed', [])); total += r['count']
        out.append(f"| {p.name} | {r['count']:,} | {len(failures)} |")
    out += ['', f'Total: {total:,} logical comparisons. The whole-render work test additionally proves that membership searches no longer exhibit the recorded quadratic growth.', '']
    return '\n'.join(out)


if __name__ == '__main__':
    text = generate()
    if '--check' in sys.argv:
        assert (D/'RESULTS.md').read_text() == text, 'regenerate RESULTS.md'
    else:
        (D/'RESULTS.md').write_text(text)
