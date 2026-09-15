#!/usr/bin/env python3
"""Prepare the small, pinned current-versus-v2 comparison corpus.

The source checkout is deliberately supplied by the caller.  This keeps the
corpus generator independent of Git and makes the snapshot used for a report
explicit in the resulting JSON.
"""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path


# The comparison snapshot is a source export without a .git directory.  Keep
# the revision supplied with the task in every generated report instead of
# inventing a revision from the export's filesystem metadata.
SOURCE_REVISION = "a6e9906f7b4a01355d336f209fd12534796419df"
LICENSE = "MIT OR Apache-2.0"


def repeat_to_bytes(pattern: str, size: int) -> str:
    """Return an ASCII pattern repeated to exactly *size* bytes."""

    encoded = pattern.encode("utf-8")
    if not encoded or any(byte >= 128 for byte in encoded):
        raise ValueError("generated fixed-size patterns must be non-empty ASCII")
    return (pattern * ((size + len(pattern) - 1) // len(pattern))).encode("utf-8")[
        :size
    ].decode("utf-8")


def authored_cases() -> list[dict[str, str]]:
    """Return deterministic authored inputs used by the primary matrix."""

    short = (
        "# Short document\n\n"
        "A paragraph with *emphasis*, **strong text**, `code`, and "
        "[a link](https://example.com/docs).\n\n"
        "- First item\n- Second item\n"
    )
    links = repeat_to_bytes(
        "[guide](https://example.com/guide) [relative](/docs/start) "
        "<mailto:team@example.com>\n\n",
        10_000,
    )
    lists = repeat_to_bytes(
        "- top item\n  - nested item\n    1. ordered child\n    2. second child\n\n",
        10_000,
    )
    code = repeat_to_bytes(
        "```rust\nfn measured(value: usize) -> usize { value + 1 }\n```\n\n"
        "Use `measured(41)` here.\n\n",
        10_000,
    )
    escaped = repeat_to_bytes(
        "Fish & chips cost < 5 euros; 8 > 3 and &amp; is an entity.\n\n",
        10_000,
    )
    task_autolink = (
        "## GFM controls\n\n"
        "- [x] Parse the document\n"
        "- [ ] Publish the result\n\n"
        "Visit https://example.com/docs and www.example.com.\n"
    )
    references = "".join(
        f"Read [reference {index}][ref-{index}] and [another][ref-{index}].\n\n"
        for index in range(128)
    ) + "".join(
        f"[ref-{index}]: https://example.com/reference/{index}\n"
        for index in range(128)
    )
    return [
        {"name": "tiny", "profile": "commonmark", "input": "Hello, **world**!"},
        {"name": "short-authored-prose", "profile": "commonmark", "input": short},
        {"name": "plain-text-10k", "profile": "commonmark", "input": repeat_to_bytes(
            "Ordinary prose stays deliberately uneventful so fixed parser costs remain visible.\n\n",
            10_000,
        )},
        {"name": "link-heavy", "profile": "commonmark", "input": links},
        {"name": "nested-lists", "profile": "commonmark", "input": lists},
        {"name": "fenced-code", "profile": "commonmark", "input": code},
        {"name": "escape-heavy", "profile": "commonmark", "input": escaped},
        {"name": "gfm-task-autolink", "profile": "gfm", "input": task_autolink},
        {"name": "reference-heavy", "profile": "commonmark", "input": references},
    ]


def fixture_case(source_root: Path, name: str, relative_path: str, profile: str) -> dict[str, str]:
    path = source_root / relative_path
    try:
        content = path.read_text(encoding="utf-8")
    except FileNotFoundError as error:
        raise SystemExit(f"missing required source fixture: {path}") from error
    return {
        "name": name,
        "profile": profile,
        "input": content,
        "origin_path": relative_path,
        "origin_kind": "repository-fixture",
    }


def real_readme(source_root: Path) -> dict[str, str]:
    path = source_root / "README.md"
    try:
        content = path.read_text(encoding="utf-8")
    except FileNotFoundError as error:
        raise SystemExit(f"missing required diagnostic document: {path}") from error
    return {
        "name": "real-readme",
        "profile": "gfm",
        "input": content,
        "origin_path": "README.md",
        "origin_kind": "repository-document",
    }


def build_cases(source_root: Path) -> list[dict[str, str]]:
    cases = authored_cases()
    for name, relative_path, profile in [
        ("commonmark-5k", "benches/fixtures/commonmark-5k.md", "commonmark"),
        ("commonmark-20k", "benches/fixtures/commonmark-20k.md", "commonmark"),
        ("commonmark-50k", "benches/fixtures/commonmark-50k.md", "commonmark"),
        ("tables-plain", "benches/fixtures/tables-plain.md", "gfm"),
        (
            "tables-commonmark-inline",
            "benches/fixtures/tables-commonmark-inline.md",
            "gfm",
        ),
        ("tables-links", "benches/fixtures/tables-links.md", "gfm"),
    ]:
        cases.append(fixture_case(source_root, name, relative_path, profile))
    cases.append(real_readme(source_root))
    return cases


def finalize_case(case: dict[str, str]) -> dict[str, object]:
    input_text = case.pop("input")
    input_bytes = input_text.encode("utf-8")
    result: dict[str, object] = {
        "name": case.pop("name"),
        "profile": case.pop("profile"),
        "input": input_text,
        "origin": {
            "kind": case.pop("origin_kind", "synthetic-authored"),
            "path": case.pop("origin_path", "make_corpus.py"),
            "license": LICENSE,
        },
        "byte_count": len(input_bytes),
        "sha256": hashlib.sha256(input_bytes).hexdigest(),
    }
    if case:
        raise AssertionError(f"unhandled case metadata: {sorted(case)}")
    return result


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source_snapshot", type=Path, help="current Ferromark source export")
    parser.add_argument("output_corpus", type=Path, help="destination JSON corpus file")
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    source_root = args.source_snapshot.expanduser().resolve()
    if not source_root.is_dir():
        raise SystemExit(f"source snapshot is not a directory: {source_root}")
    cases = [finalize_case(case) for case in build_cases(source_root)]
    document = {
        "schema": 1,
        "source_snapshot": str(source_root),
        "source_revision": SOURCE_REVISION,
        "license": LICENSE,
        "case_count": len(cases),
        "cases": cases,
    }
    output_path = args.output_corpus.expanduser().resolve()
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(
        json.dumps(document, ensure_ascii=False, indent=2) + "\n",
        encoding="utf-8",
    )


if __name__ == "__main__":
    main()
