use md_fast::{BlockParser, fixup_list_tight};

fn main() {
    let input = "> foo\n\n> bar\n";
    let mut parser = BlockParser::new(input.as_bytes());
    let mut events = Vec::new();
    parser.parse(&mut events);
    
    fixup_list_tight(&mut events);
    
    println!("Events for: {:?}", input);
    for (i, e) in events.iter().enumerate() {
        println!("  {}: {:?}", i, e);
    }
}
