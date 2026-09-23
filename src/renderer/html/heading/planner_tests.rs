//! The claim-storage planner must plan exactly what the map-keyed planner it
//! replaced planned.
//!
//! `HeadingIdPlanner` now writes each candidate into one reused string,
//! looks it up by a hash computed once, and claims a free candidate in place.
//! The map-keyed planner below is kept verbatim as the oracle. Random request
//! sequences drawn from a collision-heavy alphabet run through both, with the
//! clears of one-shot rendering and the snapshot/restore of provisional
//! incremental fragments interleaved, and every planned ID has to match. The
//! same sequences run again with the hash narrowed to a few bits, because
//! distinct IDs practically never share a full 64-bit hash and the collision
//! chain would otherwise go untested.

use std::cell::Cell;
use std::fmt::Write as _;

use compact_str::CompactString;
use rustc_hash::FxHashMap;

use super::{HeadingIdPlanner, PlannedId, slugify_heading};

thread_local! {
    /// Mask applied to every planner hash computed on this thread.
    pub(super) static HASH_MASK: Cell<u64> = const { Cell::new(u64::MAX) };
}

/// Restores the full hash when a test that narrowed it ends, even by panic.
struct NarrowedHash;

impl NarrowedHash {
    fn new(mask: u64) -> Self {
        HASH_MASK.with(|cell| cell.set(mask));
        Self
    }
}

impl Drop for NarrowedHash {
    fn drop(&mut self) {
        HASH_MASK.with(|cell| cell.set(u64::MAX));
    }
}

/// The planner before claim storage, verbatim apart from its name.
#[derive(Debug, Clone, Default)]
pub(in crate::renderer::html) struct MapKeyedPlanner {
    next_suffix: FxHashMap<CompactString, usize>,
}

impl MapKeyedPlanner {
    pub(in crate::renderer::html) fn clear(&mut self) {
        self.next_suffix.clear();
    }

    pub(in crate::renderer::html) fn plan(&mut self, base: &str) -> String {
        let mut id = String::new();
        self.plan_into(base, &mut id);
        id
    }

    pub(in crate::renderer::html) fn plan_into(&mut self, base: &str, output: &mut String) {
        output.clear();
        let Some(mut suffix) = self.next_suffix.get(base).copied() else {
            output.push_str(base);
            self.next_suffix.insert(CompactString::from(base), 1);
            return;
        };

        loop {
            output.clear();
            output.push_str(base);
            let _ = write!(output, "-{suffix}");
            suffix = suffix.saturating_add(1);
            if !self.next_suffix.contains_key(output.as_str()) {
                self.next_suffix
                    .insert(CompactString::from(output.as_str()), 1);
                if let Some(next) = self.next_suffix.get_mut(base) {
                    *next = suffix;
                }
                return;
            }
        }
    }
}

/// Deterministic xorshift, so a failure is reproducible from the test alone.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn pick<'a>(&mut self, items: &[&'a str]) -> &'a str {
        items[(self.next() % items.len() as u64) as usize]
    }
}

/// Bases whose generated suffixes collide with each other and with later
/// explicit requests: `a` then `a-1` then `a` again, `-1` style fragments,
/// the empty string, and non-ASCII IDs.
const BASES: [&str; 22] = [
    "a",
    "a",
    "a-1",
    "a-1-1",
    "a-2",
    "a-1-2",
    "b",
    "b-1",
    "section",
    "section-1",
    "",
    "-1",
    "-",
    "1",
    "日本",
    "日本-1",
    "x\"y",
    "a&b",
    "an-extended-heading-longer-than-twenty-four-bytes",
    "an-extended-heading-longer-than-twenty-four-bytes-1",
    "A",
    "a-10",
];

/// Heading texts for `claim_slug`, whose slugs land on the same bases.
const TEXTS: [&str; 18] = [
    "A",
    "a",
    "a 1",
    "A-1",
    "a 1 1",
    "Section",
    "!!!",
    "",
    "日本",
    "日本 1",
    "B",
    "b-1",
    "Options & Defaults",
    "options defaults 1",
    "An extended heading, longer than twenty-four bytes",
    "\u{0130}stanbul",
    "a 10",
    "a-2",
];

