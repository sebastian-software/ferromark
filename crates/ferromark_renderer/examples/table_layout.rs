//! Render a table with horizontal spans and external CSS column widths.
use ferromark_allocator::Allocator;
use ferromark_parser::{Parser, ParserOptions};
use ferromark_renderer::{HtmlRenderer, HtmlRendererOptions};

fn main() {
    let source = "| Item | Net | Tax |\n| :--- | ---: | ---: |\n| Book | 20.00 | 1.40 |\n| Gift | Included ||\n\n: Prices *today* {#prices .price-list}";
    let allocator = Allocator::for_source_len(source.len());
    let document = Parser::with_options(
        &allocator,
        source,
        ParserOptions { merged_table_cells: true, table_attributes: true, ..ParserOptions::gfm() },
    )
    .parse()
    .expect("valid example Markdown");
    let html = HtmlRenderer::with_options(HtmlRendererOptions {
        table_colgroup: true,
        ..HtmlRendererOptions::gfm()
    })
    .render(&document);
    #[allow(clippy::print_stdout)]
    {
        print!(
            "<!doctype html>\n<html lang=\"en\"><meta charset=\"utf-8\"><title>Table layout</title>\n<style>\nbody {{ font: 1rem/1.5 system-ui; margin: 3rem auto; max-width: 48rem; padding: 0 1rem; }}\n#prices {{ width: 100%; table-layout: fixed; border-collapse: collapse; }}\n#prices > colgroup > .col-1 {{ width: 60%; }}\n#prices > colgroup > .col-2 {{ width: 25%; }}\n#prices > colgroup > .col-3 {{ width: 15%; }}\n#prices th, #prices td {{ padding: .6rem; border-bottom: 1px solid #aaa; overflow-wrap: anywhere; }}\n#prices caption {{ text-align: left; margin-bottom: 1rem; }}\n</style>\n<body>\n{html}</body></html>\n"
        );
    }
}
