#!/usr/bin/env python3
"""Conservative HTML comparison helpers for the native engine benchmark.

The parser only removes serialization choices that HTML treats as equivalent:
entity spelling, normal-flow whitespace, void-tag slashes, boolean attribute
spelling, table alignment spelling, and UTF-8 URL spelling. It deliberately
keeps text, links, fragments, code whitespace, task state, and all other
attributes significant.
"""

from __future__ import annotations

from html.parser import HTMLParser
import re
from urllib.parse import quote, urlsplit


VOID = {
    "area", "base", "br", "col", "embed", "hr", "img", "input",
    "link", "meta", "param", "source", "track", "wbr",
}
BLOCK = {
    "address", "article", "aside", "blockquote", "div", "dl", "fieldset",
    "figcaption", "figure", "footer", "form", "h1", "h2", "h3", "h4",
    "h5", "h6", "header", "hr", "li", "main", "nav", "ol", "p", "pre",
    "section", "table", "tbody", "td", "tfoot", "th", "thead", "tr", "ul",
}
LITERAL = {"code", "pre", "script", "style", "textarea"}
HEADINGS = {f"h{level}" for level in range(1, 7)}
BOOLEAN = {"checked", "disabled"}


def _unicode_url(value: str) -> str:
    """Normalize UTF-8 URL spelling without decoding reserved characters."""
    try:
        parts = urlsplit(value)
    except ValueError:
        return value
    if parts.scheme not in {"", "http", "https"} or not parts.netloc.isascii():
        return value
    return "".join(quote(char, safe="") if ord(char) > 127 else char for char in value)


def _alignment_style(value: str) -> str:
    """Normalize one CSS text-align declaration for table serialization."""
    return f"text-align:{value.strip().lower()}"


def _attrs(tag: str, attrs: list[tuple[str, str | None]]) -> tuple[tuple[str, object], ...]:
    normalized: dict[str, object] = {}
    for key, value in attrs:
        key = key.lower()
        if key in BOOLEAN:
            value = True
        elif value is not None and key in {"href", "src"}:
            value = _unicode_url(value)
        normalized[key] = value

    if tag in {"td", "th"}:
        align = normalized.get("align")
        style = normalized.get("style")
        if align is not None and style is None:
            # The simple HTML align spelling has a direct CSS equivalent.
            normalized.pop("align")
            normalized["style"] = _alignment_style(str(align))
        elif style is not None:
            declarations = [part.strip() for part in str(style).split(";") if part.strip()]
            if len(declarations) == 1 and ":" in declarations[0]:
                prop, css_value = declarations[0].split(":", 1)
                if prop.strip().lower() == "text-align":
                    normalized["style"] = _alignment_style(css_value)
            # Combined styles stay byte-preserved. In particular, do not let
            # an align attribute silently override an explicit style.

    return tuple(sorted(normalized.items()))


class CanonicalHTML(HTMLParser):
    """Tokenize HTML while retaining all semantic content."""

    def __init__(self, text: str):
        super().__init__(convert_charrefs=True)
        self.tokens: list[tuple] = []
        self._literal_depth = 0
        self.feed(text)
        self.close()
        self.tokens = self._normalize_flow(self.tokens)

    def handle_starttag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        tag = tag.lower()
        self.tokens.append(("start", tag, _attrs(tag, attrs)))
        if tag in LITERAL:
            self._literal_depth += 1

    def handle_startendtag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        # A slash on a non-void HTML element does not close it. Keep only the
        # start token so <div/>x remains distinct from <div></div>x.
        self.handle_starttag(tag, attrs)
        # Void elements have no end token in either spelling.

    def handle_endtag(self, tag: str) -> None:
        tag = tag.lower()
        if tag not in VOID:
            self.tokens.append(("end", tag))
        if tag in LITERAL:
            self._literal_depth = max(0, self._literal_depth - 1)

    def handle_data(self, data: str) -> None:
        if self.tokens and self.tokens[-1][0] == "text":
            previous = self.tokens[-1]
            self.tokens[-1] = ("text", previous[1] + data, previous[2])
        else:
            self.tokens.append(("text", data, self._literal_depth > 0))

    def handle_comment(self, data: str) -> None:
        self.tokens.append(("comment", data))

    def handle_decl(self, data: str) -> None:
        self.tokens.append(("declaration", data))

    def handle_pi(self, data: str) -> None:
        self.tokens.append(("pi", data))

    @staticmethod
    def _normalize_flow(tokens: list[tuple]) -> list[tuple]:
        result: list[tuple] = []
        for index, token in enumerate(tokens):
            if token[0] != "text" or token[2]:
                result.append(token)
                continue
            value = re.sub(r"[ \t\r\n\f]+", " ", token[1])
            before = tokens[index - 1] if index else None
            after = tokens[index + 1] if index + 1 < len(tokens) else None

            def boundary(candidate: tuple | None) -> bool:
                return candidate is None or (candidate[0] in {"start", "end"} and candidate[1] in BLOCK)

            if boundary(before):
                value = value.lstrip(" ")
            if boundary(after):
                value = value.rstrip(" ")
            if value:
                result.append(("text", value, False))
        return result


def canonical(text: str) -> list[tuple]:
    """Return conservative serialization tokens for *text*."""
    return CanonicalHTML(text).tokens


def without_heading_ids(tokens: list[tuple]) -> list[tuple]:
    """Remove heading IDs for diagnostic classification only."""
    result = []
    for token in tokens:
        if token[0] == "start" and token[1] in HEADINGS:
            result.append((token[0], token[1], tuple((key, value) for key, value in token[2] if key != "id")))
        else:
            result.append(token)
    return result


def classify(left: str, right: str) -> str:
    """Classify a pair as exact, serializable, heading-ID-only, or other."""
    if left == right:
        return "exact"
    left_tokens, right_tokens = canonical(left), canonical(right)
    if left_tokens == right_tokens:
        return "serialization-equivalent"
    if without_heading_ids(left_tokens) == without_heading_ids(right_tokens):
        return "heading-id-only"
    return "other"


def classify_group(outputs: dict[str, str]) -> str:
    """Classify all outputs together without selecting an oracle engine."""
    if not outputs:
        raise ValueError("cannot classify an empty output group")
    values = list(outputs.values())
    if all(value == values[0] for value in values[1:]):
        return "exact"
    streams = [canonical(value) for value in values]
    if all(stream == streams[0] for stream in streams[1:]):
        return "serialization-equivalent"
    stripped = [without_heading_ids(stream) for stream in streams]
    if all(stream == stripped[0] for stream in stripped[1:]):
        return "heading-id-only"
    return "other"


def admitted(status: str) -> bool:
    """Whether a status is safe for strict throughput aggregation."""
    return status in {"exact", "serialization-equivalent"}


def groups(outputs: dict[str, str]) -> list[list[str]]:
    """Group every engine by canonical output without choosing an oracle."""
    if not outputs:
        raise ValueError("cannot group an empty output set")
    result: dict[tuple, list[str]] = {}
    for name in sorted(outputs):
        result.setdefault(tuple(canonical(outputs[name])), []).append(name)
    return list(result.values())
