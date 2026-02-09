/// MDX Segmenter — split MDX into typed blocks, render only the Markdown.
///
/// Run with: `cargo run --features mdx --example mdx_segment`

fn main() {
    let input = r#"import { Card, Button } from './components'
export const meta = { title: 'Getting Started' }

# Getting Started

Welcome to the **documentation**. Here's a quick overview.

<Card title="Installation">

## Install via cargo

```bash
cargo add ferromark
```

</Card>

<Button variant="primary" />

{new Date().getFullYear()}
"#;

    let segments = ferromark::mdx::segment(input);

    println!("=== MDX Segments ===\n");

    for (i, seg) in segments.iter().enumerate() {
        match seg {
            ferromark::mdx::Segment::Esm(s) => {
                println!("[{i}] ESM");
                println!("    {}", s.trim());
            }
            ferromark::mdx::Segment::Markdown(s) => {
                let html = ferromark::to_html(s);
                println!("[{i}] Markdown");
                println!("    input:  {}", s.trim().replace('\n', "\\n"));
                println!("    html:   {}", html.trim().replace('\n', "\\n"));
            }
            ferromark::mdx::Segment::JsxBlockOpen(s) => {
                println!("[{i}] JSX Open");
                println!("    {}", s.trim());
            }
            ferromark::mdx::Segment::JsxBlockClose(s) => {
                println!("[{i}] JSX Close");
                println!("    {}", s.trim());
            }
            ferromark::mdx::Segment::JsxBlockSelfClose(s) => {
                println!("[{i}] JSX Self-Close");
                println!("    {}", s.trim());
            }
            ferromark::mdx::Segment::Expression(s) => {
                println!("[{i}] Expression");
                println!("    {}", s.trim());
            }
        }
        println!();
    }

    // --- Render demo ---
    println!("=== MDX Rendered ===\n");

    let output = ferromark::mdx::render(input);

    for esm in &output.esm {
        println!("ESM: {}", esm.trim());
    }
    match output.front_matter {
        Some(fm) => println!("Front matter: {}", fm.trim()),
        None => println!("Front matter: (none)"),
    }
    println!("\nBody:");
    for line in output.body.lines() {
        println!("  {line}");
    }
}