/// One claimed ID and the text the oracle planned for it.
type Claimed = Vec<(PlannedId, String)>;

fn run_against_map_keyed_planner(seed: u64, rounds: usize) {
    let mut rng = Rng(seed);
    let mut planner = HeadingIdPlanner::new();
    let mut oracle = MapKeyedPlanner::default();
    let mut claimed: Claimed = Vec::new();
    let mut snapshot: Option<(HeadingIdPlanner, MapKeyedPlanner, Claimed)> = None;
    let mut reused = String::from("stale output that plan_into must clear");

    for round in 0..rounds {
        match rng.next() % 48 {
            // A new one-shot document.
            0 => {
                planner.clear();
                oracle.clear();
                claimed.clear();
                snapshot = None;
            }
            // A provisional fragment starts from a snapshot of the state ...
            1 => snapshot = Some((planner.clone(), oracle.clone(), claimed.clone())),
            // ... and the state is restored when it is discarded.
            2 => {
                if let Some((saved_planner, saved_oracle, saved_claimed)) = snapshot.take() {
                    planner = saved_planner;
                    oracle = saved_oracle;
                    claimed = saved_claimed;
                }
            }
            _ => {}
        }

        match rng.next() % 4 {
            0 => {
                let base = rng.pick(&BASES);
                assert_eq!(
                    planner.plan(base),
                    oracle.plan(base),
                    "round {round}: {base:?}"
                );
            }
            1 => {
                let base = rng.pick(&BASES);
                let expected = oracle.plan(base);
                planner.plan_into(base, &mut reused);
                assert_eq!(reused, expected, "round {round}: {base:?}");
            }
            2 => {
                let base = rng.pick(&BASES);
                let expected = oracle.plan(base);
                let id = planner.claim(base);
                assert_eq!(planner.id(id), expected, "round {round}: {base:?}");
                claimed.push((id, expected));
            }
            _ => {
                let text = rng.pick(&TEXTS);
                let expected = oracle.plan(&slugify_heading(text));
                let id = planner.claim_slug(text);
                assert_eq!(planner.id(id), expected, "round {round}: {text:?}");
                claimed.push((id, expected));
            }
        }

        // Later claims never move or rewrite an earlier one.
        if round.is_multiple_of(97) {
            for (id, expected) in &claimed {
                assert_eq!(planner.id(*id), expected, "round {round}");
            }
        }
    }
}

#[test]
fn claim_storage_plans_what_the_map_keyed_planner_planned() {
    for seed in [0x5EED_0001, 0xDEAD_BEEF_1234_5678, 0x0123_4567_89AB_CDEF] {
        run_against_map_keyed_planner(seed, 20_000);
    }
}

#[test]
fn claim_storage_plans_what_the_map_keyed_planner_planned_when_hashes_collide() {
    // Four hash values for every ID: long chains of distinct IDs that share a
    // hash, so each lookup walks past claims that do not match.
    let narrowed = NarrowedHash::new(0b11);
    for seed in [0x5EED_0002, 0xFEED_FACE_CAFE_F00D] {
        run_against_map_keyed_planner(seed, 10_000);
    }
    drop(narrowed);

    // One hash value for every ID: a single chain holds the whole document.
    let _narrowed = NarrowedHash::new(0);
    run_against_map_keyed_planner(0x5EED_0003, 5_000);
}

#[test]
fn claimed_ids_survive_later_claims_and_duplicates() {
    let _narrowed = NarrowedHash::new(0b1);
    let mut planner = HeadingIdPlanner::new();
    let first = planner.claim_slug("Getting Started");
    let explicit = planner.claim("getting-started-1");
    let duplicate = planner.claim_slug("Getting started");
    let second_duplicate = planner.claim("getting-started");

    assert_eq!(planner.id(first), "getting-started");
    assert_eq!(planner.id(explicit), "getting-started-1");
    assert_eq!(planner.id(duplicate), "getting-started-2");
    assert_eq!(planner.id(second_duplicate), "getting-started-3");

    planner.clear();
    let after_clear = planner.claim_slug("Getting Started");
    assert_eq!(planner.id(after_clear), "getting-started");
}
