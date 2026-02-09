//! CommonMark specification tests.
//!
//! Runs tests from the CommonMark spec.json file to track compliance.

use ferromark::{to_html_with_options, Options};
use serde::Deserialize;
use std::fs;

/// CommonMark spec tests use default options with heading_ids disabled,
/// since heading IDs are not part of the CommonMark spec.
fn spec_to_html(input: &str) -> String {
    let mut options = Options::default();
    options.heading_ids = false;
    to_html_with_options(input, &options)
}

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

/// Sections that are intentionally out of scope.
#[allow(dead_code)]
const OUT_OF_SCOPE_SECTIONS: &[&str] = &[];

/// Check if label contains unescaped brackets
#[allow(dead_code)]
fn has_unescaped_bracket(label: &str) -> bool {
    let bytes = label.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\\' && i + 1 < bytes.len() {
            // Skip escaped character
            i += 2;
        } else if bytes[i] == b'[' || bytes[i] == b']' {
            return true;
        } else {
            i += 1;
        }
    }
    false
}

/// Check if a test uses reference link definitions (pattern: [label]: url)
#[allow(dead_code)]
fn uses_reference_links(markdown: &str) -> bool {
    // Reference definition pattern: starts with optional spaces, [label]:
    for line in markdown.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('[') {
            if let Some(bracket_end) = trimmed[1..].find("]:") {
                // Found a potential reference definition
                let label = &trimmed[1..bracket_end + 1];
                // Label should not be empty and should not contain unescaped brackets
                if !label.is_empty() && !has_unescaped_bracket(label) {
                    return true;
                }
            }
        }
        // Also check for continuation lines that contain ]: followed by URL/space
        // This catches multi-line reference labels like "[Foo\n  bar]: /url"
        if (trimmed.contains("]: ") || trimmed.contains("]:/") || trimmed.contains("]: <"))
            && !trimmed.starts_with('[')
        {
            // Could be a reference definition continuation
            // Check if there's a [ on an earlier line that's unclosed
            let mut found_open_bracket = false;
            for prev_line in markdown.lines() {
                if prev_line == line {
                    break; // Stop at current line
                }
                if prev_line.trim_start().starts_with('[') && !prev_line.contains(']') {
                    found_open_bracket = true;
                }
            }
            if found_open_bracket {
                return true;
            }
        }
    }
    false
}

/// Check if test requires proper 4-space indent handling.
/// This includes both indented code blocks and paragraph continuation rules.
#[allow(dead_code)]
fn requires_4space_handling(test: &SpecTest) -> bool {
    // Check if any line starts with 4+ spaces (after a non-blank line)
    let lines: Vec<&str> = test.markdown.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        let spaces = line.bytes().take_while(|&b| b == b' ').count();
        if spaces >= 4 && line.len() > spaces {
            // This line has 4+ space indent with content
            // If it's after a non-blank line and expected output treats it as paragraph content,
            // we can't handle this properly
            if i > 0 && !lines[i - 1].trim().is_empty() {
                // Previous line has content - this might be lazy continuation
                return true;
            }
            // If expected output has <pre><code>, it's indented code block
            if test.html.contains("<pre><code>") {
                return true;
            }
        }
    }
    false
}

/// Check if test requires setext heading (underline-style)
#[allow(dead_code)]
fn requires_setext(test: &SpecTest) -> bool {
    // Setext: line followed by === or ---
    let lines: Vec<&str> = test.markdown.lines().collect();
    for i in 1..lines.len() {
        let line = lines[i].trim();
        if (line.chars().all(|c| c == '=' || c == ' ') && line.contains('='))
            || (line.chars().all(|c| c == '-' || c == ' ') && line.contains('-'))
        {
            // Check if previous line has content and expected output has heading
            if !lines[i - 1].trim().is_empty()
                && (test.html.contains("<h1>") || test.html.contains("<h2>"))
            {
                return true;
            }
        }
    }
    false
}

#[allow(dead_code)]
fn is_in_scope(section: &str) -> bool {
    !OUT_OF_SCOPE_SECTIONS.contains(&section)
}

