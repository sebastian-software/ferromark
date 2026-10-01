"""Public API option guards, retaining unavoidable competitor differences."""
import argparse
import json
from pathlib import Path

MANIFEST = Path(__file__).with_name('comparisons.json')
PROJECTS = json.loads(MANIFEST.read_text())
PAIRS = tuple((p['track'], p['id']) for p in PROJECTS if p['lane'] == 'pair')
COMMONMARK_ONLY = {p['id'] for p in PROJECTS if p['profile'] == 'commonmark-only'}
SPECIAL = {'remarkable', 'markdown-exit', 'markdown-it-ts', 'satteri', 'md4x-napi', 'md4x-wasm', 'ox-content-napi'}


TIMING_KEYS = ('rounds', 'samples', 'window_ms', 'warmup_ms')
TIMING_PROFILES = json.loads(MANIFEST.with_name('scoring-policy.json').read_text())['timing_profiles']


def timing_identity(config):
    return next((name for name, values in TIMING_PROFILES.items()
                 if all(config[key] == values[key] for key in TIMING_KEYS)), 'diagnostic')


def validate_timing(config, policy=None):
    # Historical campaigns have no named profile and retain their original windows.
    profiles = policy.get('timing_profiles') if policy else None
    name = config.get('timing_profile', 'standard')
    if profiles is None:
        assert name == 'standard', 'historical campaigns require standard timing'
        values = dict(rounds=3, samples=6, window_ms=40, warmup_ms=60)
    else:
        assert config.get('timing_profile') in profiles, 'missing or unknown timing profile'
        values = profiles[name]
    assert all(config[key] == values[key] for key in TIMING_KEYS), 'timing differs from the committed profile'
    return name


def timing_arguments(name):
    return [arg for key in TIMING_KEYS for arg in ('--' + key.replace('_', '-'), str(TIMING_PROFILES[name][key]))]



def campaign_projects(projects, scope='main'):
    """Keep optional adapters executable without requiring them in every campaign."""
    if scope not in ('main', 'extended'):
        raise ValueError('Comparison scope must be main or extended')
    return [project for project in projects if scope == 'extended' or not project.get('optional', False)]


def campaign_pairs(scope='main'):
    return tuple((project['track'], project['id']) for project in campaign_projects(PROJECTS, scope)
                 if project['lane'] == 'pair')

def project(engine):
    return next(p for p in PROJECTS if p['id'] == engine)


def scope(engine):
    if engine in COMMONMARK_ONLY:
        return 'CommonMark only on all inputs'
    return {
        'satteri': 'Frozen profiles; public GFM also enables literal autolinks',
        'md4x-napi': 'Fixed MD4X extensions; baseline follows frozen profiles',
        'md4x-wasm': 'Fixed MD4X extensions; baseline follows frozen profiles',
        'ox-content-napi': 'Frozen shared parser flags; public renderer builtins remain enabled',
        'tanstack-markdown': 'Frozen baseline profiles; TanStack syntax subset with fixed table/strike/task/footnote support',
    }.get(engine, 'CommonMark or tables/strikethrough/tasks, per frozen input profile')


def groups(cases):
    return {profile: [case for case in cases if case['profile'] == profile]
            for profile in sorted({case['profile'] for case in cases})}


