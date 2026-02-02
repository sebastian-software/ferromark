//! CommonMark specification tests.
//!
//! Runs tests from the CommonMark spec.json file to track compliance.

use md_fast::to_html;
use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
struct SpecTest {
    markdown: String,
    html: String,
    example: u32,
    section: String,
}

fn load_spec_tests() -> Vec<SpecTest> {
    let spec_json = fs::read_to_string("tests/spec.json")
        .expect("Failed to read tests/spec.json");
    serde_json::from_str(&spec_json).expect("Failed to parse spec.json")
}

/// Run all spec tests and report results.
/// This is marked as ignored by default since it's for reporting, not CI.
#[test]
#[ignore]
fn commonmark_spec_report() {
    let tests = load_spec_tests();
    let mut passed = 0;
    let mut failed = 0;
    let mut by_section: std::collections::HashMap<String, (u32, u32)> =
        std::collections::HashMap::new();

    for test in &tests {
        let output = to_html(&test.markdown);
        let is_pass = output == test.html;

        let entry = by_section.entry(test.section.clone()).or_insert((0, 0));
        if is_pass {
            passed += 1;
            entry.0 += 1;
        } else {
            failed += 1;
            entry.1 += 1;
        }
    }

    println!("\n=== CommonMark Spec Compliance Report ===\n");
    println!("Total: {} passed, {} failed out of {}", passed, failed, tests.len());
    println!("Pass rate: {:.1}%\n", (passed as f64 / tests.len() as f64) * 100.0);

    println!("By section:");
    let mut sections: Vec<_> = by_section.iter().collect();
    sections.sort_by_key(|(name, _)| *name);

    for (section, (p, f)) in sections {
        let total = p + f;
        let pct = (*p as f64 / total as f64) * 100.0;
        let status = if *f == 0 { "✓" } else { " " };
        println!("  {} {:40} {:3}/{:3} ({:5.1}%)", status, section, p, total, pct);
    }
}

/// Test a specific section of the CommonMark spec.
fn run_section_tests(section_name: &str) -> (u32, u32, Vec<(u32, String, String, String)>) {
    let tests = load_spec_tests();
    let mut passed = 0;
    let mut failed = 0;
    let mut failures = Vec::new();

    for test in tests.iter().filter(|t| t.section == section_name) {
        let output = to_html(&test.markdown);
        if output == test.html {
            passed += 1;
        } else {
            failed += 1;
            failures.push((test.example, test.markdown.clone(), test.html.clone(), output));
        }
    }

    (passed, failed, failures)
}

// === Section-specific tests ===
// These help track progress on specific CommonMark sections.

#[test]
fn spec_thematic_breaks() {
    let (passed, failed, failures) = run_section_tests("Thematic breaks");
    if !failures.is_empty() {
        for (ex, md, expected, got) in &failures[..failures.len().min(3)] {
            eprintln!("\nExample {}: {:?}", ex, md);
            eprintln!("  Expected: {:?}", expected);
            eprintln!("  Got:      {:?}", got);
        }
    }
    eprintln!("\nThematic breaks: {}/{} passed", passed, passed + failed);
    // Don't assert - just report for now
}

#[test]
fn spec_atx_headings() {
    let (passed, failed, failures) = run_section_tests("ATX headings");
    if !failures.is_empty() {
        for (ex, md, expected, got) in &failures[..failures.len().min(3)] {
            eprintln!("\nExample {}: {:?}", ex, md);
            eprintln!("  Expected: {:?}", expected);
            eprintln!("  Got:      {:?}", got);
        }
    }
    eprintln!("\nATX headings: {}/{} passed", passed, passed + failed);
}

#[test]
fn spec_fenced_code_blocks() {
    let (passed, failed, failures) = run_section_tests("Fenced code blocks");
    if !failures.is_empty() {
        for (ex, md, expected, got) in &failures[..failures.len().min(3)] {
            eprintln!("\nExample {}: {:?}", ex, md);
            eprintln!("  Expected: {:?}", expected);
            eprintln!("  Got:      {:?}", got);
        }
    }
    eprintln!("\nFenced code blocks: {}/{} passed", passed, passed + failed);
}

#[test]
fn spec_paragraphs() {
    let (passed, failed, failures) = run_section_tests("Paragraphs");
    if !failures.is_empty() {
        for (ex, md, expected, got) in &failures[..failures.len().min(3)] {
            eprintln!("\nExample {}: {:?}", ex, md);
            eprintln!("  Expected: {:?}", expected);
            eprintln!("  Got:      {:?}", got);
        }
    }
    eprintln!("\nParagraphs: {}/{} passed", passed, passed + failed);
}

