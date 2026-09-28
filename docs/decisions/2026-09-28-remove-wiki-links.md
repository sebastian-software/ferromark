# Remove wiki link syntax

- Status: Accepted
- Date: 2026-09-28

## Context

`ParserOptions::wiki_links` (`wikiLinks` in Node.js) parsed Obsidian-style
`[[Page]]` and `[[Page|Label]]` into ordinary `Link` nodes whose URL was the raw
target. The option arrived with the OX-Content core when v2 was initialized;
Ferromark v1 never had it, and no Ferromark user or issue asked for it.

Wiki links are an alternative spelling of a link that is only useful together
with application routing: the raw target is not a URL, so every integration
had to rewrite it in a link hook anyway. Standard Markdown links already
express the same thing portably. The option still cost code and maintenance:
a dedicated `]]` scan with its own memo table, label probing, and tests for
quadratic inputs, nesting depth, and source spans.

## Decision

Remove wiki link syntax in 3.0, together with the other breaking Rust API
changes of that release. `[[Page]]` is ordinary text again in every profile;
nested bracket text, reference links, and link nesting rules are unchanged.

Remove `ParserOptions::wiki_links`, the Node.js `wikiLinks` option, and the
parser code behind them. The Node.js packed option layout keeps bit 24 unused,
so the bits of later options stay stable.

Applications that want wiki-style links can resolve `[[Page]]` in their own
text transform and emit ordinary links.

## Consequences

Rust code that sets `wiki_links` no longer compiles; Node.js calls that pass
`wikiLinks` throw `TypeError: unknown option "wikiLinks"`. Documents
that relied on the syntax render the brackets as text. The Ferromark Flavored
Markdown profile never included wiki links, so it is unaffected.
