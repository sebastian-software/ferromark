"""Count ATX/setext headings and their text lengths per document: headings.py <case-name>...

Reads corpus.json from ROUND6_WORK, the round's scratch directory (the corpus is not archived; its
SHA-256 is in every run.json). It avoids `statistics`: that module imports `numbers`, which
numbers.py next to this script would shadow.
"""
import json
import os
import re
import sys
from pathlib import Path

corpus = json.loads((Path(os.environ["ROUND6_WORK"]) / "corpus.json").read_text())
cases = {c["name"]: c for c in corpus["cases"]}
for name in sys.argv[1:]:
    text = cases[name]["input"]
    atx = [m.group(2) for m in re.finditer(r"^(#{1,6})[ \t]+(.*)$", text, re.M)]
    setext = re.findall(r"^(.+)\n(=+|-+)[ \t]*$", text, re.M)
    lengths = [len(h) for h in atx] + [len(h[0]) for h in setext]
    distinct = len({h.lower() for h in atx})
    print(f"{name:34} {len(text):7} bytes  headings {len(lengths):4} (distinct atx {distinct})"
          f"  mean len {sum(lengths) / len(lengths) if lengths else 0:.1f}  non-ascii {sum(any(ord(c) > 127 for c in h) for h in atx)}")