#[test]
fn spec_block_quotes() {
    let (passed, failed, failures) = run_section_tests("Block quotes");
    if !failures.is_empty() {
        for (ex, md, expected, got) in &failures[..failures.len().min(3)] {
            eprintln!("\nExample {}: {:?}", ex, md);
            eprintln!("  Expected: {:?}", expected);
            eprintln!("  Got:      {:?}", got);
        }
    }
    eprintln!("\nBlock quotes: {}/{} passed", passed, passed + failed);
}

#[test]
fn spec_list_items() {
    let (passed, failed, failures) = run_section_tests("List items");
    if !failures.is_empty() {
        for (ex, md, expected, got) in &failures[..failures.len().min(3)] {
            eprintln!("\nExample {}: {:?}", ex, md);
            eprintln!("  Expected: {:?}", expected);
            eprintln!("  Got:      {:?}", got);
        }
    }
    eprintln!("\nList items: {}/{} passed", passed, passed + failed);
}

#[test]
fn spec_lists() {
    let (passed, failed, failures) = run_section_tests("Lists");
    if !failures.is_empty() {
        for (ex, md, expected, got) in &failures[..failures.len().min(3)] {
            eprintln!("\nExample {}: {:?}", ex, md);
            eprintln!("  Expected: {:?}", expected);
            eprintln!("  Got:      {:?}", got);
        }
    }
    eprintln!("\nLists: {}/{} passed", passed, passed + failed);
}

#[test]
fn spec_emphasis() {
    let (passed, failed, failures) = run_section_tests("Emphasis and strong emphasis");
    if !failures.is_empty() {
        for (ex, md, expected, got) in &failures[..failures.len().min(5)] {
            eprintln!("\nExample {}: {:?}", ex, md);
            eprintln!("  Expected: {:?}", expected);
            eprintln!("  Got:      {:?}", got);
        }
    }
    eprintln!("\nEmphasis: {}/{} passed", passed, passed + failed);
}

#[test]
fn spec_hard_line_breaks() {
    let (passed, failed, failures) = run_section_tests("Hard line breaks");
    if !failures.is_empty() {
        for (ex, md, expected, got) in &failures[..failures.len().min(5)] {
            eprintln!("\nExample {}: {:?}", ex, md);
            eprintln!("  Expected: {:?}", expected);
            eprintln!("  Got:      {:?}", got);
        }
    }
    eprintln!("\nHard line breaks: {}/{} passed", passed, passed + failed);
}

#[test]
fn spec_images() {
    let (passed, failed, failures) = run_section_tests("Images");
    if !failures.is_empty() {
        for (ex, md, expected, got) in &failures[..failures.len().min(5)] {
            eprintln!("\nExample {}: {:?}", ex, md);
            eprintln!("  Expected: {:?}", expected);
            eprintln!("  Got:      {:?}", got);
        }
    }
    eprintln!("\nImages: {}/{} passed", passed, passed + failed);
}

#[test]
fn spec_links() {
    let (passed, failed, failures) = run_section_tests("Links");
    if !failures.is_empty() {
        for (ex, md, expected, got) in &failures[..failures.len().min(5)] {
            eprintln!("\nExample {}: {:?}", ex, md);
            eprintln!("  Expected: {:?}", expected);
            eprintln!("  Got:      {:?}", got);
        }
    }
    eprintln!("\nLinks: {}/{} passed", passed, passed + failed);
}

#[test]
fn spec_code_spans() {
    let (passed, failed, failures) = run_section_tests("Code spans");
    if !failures.is_empty() {
        for (ex, md, expected, got) in &failures[..failures.len().min(5)] {
            eprintln!("\nExample {}: {:?}", ex, md);
            eprintln!("  Expected: {:?}", expected);
            eprintln!("  Got:      {:?}", got);
        }
    }
    eprintln!("\nCode spans: {}/{} passed", passed, passed + failed);
}

#[test]
fn spec_backslash_escapes() {
    let (passed, failed, failures) = run_section_tests("Backslash escapes");
    if !failures.is_empty() {
        for (ex, md, expected, got) in &failures[..failures.len().min(5)] {
            eprintln!("\nExample {}: {:?}", ex, md);
            eprintln!("  Expected: {:?}", expected);
            eprintln!("  Got:      {:?}", got);
        }
    }
    eprintln!("\nBackslash escapes: {}/{} passed", passed, passed + failed);
}
