#!/usr/bin/env python3
"""Convert frozen Wikipedia article HTML into prose Markdown (requires lxml)."""

import argparse
import gzip
import hashlib
import json
from pathlib import Path
import re
from urllib.parse import urljoin

from lxml import html
import lxml.etree

ROOT = Path(__file__).resolve().parent
TITLES = ("Rainbow", "Tea", "Chess", "Volcano")
STOP = {"See also", "Notes", "References", "Citations", "Further reading", "External links", "Footnotes", "Bibliography"}
REMOVE_CLASSES = {"infobox", "navbox", "sidebar", "vertical-navbox", "reflist", "reference",
                  "mw-editsection", "hatnote", "shortdescription", "metadata", "noprint",
                  "thumb", "sistersitebox", "authority-control", "toc", "mwe-math-element"}


def escape(text):
    return re.sub(r"([\\`*_\[\]<>])", r"\\\1", text or "")


def inline(node, base, markup=True):
    result = escape(node.text)
    for child in node:
        if not isinstance(child.tag, str):
            result += escape(child.tail)
            continue
        text = inline(child, base, markup)
        if markup and child.tag == "a" and child.get("href") and text.strip():
            url = urljoin(base, child.get("href")).replace(" ", "%20").replace("(", "%28").replace(")", "%29").replace(">", "%3E")
            text = f"[{text}]({url})"
        elif markup and child.tag in {"b", "strong"} and text.strip():
            text = "**" + text.strip() + "**"
        elif markup and child.tag in {"i", "em"} and text.strip():
            text = "*" + text.strip() + "*"
        elif child.tag == "br":
            text = " "
        elif child.tag in {"img", "math", "script", "style"}:
            text = ""
        result += text + escape(child.tail)
    return re.sub(r"\s+", " ", result).strip() if node.tag in {"p", "li", "h1", "h2", "h3", "h4", "h5", "h6"} else result


def listing(node, base, level=0, markup=True):
    lines = []
    for index, item in enumerate(node.findall("li"), 1):
        nested = [child for child in item if child.tag in {"ul", "ol"}]
        for child in nested:
            item.remove(child)
        marker = f"{index}." if node.tag == "ol" else "-"
        lines.append("    " * level + marker + " " + inline(item, base, markup))
        for child in nested:
            lines.append(listing(child, base, level + 1, markup))
    return "\n".join(lines)


def blocks(node, base, markup=True):
    for child in node:
        if not isinstance(child.tag, str):
            continue
        if re.fullmatch(r"h[1-6]", child.tag):
            yield ("heading", int(child.tag[1]), inline(child, base, markup))
        elif child.tag == "p":
            text = inline(child, base, markup)
            if text:
                yield ("paragraph", 0, text)
        elif child.tag in {"ul", "ol"}:
            text = listing(child, base, markup=markup)
            if text:
                yield ("list", 0, text)
        elif child.tag == "blockquote":
            text = inline(child, base, markup).strip()
            if text:
                yield ("quote", 0, "> " + text)
        else:
            yield from blocks(child, base, markup)


