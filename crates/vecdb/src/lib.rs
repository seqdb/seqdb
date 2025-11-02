#![doc = include_str!("../README.md")]
#![doc = "\n## Examples\n"]
#![doc = "\n### Raw\n\n```rust"]
#![doc = include_str!("../examples/raw.rs")]
#![doc = "```\n"]
#![doc = "\n### Compressed\n\n```rust"]
#![doc = include_str!("../examples/compressed.rs")]
#![doc = "```"]

pub use rawdb::{Database, Error as SeqDBError, PAGE_SIZE, Reader};
#[cfg(feature = "derive")]
pub use vecdb_derive::StoredCompressed;

mod error;
mod exit;
mod iterators;
mod stamp;
mod traits;
mod variants;
mod version;

use variants::*;

pub use error::*;
pub use exit::*;
pub use iterators::*;
pub use stamp::*;
pub use traits::*;
pub use variants::{
    CompressedVec, Computation, ComputedVec, ComputedVecFrom1, ComputedVecFrom2, ComputedVecFrom3,
    EagerVec, Format, ImportOptions, LazyVecFrom1, LazyVecFrom2, LazyVecFrom3, RawVec, StoredVec,
};
pub use version::*;

const ONE_KIB: usize = 1024;
const BUFFER_SIZE: usize = 512 * ONE_KIB;

// Branch prediction hints
#[inline(always)]
#[cold]
pub fn cold() {}

#[inline(always)]
pub fn likely(b: bool) -> bool {
    if !b {
        cold();
    }
    b
}

#[inline(always)]
pub fn unlikely(b: bool) -> bool {
    if b {
        cold();
    }
    b
}
