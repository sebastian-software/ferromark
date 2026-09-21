#!/usr/bin/env python3
"""Publish archived measurements to the website and README without changing the archive.

The archived report's own ``publish.py`` derives every figure from the raw
evidence. This wrapper runs it in a temporary copy so the archive's checksums
stay valid, then writes three things from that one source: the website's native
comparison section (``homepage/app/routes/guide/benchmarks.mdx``), the homepage
figures (``homepage/app/data/native-benchmarks.json``), and the marked sentence
in ``README.md.src`` and its generated ``README.md``. ``--check`` fails when any
of them drifts from the archive.
"""

import argparse
import json
import re
import shutil
import subprocess
import sys
import tempfile
import textwrap
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
REPORT = Path("docs/reports/2026-09-21-native-release-fixed")
GUIDE = ROOT / "homepage/app/routes/guide/benchmarks.mdx"
FIGURES = ROOT / "homepage/app/data/native-benchmarks.json"
READMES = (ROOT / "README.md.src", ROOT / "README.md")
README_START = "<!-- native-benchmarks -->"
README_END = "<!-- /native-benchmarks -->"
PREFIX = "https://github.com/sebastian-software/ferromark/blob/main/"


def speed(value):
    """The landing page and README state one decimal: 2.07 becomes 2.1×."""
    return f"{value:.1f}×"


def readme_block(figures):
    by_id = {figure["id"]: figure for figure in figures["figures"]}
    sentence = (
        f"On {figures['documents']['fiveEngineAgreement']} real documents rendered to "
        f"equivalent HTML, v2 completes Markdown to HTML "
        f"{speed(by_id['v1']['fresh'])} faster than Ferromark v1, "
        f"{speed(by_id['pulldown-cmark']['fresh'])} faster than pulldown-cmark, "
        f"{speed(by_id['md4c']['fresh'])} faster than md4c, and "
        f"{speed(by_id['bun']['fresh'])} faster than Bun's native engine "
        f"({figures['machine']}, fresh parser state, {figures['measured']})."
    )
    return README_START + "\n" + textwrap.fill(sentence, width=80) + "\n" + README_END


def readme_span(text, path):
    start = text.find(README_START)
    end = text.find(README_END, start)
    if start == -1 or end == -1:
        raise SystemExit(f"{path} has no {README_START} … {README_END} block")
    return start, end + len(README_END)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    with tempfile.TemporaryDirectory(prefix="ferromark-readme-") as directory:
        temporary = Path(directory)
        # The archived publisher expects an unthemed root README.md and rewrites
        # its report too. Isolate both writes so all archived checksums remain valid.
        shutil.copytree(ROOT / REPORT, temporary / REPORT)
        original = GUIDE.read_text()
        (temporary / "README.md").write_text(original.replace("](" + PREFIX, "]("))
        subprocess.run(
            [sys.executable, str(temporary / REPORT / "publish.py"), "--update-readme",
             "--website-json", str(temporary / "native-benchmarks.json")],
            check=True,
            cwd=temporary,
        )
        guide = (temporary / "README.md").read_text()
        guide = re.sub(r"\]\(((?:docs|benchmarks|crates)/[^)]+)\)",
                       lambda match: "](" + PREFIX + match[1] + ")", guide)
        figures_text = (temporary / "native-benchmarks.json").read_text()
    guide = guide.replace("| <512 B |", "| &lt;512 B |")
    block = readme_block(json.loads(figures_text))
    if args.check:
        if original != guide:
            raise SystemExit("Website benchmark drift: run python3 scripts/publish-native-readme.py")
        if not FIGURES.exists() or FIGURES.read_text() != figures_text:
            raise SystemExit("Homepage figure drift: run python3 scripts/publish-native-readme.py")
        for path in READMES:
            text = path.read_text()
            start, end = readme_span(text, path)
            if text[start:end] != block:
                raise SystemExit(f"{path.name} benchmark drift: run python3 scripts/publish-native-readme.py")
        print("Website, homepage and README benchmark figures match the archived measurements")
    else:
        GUIDE.write_text(guide)
        FIGURES.write_text(figures_text)
        for path in READMES:
            text = path.read_text()
            start, end = readme_span(text, path)
            path.write_text(text[:start] + block + text[end:])
        print("Updated the website benchmark guide, the homepage figures and the README sentence")


if __name__ == "__main__":
    main()
