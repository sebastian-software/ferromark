# RFC: optional resolved-inline sink

## Decision

Do not add a direct HTML sink in the current architecture. Keep the resolved
`InlineEvent` sequence as the one shared boundary between inline resolution and
output. Reopen this RFC only if profiling isolates event materialization as a
dominant cost and a sink can consume one shared resolved representation without
duplicating renderer semantics.

## Evidence

On the portable current baseline, CommonMark 50 KB produces 2,081 inline events
from 1,018 inline parses; mixed 250 KB produces 10,405 events from 5,090 parses.
Those counts are material, but they do not establish that event storage is the
dominant cost. The resolved-event generation still sorts emit points, while the
HTML renderer owns stateful image-alt handling, link policy, raw-HTML policy,
and footnote numbering.

## Evaluated boundary

The smallest credible abstraction would let `InlineParser` emit the already
resolved operations to either an event collector or an HTML writer. In practice,
the writer needs the same image nesting, reference lookup, URL policy, raw HTML
filtering, and footnote-number state as `render_inline_event`. Moving those
rules into a second sink would duplicate behavior; moving them into the parser
would make parsing depend on an HTML policy and weaken the transform boundary.

## Reopen gate

Require all of the following before a new prototype:

- a profile showing event allocation or event iteration as a dominant 50 KB
  cost after resolution;
- one resolved operation model consumed by both the event collector and HTML
  sink;
- byte-identical HTML across images, links, raw HTML policy, footnotes, and
  nested emphasis;
- a credible transform/plugin consumer of the same operation model; and
- a repeated 2% 50 KB gain or a substantial allocation reduction.
