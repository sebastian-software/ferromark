use std::io::{self, BufRead};
fn main() {
    for line in io::stdin().lock().lines() {
        let line = line.unwrap();
        let words: Vec<_> = line.split_whitespace().map(|word| {
            if word.starts_with("_R") { format!("{:#}", rustc_demangle::demangle(word)) } else { word.to_owned() }
        }).collect();
        println!("{}", words.join(" "));
    }
}
