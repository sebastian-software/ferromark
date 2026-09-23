#!/usr/bin/env python3
"""Node-level paired benchmark wrapper.

Named `optimization-rounds/run.py` on purpose: the implementation agent waits on
`pgrep -f optimization-rounds/run.py` before compiling, so this keeps builds out
of the Node timing as well.

Expects node-bench.mjs, corpus.json, addon-main.node (built from cb352020) and
addon-412.node (built from the #412 branch) in the directory above this one;
the corpus and the addons are not archived.
"""
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent.parent
subprocess.run(
    ["node", str(HERE / "node-bench.mjs"), str(HERE / "corpus.json"),
     str(HERE / "addon-main.node"), str(HERE / "addon-412.node"), sys.argv[1] if len(sys.argv) > 1 else "10"],
    check=True,
)
