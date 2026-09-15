#!/usr/bin/env python3
"""Run the frozen ARM report publisher against the authored README in isolation."""

import argparse
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[1]
REPORT = Path("docs/reports/2026-09-15-native-arm")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    source = ROOT / "README.md.src"
    with tempfile.TemporaryDirectory(prefix="ferromark-readme-") as directory:
        temporary = Path(directory)
        # The archived publisher expects an unthemed root README.md and rewrites
        # its report too. Isolate both writes so all archived checksums remain valid.
        shutil.copytree(ROOT / REPORT, temporary / REPORT)
        shutil.copyfile(source, temporary / "README.md")
        subprocess.run(
            [sys.executable, str(temporary / REPORT / "publish.py"), "--update-readme"],
            check=True,
            cwd=temporary,
        )
        generated = (temporary / "README.md").read_bytes()
    if args.check:
        if source.read_bytes() != generated:
            raise SystemExit("README benchmark drift: run python3 scripts/publish-native-readme.py")
        print("README benchmark section matches the archived measurements")
    else:
        source.write_bytes(generated)
        print("Updated README.md.src; run mise run readme:write")


if __name__ == "__main__":
    main()
