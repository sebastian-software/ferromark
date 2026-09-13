"""Check each range-walk guard against counted and uncounted quadratic rewrites.

Only a temporary standalone crate is mutated. This is a negative-control check,
not a wall-clock performance test. The original measured source stays frozen.
"""
from pathlib import Path
import io
import json
import shutil
import subprocess
import tarfile
import tempfile

D = Path(__file__).resolve().parents[1]
R = D.parents[2]
metadata = json.loads((D / 'metadata.json').read_text())
test = 'inline::tests::code_html_membership_work_scales_linearly'
results = []

with tempfile.TemporaryDirectory(prefix='ferromark-review-guard-') as temporary:
    root = Path(temporary)
    archive = subprocess.check_output(
        ['git', 'archive', metadata['baseline_revision'], 'src'], cwd=R)
    with tarfile.open(fileobj=io.BytesIO(archive)) as tar:
        tar.extractall(root, filter='data')
    for patch in [D / 'measurements/final/patch.diff', D / 'validation/review-guard.diff']:
        subprocess.run(['patch', '-s', '-p1', '-d', str(root)],
                       input=patch.read_bytes(), check=True)
    shutil.copyfile(D / 'source-Cargo.toml', root / 'Cargo.toml')
    subprocess.run(['cargo', 'generate-lockfile', '--offline'], cwd=root, check=True)
    paths = [root / 'src/inline/mod.rs', root / 'src/inline/code_span.rs']
    original = {path: path.read_text() for path in paths}

    def run(name, expected):
        result = subprocess.run(
            ['cargo', 'test', '--offline', '--locked', '--all-features', '--lib',
             test, '--', '--exact', '--nocapture'],
            cwd=root, text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
        # A compiler error is not a successful negative control.
        assert 'running 1 test' in result.stdout, result.stdout
        assert result.returncode == expected, result.stdout
        if expected:
            assert 'must exercise' in result.stdout or 'probes grew from' in result.stdout
        results.append(dict(variant=name, exit_code=result.returncode,
                            output=result.stdout.replace(str(root), '<temporary>')))
        print(name, 'passed' if expected == 0 else 'rejected as expected', flush=True)

    run('current', 0)
    for site in ['CodeOpenerInHtml', 'AutolinkInCode', 'HtmlInCode']:
        for counted in [False, True]:
            for path, source in original.items():
                path.write_text(source)
            if site == 'CodeOpenerInHtml':
                path = paths[1]
                source = original[path].replace('    let mut html_idx = 0;\n', '')
                start = source.index('        while html_idx < html_spans.len()')
                end = source.index('        if opener_pos > 0', start)
                probe = ('super::record_range_probe(super::RangeProbe::CodeOpenerInHtml);'
                         if counted else '')
                replacement = f'''        if html_spans.iter().any(|&(start, end)| {{
            {probe}
            opener_pos as u32 >= start && (opener_pos as u32) < end
        }}) {{
            continue;
        }}
'''
            else:
                path = paths[0]
                source = original[path]
                function = ('filter_autolinks_in_code_spans' if site == 'AutolinkInCode'
                            else 'filter_html_spans_in_code_spans')
                start = source.index('fn ' + function + '(')
                end = source.index('\nfn ', start + 1)
                ty = 'Autolink' if site == 'AutolinkInCode' else 'HtmlSpan'
                probe = f'record_range_probe(RangeProbe::{site});' if counted else ''
                replacement = f'''fn {function}(items: &mut Vec<{ty}>, code_spans: &[CodeSpan]) {{
    items.retain(|item| !code_spans.iter().any(|code| {{
        {probe}
        item.start >= code.opener_pos && item.start < code.closer_end
    }}));
}}
'''
            path.write_text(source[:start] + replacement + source[end:])
            run(site + ('-counted' if counted else '-uncounted'), 101)

print(json.dumps(results, indent=2))