def convert(raw, title, markup=True):
    doc = html.fromstring(raw)
    roots = doc.xpath('//div[contains(concat(" ", normalize-space(@class), " "), " mw-parser-output ")]')
    assert roots, title
    root = max(roots, key=lambda node: len(node.xpath('.//p')))
    for node in list(root.iterdescendants()):
        if not isinstance(node.tag, str) or node.getparent() is None:
            continue
        classes = set(node.get("class", "").split())
        if node.tag in {"table", "figure", "script", "style", "link", "meta", "math", "img", "pre", "code"} or classes & REMOVE_CLASSES:
            node.drop_tree()
    selected, lead, first = [], [], None
    in_lead = True
    for kind, level, text in blocks(root, f"https://en.wikipedia.org/wiki/{title}", markup):
        plain = re.sub(r"\\(.)", r"\1", text).strip()
        if kind == "heading" and level == 2 and plain in STOP:
            break
        if kind == "heading":
            in_lead = False
            text = ("#" * level + " " if markup else "") + text
        elif kind == "paragraph" and first is None:
            first = text
        selected.append(text)
        if in_lead:
            lead.append(text)
    assert first and lead and selected
    def document(parts):
        return ("# " if markup else "") + f"{title}\n\n" + "\n\n".join(parts) + "\n"
    return {"first-paragraph": document([first]), "lead": document(lead), "article-body": document(selected)}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--import-dir", type=Path, help="Import ferromark-TITLE.html downloads instead of existing frozen sources")
    args = parser.parse_args()
    sources = ROOT / "wiki-sources"
    sources.mkdir(exist_ok=True)
    cases, articles = [], []
    for title in TITLES:
        slug = title.lower()
        raw = (args.import_dir / f"ferromark-{slug}.html").read_bytes() if args.import_dir else gzip.decompress((sources / f"{slug}.html.gz").read_bytes())
        source = raw.decode("utf-8")
        revision = re.search(r'"wgRevisionId":(\d+)', source).group(1)
        assert "creativecommons.org/licenses/by-sa/4.0" in source
        if args.import_dir:
            (sources / f"{slug}.html.gz").write_bytes(gzip.compress(raw, mtime=0))
        origin = {
            "kind": "encyclopedia", "title": title, "authors": "Wikipedia contributors",
            "url": f"https://en.wikipedia.org/w/index.php?title={title}&oldid={revision}",
            "history_url": f"https://en.wikipedia.org/w/index.php?title={title}&action=history",
            "revision": revision, "license": "CC-BY-SA-4.0",
            "license_url": "https://creativecommons.org/licenses/by-sa/4.0/",
            "source_html_sha256": hashlib.sha256(raw).hexdigest(),
            "source_snapshot": f"wiki-sources/{slug}.html.gz",
            "transformations": [
                "Select main mw-parser-output article body; stop before the first level-2 appendix (See also, Notes, References, etc.).",
                "Remove navigation, infoboxes/tables, images/figures, citation markers, edits, scripts/styles, math markup, and any code/pre elements. No image files redistributed.",
                "Convert paragraph/heading/list/blockquote structure and emphasis/links to Markdown; collapse HTML whitespace; escape Markdown punctuation; resolve hrefs to absolute URLs.",
                "Create three explicitly correlated views: first paragraph, complete lead, and article body. Prepend the article title. No repetition or byte padding.",
                "Also retain one full prose view per topic: link labels without destinations, no emphasis markers, and headings as plain paragraphs; lists/quotes remain structured. All linked originals stay in the corpus. This separates ordinary prose from link-dense encyclopedia markup.",
            ],
        }
        articles.append(origin)
        variants = convert(raw, title)
        variants["plain-prose"] = convert(raw, title, markup=False)["article-body"]
        for variant, text in variants.items():
            assert "```" not in text and "~~~" not in text
            cases.append({"name": f"wiki-{slug}-{variant}", "input": text, "profile": "commonmark",
                          "category": "plain-prose" if variant == "plain-prose" else "encyclopedia",
                          "collection": "encyclopedia-prose" if variant == "plain-prose" else "encyclopedia",
                          "origin": {**origin, "excerpt": variant}})
    payload = {"schema": 1, "converter": "make_wiki_corpus.py", "lxml_version": lxml.etree.LXML_VERSION,
               "conversion_validation": "Before timing, the first converter draft used angle-delimited link destinations. Main duplicated those links in its output. The final converter uses ordinary parenthesized destinations with parentheses percent-encoded; the minimized angle-link case is retained separately by make_corpus.py. Document/topic/excerpt selection did not change.",
               "selection": "Four fixed, non-programming topics × first paragraph, lead, body, and full prose adaptation. Size diversity through real article structure, never repetition or timing selection.", "cases": cases}
    (ROOT / "wiki-cases.json.gz").write_bytes(gzip.compress(json.dumps(payload, indent=2, ensure_ascii=False).encode("utf-8"), mtime=0))
    lines = ["# Wikipedia corpus attribution", "", "Copyright Wikipedia contributors. The article text, derived Markdown, and rendered HTML derivatives remain under [CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/). This does not change the parser or harness license.", "", "These are mechanical Markdown conversions of Wikipedia HTML, not native Markdown articles. The manifest records every transformation. Tables, images, reference apparatus, and math markup are excluded; article prose, headings, emphasis, lists, and links are retained. No Wikipedia endorsement is implied.", "", "Raw downloaded HTML snapshots are retained compressed for conversion review. No linked image files are redistributed. Each first-paragraph/lead/body family overlaps and must not be treated as three independent articles.", ""]
    for article in articles:
        lines.append(f"- [{article['title']}, revision {article['revision']}]({article['url']}) — [contributors/history]({article['history_url']}); source SHA-256 `{article['source_html_sha256']}`.")
    lines += ["", "Reproduce the conversion with Python and lxml (the snapshot records the lxml version):", "", "```sh", "python3 benchmarks/broad-comparison/make_wiki_corpus.py", "```", "", "To acquire new sources, download each named article HTML to `ferromark-title.html` and pass `--import-dir DIR`. That creates a new corpus revision; do not overwrite an archived benchmark's inputs.", ""]
    (ROOT / "WIKIPEDIA-ATTRIBUTION.md").write_text("\n".join(lines), encoding="utf-8")
    print([(c["name"], len(c["input"].encode("utf-8"))) for c in cases])


if __name__ == "__main__":
    main()
