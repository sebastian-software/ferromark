"""Public API option guards, retaining unavoidable competitor differences."""
import json
from pathlib import Path

MANIFEST = Path(__file__).with_name('comparisons.json')
PROJECTS = json.loads(MANIFEST.read_text())
PAIRS = tuple((p['track'], p['id']) for p in PROJECTS if p['lane'] == 'pair')
COMMONMARK_ONLY = {p['id'] for p in PROJECTS if p['profile'] == 'commonmark-only'}
SPECIAL = {'remarkable', 'markdown-exit', 'markdown-it-ts', 'satteri', 'md4x-napi', 'md4x-wasm', 'ox-content-napi'}


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
    }.get(engine, 'CommonMark or tables/strikethrough/tasks, per frozen input profile')


def groups(cases):
    return {profile: [case for case in cases if case['profile'] == profile]
            for profile in sorted({case['profile'] for case in cases})}


def behavior_checks(native, command, directory, engines, profiles):
    engine = engines[1]
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
