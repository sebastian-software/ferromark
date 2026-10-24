use smallvec::SmallVec;
use std::mem::size_of;

fn main() {
    println!("old_names_bytes={}", size_of::<SmallVec<[Option<&str>; 16]>>());
    println!("initial_names_and_span_bytes={}", size_of::<SmallVec<[(Option<&str>, [u32; 2]); 16]>>());
    println!("source_backed_names_bytes={}", size_of::<SmallVec<[&str; 16]>>());
}
