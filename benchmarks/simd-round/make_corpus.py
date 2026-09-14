#!/usr/bin/env python3
"""Reuse the complete broad corpus and add labeled link-scan diagnostics."""

import argparse
import gzip
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    source = ROOT / "docs/reports/2026-09-14-broad-markdown/corpus.json.gz"
    payload = json.loads(gzip.decompress(source.read_bytes()))
    cases = payload["cases"]
    assert len(cases) == 57
    for case in cases:
        case["suite"] = "broad"

    # These are stress/semantic diagnostics, not additional real documents.
    # Freeze selection before timing; do not mix them into broad-corpus means.
    diagnostics = {
        "empty": ("commonmark", ""),
        "short-link": ("commonmark", "[a](b)\n"),
        "short-title": ("commonmark", '[a](b "c")\n'),
        "short-entity": ("commonmark", '[a](/?x&amp;y "&#65;")\n'),
        "short-reference": ("commonmark", '[a][b]\n\n[b]: /c "d"\n'),
        "long-clean-links": ("gfm", "\n\n".join(
            f'[Guide {i}](https://example.org/documentation/{"section/" * 16}{i} '
            f'"A detailed explanation of the document processing pipeline for section {i}")'
            for i in range(48))),
        "many-short-links": ("gfm", " ".join(f'[x{i}](/p/{i} "t")' for i in range(128))),
        "escaped-links": ("gfm", "\n\n".join(
            rf'[Guide {i}](https://example.org/a\(b\)/{i}?x=1&amp;y=2 "A \"quoted\" &copy; title")'
            for i in range(48))),
        "unicode-links": ("commonmark", "\n\n".join(
            f'[Überblick {i}](https://example.org/{"日本語/Änderungen/" * 8}{i} '
            f'"Größe &amp; résumé {i}")' for i in range(48))),
        "long-references": ("gfm", "\n\n".join(
            f'[See guide {i}][ref-{i}]\n\n[ref-{i}]: '
            f'https://example.org/{"reference/" * 16}{i} "A reference &amp; its title"'
            for i in range(48))),
        "dense-escapes": ("commonmark", '[dense](<' + r'\!\?\&' * 256 + '>)\n'),
        "dense-entities": ("commonmark", '[dense](<' + '&amp;&#65;&copy;' * 256 + '>)\n'),
        "malformed-entities": ("commonmark", '[unchanged](<' + '&bad;&#xZZ;&#99999999;' * 128 + '>)\n'),
        "mdx-links": ("mdx", 'import Note from "./note"\n\n# Linked notes\n\n'
            '<Note tone="quiet">\n\n' + "\n\n".join(
                f'[Chapter {i}](https://example.org/{"mdx/chapter/" * 4}{i} "Résumé &amp; notes")'
                for i in range(32)) + '\n\n</Note>\n\n{page.title}\n'),
        "extension-links": ("extensions", '# References {#references}\n\n'
            'H~2~O, x^2^ and $x + y$ beside [the guide](/a\\(b\\)?x=1&amp;y=2 "A &copy; title").\n\n'
            'Term\n: Definition with [a reference][help].\n\n'
            '| Subject | Link |\n| --- | --- |\n| Math | [help][help] |\n\n'
            '[help]: /docs/guide "A reference &amp; title"\n\n'
            'A footnote[^n] and [[Wiki page]].\n\n[^n]: See [more](/notes).\n'),
    }
    for name, (profile, text) in diagnostics.items():
        data = text.encode()
        cases.append({
            "name": f"scan-{name}", "profile": profile, "input": text,
            "byte_count": len(data), "sha256": hashlib.sha256(data).hexdigest(),
            "suite": "diagnostic", "category": "link-scan-diagnostic",
            "collection": "authored-diagnostics",
            "origin": {"kind": "authored-example", "license": "MIT"},
        })
    result = {
        "schema": 1,
        "selection": "All 57 frozen broad cases, plus 15 synthetic link-scan diagnostics selected before timing. Broad statistics exclude diagnostics. Wikipedia views overlap; this is a corpus, not a population estimate.",
        "broad_corpus_sha256": hashlib.sha256(source.read_bytes()).hexdigest(),
        "cases": cases,
    }
    args.output.write_text(json.dumps(result, indent=2, ensure_ascii=False) + "\n")
    print(f"{len(cases)} cases written to {args.output}")


if __name__ == "__main__":
    main()
