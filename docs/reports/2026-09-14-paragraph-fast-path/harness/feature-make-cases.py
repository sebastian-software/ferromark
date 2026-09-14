#!/usr/bin/env python3
"""Freeze a focused feature-scan corpus and its runtime JSON profiles."""

import argparse
import copy
import hashlib
import importlib.util
import json
from pathlib import Path
import re


HERE = Path(__file__).resolve().parent
RUNTIME_CASES = HERE.parent / "runtime-profiles" / "make_cases.py"


def load_runtime_cases():
    spec = importlib.util.spec_from_file_location("runtime_profile_cases", RUNTIME_CASES)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module.make_cases()


def config_key(config):
    encoded = json.dumps(config, sort_keys=True, separators=(",", ":"))
    return hashlib.sha256(encoded.encode()).hexdigest()[:12]


PROSE = (
    "A small document can describe an observation and give the reader enough "
    "context. The next paragraph explains the result in ordinary words without "
    "special notation.\n\n"
)


def scaled(source, target):
    return source * max(1, (target + len(source) - 1) // len(source))


def add_case(cases, name, source, category, config, **metadata):
    raw = source.encode("utf-8")
    cases.append({
        "name": name,
        "category": category,
        "input": source,
        "profile": None,
        "runtime_options": copy.deepcopy(config),
        "byte_count": len(raw),
        "sha256": hashlib.sha256(raw).hexdigest(),
        "origin": {"kind": "authored-synthetic-probe", "license": "MIT"},
        **metadata,
    })


def diagnostics():
    def cfg(parser=None, renderer=None):
        return {"parser": parser or {}, "renderer": renderer or {}}

    def defs(**parser):
        return cfg({"definition_lists": True, **parser})

    def comments(**parser):
        return cfg({"line_comments": True, **parser})

    return [
        ("definition-near-miss-punctuation", "Term\n:: not a definition\n\n", defs()),
        ("definition-near-miss-indented", "Term\n : one space before marker\n\n", defs()),
        ("definition-near-miss-blank", "Term\n\n: orphan marker\n\n", defs()),
        ("definition-late-after-prose", "Introductory paragraph.\n\n" * 8 + "Term\n: a late definition\n", defs()),
        ("definition-late-after-list", "- first\n- second\n\n" * 8 + "Term\n: a late definition\n", defs()),
        ("definition-nested-list", "- outer\n  - inner\n\n  Term\n  : nested definition\n", defs()),
        ("definition-nested-quote", "> Term\n> : quoted definition\n\n", defs()),
        ("definition-fenced-lookalike", "```text\nTerm\n: literal text\n```\n\nTerm\n: real definition\n", defs()),
        ("comment-inline-lookalike", "Text // this is ordinary text\nSecond line.\n", comments()),
        ("comment-indented", "Paragraph.\n  // indented source note\nAfter note.\n", comments()),
        ("comment-fenced", "```text\n// code, not a source comment\n```\n\nAfter.\n", comments()),
        ("comment-quote", "> // quoted source note\n> visible quote\n", comments()),
        ("comment-url-path", "Visit https://example.org/a//b and continue.\n", comments(autolinks=True)),
        ("comment-url-query", "[link](https://example.org/a?next=//b) and text.\n", comments()),
        ("comment-email", "Contact person@example.org // trailing note\n", comments(autolinks=True)),
        ("definition-crlf", "Term\r\n: CRLF definition\r\n\r\nBody\r\n", defs()),
        ("definition-bom", "\ufeffTerm\n: definition after BOM\n\n", defs()),
        ("definition-span-sensitive", "# Heading\n\nTerm\n: definition\n\n- item\n", cfg({"definition_lists": True}, {"source_spans": True})),
        ("comment-span-sensitive", "# Heading\r\n\r\n// note\r\nBody\r\n", cfg({"line_comments": True}, {"source_spans": True})),
    ]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("output", type=Path, help="new directory receiving corpus.json and configs")
    args = parser.parse_args()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    config_dir = output / "configs"
    config_dir.mkdir()

    runtime = load_runtime_cases()
    cases = []
    profiles = {}

    def profile(config, label):
        config = copy.deepcopy(config)
        key = config_key(config)
        path = config_dir / f"{label}-{key}.json"
        if key not in profiles:
            path.write_text(json.dumps(config, indent=2, sort_keys=True) + "\n")
            profiles[key] = path.resolve()
        return config, str(profiles[key])

    # Keep the prior study's exact plain/active inputs and sizes for the two
    # scanner features under investigation.  Only the enabled configuration is
    # timed here; the optimization runner supplies the same profile to both
    # frozen workers.
    wanted = {"parser.definition_lists", "parser.line_comments"}
    for source_case in runtime["cases"]:
        if source_case.get("feature") not in wanted or source_case.get("workload") not in {"plain", "active"}:
            continue
        config, path = profile(source_case["on"], source_case["feature"].replace("parser.", ""))
        add_case(
            cases,
            f"feature--{source_case['name']}",
            source_case["input"],
            f"feature-{source_case['feature'].split('.', 1)[1]}",
            config,
            profile=path,
            feature=source_case["feature"],
            workload=source_case["workload"],
            target_bytes=source_case["target_bytes"],
            origin=copy.deepcopy(source_case["origin"]),
        )

    # The runtime study's 13 mixed documents are replayed with the docs
    # profile plus definition lists, alongside exact CommonMark controls.
    docs_cases = [
        case for case in runtime["cases"]
        if case.get("group") == "profile" and case.get("feature") == "docs"
    ]
    assert len(docs_cases) == 13, len(docs_cases)
    for source_case in docs_cases:
        docs_config = copy.deepcopy(source_case["on"])
        docs_config["parser"]["definition_lists"] = True
        control_config = copy.deepcopy(source_case["off"])
        for label, config, category in (
            ("deflists", docs_config, "mixed-docs-definition-lists"),
            ("control", control_config, "mixed-docs-control"),
        ):
            config, path = profile(config, f"docs-{label}")
            document = source_case["document"]
            safe_document = re.sub(r"[^A-Za-z0-9_.-]+", "-", document)
            add_case(
                cases,
                f"mixed--{label}--{safe_document}",
                source_case["input"],
                category,
                config,
                profile=path,
                document=document,
                feature="docs+definition_lists" if label == "deflists" else "commonmark",
                workload=source_case["workload"],
                origin=copy.deepcopy(source_case["origin"]),
            )

    for name, source, config in diagnostics():
        config, path = profile(config, name)
        add_case(cases, f"diagnostic--{name}", source, "diagnostic", config,
                 profile=path, feature="definition_lists" if "definition" in name else "line_comments")

    # Long controls distinguish a late valid marker from a scan that can stop
    # after the opening prose.  The URL/colon and slash-heavy controls keep
    # punctuation common while leaving the relevant extension mostly absent.
    for target in (4096, 65536):
        source = scaled(PROSE, target) + "Late term\n: a valid late definition\n"
        config, path = profile({"parser": {"definition_lists": True}, "renderer": {}}, "late-definition")
        add_case(cases, f"diagnostic--late-definition-prose-{target}", source,
                 "diagnostic-late-definition", config, profile=path,
                 feature="definition_lists", target_bytes=target)

        url_defs = scaled(
            "Term https://example.org/a:b?q=1#part\n: definition with inline: colon\n\n",
            target,
        )
        config, path = profile({"parser": {"definition_lists": True}, "renderer": {}}, "url-colon-definition")
        add_case(cases, f"diagnostic--url-colon-definitions-{target}", url_defs,
                 "diagnostic-url-colon", config, profile=path,
                 feature="definition_lists", target_bytes=target)

        for label, source in [
            ("indented-definition-body", "Term\n: first body\n" +
             scaled("    Indented body text and more words.\n", target) + "\nNext\n: last body\n"),
            ("multiline-definition-terms", scaled("An ordinary term with words\n", target) + ": body\n"),
            ("inline-colons-without-definitions", scaled(
                "A URL https://example.org/a:b and time 12:34. Words: still ordinary prose.\n\n", target)),
        ]:
            add_case(cases, f"diagnostic--{label}-{target}", source,
                     f"diagnostic-{label}", config, profile=path,
                     feature="definition_lists", target_bytes=target)

        slash_paths = scaled(
            "path /a//b///c/d/e/f and https://example.org/a//b\n",
            target,
        )
        config, path = profile({"parser": {"line_comments": True}, "renderer": {}}, "slash-paths")
        add_case(cases, f"diagnostic--slash-paths-without-comments-{target}", slash_paths,
                 "diagnostic-slash-paths", config, profile=path,
                 feature="line_comments", target_bytes=target)

    # Plain prose with each scanner explicitly disabled is a same-shape
    # baseline control at all three sizes.
    plain_by_feature = {
        "definition_lists": "parser.definition_lists",
        "line_comments": "parser.line_comments",
    }
    for feature, runtime_feature in plain_by_feature.items():
        for source_case in runtime["cases"]:
            if (source_case.get("feature") != runtime_feature or
                    source_case.get("workload") != "plain"):
                continue
            config, path = profile({"parser": {feature: False}, "renderer": {}}, "feature-off")
            add_case(cases, f"control--{feature}-plain-{source_case['target_bytes']}",
                     source_case["input"], "feature-off-plain", config,
                     profile=path, feature=runtime_feature, workload="plain",
                     target_bytes=source_case["target_bytes"])

    assert 50 <= len(cases) <= 80, len(cases)
    data = {
        "schema": 1,
        "description": "Focused definition-list and line-comment feature-scan corpus.",
        "cases": cases,
    }
    (output / "corpus.json").write_text(json.dumps(data, indent=2, ensure_ascii=False) + "\n")
    print(f"{len(cases)} cases, {len(profiles)} runtime profiles")


if __name__ == "__main__":
    main()
