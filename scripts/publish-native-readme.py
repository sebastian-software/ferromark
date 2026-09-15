#!/usr/bin/env python3
"""Publish archived ARM measurements to the website without changing the archive."""

import argparse
from pathlib import Path
import shutil
import re
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[1]
REPORT = Path("docs/reports/2026-09-15-native-arm")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    source = ROOT / "homepage/app/routes/guide/benchmarks.mdx"
    with tempfile.TemporaryDirectory(prefix="ferromark-readme-") as directory:
        temporary = Path(directory)
        # The archived publisher expects an unthemed root README.md and rewrites
        # its report too. Isolate both writes so all archived checksums remain valid.
        shutil.copytree(ROOT / REPORT, temporary / REPORT)
        original = source.read_text()
        prefix = "https://github.com/sebastian-software/ferromark/blob/main/"
        (temporary / "README.md").write_text(original.replace("](" + prefix, "]("))
        subprocess.run(
            [sys.executable, str(temporary / REPORT / "publish.py"), "--update-readme"],
            check=True,
            cwd=temporary,
        )
        generated = (temporary / "README.md").read_text()
        generated = re.sub(r"\]\(((?:docs|benchmarks|crates)/[^)]+)\)",
                           lambda match: "](" + prefix + match[1] + ")", generated)
    generated = generated.replace("| <512 B |", "| &lt;512 B |")
    if args.check:
        if source.read_text() != generated:
            raise SystemExit("Website benchmark drift: run python3 scripts/publish-native-readme.py")
        print("Website benchmark section matches the archived measurements")
    else:
        source.write_text(generated)
        print("Updated the website benchmark guide")


if __name__ == "__main__":
    main()
