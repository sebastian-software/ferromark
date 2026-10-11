//! Collect occurrences; a consuming application supplies destination rules.

use ferromark::{Allocator, Parser};

#[allow(clippy::print_stdout)]
fn main() -> Result<(), ferromark::ParseError> {
    let source = "[Guide][guide] ![Logo](./logo.svg)\n\n[guide]: /guide\n";
    let allocator = Allocator::new();
    let document = Parser::new(&allocator, source).parse()?;
    for reference in document.collect_references(source) {
        println!(
            "{:?}: {} {:?}",
            reference.kind, reference.destination, reference.span
        );
        // Resolve local paths or external URLs according to your application.
    }
    Ok(())
}
