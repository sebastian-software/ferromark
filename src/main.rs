//! ferromark CLI - Ultra-high-performance Markdown to HTML compiler

use std::io::{self, Read, Write};

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    // Simple usage: read from stdin or file
    let input = if args.len() > 1 && args[1] != "-" {
        std::fs::read_to_string(&args[1])?
    } else {
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf)?;
        buf
    };

    let html = ferromark::to_html(&input);
    io::stdout().write_all(html.as_bytes())?;

    Ok(())
}
