#!/usr/bin/env python3
"""Verify rendered output before measuring; retain raw HTML, samples, and source metadata."""
import argparse
import hashlib
from html.parser import HTMLParser
import json
from pathlib import Path
import platform
import re
import statistics
import subprocess
import time

from prepare import BUN_REV, MI_REV, HWY_REV, MD4C_REV, TOOLCHAIN, HERE, REPO, git

PARSERS = {"ferromark", "bun_md", "pulldown-cmark", "comrak", "md4c"}
HEADLINE_CASES = [
    "commonmark/publication-2k", "commonmark/publication-5k", "commonmark/publication-10k",
    "tables/gfm-tables", "strikethrough/strikethrough", "gfm_overlap/gfm-tables",
    "commonmark/links", "commonmark/entities",
]


def mismatches(outputs):
    if set(outputs) != PARSERS:
        raise ValueError("Every case must include all five parser outputs")
    reference = CanonicalHTML(outputs["ferromark"]).tokens
    return [name for name, html in outputs.items() if CanonicalHTML(html).tokens != reference]


def workload_review(case, outputs):
    """Separate workload eligibility from HTML fidelity; never hide the raw diff."""
    if set(outputs) != PARSERS:
        raise ValueError("Every case must include all five parser outputs")
    streams = [CanonicalHTML(html).tokens for html in outputs.values()]
    def equal(items):
        return all(item == items[0] for item in items[1:])
    if equal(streams):
        return {"comparable": True, "html_equivalent": True, "accepted_differences": []}
    lane = case.split("/")[0]
    reviews = []
    if lane in ("tables", "gfm_overlap"):
        reviews.append(("table-alignment", _review_alignment))
    if lane in ("task_lists", "gfm_overlap"):
        reviews.append(("task-presentation", _review_tasks))
    for name, transform in reviews:
        if equal([transform(tokens) for tokens in streams]):
            return {"comparable": True, "html_equivalent": False, "accepted_differences": [name]}
    if len(reviews) == 2 and equal([_review_tasks(_review_alignment(tokens)) for tokens in streams]):
        return {"comparable": True, "html_equivalent": False,
                "accepted_differences": [name for name, _ in reviews]}
    return {"comparable": False, "html_equivalent": False, "accepted_differences": []}


def _review_alignment(tokens):
    result = []
    for token in tokens:
        if token[0] == "start" and token[1] in ("td", "th"):
            attrs = dict(token[2])
            if "style" in attrs:
                styles = [s for s in attrs["style"].split(";") if not s.startswith("text-align:")]
                if styles:
                    attrs["style"] = ";".join(styles)
                else:
                    del attrs["style"]
            token = ("start", token[1], sorted(attrs.items()))
        result.append(token)
    return result


def _review_tasks(tokens):
    def checkbox(token):
        return token[0] == "start" and token[1] == "input" and dict(token[2]).get("type") == "checkbox"
    result = []
    for token in tokens:
        if token[0] == "start" and (token[1] == "li" or checkbox(token)):
            attrs = dict(token[2])
            if "class" in attrs:
                keep = [c for c in attrs["class"].split() if c not in ("task-list-item", "task-list-item-checkbox")]
                if keep:
                    attrs["class"] = " ".join(keep)
                else:
                    del attrs["class"]
            token = ("start", token[1], sorted(attrs.items()))
        if token[0] == "text" and result and checkbox(result[-1]):
            # Space beside a checkbox is a renderer convention, not a word boundary.
            text = token[1].lstrip(" ")
            if not text:
                continue
            token = ("text", text)
        result.append(token)
    # Both conventions implement the same task: checkbox inside the first
    # paragraph, or immediately before that paragraph in the same list item.
    for i in range(len(result) - 2):
        if (result[i][0:2] == ("start", "li") and checkbox(result[i + 1])
                and result[i + 2][0:2] == ("start", "p")):
            result[i + 1], result[i + 2] = result[i + 2], result[i + 1]
    return result


