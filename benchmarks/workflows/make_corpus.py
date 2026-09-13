#!/usr/bin/env python3
"""Freeze a declared, unpadded corpus before measuring. Never select by speed."""
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[2]
HERE = Path(__file__).resolve().parent
DOCUMENTATION = [
    "CONTRIBUTING.md", "node/ferromark/README.md", "docs/migration-0.2.md",
    "docs/migration-0.3.md", "docs/migration-0.4.md", "docs/migration-0.8.md",
    "docs/releasing.md", "docs/readme-theme.md", "docs/markdown-extensions.md",
    "docs/mdx.md", "docs/README.md", "docs/adr/readme-theme-composition.md",
]
GUIDES = [f"homepage/app/routes/guide/{name}.mdx" for name in
          ("quick-start", "features", "mdx-examples")]
PREVIEWS = [
    ("acknowledgment", "Thanks, that fixed it! **Confirmed** on Linux and macOS.\n"),
    ("question", "Could we keep `renderPolicy` explicit here? The [integration guide](/guide/rendering) says the default also applies to images.\n"),
    ("review", "> We should cache the output.\n\nI would start with the parser state. The document can change between requests, but the allocated scratch space can be reused.\n"),
    ("checklist", "## Before merging\n\n- [x] Add the regression fixture\n- [x] Run the package tests\n- [ ] Check the Windows build\n\nThe remaining issue is tracked in [the release checklist](/releases/next).\n"),
    ("code-example", "The empty case reproduces with:\n\n```rust\nlet input = String::new();\nassert_eq!(render(&input), \"\");\n```\n\nThe nonempty case still produces the expected paragraph.\n"),
    ("unicode", "Überschrift und Fußnoten funktionieren. 日本語の段落も確認しました。\n\nThe café example preserves **naïve** and *résumé* correctly. ✅\n"),
    ("table", "Here are the environments I checked:\n\n| Platform | Result |\n| --- | --- |\n| Linux x64 | Pass |\n| macOS arm64 | Pass |\n| Windows x64 | Pending |\n\nNo configuration overrides were used.\n"),
    ("link-and-image", "The [before/after comparison](https://example.org/review?mode=diff&view=split) shows the spacing change.\n\n![Screenshot of the expected layout](/images/review.png \"Expected layout\")\n\nContact <reviewer@example.org> if the fixture needs updating.\n"),
    ("correction", "~~The problem affects every page.~~ It affects pages with a reference link inside a list item.\n\n1. Open the sample page.\n2. Follow [the reference][issue].\n3. Compare the generated HTML.\n\n[issue]: /issues/reference-links\n"),
    ("draft", "## Proposed rollout\n\nShip the patch to staging first, then compare the output of the documentation build. Keep the previous artifact until the links and navigation have been checked.\n\n> [!NOTE]\n> This is a draft rollout plan; the release checklist remains the source of truth.\n"),
    ("raw-html", "A user pasted this HTML into a comment:\n\n<div onclick=\"alert(1)\">click me</div>\n\nIt should be displayed as source text, not inserted as an interactive element.\n"),
    ("unsafe-link", "An untrusted example: [do not follow](javas&#99;ript:alert%281%29).\n\nThe legitimate [documentation link](https://example.org/docs) should remain usable.\n"),
]


def digest(text):
    return hashlib.sha256(text.encode()).hexdigest()


def main():
    revision = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip()
    provenance = []

    def snapshot(path):
        text = (ROOT / path).read_text()
        tracked = subprocess.check_output(["git", "show", f"{revision}:{path}"], cwd=ROOT).decode()
        if text != tracked:
            raise ValueError(f"Commit the source document first: {path}")
        provenance.append({"id": path, "source_path": path, "revision": revision,
                           "sha256": digest(text), "bytes": len(text.encode()),
                           "license": "MIT OR Apache-2.0", "transformation": "none"})
        return {"id": path, "input": text}

    corpus = {"schema": 1, "previews": [{"id": name, "input": text} for name, text in PREVIEWS],
              "guides": [snapshot(p) for p in GUIDES],
              "documentation": [snapshot(p) for p in DOCUMENTATION],
              "provenance": provenance,
              "preview_provenance": "Authored examples in make_corpus.py, MIT OR Apache-2.0; not production traffic."}
    (HERE / "corpus.json").write_text(json.dumps(corpus, ensure_ascii=False, indent=2) + "\n")
    for group in ("previews", "guides", "documentation"):
        sizes = [len(d["input"].encode()) for d in corpus[group]]
        print(f"{group}: {len(sizes)} documents, {sum(sizes)} bytes, {min(sizes)}–{max(sizes)} bytes/document")


if __name__ == "__main__":
    main()
