#!/usr/bin/env python3
"""Verify rendered output before measuring; retain raw HTML, samples, and source metadata."""
import argparse
import hashlib
from html.parser import HTMLParser
import json
from pathlib import Path
import platform
import statistics
import subprocess
import time

from prepare import BUN_REV, MI_REV, HWY_REV, TOOLCHAIN, HERE, REPO, git


class CanonicalHTML(HTMLParser):
    """Limited serialization normalization; not a browser DOM or sanitizer."""
    VOID = {"area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source", "track", "wbr"}

    def __init__(self, text):
        super().__init__(convert_charrefs=True)
        self.tokens = []
        self.literal = 0
        self.feed(text)
        self.close()

    def handle_starttag(self, tag, attrs):
        attrs = dict(attrs)
        for name in ("disabled", "checked"):
            if name in attrs:
                attrs[name] = ""
        if tag in ("td", "th") and "align" in attrs:
            attrs["style"] = "text-align:" + attrs.pop("align")
        if tag in ("td", "th") and "style" in attrs:
            attrs["style"] = attrs["style"].replace(" ", "").rstrip(";")
        self.tokens.append(("start", tag, sorted(attrs.items())))
        if tag in ("pre", "code"):
            self.literal += 1

    def handle_startendtag(self, tag, attrs):
        self.handle_starttag(tag, attrs)
        if tag not in self.VOID:
            self.handle_endtag(tag)

    def handle_endtag(self, tag):
        if tag not in self.VOID:
            self.tokens.append(("end", tag))
        if tag in ("pre", "code"):
            self.literal -= 1

    def handle_data(self, text):
        if not self.literal and "\n" in text and not text.strip():
            return
        if self.tokens and self.tokens[-1][0] == "text":
            self.tokens[-1] = ("text", self.tokens[-1][1] + text)
        else:
            self.tokens.append(("text", text))

    def handle_comment(self, text):
        self.tokens.append(("comment", text))

    def handle_decl(self, text):
        self.tokens.append(("decl", text))

    def handle_pi(self, text):
        self.tokens.append(("pi", text))


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def capture(args):
    return subprocess.check_output(args, text=True).strip()


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("bun", type=Path)
    p.add_argument("result", type=Path, help="New output directory; existing results are never overwritten")
    args = p.parse_args()
    bun, result = args.bun.resolve(), args.result.resolve()
    result.mkdir(parents=True, exist_ok=False)
    binary = bun / "target/release/ferromark-bun-comparison"
    if git(bun, "rev-parse", "HEAD") != BUN_REV or git(bun, "diff", "HEAD", "--", "src", "scripts/build"):
        raise SystemExit("Pinned Bun sources changed; refusing measurement")
    if (bun / "ferromark-comparison/driver.rs").read_bytes() != (HERE / "driver.rs").read_bytes():
        raise SystemExit("Rebuild after changing driver.rs")
    (result / "ferromark.patch").write_text(git(REPO, "diff", "HEAD", "--", "src", "Cargo.toml", "Cargo.lock") + "\n")
    (result / "bun-workspace.patch").write_text(git(bun, "diff", "HEAD", "--", "Cargo.toml") + "\n")
    (result / "Cargo.lock").write_bytes((bun / "Cargo.lock").read_bytes())
    metadata = {
        "bun_revision": BUN_REV, "mimalloc_revision": MI_REV, "highway_revision": HWY_REV,
        "ferromark_revision": git(REPO, "rev-parse", "HEAD"),
        "ferromark_source_sha256": {str(f.relative_to(REPO)): sha(f) for f in sorted((REPO / "src").rglob("*.rs"))},
        "harness_sha256": {f.name: sha(f) for f in sorted(HERE.iterdir()) if f.suffix in (".rs", ".py", ".c", ".h")},
        "binary_sha256": sha(binary), "platform": platform.platform(),
        "cpu": capture(["sysctl", "-n", "machdep.cpu.brand_string"]),
        "rustc": capture(["rustc", f"+{TOOLCHAIN}", "-Vv"]),
        "clang": capture(["clang++", "--version"]),
        "rustflags": "-C target-cpu=generic", "profile": "Bun release: opt-level=3, fat LTO, 1 CGU, panic=abort",
        "allocation": "fresh owned HTML output, process-wide original Bun mimalloc for all four parsers",
        "started_unix": time.time(),
        "fixture_sha256": {str(f.relative_to(REPO)): sha(f) for f in sorted((REPO / "benches/fixtures").glob("*.md"))},
        "spec_sha256": sha(REPO / "tests/spec.json"),
    }
    stamp = json.loads((bun / "ferromark-comparison/build-info.json").read_text())
    for key in ("binary_sha256", "ferromark_source_sha256"):
        if stamp[key] != metadata[key]:
            raise SystemExit(f"Build inputs changed ({key}); rerun prepare.py")
    metadata["native_archives_sha256"] = stamp["archives"]
    (result / "metadata.json").write_text(json.dumps(metadata, indent=2) + "\n")
    for mode in ("verify", "spec"):
        with (result / f"{mode}.jsonl").open("w") as out:
            subprocess.run([binary, mode], stdout=out, check=True)
    cases = []
    allowed = []
    for line in (result / "verify.jsonl").read_text().splitlines():
        row = json.loads(line)
        outputs = row["outputs"]
        reference = outputs["ferromark"]
        mismatches = [name for name, html in outputs.items() if CanonicalHTML(html).tokens != CanonicalHTML(reference).tokens]
        cases.append({"case": row["case"], "bytes": row["bytes"], "exact": all(h == reference for h in outputs.values()),
                      "mismatches": mismatches, "output_sha256": {k: hashlib.sha256(v.encode()).hexdigest() for k, v in outputs.items()}})
        if not mismatches:
            allowed.append(row["case"])
    spec = {}
    for line in (result / "spec.jsonl").read_text().splitlines():
        row = json.loads(line)
        for name, html in row["outputs"].items():
            counts = spec.setdefault(name, {"total": 0, "exact": 0, "normalized": 0, "mismatched_examples": []})
            counts["total"] += 1
            counts["exact"] += html == row["expected"]
            match = CanonicalHTML(html).tokens == CanonicalHTML(row["expected"]).tokens
            counts["normalized"] += match
            if not match:
                counts["mismatched_examples"].append(row["example"])
    (result / "verification.json").write_text(json.dumps({"cases": cases, "spec": spec}, indent=2) + "\n")
    (result / "allowlist.json").write_text(json.dumps(allowed, indent=2) + "\n")
    print(f"Verified {len(allowed)}/{len(cases)} benchmark cases; excluded differences recorded.", flush=True)
    if not allowed:
        raise SystemExit("No comparable output; no timings collected")
    with (result / "samples.jsonl").open("w") as out:
        subprocess.run([binary, "bench", result / "allowlist.json"], stdout=out, check=True)
    summary = []
    for line in (result / "samples.jsonl").read_text().splitlines():
        row = json.loads(line)
        median = statistics.median(row["ns_per_render"])
        summary.append({**row, "median_ns": median, "mib_per_second": row["bytes"] / median * 1e9 / 1048576})
    (result / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    print(result)


if __name__ == "__main__":
    main()
