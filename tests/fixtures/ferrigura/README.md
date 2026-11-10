# Ferrigura integration fixtures

`diagrams.json` pins the sanitized SVG output of the five shared diagrams.

`AtkinsonHyperlegibleMono-Regular.ttf` and `OFL-AtkinsonMono.txt` are unmodified
files from the published `ferrigura-text` 0.7.0 crate, source commit
`cc4e69456e0d168836b1683eb64637e067cb08ca`. The font is licensed under OFL-1.1;
retain its accompanying license. It exercises caller-supplied font geometry,
embedded font bytes, and CSS through Ferromark's HTML and JSX SVG policy.

## Ferrigura 0.6 to 0.7 comparison

The five actual registry renderings retain identical view boxes, text labels,
element counts, and embedded font payloads. The flowchart is byte-identical.
The other four hashes change for these reviewed engine behaviors:

- Sequence participant boxes use the compact default (76 to 32 pixels), with
  corresponding box/text offsets. The message line is unchanged. Inline
  unavailable-family overrides are removed so text inherits the embedded face.
- Class nodes gain palette color IDs; node/relation CSS uses a defined stroke
  width of 2 instead of an unresolved variable.
- State start/end paths gain explicit fill/stroke colors; state-note CSS gains
  a text rule. Path geometry is unchanged.
- ER's existing edge-label background gains explicit opacity 1, preserving its
  coordinates and dimensions.

These are the published engine's compatibility, color, and label-mask changes;
the hash refresh does not assert that every difference comes from custom fonts.
