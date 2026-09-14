#!/usr/bin/env python3
"""Build the bounded, deterministic cmark differential corpus."""

from __future__ import annotations


def make_cases() -> list[dict[str, str]]:
    cases: list[dict[str, str]] = []

    def add(category: str, profile: str, name: str, markdown: str) -> None:
        cases.append({"id": name, "category": category, "profile": profile, "markdown": markdown})

    structural = [
        "- one\n  - two\n    - three\n\n> quote with **bold _nested_** and `code`\n",
        "> > quoted\n> > - item\n> >   - nested\n\nback\n",
        "1. alpha\n   1. beta\n      1. gamma\n\n2. delta\n",
        "- [x] done\n- [ ] todo\n\n> [link](https://example.com/a_(b))\n",
        "[outer [inner](u)](v) and **bold *em* `code`**\n",
        "***strong and em*** with ___mixed___ and **`code`**\n",
        "```` js extra words\n<>&\n````\n\n~~~python\nprint(1)\n~~~\n",
        "[ref]: /url_(x) \"title\"\n\n> [ref] and ![alt][ref]\n",
        "- a\n\n  > quote\n  >\n  > `literal`\n\n- b\n",
        "A  \nline with\nsoft break and\\\nhard break\n",
        "<https://example.com/a_(b)> and <mailto:a@example.com>\n",
        "foo\n===\n\nbar\n---\n",
        "## heading *with* [link](u)\n\n    indented `code`\n",
        "<div>raw <em>html</em></div>\n\nparagraph\n",
        "- a\n  continuation with [x](u) and ![i](v)\n",
        "`a  b` and `` a ` b `` and ```x```\n",
        "***a **b _c_** d***\n\n[a *b*](u)\n",
        "> # heading\n>\n> 1. one\n> 2. two\n",
        "[a](<foo bar>) [b](foo\\bar) [c](foo%23bar#x)\n",
        "- one\n\n  ```\n  x\n  ```\n\n  - two\n",
    ]
    for i, source in enumerate(structural, 1):
        add("nested-structure", "commonmark", f"structure-{i:02d}", source)

    for depth in range(1, 9):
        list_lines = ["  " * (level - 1) + "- " + f"level {level}" for level in range(1, depth + 1)]
        add("generated-nesting", "commonmark", f"list-depth-{depth:02d}", "\n".join(list_lines) + "\n")
        quote = "\n".join("> " * level + "nested" for level in range(1, depth + 1))
        add("generated-nesting", "commonmark", f"quote-depth-{depth:02d}", quote + "\n")

    for i, marker in enumerate(("*", "_", "~"), 1):
        source = f"> - {marker}outer {marker}\n>   - **{marker}inner{marker}**\n>\n>     `[code]`\n"
        add("generated-nesting", "commonmark", f"mixed-nesting-{i:02d}", source)
    for i in range(1, 4):
        source = "\n".join(
            f"{'> ' * level}- [x] **item {level}** with `{marker}`"
            for level, marker in enumerate(("a", "b", "c", "d"), 1)
        ) + "\n"
        add("generated-nesting", "commonmark", f"mixed-list-quote-{i:02d}", source)
    for i in range(1, 9):
        source = f"{i}. **bold** and *em* with [link](u{i})\n\n" + "> " * (i % 4 + 1) + "quote `code`\n"
        add("generated-nesting", "commonmark", f"mixed-inline-{i:02d}", source)

    tilde_cases = [
        "~foo ~ bar~\n",
        "~~foo ~~ bar~~\n",
        "~a [b](u~r)~\n",
        "~~a [b](u~~r)~~\n",
        "~a `~` b~\n",
        r"~a\~ b~" + "\n",
        "~one~ ~~two~~ ~~~three~~~\n",
        "~~~one~~ two~ three~~~\n",
        "a ~~b~~ c ~d~ e ~~~f~~~\n",
        "[x](u \"title ~one~\") and [y](v \"~~two~~\")\n",
        "~~[x](u \"~title~\")~~\n",
        "~a **b ~~c~~** d~\n",
    ]
    for i, source in enumerate(tilde_cases, 1):
        add("tilde-binding", "gfm", f"tilde-{i:02d}", source)

    line_bases = [
        "- a\n  - b\n\n> c\n",
        "[x](url) and `code`\n",
        "# heading\n\ntext\n",
        "<https://example.com/a>\n",
        "| a | b |\n|---|---|\n| c | d |\n",
        "~~~\na\n~~~\n",
        "a  \nb\n",
        "\n\n- x\n\n\n- y\n",
    ]
    for i, source in enumerate(line_bases, 1):
        for ending, label in [("\r\n", "crlf"), ("\r", "cr")]:
            add("line-endings", "commonmark", f"line-{i:02d}-{label}", source.replace("\n", ending))

    unicode_cases = [
        "こんにちは **世界** — café — 😀\n",
        "[中文](https://例え.テスト/道) and `é`\n",
        "\u0000 before NUL\n",
        "after NUL \u0000 and [link](u)\n",
        "\ufeff BOM and nbsp space\n",
        "emoji 👩‍💻 and combining é\n",
        "\U0001f4a9\n\n- naïve\n",
        "<https://example.com/ümlaut?q=é>\n",
    ]
    for i, source in enumerate(unicode_cases, 1):
        add("unicode-and-nul", "commonmark", f"unicode-{i:02d}", source)

    html_cases = [
        "<script>alert(1)</script>\n",
        "<style> a { color: red; } </style>\n",
        "<div title=\"x & y\">text</div>\n",
        "<!-- comment -->\n\ntext\n",
        "<custom-tag/>\n",
        "<table><tr><td>x</td></tr></table>\n",
        "<br>after\n",
        "<x a='1' b=2>raw</x>\n",
    ]
    for i, source in enumerate(html_cases, 1):
        add("raw-html", "commonmark", f"html-{i:02d}", source)

    gfm_cases = [
        "| head | other |\n| :--- | ---: |\n| a | b |\n",
        "- [x] checked\n- [ ] open\n",
        "~~strike **bold**~~\n",
        "www.example.com and https://example.com\n",
        "A\n===\n\n| a | b |\n|---|---|\n| c | d |\n",
        "- item\n  - [x] nested task\n",
        "~~a `code` and [link](u)~~\n",
        "| a | b | c |\n|---|:---:|---:|\n| 1 | 2 | 3 |\n",
        "[^1]: footnote\n\ntext[^1]\n",
        "<https://example.com> and user@example.com\n",
        "~~one~ two~~ and ~three~~\n",
        "- [ ] **task** with `code`\n",
    ]
    for i, source in enumerate(gfm_cases, 1):
        add("gfm-extensions", "gfm", f"gfm-{i:02d}", source)

    assert 100 <= len(cases) <= 300, len(cases)
    return cases


if __name__ == "__main__":
    import argparse
    import json
    from pathlib import Path

    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="verify the checked-in JSON exactly matches this generator")
    parser.add_argument("--output", type=Path, default=Path(__file__).with_name("cmark-fixtures.json"))
    args = parser.parse_args()
    expected = json.dumps({"generator": "cmark_fixtures.py", "cases": make_cases()}, indent=2, ensure_ascii=False) + "\n"
    if args.check:
        actual = args.output.read_text(encoding="utf-8")
        if actual != expected:
            raise SystemExit(f"fixture drift: {args.output}")
        print(f"fixture generator matches {args.output}")
    else:
        args.output.write_text(expected, encoding="utf-8")
        print(f"wrote {len(make_cases())} cases to {args.output}")
