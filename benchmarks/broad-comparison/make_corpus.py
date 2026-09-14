#!/usr/bin/env python3
"""Freeze a stratified corpus without consulting timings or output admission."""

import argparse
from collections import Counter
import gzip
import hashlib
import json
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parent
BINS = ((512, "<512 B"), (2048, "512 B–2 KiB"), (10240, "2–10 KiB"),
        (51200, "10–50 KiB"), (262144, "50–256 KiB"), (float("inf"), ">=256 KiB"))

# Authored examples model different comment shapes; they are not scraped users'
# comments, repeated templates, or padding to reach a target byte count.
COMMENTS = {
    "ack": "Thanks, this fixes the issue for me.\n",
    "question": "Does this also apply when the connection is already open? I can still reproduce the original behavior after refreshing the page, but only on the first request.\n",
    "review": "The change looks reasonable. Could we keep the existing error message when the file is missing?\n\nThat message is linked from the troubleshooting guide, and a few users may search for its exact wording. The new wording would still make sense for files that exist but cannot be read.\n",
    "links": "I found a related discussion in [the design notes](https://example.org/design#timeouts). The proposed default seems fine for local files, but a network mount can take longer.\n\nCould the caller supply a timeout? See also <https://example.org/issues/42> for a small reproduction.\n",
    "checklist": "A few things to check before merging:\n\n- [x] Preserve the public function name\n- [x] Update the example in the guide\n- [ ] Try an empty input\n- [ ] Confirm that the error still includes the file path\n\nThe implementation otherwise looks good. I can help with the last two cases tomorrow.\n",
    "quote": "> The result should remain stable when the input order changes.\n\nI agree for distinct entries. What should happen when two entries have the same key? Keeping the first one would match the current behavior; keeping the last one might be easier to explain, but would change existing results.\n",
    "unicode": "Das Verhalten lässt sich auch mit Umlauten reproduzieren: **Änderungen**, *Größe* und `straße.md`.\n\nIm Suchfeld funktioniert „größer“, im exportierten Dokument fehlt dagegen der Link. Mit 日本語 und Ελληνικά sehe ich dasselbe Problem. Könnte die Normalisierung an zwei verschiedenen Stellen stattfinden?\n",
    "inline-code": "I think `render_into()` should clear the destination before writing the result. Otherwise a caller that reuses the same `Vec<u8>` can accidentally append a second document.\n\nWe should document whether capacity is retained. That is the part that matters for the repeated-call use case.\n",
    "reproduction": "I can reproduce this with an empty title:\n\n```rust\nlet input = \"# \\n\\nA paragraph.\\n\";\nlet html = render(input);\nassert!(!html.is_empty());\n```\n\nExpected: the paragraph is still rendered. Actual: the output is empty. This happens with both a trailing newline and no newline at the end of the file.\n",
    "table": "Here are the cases I checked locally:\n\n| Input | First call | Second call |\n| --- | --- | --- |\n| Empty | OK | OK |\n| One paragraph | OK | OK |\n| Nested list | OK | Missing item |\n\nThe failure only appears when the same renderer is used twice. Creating a new renderer for each call makes all three cases pass.\n",
    "review-long": "Thanks for putting the example together. I tried it against the three inputs from the issue and can confirm that the original failure is fixed.\n\nThere are two details I would like to clarify before this becomes the default:\n\n1. A caller may keep a reference to the previous result while processing the next document. An owned return value makes that straightforward; a borrowed result needs a short example showing where the borrow ends.\n2. The empty-input case should behave the same as the one-shot function. Returning a newline in one path and an empty string in the other would make snapshot tests surprising.\n\nNeither point requires a new option. A short paragraph in the API documentation and an example using two different documents should be enough.\n\nI also checked the [migration guide](https://example.org/migration). It currently says that the output buffer is always replaced. Please update that sentence to explain when capacity can be reused.\n",
    "incident": "### Reproduction on a clean checkout\n\nThe first page builds successfully, but the second page contains a link from the first one. Restarting the process clears the problem. I tested this with a small project containing two Markdown files and no plugins.\n\n**Steps**\n\n1. Create a page with a reference-style link.\n2. Create another page that uses the same label but does not define it.\n3. Render both pages through the same renderer.\n\nThe second page should display the unresolved label as text. Instead, it links to the destination from the first page. Reversing the file order changes which page fails.\n\n```markdown\nSee [the guide][help].\n\n[help]: /getting-started\n```\n\nI have not seen the problem when invoking the command separately for each page. It may be worth checking whether reference definitions are cleared along with the output buffer. The behavior is the same on a fresh checkout and on the release package.\n\n> Is this specific to nested links?\n\nNo. A single ordinary paragraph is enough. The larger example in the original report happened to contain a list, but the list is not needed to reproduce the failure.\n",
}


def size_bin(size):
    return next(label for upper, label in BINS if size < upper)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    cases = [{"name": f"comment-{name}", "profile": "gfm", "input": value,
              "category": "comments", "collection": "authored-comments",
              "origin": {"kind": "authored-example", "license": "MIT",
                         "description": "Authored GitHub-style comment shape, not a real user comment.",
                         "transformations": []}}
             for name, value in COMMENTS.items()]
    cases.append({"name": "guard-angle-link", "profile": "commonmark",
                  "input": "[The guide](<https://example.org/guide>)\n",
                  "category": "syntax-guard", "collection": "syntax-guards",
                  "origin": {"kind": "authored-example", "license": "MIT",
                             "description": "Minimized mismatch found during pre-timing HTML-to-Markdown converter validation: main duplicates angle-delimited link destinations.",
                             "transformations": []}})
    for filename in ("local-cases.json.gz", "wiki-cases.json.gz"):
        cases.extend(json.loads(gzip.decompress((ROOT / filename).read_bytes()))["cases"])
    assert len({c["name"] for c in cases}) == len(cases)
    for case in cases:
        assert re.fullmatch(r"[a-z0-9-]+", case["name"]), case["name"]
        assert case["profile"] in {"gfm", "commonmark"}
        raw = case["input"].encode("utf-8")
        case.update(byte_count=len(raw), sha256=hashlib.sha256(raw).hexdigest(), size_bin=size_bin(len(raw)))
        case["features"] = {
            "lines": len(case["input"].splitlines()),
            "fence_marker_lines": len(re.findall(r"(?m)^\s*(?:`{3,}|~{3,})", case["input"])),
            "atx_headings": len(re.findall(r"(?m)^#{1,6}\s", case["input"])),
            "inline_link_markers": case["input"].count("]("),
            "pipe_characters": case["input"].count("|"),
        }
    payload = {
        "schema": 1,
        "selection": "Document selection precedes output checks; final inputs are frozen after converter validation and before timing. Unchanged legacy collection, external documentation selected by size and source, encyclopedia prose/excerpts, varied authored comment shapes, and a minimized syntax guard discovered during conversion validation. No repetition, padding, or performance-based selection.",
        "cases": cases,
        "counts_by_size": dict(Counter(c["size_bin"] for c in cases)),
        "counts_by_category": dict(Counter(c["category"] for c in cases)),
        "counts_by_collection": dict(Counter(c["collection"] for c in cases)),
    }
    args.output.write_text(json.dumps(payload, indent=2, ensure_ascii=False) + "\n")
    print(json.dumps({k: v for k, v in payload.items() if k != "cases"}, indent=2, ensure_ascii=False))


if __name__ == "__main__":
    main()
