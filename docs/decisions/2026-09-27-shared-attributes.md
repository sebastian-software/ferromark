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
are ASCII letters followed by ASCII letters, digits, `-`, or `_`; names
are normalized to lowercase. Values may be unquoted without whitespace, or
single/double quoted. Quoted values may contain spaces; a backslash may escape
the matching quote or another backslash. Empty quoted values are valid, so
`hidden=""` is supported. A malformed block stays literal in full. `id=`
and `class=` merge into the same AST ID/class fields as shorthand tokens.
Classes accumulate in source order; the last explicit ID wins, and the last
value for a repeated key wins. Unlike the older ID/class-only extension,
repeated IDs are valid when the full grammar is enabled.
Inline suffix recognition reads at most 512 bytes after the opening brace and
accepts at most 64 tokens; longer blocks remain literal. This bound prevents
repeated malformed openers from repeatedly scanning a long paragraph.

The AST stores original custom keys without an HTML-specific prefix.
`Attribute { name, value }` entries live in optional boxed metadata on links,
images, headings, and figures, so the default AST paths carry only one absent
pointer. Tables already hold optional metadata; bracketed spans exist only
when enabled. The HTML renderer keeps recognized standard names unchanged in
trusted output. In untrusted output, only `lang`, `dir`, `title`, `width`,
`height`, `loading`, `decoding`, `hreflang`, `role`, `translate`, `spellcheck`,
`aria-*`, and explicit `data-*` remain HTML attributes. Other names, including
`style`, `name`, and URL-bearing attributes, receive `data-`.
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
policy and URL handling otherwise remain unchanged for Markdown-owned values.
Authored URL-bearing attributes are mapped to inert data in untrusted output,
so they cannot bypass the renderer's URL policy.

The link parser resolves inline and reference links before considering a span.
Thus `[text](url){...}` is an attributed link, and `[text]{...}` is a span when
the bracketed-span option is enabled and no reference resolves. A span can
contain links and other ordinary inline nodes. Code, math, raw HTML, and MDX
are not reparsed for attributes. Visitors, source spans, hooks, transforms,
plain-text extraction, and outline ID planning traverse the native span node.
When MDX is also enabled, an attached attribute suffix or bracketed span takes
precedence at its attachment point; a free `{expression}` remains MDX.

## Compatibility

Both new options are disabled in all presets, preserving CommonMark and GFM
output. Enabling `extended_attributes` intentionally replaces the older
permissive heading suffix parser with the strict shared grammar; malformed or
mixed blocks stay literal instead of silently dropping tokens. With the new
option off, the earlier heading behavior is unchanged. The ID/class-only
image and table grammar from #466 likewise stays unchanged when the new option
is off. Generated `<col>` classes are unaffected.
The public `Node::Span` variant, `ParserOptions` fields, and optional attribute
storage on existing AST nodes require a major Rust API release. Callers that
construct or exhaustively match these types need to update their code.

## Performance

Disabled paths check option flags before suffix scanning or span construction.
The `shared_attributes` Criterion target measured ordinary prose, valid
attributes, and repeated malformed blocks with both options on and off. On an
Apple Silicon local release build, median parse times were:

| Input | Both off | Attributes only | Spans only | Both on |
| --- | ---: | ---: | ---: | ---: |
| Ordinary prose (4.6 KB) | 12.63 µs | 12.51 µs | 12.33 µs | 12.40 µs |
| Valid annotations (5.6 KB) | 15.90 µs | 20.35 µs | 22.54 µs | 26.83 µs |
| Repeated malformed blocks (3.4 KB) | 10.07 µs | 10.01 µs | 14.24 µs | 14.30 µs |
| Distant closing brace after malformed spans (7.2 KB) | 12.53 µs | 12.60 µs | 1.57 ms | 1.56 ms |

These figures describe the initial implementation before review fixes. The
distant-brace input is adversarial; the 512-byte and 64-token limits reduce
its enabled-path median from 1.57 ms to 276 µs (spans only) and 278 µs (both
options on) in a focused local rerun, an 82% reduction. The disabled case
stayed at 12.5 µs. A follow-up external release harness compared the
stacked PR to `main` with GFM defaults and both new options off. It used 96
repeated blocks, seven 250 ms windows after a 200 ms warmup, and two
alternating process cycles on the same arm64 Mac:

| Workload | Main parse | Stacked PR parse | Main parse + render | Stacked PR parse + render |
| --- | ---: | ---: | ---: | ---: |
| Ordinary prose | 10.92 µs | 10.97 µs (+0.5%) | 15.28 µs | 15.25 µs (−0.2%) |
| Ordinary images | 10.65 µs | 11.00 µs (+3.3%) | 16.28 µs | 16.72 µs (+2.7%) |
| Ordinary links | 12.99 µs | 13.35 µs (+2.7%) | 19.48 µs | 19.72 µs (+1.2%) |

Against the reviewed #471 base, the added #473 default-path cost in this run
was 0.0% for prose, +0.7% for images, and +2.0% for links in parse-only mode.
These are descriptive local timings, not cross-machine guarantees. Default
link and heading nodes have no attribute allocation; a size guard limits the
ordinary link structure to 80 bytes.
