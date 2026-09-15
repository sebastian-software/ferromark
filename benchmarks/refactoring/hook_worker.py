#!/usr/bin/env python3
"""Emit the runtime worker with all HTML entry points using no-op hooks.

Keep this separate from the default worker: no extra runtime option or branch is
inserted into the default measurement. Both hook builds must use identical bytes.
"""
from pathlib import Path
import sys

source = (Path(__file__).resolve().parents[1] / 'runtime-profiles/worker.rs').read_text()
replacements = {
    '.render(&document)': (3, '.render_with_hooks(&document, &mut ferromark::NoHtmlRenderHooks)'),
    '.render_borrowed(&document)': (2, '.render_borrowed_with_hooks(&document, &mut ferromark::NoHtmlRenderHooks)'),
    '.render_borrowed(document)': (1, '.render_borrowed_with_hooks(document, &mut ferromark::NoHtmlRenderHooks)'),
    '.render_borrowed(&self.render_documents[index])': (1, '.render_borrowed_with_hooks(&self.render_documents[index], &mut ferromark::NoHtmlRenderHooks)'),
}
for old, (count, new) in replacements.items():
    assert source.count(old) == count, f'worker changed: expected {count} occurrences of {old}'
    source = source.replace(old, new)
assert '.render(' not in source and '.render_borrowed(' not in source
sys.stdout.write(source)
