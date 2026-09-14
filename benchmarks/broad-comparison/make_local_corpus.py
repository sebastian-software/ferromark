#!/usr/bin/env python3
"""Select unchanged documents from the earlier frozen corpus, without timings."""

import argparse
import gzip
import hashlib
import json
from pathlib import Path
import re
import shutil
from urllib.parse import quote

ROOT = Path(__file__).resolve().parent
SELECTION = {
    "rust-book": [
        "2018-edition/src/ch03-04-comments.md", "src/appendix-02-operators.md",
        "src/ch00-00-introduction.md", "src/ch17-00-async-await.md",
    ],
    "vue-docs": [
        "src/guide/extras/ways-of-using-vue.md", "src/guide/built-ins/suspense.md",
        "src/guide/components/slots.md", "src/guide/extras/reactivity-in-depth.md",
    ],
    "vite-docs": [
        "docs/guide/philosophy.md", "docs/guide/performance.md",
        "docs/guide/api-plugin.md", "docs/guide/features.md",
    ],
    "typescript-handbook": [
        "packages/documentation/copy/en/handbook-v2/The Handbook.md",
        "packages/documentation/copy/en/reference/Advanced Types.md",
        "packages/documentation/copy/en/project-config/Compiler Options.md",
        "packages/documentation/copy/en/release-notes/TypeScript 5.0.md",
    ],
}
LICENSES = {"rust-book": "MIT OR Apache-2.0", "vue-docs": "CC-BY-4.0",
            "vite-docs": "MIT", "typescript-handbook": "CC-BY-4.0"}
REFERENCE_NAMES = {"appendix-02-operators.md", "api-plugin.md", "Advanced Types.md", "Compiler Options.md"}


def slug(value):
    return re.sub(r"[^a-z0-9]+", "-", value.lower()).strip("-")


def checked(text, record):
    raw = text.encode("utf-8")
    assert len(raw) == record["bytes"]
    assert hashlib.sha256(raw).hexdigest() == record["sha256"]
    return text


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("legacy", type=Path, help="Earlier workflow corpus.json")
    parser.add_argument("external", type=Path, help="OX core-port cases.json")
    parser.add_argument("manifest", type=Path, help="Earlier corpus-manifest.json")
    parser.add_argument("licenses", type=Path, help="Earlier archived licenses directory")
    args = parser.parse_args()
    legacy = json.loads(args.legacy.read_text(encoding="utf-8"))
    records = {r["id"]: r for r in legacy["provenance"]}
    cases = []
    for item in legacy["documentation"]:
        record = records[item["id"]]
        path = record["source_path"]
        cases.append({
            "name": "legacy-" + slug(path.removesuffix(".md")), "profile": "gfm",
            "input": checked(item["input"], record),
            "category": "readme" if Path(path).name == "README.md" else "technical-docs",
            "collection": "legacy-docs",
            "origin": {"kind": "repository-document", "repository": "https://github.com/sebastian-software/ferromark",
                       "url": f"https://github.com/sebastian-software/ferromark/blob/{record['revision']}/{quote(path)}",
                       "revision": record["revision"], "path": path,
                       "license": record["license"], "transformations": []},
        })
    assert len(cases) == 12
    external = {c["case"]: c for c in json.loads(args.external.read_text(encoding="utf-8"))}
    manifests = {m["project"]: m for m in json.loads(args.manifest.read_text(encoding="utf-8"))}
    for project, paths in SELECTION.items():
        manifest = manifests[project]
        files = {f["path"]: f for f in manifest["files"]}
        repository = manifest["repository"].removesuffix(".git")
        for path in paths:
            item = external[f"corpus/{project}/{path}/matched"]
            cases.append({
                "name": project + "-" + slug(Path(path).stem), "profile": "gfm",
                "input": checked(item["input"], files[path]),
                "category": "reference" if Path(path).name in REFERENCE_NAMES else "technical-docs",
                "collection": "external-docs",
                "origin": {"kind": "repository-document", "repository": repository,
                           "url": f"{repository}/blob/{manifest['revision']}/{quote(path)}",
                           "revision": manifest["revision"], "path": path,
                           "license": LICENSES[project], "transformations": [],
                           "notes": "Original framework/site directives, frontmatter, and raw HTML are retained. Timed as GFM text, without site preprocessing or framework execution."},
            })
    payload = {"schema": 1, "selection": "All 12 legacy documents plus four predeclared documents from each of four external sources. Selected for size, prose, references, tables, lists, and code before timing. No edits, padding, repeats, or output-based selection.",
               "upstream_manifest_sha256": hashlib.sha256(args.manifest.read_bytes()).hexdigest(), "cases": cases}
    (ROOT / "local-cases.json.gz").write_bytes(gzip.compress(json.dumps(payload, indent=2, ensure_ascii=False).encode("utf-8"), mtime=0))
    destination = ROOT / "licenses"
    destination.mkdir(exist_ok=True)
    for filename in ("vue-docs-LICENSE", "vite-docs-LICENSE", "rust-book-LICENSE-MIT", "rust-book-LICENSE-APACHE", "typescript-handbook-LICENSE"):
        shutil.copyfile(args.licenses / filename, destination / filename)
    shutil.copyfile(ROOT.parent / "current-comparison/LICENSE-FERROMARK-MIT", destination / "ferromark-LICENSE-MIT")
    (destination / "ATTRIBUTION.md").write_text(
        "# Source attribution\n\nOriginal document paths, revisions, hashes, and source links are in\n"
        "`../local-cases.json.gz` and in the report's frozen corpus manifest.\n\n"
        "- Vue documentation: copyright 2019-present Yuxi (Evan) You and Vue\n"
        "  documentation contributors. CC BY 4.0; see `vue-docs-LICENSE`.\n"
        "- Vite documentation: copyright 2019-present VoidZero Inc. and Vite\n"
        "  contributors. MIT; see `vite-docs-LICENSE`.\n"
        "- The Rust Programming Language: The Rust Project Developers. MIT or\n"
        "  Apache-2.0; see `rust-book-LICENSE-MIT` and `rust-book-LICENSE-APACHE`.\n"
        "- TypeScript documentation: Microsoft and TypeScript-Website contributors.\n"
        "  CC BY 4.0; see `typescript-handbook-LICENSE`.\n"
        "- Ferromark documentation: original project authors, MIT or Apache-2.0;\n"
        "  see `ferromark-LICENSE-MIT`.\n\n"
        "All selected Markdown documents are unchanged. Archived HTML outputs are\n"
        "mechanical transformations by the named renderers. No image files are\n"
        "redistributed. Nothing implies endorsement by the source projects.\n",
        encoding="utf-8")
    (destination / "LOCAL-CORPUS.md").write_text(
        "# Local corpus attribution\n\nThe unchanged inputs retain their original authorship and licenses.\n"
        "The Vue and TypeScript documentation snapshots are CC BY 4.0; see the\n"
        "copied license texts and original project attribution. Vite uses MIT;\n"
        "Rust Book and Ferromark offer MIT or Apache-2.0. The MIT license is\n"
        "included for Ferromark. Each case links to its exact upstream revision.\n\n"
        "This benchmark selects only the 16 external documents named in\n"
        "make_local_corpus.py, plus 12 Ferromark documents. No text was changed.\n",
        encoding="utf-8")
    print([(c["name"], len(c["input"].encode("utf-8"))) for c in cases])


if __name__ == "__main__":
    main()
