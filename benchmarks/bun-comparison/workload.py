"""Shared Markdown workload review; rendering differences stay outside timing."""
from html.parser import HTMLParser
import re

PARSERS = {"ferromark", "bun_md", "pulldown-cmark", "comrak", "md4c"}


def mismatches(outputs):
    if set(outputs) != PARSERS:
        raise ValueError("Every case must include all five parser outputs")
    reference = CanonicalHTML(outputs["ferromark"]).tokens
    return [name for name, html in outputs.items() if CanonicalHTML(html).tokens != reference]


def workload_review(case, outputs, *, parsers=PARSERS):
    """Separate workload eligibility from HTML fidelity; never hide the raw diff."""
    if len(parsers) < 2 or "ferromark" not in parsers or set(outputs) != set(parsers):
        raise ValueError("Every case must include exactly the declared parser outputs, including ferromark")
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
        if token[0] == "start" and (token[1] in ("ul", "ol", "li") or checkbox(token)):
            attrs = dict(token[2])
            if "class" in attrs:
                # Sätteri also labels the enclosing list. Only this reviewed
                # presentation class is ignored there; list structure remains.
                ignored = ("contains-task-list",) if token[1] in ("ul", "ol") else ("task-list-item", "task-list-item-checkbox")
                keep = [c for c in attrs["class"].split() if c not in ignored]
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