class CanonicalHTML(HTMLParser):
    """Limited serialization normalization; not a browser DOM or sanitizer."""
    VOID = {"area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source", "track", "wbr"}

    BLOCK = {"p", "div", "blockquote", "ul", "ol", "li", "pre", "h1", "h2", "h3", "h4", "h5", "h6",
             "table", "thead", "tbody", "tfoot", "tr", "td", "th", "hr"}
    LITERAL = {"pre", "code", "textarea", "script", "style"}

    def __init__(self, text):
        super().__init__(convert_charrefs=True)
        self.tokens = []
        self.literal = 0
        self.feed(text)
        self.close()
        # Default HTML flow only: retain word separators, trim at known block
        # boundaries, and never alter literal/code text or non-breaking spaces.
        raw, self.tokens = self.tokens, []
        literal = 0
        for i, token in enumerate(raw):
            kind, value = token[:2]
            if kind == "text" and not literal:
                value = re.sub(r"[ \t\r\n\f]+", " ", value)
                before = raw[i - 1] if i else None
                after = raw[i + 1] if i + 1 < len(raw) else None
                def boundary(t):
                    return t is None or (t[0] in ("start", "end") and t[1] in self.BLOCK)
                if boundary(before):
                    value = value.lstrip(" ")
                if boundary(after):
                    value = value.rstrip(" ")
                if not value:
                    continue
                token = ("text", value)
            self.tokens.append(token)
            if kind == "start" and value in self.LITERAL:
                literal += 1
            elif kind == "end" and value in self.LITERAL:
                literal = max(0, literal - 1)

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
    p.add_argument("--verify-only", action="store_true")
    p.add_argument("--case", action="append", help="Measure only named cases, each in three publication runs")
    p.add_argument("--screening", action="store_true", help="Short smoke run, never used for publication")
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
        "bun_revision": BUN_REV, "mimalloc_revision": MI_REV, "highway_revision": HWY_REV, "md4c_revision": MD4C_REV,
        "ferromark_revision": git(REPO, "rev-parse", "HEAD"),
        "ferromark_source_sha256": {str(f.relative_to(REPO)): sha(f) for f in sorted((REPO / "src").rglob("*.rs"))},
        "harness_sha256": {f.name: sha(f) for f in sorted(HERE.iterdir()) if f.suffix in (".rs", ".py", ".c", ".h")},
        "binary_sha256": sha(binary), "platform": platform.platform(),
        "cpu": capture(["sysctl", "-n", "machdep.cpu.brand_string"]),
        "rustc": capture(["rustc", f"+{TOOLCHAIN}", "-Vv"]),
        "clang": capture(["clang++", "--version"]),
        "rustflags": "-C target-cpu=generic", "profile": "Bun release: opt-level=3, fat LTO, 1 CGU, panic=abort",
        "allocation": "fresh parser and owned output; Bun mimalloc for Rust and md4c C allocation calls",
        "started_unix": time.time(),
        "fixture_sha256": {str(f.relative_to(REPO)): sha(f) for f in sorted((REPO / "benches/fixtures").glob("*.md"))},
        "spec_sha256": sha(REPO / "tests/spec.json"),
    }
    stamp = json.loads((bun / "ferromark-comparison/build-info.json").read_text())
    for key in ("binary_sha256", "ferromark_source_sha256"):
        if stamp[key] != metadata[key]:
            raise SystemExit(f"Build inputs changed ({key}); rerun prepare.py")
    adapters = {f.name: sha(f) for f in sorted(HERE.iterdir()) if f.suffix in (".rs", ".c", ".h")}
    if stamp["adapter_sha256"] != adapters or stamp["md4c_revision"] != MD4C_REV:
        raise SystemExit("Native adapter changed; rerun prepare.py")
    metadata["native_archives_sha256"] = stamp["archives"]
    metadata["md4c_source_sha256"] = stamp["md4c_source_sha256"]
    headline_cases = args.case or HEADLINE_CASES
    samples, window_ms, warmup_ms = (5, 20, 50) if args.screening else (80, 63, 3000)
    metadata["protocol"] = {"samples":samples, "window_ms":window_ms, "warmup_ms":warmup_ms,
        "headline_runs":1 if args.screening else 3, "mode":"screening" if args.screening else "publication",
        "headline_cases":headline_cases, "selected_cases":args.case, "statistic":"median of run medians", "timer_check_batch":16}
    (result / "metadata.json").write_text(json.dumps(metadata, indent=2) + "\n")
    subprocess.run([binary, "selftest"], check=True)
    for mode in ("verify", "spec", "catalog", "options"):
        with (result / f"{mode}.jsonl").open("w") as out:
            subprocess.run([binary, mode], stdout=out, check=True)
    cases = []
    allowed = []
    for line in (result / "verify.jsonl").read_text().splitlines():
        row = json.loads(line)
        outputs = row["outputs"]
        reference = outputs["ferromark"]
        differences = mismatches(outputs)
        review = workload_review(row["case"], outputs)
        cases.append({"case": row["case"], "workload": review, "bytes": row["bytes"], "exact": all(h == reference for h in outputs.values()),
                      "mismatches": differences, "output_sha256": {k: hashlib.sha256(v.encode()).hexdigest() for k, v in outputs.items()}})
        if review["comparable"]:
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
    print(f"Admitted {len(allowed)}/{len(cases)} benchmark workloads; HTML differences recorded.", flush=True)
    if args.verify_only:
        return
    if not allowed:
        raise SystemExit("No comparable output; no timings collected")
    if not set(headline_cases).issubset(allowed):
        raise SystemExit("A headline case failed workload eligibility; refusing publication")
    if args.case:
        allowed = [case for case in allowed if case in args.case]
        (result / "allowlist.json").write_text(json.dumps(allowed, indent=2) + "\n")
    (result / "headline-allowlist.json").write_text(json.dumps(headline_cases) + "\n")
    collected = {}
    for repetition in range(1 if args.screening else 3):
        allowlist = "allowlist.json" if repetition == 0 else "headline-allowlist.json"
        path = result / f"samples-{repetition}.jsonl"
        with path.open("w") as out:
            subprocess.run([binary, "bench", result / allowlist, str(samples), str(window_ms), str(warmup_ms), str(repetition)], stdout=out, check=True)
        for line in path.read_text().splitlines():
            row = json.loads(line)
            entry = collected.setdefault((row["case"],row["parser"]), {**row, "runs":[]})
            entry["runs"].append(row["ns_per_render"])
        print(f"Completed measurement run {repetition + 1}.", flush=True)
    summary = []
    for row in collected.values():
        medians = [statistics.median(run) for run in row.pop("runs")]
        median = statistics.median(medians)
        row.pop("ns_per_render")
        summary.append({**row, "run_medians_ns":medians, "median_ns": median,
                        "mib_per_second": row["bytes"] / median * 1e9 / 1048576})
    (result / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    metadata["finished_unix"] = time.time()
    (result / "metadata.json").write_text(json.dumps(metadata, indent=2) + "\n")
    print(result)


if __name__ == "__main__":
    main()
