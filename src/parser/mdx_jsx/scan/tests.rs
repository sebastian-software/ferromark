#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types
)]

use std::collections::HashMap;

use super::{find_matching_close, record_matching_closes, scan_jsx_open};

/// The record answers for every opener one walk passes, so each answer has
/// to be the one a walk starting at that opener reaches — the openers it
/// reports and the ones its range leaves unreported.
#[test]
fn recorded_closers_agree_with_a_walk_from_each_opener() {
    let allocator = crate::allocator::Allocator::new();
    for source in [
        "<A><A><A></A>",
        "<A></A><A></A>",
        "<A><A></A></A>",
        "<A><B></B></A>",
        "<A><A/><A></A>",
        "<A>{'</A>'}</A>",
        "<A>`</A>`</A>",
        "<A>{</A>",
        "<A>{{{{</A>",
        "<A>text</A>more<A>",
        "<><></></>",
        "<A.B><A.B></A.B>",
        "<A x={1} y=\"2\"><A></A>",
        "<A></B></A>",
        "<A",
        "<A>",
    ] {
        for name in [Some("A"), Some("A.B"), None] {
            for from in 0..=source.len() {
                let mut recorded = HashMap::new();
                let walk = record_matching_closes(source, from, name, &mut |opener, closed| {
                    recorded.insert(opener, closed);
                });
                assert_eq!(
                    walk.closed,
                    find_matching_close(source, from, name).is_some(),
                    "outer answer for {source:?} from {from} as {name:?}"
                );
                for (opener, closed) in &recorded {
                    assert_eq!(
                        *closed,
                        find_matching_close(source, *opener, name).is_some(),
                        "recorded answer for {source:?} at {opener} as {name:?}"
                    );
                }
                if walk.closed {
                    continue;
                }
                // The range the walk read is the claim the memo makes about
                // every opening tag in it that went unreported: nothing
                // closes it.
                let (read_from, until) = walk.read;
                for at in read_from..until {
                    let mut attributes = allocator.new_vec();
                    let Some(open) = scan_jsx_open(source, at, 0, &mut attributes) else {
                        continue;
                    };
                    if open.self_closing || open.name != name || recorded.contains_key(&open.end) {
                        continue;
                    }
                    assert_eq!(
                        find_matching_close(source, open.end, name),
                        None,
                        "unreported opener in {source:?} at {at} as {name:?} does close"
                    );
                }
            }
        }
    }
}

/// An opener is recorded at the end of its opening tag, which is the
/// position the next opener asks under.
#[test]
fn recorded_openers_are_keyed_by_the_end_of_the_opening_tag() {
    let allocator = crate::allocator::Allocator::new();
    for source in ["<A><A></A>", "<A x=\"1\"><A></A>", "<><></>"] {
        let mut attributes = allocator.new_vec();
        let open = scan_jsx_open(source, 0, 0, &mut attributes).expect("opening tag");
        let mut recorded = HashMap::new();
        record_matching_closes(source, open.end, open.name, &mut |opener, closed| {
            recorded.insert(opener, closed);
        });
        let mut nested_attributes = allocator.new_vec();
        let nested =
            scan_jsx_open(source, open.end, 0, &mut nested_attributes).expect("nested opening tag");
        assert!(
            recorded.contains_key(&nested.end),
            "{source:?} did not record the nested opener"
        );
    }
}
