# Ferromark terminology

`transform()` in the Node API means converting Markdown source to HTML. In the
Rust AST API, an individual operation over a parsed `Document` is a **pass**;
ordered passes are composed by `TransformPipeline` in the optional
`ferromark-transforms` crate.

Passes run after parsing and before rendering. They mutate Ferromark's native
arena-backed AST and follow the caller's order. The parser and renderer do not
load or run the optional transform crate. See
[ADR-0022](docs/arch/ADR-0022-native-transform-pipeline.md) for the lifecycle,
source-span and URL-protection contracts.
