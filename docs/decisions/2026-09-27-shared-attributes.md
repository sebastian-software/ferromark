# Shared attributes and bracketed spans

- Status: Accepted
- Date: 2026-09-27
- Related: #466, #467, #468

## Context

Heading, table, image, and figure ID/class suffixes already exist. Authors need
ordinary HTML metadata and custom application data on those elements, links,
and inline text, while retaining native AST structure and safe HTML escaping.

## Decision

Two new parser options are off by default in Rust and Node. `extended_attributes`
(`extendedAttributes`) enables the full grammar on eligible heading, table,
image, figure, and link attachment points. `bracketed_spans` (`bracketedSpans`)
enables `[inline Markdown]{attributes}` independently, with the full grammar.
Existing `table_attributes` still gates the table metadata line, and
`image_captions` still gates a figure caption. The extended option alone
enables image and heading suffixes; the existing `image_attributes` and
`heading_attributes` options retain their ID/class-only behavior when the
extended option is off. Links accept suffixes only with the extended option.

An attribute block must contain at least one token. `#id` sets the ID, `.class`
appends a class, and `name=value` sets a key/value attribute. A class always
needs its dot shorthand or `class="one two"`. Bare words are invalid. Names
are ASCII letters followed by ASCII letters, digits, `-`, `_`, or `:`; names
are normalized to lowercase. Values may be unquoted without whitespace, or
single/double quoted. Quoted values may contain spaces; a backslash may escape
the matching quote or another backslash. Empty quoted values are valid, so
`hidden=""` is supported. A malformed block stays literal in full. `id=`
and `class=` merge into the same AST ID/class fields as shorthand tokens.
Classes accumulate in source order; the last explicit ID wins, and the last
value for a repeated key wins. Unlike the older ID/class-only extension,
repeated IDs are valid when the full grammar is enabled.
Inline suffix recognition reads at most 4,096 bytes after the opening brace;
longer blocks remain literal. This bound prevents repeated malformed openers
from repeatedly scanning a long paragraph.

The AST stores original custom keys without an HTML-specific prefix.
`Attribute { name, value }` entries live beside ID and class fields on links,
images, headings, tables, figures, and bracketed spans. The HTML renderer keeps
recognized standard names unchanged regardless of element type, as well as
`aria-*` and explicit `data-*`; all other valid names receive `data-`.
Recognition uses the fixed list in `renderer/html/renderer/attributes.rs`.
When both `sku` and `data-sku` are authored, explicit `data-sku` wins regardless
of source order. The renderer emits one attribute per normalized key and
escapes every value. This mapping does not change the AST or invoke typography
by `lang`.

Markdown-owned values take precedence: link `href`, image `src` and `alt`, and
quoted image/link titles cannot be replaced by metadata. Explicit IDs/classes
are emitted from their normalized AST fields, once. On an external link for
which the renderer generates `target="_blank"` and its `rel`, those generated
values take precedence; other links may author `target` and `rel`. HTML output
policy and URL handling otherwise remain unchanged.

The link parser resolves inline and reference links before considering a span.
Thus `[text](url){...}` is an attributed link, and `[text]{...}` is a span when
the bracketed-span option is enabled and no reference resolves. A span can
contain links and other ordinary inline nodes. Code, math, raw HTML, and MDX
are not reparsed for attributes. Visitors, source spans, hooks, transforms,
plain-text extraction, and outline ID planning traverse the native span node.

## Compatibility

Both new options are disabled in all presets, preserving CommonMark and GFM
output. Enabling `extended_attributes` intentionally replaces the older
permissive heading suffix parser with the strict shared grammar; malformed or
mixed blocks stay literal instead of silently dropping tokens. With the new
option off, the earlier heading behavior is unchanged. The ID/class-only
image and table grammar from #466 likewise stays unchanged when the new option
is off. Generated `<col>` classes are unaffected.

## Performance

Disabled paths check option flags before suffix scanning or span construction.
The `shared_attributes` Criterion target measures ordinary prose, valid
attributes, and repeated malformed blocks with both options on and off. On an
Apple Silicon local release build, median parse times were:

| Input | Both off | Attributes only | Spans only | Both on |
| --- | ---: | ---: | ---: | ---: |
| Ordinary prose (4.6 KB) | 12.63 µs | 12.51 µs | 12.33 µs | 12.40 µs |
| Valid annotations (5.6 KB) | 15.90 µs | 20.35 µs | 22.54 µs | 26.83 µs |
| Repeated malformed blocks (3.4 KB) | 10.07 µs | 10.01 µs | 14.24 µs | 14.30 µs |
| Distant closing brace after malformed spans (7.2 KB) | 12.53 µs | 12.60 µs | 1.57 ms | 1.56 ms |

The distant-brace input is an adversarial case. Each candidate is capped at
4,096 bytes, so repeated attempts remain bounded, but enabling spans has a
substantial cost on that input. The measured cost is recorded here rather than
treated as representative of normal prose.
