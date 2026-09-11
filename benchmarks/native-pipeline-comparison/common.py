"""Pinned inputs and the persistent-worker protocol for native pipeline comparisons."""
import json
import os
from pathlib import Path
import select
import subprocess
import sys

HERE = Path(__file__).resolve().parent
REPO = HERE.parent.parent
sys.path.insert(0, str(REPO / "benchmarks/cmark-comparison"))
from support import capture, cases, sha, source_hashes, write_json
sys.path.insert(0, str(REPO / "benchmarks/bun-comparison"))
from workload import CanonicalHTML, workload_review
sys.path.insert(0, str(HERE))

ENGINES = ("goldmark-v1", "goldmark-v2", "satteri")
SATTERI = "b3d38e1e341c809b20b76a655e9b1601d11bd1f0"
GO_VERSION = "go1.27.1"
GO_MODULES = {"github.com/yuin/goldmark": "v1.8.6", "github.com/yuin/goldmark/v2": "v2.0.2"}


def catalog():
    return cases("cmark-gfm")


def probes():
    text = "| A |\n| --- |\n| B |\n\n~~old~~\n\n- [x] done\n- [ ] pending\n\nwww.example.com\n\n<script>x</script>\n"
    result = [{"case": f"switches/{flags}", "flags": flags, "input": text} for flags in range(8)]
    result += [{"case": "dialect/single-tilde", "flags": 2, "input": "~single~\n"},
               {"case": "policy/trusted-urls", "flags": 0, "input": "[link](javascript:alert)\n"}]
    return result


def mdx_cases():
    return [{"case": name, "flags": 0, "input": text} for name, text in (
        ("mdx/markdown", "# Heading\n\nSome **strong** content.\n"),
        ("mdx/root-jsx", "import Card from './Card'\n\n<Card>\n\n# Heading\n\n</Card>\n"),
        ("mdx/inline-expression", "export const name = 'World';\n\nHello {name}!\n"),
        ("mdx/container-jsx", "> <Card>\n>\n> # Heading\n>\n> </Card>\n"),
        ("mdx/invalid-javascript", "export const = ;\n\n# Heading\n"),
    )]


def local_hashes():
    paths = list((REPO / "src").rglob("*.rs")) + list((REPO / "crates").rglob("*.rs"))
    paths += list((REPO / "crates").rglob("Cargo.toml"))
    paths += [REPO / name for name in ("Cargo.toml", "Cargo.lock", "rust-toolchain.toml", "tests/spec.json",
        "benchmarks/cmark-comparison/support.py", "benchmarks/bun-comparison/workload.py")]
    paths += [p for p in HERE.rglob("*") if p.is_file() and "target" not in p.relative_to(HERE).parts
              and p.suffix in (".py", ".rs", ".toml", ".lock", ".go", ".mod", ".sum")]
    paths += list((REPO / "benches/fixtures").glob("tables-*.md"))
    return {str(p.relative_to(REPO)): sha(p) for p in sorted(paths)}


def worker_env():
    env = os.environ.copy()
    for key in ("GODEBUG", "GOMEMLIMIT", "LD_PRELOAD", "DYLD_INSERT_LIBRARIES"):
        env.pop(key, None)
    env.update(GOGC="100", GOMAXPROCS="1")
    return env


class Worker:
    def __init__(self, work, engine, input_file):
        binary = work / ("goldmark-driver" if engine.startswith("goldmark-") else "rust-driver")
        self.engine = engine
        self.process = subprocess.Popen([str(binary), engine, str(input_file)], stdin=subprocess.PIPE,
            stdout=subprocess.PIPE, stderr=None, text=True, bufsize=1, env=worker_env())

    def request(self, op, index, **fields):
        proc = self.process
        proc.stdin.write(json.dumps({"op": op, "index": index, **fields}) + "\n")
        proc.stdin.flush()
        if not select.select([proc.stdout], [], [], 60)[0]:
            raise TimeoutError(f"{self.engine} did not respond within 60 seconds")
        line = proc.stdout.readline()
        if not line:
            raise RuntimeError(f"{self.engine} exited before responding")
        row = json.loads(line)
        if row["engine"] != self.engine:
            raise ValueError("Wrong worker engine")
        return row

    def close(self):
        proc = self.process
        proc.stdin.close()
        try:
            code = proc.wait(timeout=10)
        except subprocess.TimeoutExpired:
            proc.kill()
            proc.wait()
            raise
        finally:
            proc.stdout.close()
        if code:
            raise RuntimeError(f"{self.engine} failed with status {code}")

    def __enter__(self):
        return self

    def __exit__(self, kind, value, traceback):
        if kind is not None:
            self.process.kill()
            self.process.wait()
            self.process.stdin.close()
            self.process.stdout.close()
        else:
            self.close()