def behavior_checks(native, command, directory, engines, profiles):
    engine = engines[1]
    if engine == 'tanstack-markdown':
        # Verify the public subset API as shipped, without inventing CommonMark switches.
        result = native.behavior_checks(command, directory, ('v2',), profiles)
        names = list(result[profiles[0]]['v2'])
        paths = [directory / ('guard-' + name + '.md') for name in names]
        for profile in profiles:
            modes = []
            for mode in native.MODES:
                worker = native.Worker(command, engine, profile, mode, paths)
                try:
                    output = worker.verify()
                    assert worker.verify() == output, ('subset state leaked', engine, mode)
                    modes.append(dict(zip(names, output)))
                finally:
                    worker.close()
            assert modes[0] == modes[1], ('lifecycle mismatch', engine)
            out = modes[0]
            assert out['empty'] == ''
            for token in ('<strong>two</strong>', '<code>three</code>', '<div>raw</div>', 'href="/path"'):
                assert token in out['common'], (engine, profile, token)
            assert '&amp;amp;' in out['common'], 'retain the released entity-handling difference'
            assert '<https://example.com>' in out['common'], 'angle autolinks are not implemented'
            assert 'x  y\n\n z' in out['literal'] and 'class="tm-code"' in out['literal']
            assert '<table>' in out['extensions'] and '<del>old</del>' in out['extensions']
            assert out['extensions'].count('type="checkbox"') == 2
            assert '<a ' not in out['extensions'], 'literal autolinks are off'
            assert 'footnotes' in out['footnotes'], 'footnotes have no public disable switch'
            assert 'title: Ordinary text' in out['frontmatter']
            assert ' id=' not in out['headings']
            assert '[[toc]]' in out['extras'] and '<dl>' not in out['extras']
            assert 'href="target.md"' in out['extras'] and 'target="_blank"' not in out['extras']
            assert '[!NOTE]' in out['callout']
            assert 'data-meta="{1}"' in out['fence-metadata']
            assert '[x][ref]' in out['undefined'] and 'href="/first"' in out['references']
            assert '<script>raw()</script>' in out['raw-tagfilter']
            result[profile][engine] = out
        return result
    if engine not in SPECIAL:
        return native.behavior_checks(command, directory, engines, profiles)
    # The established Ferromark guards also materialize the shared fixtures.
    result = native.behavior_checks(command, directory, ('v2',), profiles)
    names = list(result[profiles[0]]['v2'])
    paths = [directory / ('guard-' + name + '.md') for name in names]
    for profile in profiles:
        modes = []
        for mode in native.MODES:
            worker = native.Worker(command, engine, profile, mode, paths)
            try:
                output = worker.verify()
                assert worker.verify() == output, ('reference state leaked', engine, mode)
                modes.append(dict(zip(names, output)))
            finally:
                worker.close()
        assert modes[0] == modes[1], ('lifecycle mismatch', engine)
        out = modes[0]
        md4x = engine.startswith('md4x-')
        ox = engine == 'ox-content-napi'
        gfm = profile != 'commonmark' or md4x
        autolinks = md4x or ox or (engine == 'satteri' and gfm)
        assert out['empty'] == ''
        for token in ('<strong>two</strong>', '<code>three</code>', '<div>raw</div>', 'href="https://example.com"'):
            assert token in out['common'], (engine, profile, token)
        assert 'x  y\n\n z\n' in out['literal']
        assert '[[page]]' in out['extensions']
        assert ('<a ' in out['extensions']) == autolinks, (engine, profile, 'literal autolinks')
        assert '[x][ref]' in out['undefined'], ('reference state leaked', engine)
        assert '<script>raw()</script>' in out['raw-tagfilter']
        for token in ('// visible', 'Term', ': meaning', '-- ...', 'x^2^', 'href="target.md"'):
            assert token in out['extras'], (engine, profile, token)
        for token in ('<dl>', '<sup>', '<sub>'):
            assert token not in out['extras'], (engine, profile, token)
        assert ('footnotes' in out['footnotes']) == md4x
        assert ('title: Ordinary text' in out['frontmatter']) == (not md4x)
        assert 'colspan' not in out['table-extras'] and '<colgroup' not in out['table-extras']
        assert (' id=' in out['headings']) == ox
        assert ('[[toc]]' in out['extras']) == (not ox)
        assert ('[!NOTE]' in out['callout']) == (not (md4x or ox))
        assert ('language-js{1}' in out['fence-metadata']) == (not ox)
        if ox:
            assert 'id="same"' in out['headings'] and 'id="same-1"' in out['headings']
            assert 'ox-callout' in out['callout'] and 'target="_blank"' in out['extensions']
        if md4x:
            assert 'markdown-alert-note' in out['callout'] and out['frontmatter'] == ''
        assert ('<table>' in out['extensions']) == gfm
        strike = '<s>old</s>' if engine in ('markdown-exit', 'markdown-it-ts') else '<del>old</del>'
        assert (strike in out['extensions']) == gfm, (engine, profile, 'strikethrough')
        assert out['extensions'].count('type="checkbox"') == (2 if gfm else 0)
        result[profile][engine] = out
    return result


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description='Generate the selected ecosystem job matrix without allocating runners.')
    parser.add_argument('--scope', choices=('main', 'extended'), default='main')
    args = parser.parse_args()
    print(json.dumps({'include': [{'track': track, 'engine': engine} for track, engine in campaign_pairs(args.scope)]}))
