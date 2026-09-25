#!/usr/bin/env python3
"""Extend the frozen 72-case corpus with structural parser diagnostics."""

import argparse
import gzip
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
BASE = ROOT / "docs/reports/2026-09-14-simd-round/corpus.json.gz"


def case(name, profile, text, category):
    raw = text.encode("utf-8")
    return {
        "name": name, "profile": profile, "input": text,
        "byte_count": len(raw), "sha256": hashlib.sha256(raw).hexdigest(),
        "suite": "diagnostic", "category": category,
        "collection": "optimization-round-diagnostics",
        "origin": {"kind": "authored-example", "license": "MIT"},
    }


def table_body(size, density):
    if density == "plain":
        content = "word " * (size // 5)
    elif density == "sparse":
        content = "word " * (size // 10) + r"\|" + " prose" * (size // 12)
    else:
        token = r"\|" if density == "dense" else r"**bold\|pipe** and *text* "
        content = token * max(1, size // len(token))
    return f"| Header | Value |\n| --- | --- |\n| {content} | end |\n"


def scanner_text(size, profile, shape):
    bits = int(profile[4:])
    plain = "Plain prose with ordinary words and Unicode café 日本語. "
    markers = (("{value} " if bits & 1 else "") +
               ("$x+y$ " if bits & 2 else "") +
               ("x^2^ " if bits & 4 else ""))
    if shape == "sparse":
        token = plain * 20 + markers
    elif shape == "disabled":
        disabled = ("{value} " if not bits & 1 else "")
        disabled += ("$literal$ " if not bits & 2 else "")
        disabled += ("literal^2^ " if not bits & 4 else "")
        token = "Words " + (disabled or "plain text ")
    else:
        token = plain
    return token * max(1, size // len(token.encode()))


def autolink_text(size, shape):
    prefix = "https://example.org/"
    if shape == "closers":
        return "See " + prefix + "path" + ")] }".replace(" ", "") * (size // 3) + " next.\n"
    if shape == "long-clean":
        return "See " + prefix + "a" * size + " next.\n"
    tokens = {
        "clean": prefix + "docs/page ",
        "balanced": prefix + "a(b(c))/page ",
        "mixed": prefix + "a(b)c]d}e/page ",
        "unicode": "https://例え.テスト/日本語/変更。続き 🙂 ",
    }
    token = tokens[shape]
    return token * max(1, size // len(token.encode()))


def container_text(size, shape):
    """Repeat a closed container shape to exercise reparsing and remapping."""
    blocks = {
        "quote": "> Quoted paragraph with **strong**, *emphasis*, and ordinary text.\n",
        "nested-quote": "> > > > Quoted paragraph with **strong** and ordinary text.\n",
        "quote-lazy": "> Quoted paragraph with *emphasis* and\nlazy continuation text.\n",
        "quote-tabs": ">\tQuoted text after a tab following the quote marker.\n",
        "quote-fence": "> ```text\n> fenced content\n> ```\n> trailing paragraph\n",
        "list-multiline": "- Item with **strong** text\n  and an indented continuation.\n",
        "list-loose": "- first paragraph with *emphasis*\n\n  second paragraph\n- next item\n",
        "nested-list": "- outer item\n  - nested item\n    - deep item with **strong** text\n",
        "list-tabs": "-\tTabbed item\n\tcontinuation after a tab.\n",
        "mixed-quote-list": "> - quoted item\n>   continuation\n>   - nested item\n>     deeper text\n",
        "gfm-table-quote": "> | Name | Value |\n> | --- | --- |\n> | first | 1 |\n",
        "gfm-lazy-boundary": "> ```\n> code\n> ```\nlazy text outside the quote\n",
        "commented-containers": "> first line\n// private quote comment\n> second line\n- first item\n// private list comment\n  continuation\n",
    }
    block = blocks[shape]
    text = block
    while len(text.encode("utf-8")) < size:
        text += block
    return text


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("output", type=Path)
    parser.add_argument(
        "--include-scanner-diagnostics", action="store_true",
        help="add large plain and sparse-marker cases for opt-0..opt-7 profiles",
    )
    parser.add_argument("--include-autolink-broad", action="store_true",
                        help="also replay the 57 real mixed documents with CommonMark and renderer autolinks")
    parser.add_argument("--include-container-diagnostics", action="store_true",
                        help="add generated blockquote/list cases, including laziness, tabs, comments and GFM blocks")
    args = parser.parse_args()
    payload = json.loads(gzip.decompress(BASE.read_bytes()))
    cases = payload["cases"]
    if len(cases) != 72:
        raise SystemExit(f"expected frozen 72-case corpus, got {len(cases)}")
    for item in cases:
        item.setdefault("suite", "broad")
    additions = []
    for size in (256, 4096, 16000):
        for density in ("plain", "sparse", "dense", "formatted"):
            additions.append(case(f"table-{density}-{size}", "gfm", table_body(size, density), "table-diagnostic"))
    for size in (64, 512, 2048):
        for shape in ("clean", "balanced", "mixed", "closers", "unicode", "long-clean"):
            additions.append(case(f"autolink-{shape}-{size}", "autolink", autolink_text(size, shape), "autolink-diagnostic"))
    if args.include_scanner_diagnostics:
        for size in (4096, 16000):
            for profile in (f"opt-{bits}" for bits in range(8)):
                for shape in ("plain", "sparse", "disabled"):
                    additions.append(case(f"scanner-{profile}-{shape}-{size}", profile,
                                          scanner_text(size, profile, shape), "scanner-diagnostic"))
    if args.include_container_diagnostics:
        for size in (4096, 16000):
            for shape in (
                "quote", "nested-quote", "quote-lazy", "quote-tabs", "quote-fence",
                "list-multiline", "list-loose", "nested-list", "list-tabs",
                "mixed-quote-list", "gfm-table-quote", "gfm-lazy-boundary",
                "commented-containers",
            ):
                profile = "gfm-comments" if shape == "commented-containers" else (
                    "gfm" if shape.startswith("gfm-") else "commonmark"
                )
                additions.append(case(f"container-{shape}-{size}", profile,
                                      container_text(size, shape), "container-diagnostic"))
    if args.include_autolink_broad:
        for original in cases:
            if original["suite"] == "broad":
                additions.append({**original,
                    "name": "autolink-broad--" + original["name"],
                    "source_profile": original["profile"], "profile": "autolink",
                    "suite": "autolink-broad"})
    existing = {item["name"] for item in cases}
    if existing & {item["name"] for item in additions}:
        raise SystemExit("diagnostic case name collision")
    cases.extend(additions)
    result = {
        "schema": 1,
        "selection": "Frozen 72-case SIMD corpus plus authored structural table, autolink, scanner, and optional container diagnostics; diagnostics are labeled and excluded from broad-corpus statistics.",
        "base_corpus_sha256": hashlib.sha256(BASE.read_bytes()).hexdigest(),
        "diagnostic_counts": {
            "table": 12,
            "autolink": 18,
            "scanner": 48 if args.include_scanner_diagnostics else 0,
            "containers": 26 if args.include_container_diagnostics else 0,
        },
        "cases": cases,
    }
    raw = json.dumps(result, indent=2, ensure_ascii=False).encode() + b"\n"
    if args.output.suffix == ".gz":
        args.output.write_bytes(gzip.compress(raw, mtime=0))
    else:
        args.output.write_bytes(raw)
    print(f"{len(cases)} cases written to {args.output}")


if __name__ == "__main__":
    main()
