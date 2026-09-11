"""Source provenance and fixtures for the native cmark comparison."""
import hashlib
import json
from pathlib import Path
import subprocess

HERE = Path(__file__).resolve().parent
REPO = HERE.parent.parent
SOURCES = {
    "cmark": {"version": "0.31.1", "revision": "bb3678d7a73cb02d35c8876ecd097072636200a8"},
    "cmark-gfm": {"version": "0.29.0.gfm.13", "revision": "587a12bb54d95ac37241377e6ddc93ea0e45439b"},
}


def capture(args, **kwargs):
    return subprocess.check_output(list(map(str, args)), text=True, **kwargs).strip()


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def write_json(path, data):
    path.write_text(json.dumps(data, indent=2, ensure_ascii=False) + "\n")


def source_hashes(source, revision):
    if capture(["git", "-C", source, "rev-parse", "HEAD"]) != revision:
        raise ValueError(f"{source} must be at {revision}")
    if capture(["git", "-C", source, "status", "--porcelain", "--untracked-files=all"]):
        raise ValueError(f"{source} must be a clean, dedicated checkout")
    files = subprocess.check_output(["git", "-C", str(source), "ls-files", "-z"]).decode().split("\0")
    return {name: sha(source / name) for name in sorted(files) if name}


def local_hashes():
    files = list((REPO / "src").rglob("*.rs")) + list((REPO / "crates").rglob("*.rs"))
    files += list((REPO / "crates").rglob("Cargo.toml"))
    files += [REPO / "Cargo.toml", REPO / "Cargo.lock", REPO / "rust-toolchain.toml"]
    files += [p for p in HERE.rglob("*") if p.is_file() and p.suffix in (".rs", ".c", ".py", ".toml", ".lock")]
    files += [REPO / "benchmarks/bun-comparison/workload.py"]
    return {str(p.relative_to(REPO)): sha(p) for p in sorted(files)}


def cases(competitor):
    # Core-only synthetic documents. Match the native publication's byte sizes;
    # the real table inputs below are exactly the fixtures added in PR #295.
    unit = "## Project notes\n\nA paragraph with **strong emphasis**, *emphasis*, and `inline code`.\n\nRead the [documentation](https://example.com/guide?a=1&b=2) for details &amp; examples.\n\n- First item\n- Second item with a [link](/relative)\n\n> A quoted paragraph with a useful explanation.\n\n```rust\nlet answer = 42;\n```\n\n"
    result = [{"case": "commonmark/short", "flags": 0, "input": "Hello, **world**!\n"}]
    for size in (2, 5, 10, 50):
        text = unit * (size * 1024 // len(unit))
        text += "x" * (size * 1024 - len(text) - 1) + "\n"
        result.append({"case": f"commonmark/{size}k", "flags": 0, "input": text})
    if competitor == "cmark-gfm":
        for name in ("tables-plain", "tables-links", "tables-commonmark-inline"):
            text = (REPO / "benches/fixtures" / f"{name}.md").read_text()
            for lane, flags in (("tables", 1), ("gfm_overlap", 7)):
                result.append({"case": f"{lane}/{name}", "flags": flags, "input": text})
        for lane, flags, text in (
            ("strikethrough", 2, "Some ~~old words~~ and **replacement words**.\n\n" * 100),
            ("task_lists", 4, "- [x] First task\n- [ ] Second task\n\n" * 100),
            ("gfm_overlap", 7, "| Item | State |\n| --- | --- |\n| **table** | ~~pending~~ done |\n\n- [x] First task\n- [ ] Second task\n\n" * 50),
        ):
            result.append({"case": f"{lane}/features", "flags": flags, "input": text})
        for lane, flags in (("tables", 1), ("strikethrough", 2), ("task_lists", 4), ("gfm_overlap", 7)):
            result.append({"case": f"{lane}/neutral", "flags": flags, "input": unit})
    return result