fn is_test_in_scope(test: &SpecTest) -> bool {
    let _ = test;
    true
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
        let output = spec_to_html(&test.markdown);
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

/// Run only IN-SCOPE spec tests and report results.
/// This excludes intentionally unsupported features like HTML blocks, setext headings, etc.
/// Also excludes link/image tests that use reference definitions.
#[test]
#[ignore]
fn commonmark_spec_report_in_scope() {
    let tests = load_spec_tests();
    let mut passed = 0;
    let mut failed = 0;
    let mut out_of_scope_count = 0;
    let mut ref_link_count = 0;
    let mut by_section: std::collections::HashMap<String, (u32, u32)> =
        std::collections::HashMap::new();

    for test in &tests {
        if !is_in_scope(&test.section) {
            out_of_scope_count += 1;
            continue;
        }
        if !is_test_in_scope(test) {
            ref_link_count += 1;
            continue;
        }

        let output = spec_to_html(&test.markdown);
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

    let total_in_scope = passed + failed;

    println!("\n=== CommonMark Spec Compliance Report (IN-SCOPE ONLY) ===\n");
    println!("Out-of-scope sections excluded: {:?}", OUT_OF_SCOPE_SECTIONS);
    println!("Out-of-scope tests skipped: {} (sections) + {} (reference links)",
             out_of_scope_count, ref_link_count);
    println!("In-scope: {} passed, {} failed out of {}", passed, failed, total_in_scope);
    println!("In-scope pass rate: {:.1}%\n", (passed as f64 / total_in_scope as f64) * 100.0);

    println!("By section:");
    let mut sections: Vec<_> = by_section.iter().collect();
    sections.sort_by_key(|(name, _)| *name);

    for (section, (p, f)) in sections {
        let total = p + f;
        let pct = (*p as f64 / total as f64) * 100.0;
        let status = if *f == 0 { "✓" } else { " " };
        println!("  {} {:40} {:3}/{:3} ({:5.1}%)", status, section, p, total, pct);
    }

    // Summary
    println!("\n--- Target Progress ---");
    let target_pct = 70.0;
    let current_pct = (passed as f64 / total_in_scope as f64) * 100.0;
    let target_tests = (total_in_scope as f64 * target_pct / 100.0).ceil() as u32;
    let tests_needed = if passed >= target_tests { 0 } else { target_tests - passed };
    println!("Current: {:.1}% ({}/{})", current_pct, passed, total_in_scope);
    println!("Target:  {:.1}% ({}/{})", target_pct, target_tests, total_in_scope);
    println!("Tests needed for target: {}", tests_needed);
}

/// Print a prioritized list of all failing CommonMark examples, grouped by section.
/// This is ignored by default since it's verbose.
#[test]
#[ignore]
fn commonmark_failures_report() {
    let tests = load_spec_tests();
    let mut failures_by_section: std::collections::HashMap<String, Vec<(u32, String, String, String)>> =
        std::collections::HashMap::new();

    for test in &tests {
        let output = spec_to_html(&test.markdown);
        if output != test.html {
            failures_by_section
                .entry(test.section.clone())
                .or_default()
                .push((test.example, test.markdown.clone(), test.html.clone(), output));
        }
    }

    let mut sections: Vec<_> = failures_by_section.into_iter().collect();
    sections.sort_by_key(|(_, failures)| std::cmp::Reverse(failures.len()));

    println!("\n=== CommonMark Failure List (All Sections) ===\n");
    for (section, failures) in sections {
        println!("-- {} ({} failures)", section, failures.len());
        for (ex, md, expected, got) in failures {
            println!("Example {}:", ex);
            println!("  Markdown: {:?}", md);
            println!("  Expected: {:?}", expected);
            println!("  Got:      {:?}", got);
        }
        println!();
    }
}

/// Test a specific section of the CommonMark spec.
fn run_section_tests(section_name: &str) -> (u32, u32, Vec<(u32, String, String, String)>) {
    let tests = load_spec_tests();
    let mut passed = 0;
    let mut failed = 0;
    let mut failures = Vec::new();

    for test in tests.iter().filter(|t| t.section == section_name) {
        let output = spec_to_html(&test.markdown);
        if output == test.html {
            passed += 1;
        } else {
            failed += 1;
            failures.push((test.example, test.markdown.clone(), test.html.clone(), output));
        }
    }

    (passed, failed, failures)
}

/// Test a specific section, only in-scope tests.
fn run_section_tests_in_scope(section_name: &str) -> (u32, u32, Vec<(u32, String, String, String)>) {
    let tests = load_spec_tests();
    let mut passed = 0;
    let mut failed = 0;
    let mut failures = Vec::new();

    for test in tests.iter().filter(|t| t.section == section_name && is_test_in_scope(t)) {
        let output = spec_to_html(&test.markdown);
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
fn spec_links_in_scope() {
    let (passed, failed, failures) = run_section_tests_in_scope("Links");
    if !failures.is_empty() {
        for (ex, md, expected, got) in &failures[..failures.len().min(10)] {
            eprintln!("\nExample {}: {:?}", ex, md);
            eprintln!("  Expected: {:?}", expected);
            eprintln!("  Got:      {:?}", got);
        }
    }
    eprintln!("\nLinks (in-scope): {}/{} passed", passed, passed + failed);
}

#[test]
fn spec_list_items_in_scope() {
    let (passed, failed, failures) = run_section_tests_in_scope("List items");
    if !failures.is_empty() {
        for (ex, md, expected, got) in &failures[..failures.len().min(10)] {
            eprintln!("\nExample {}: {:?}", ex, md);
            eprintln!("  Expected: {:?}", expected);
            eprintln!("  Got:      {:?}", got);
        }
    }
    eprintln!("\nList items (in-scope): {}/{} passed", passed, passed + failed);
}

#[test]
fn spec_lists_in_scope() {
    let (passed, failed, failures) = run_section_tests_in_scope("Lists");
    if !failures.is_empty() {
        for (ex, md, expected, got) in &failures[..failures.len().min(10)] {
            eprintln!("\nExample {}: {:?}", ex, md);
            eprintln!("  Expected: {:?}", expected);
            eprintln!("  Got:      {:?}", got);
        }
    }
    eprintln!("\nLists (in-scope): {}/{} passed", passed, passed + failed);
}

#[test]
fn spec_entity_refs_in_scope() {
    let (passed, failed, failures) = run_section_tests_in_scope("Entity and numeric character references");
    if !failures.is_empty() {
        for (ex, md, expected, got) in &failures[..failures.len().min(10)] {
            eprintln!("\nExample {}: {:?}", ex, md);
            eprintln!("  Expected: {:?}", expected);
            eprintln!("  Got:      {:?}", got);
        }
    }
    eprintln!("\nEntity refs (in-scope): {}/{} passed", passed, passed + failed);
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

#[test]
fn spec_autolinks() {
    let (passed, failed, failures) = run_section_tests("Autolinks");
    if !failures.is_empty() {
        for (ex, md, expected, got) in &failures[..failures.len().min(5)] {
            eprintln!("\nExample {}: {:?}", ex, md);
            eprintln!("  Expected: {:?}", expected);
            eprintln!("  Got:      {:?}", got);
        }
    }
    eprintln!("\nAutolinks: {}/{} passed", passed, passed + failed);
}
