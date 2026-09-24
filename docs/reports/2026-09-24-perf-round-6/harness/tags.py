"""Count raw-HTML tag names per document: tags.py <case-name>...

Reads corpus.json from ROUND6_WORK, the round's scratch directory (the corpus is not archived; its
SHA-256 is in every run.json).
"""
import collections
import json
import os
import re
import sys
from pathlib import Path

corpus = json.loads((Path(os.environ["ROUND6_WORK"]) / "corpus.json").read_text())
cases = {c["name"]: c for c in corpus["cases"]}
for name in sys.argv[1:]:
    text = cases[name]["input"]
    names = collections.Counter(m.group(1).lower() for m in re.finditer(r"</?([A-Za-z][A-Za-z0-9-]*)", text))
    print(name, cases[name]["profile"], len(text), "bytes, '<' count", text.count("<"))
    print("  ", names.most_common(12))
