# Wikipedia corpus attribution

Copyright Wikipedia contributors. The article text, derived Markdown, and rendered HTML derivatives remain under [CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/). This does not change the parser or harness license.

These are mechanical Markdown conversions of Wikipedia HTML, not native Markdown articles. The manifest records every transformation. Tables, images, reference apparatus, and math markup are excluded; article prose, headings, emphasis, lists, and links are retained. No Wikipedia endorsement is implied.

Raw downloaded HTML snapshots are retained compressed for conversion review. No linked image files are redistributed. Each first-paragraph/lead/body family overlaps and must not be treated as three independent articles.

- [Rainbow, revision 1373682866](https://en.wikipedia.org/w/index.php?title=Rainbow&oldid=1373682866) — [contributors/history](https://en.wikipedia.org/w/index.php?title=Rainbow&action=history); source SHA-256 `e6dd68d13ca5d87f185818943275d9cc26ab8886db1fbd8056d79cad6741509c`.
- [Tea, revision 1374563032](https://en.wikipedia.org/w/index.php?title=Tea&oldid=1374563032) — [contributors/history](https://en.wikipedia.org/w/index.php?title=Tea&action=history); source SHA-256 `59e215bd5832f8f02d93e8cb5f91a16810146a374c8920bfaa1cd8f16e5f5865`.
- [Chess, revision 1374640519](https://en.wikipedia.org/w/index.php?title=Chess&oldid=1374640519) — [contributors/history](https://en.wikipedia.org/w/index.php?title=Chess&action=history); source SHA-256 `a35bd65d74d00a6faae935cf4a1602f207b0ec8a42deebaea1ec56565a654f76`.
- [Volcano, revision 1372671594](https://en.wikipedia.org/w/index.php?title=Volcano&oldid=1372671594) — [contributors/history](https://en.wikipedia.org/w/index.php?title=Volcano&action=history); source SHA-256 `5f16cbc99988800c26f45dd03b60940f97a7c261c375cb99077e9ec3e49329ef`.

Reproduce the conversion with Python and lxml (the snapshot records the lxml version):

```sh
python3 benchmarks/broad-comparison/make_wiki_corpus.py
```

To acquire new sources, download each named article HTML to `ferromark-title.html` and pass `--import-dir DIR`. That creates a new corpus revision; do not overwrite an archived benchmark's inputs.
