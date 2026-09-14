#!/usr/bin/env python3
"""Deterministic differential checks for definition/comment scanner boundaries."""

import argparse
import gzip
import hashlib
import importlib.util
import json
from pathlib import Path
import random


RUNNER = Path(__file__).resolve().parents[1] / "optimization-rounds" / "run.py"
spec = importlib.util.spec_from_file_location("optimization_runner", RUNNER)
runner = importlib.util.module_from_spec(spec)
spec.loader.exec_module(runner)


def sources():
    rng = random.Random(20260914)
    fragments = [
        "Ordinary **strong** prose: with a colon.\n\n",
        "Term\n: **definition**\n\n",
        "First term\nSecond term\n\n : first body\n : second body\n\n",
        "Term\n: body\n\n    - nested item\n      continued\n\n",
        "Term\n: body\n  // hidden\n    **continued**\n\n",
        "> Term\n> : body\n>\n>     - item\n\n",
        "- Term\n  : body\n\n",
        "- first\n  // physical comment\n  **second**\n\n",
        "> first\n// physical comment\n> **second**\n\n",
        "first\n// hidden\n**second**\n\n",
        "Heading\n// hidden\n**second**\n---\n\n",
        "Heading\n// trailing\n---\n\n",
        "first\n// trailing\n# Heading\n\n",
        "https://example.org/a//b\nWords // inline\n\n",
        "Term\n:: invalid\n\nTerm\n    : indented code\n\n",
        "Term\n:\n\n",
        "```text\nTerm\n: code\n// code\n```\n\n",
        "<div>\nTerm\n: html\n// html\n</div>\n\n",
        "[label]:\n// hidden\n/url \"title\"\n\n[label]\n\n",
        "[label]: /url\n\nText [label]\n// hidden\n**after**\n\n",
        "Note[^n]\n\n[^n]: first\n  // hidden\n    **second**\n\n",
        "A | B\n// hidden\n- | -\n1 | 2\n// tail\n\n",
        "    // code\n\t: code\n\n",
        "Term\n\t: invalid marker\n\n",
        "# Heading {#id}\n\n",
        "`unclosed\nTerm\n: definition candidate\n\n",
        "Ordinary `unclosed\n: definition candidate\n\n",
        "Ordinary `closed` term\n: body\n\n",
        "\u00a0***\n: candidate after Unicode whitespace\n\n",
        "\u00e4 term\n: body\n\n",
        "Term\n// note\nSecond **term**\n: body\n\n",
        "Term\n: first\n// note\n: second\n\n",
        "Term\n: first\n    // literal\n\n",
    ]
    documents = list(fragments)
    for _ in range(400 - len(fragments)):
        source = "".join(rng.choices(fragments, k=rng.randint(1, 8)))
        if rng.randrange(3) == 0:
            source = source.rstrip("\n")
        ending = rng.choice(["\n", "\r\n", "\r"])
        source = source.replace("\n", ending)
        if rng.randrange(5) == 0:
            source = "\ufeff" + source
        documents.append(source)
    return documents


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("build", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=False)
    build = json.loads((args.build / "build.json").read_text())
    for engine in runner.ENGINES:
        entry = build["engines"][engine]
        assert runner.digest(Path(entry["binary"])) == entry["binary_sha256"]
    documents = sources()
    paths = []
    for index, source in enumerate(documents):
        path = (args.output / f"{index:04}.md").resolve()
        path.write_text(source, newline="")
        paths.append(path)
    configs = []
    for definitions, comments in [(True, False), (False, True), (True, True)]:
        configs.append({"parser": {
            "definition_lists": definitions, "line_comments": comments,
            "footnotes": True, "tables": True, "autolinks": True,
            "heading_attributes": True,
        }, "renderer": {"source_spans": True, "heading_ids": True}})
    digest = hashlib.sha256()
    comparisons = 0
    for config_index, config in enumerate(configs):
        config_path = (args.output / f"config-{config_index}.json").resolve()
        config_path.write_text(json.dumps(config))
        for offset in range(0, len(paths), 40):
            batch = paths[offset:offset + 40]
            results = []
            for engine in runner.ENGINES:
                worker = runner.Worker(build["engines"][engine]["binary"],
                                       str(config_path), "reuse", batch)
                try:
                    results.append(worker.verify())
                finally:
                    worker.close()
            assert len(results[0]) == len(results[1]) == len(batch)
            for index, (before, after) in enumerate(zip(*results)):
                for key in ("html", "ast_debug", "children"):
                    if before[key] != after[key]:
                        (args.output / "failure.json").write_text(json.dumps({
                            "config": config, "source": documents[offset + index],
                            "baseline": before, "candidate": after,
                        }, indent=2, ensure_ascii=False))
                        raise AssertionError((config_index, offset + index, key))
                    digest.update(json.dumps(before[key], ensure_ascii=False).encode())
                comparisons += 1
    corpus = json.dumps({"inputs": documents, "configs": configs}, ensure_ascii=False).encode()
    (args.output / "corpus.json.gz").write_bytes(gzip.compress(corpus, mtime=0))
    result = {"status": "exact HTML, AST Debug (including spans), and children",
              "inputs": len(documents), "configurations": len(configs),
              "comparisons": comparisons, "seed": 20260914,
              "corpus_sha256": hashlib.sha256(corpus).hexdigest(),
              "output_sha256": digest.hexdigest(),
              "generator_sha256": runner.digest(Path(__file__)),
              "engines": build["engines"]}
    (args.output / "result.json").write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps({k: v for k, v in result.items() if k != "engines"}, indent=2))


if __name__ == "__main__":
    main()
