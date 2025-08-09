#![doc = include_str!("../README.md")]
#![doc = "\n## Examples\n"]
#![doc = "\n### Raw\n\n```rust"]
#![doc = include_str!("../examples/raw.rs")]
#![doc = "```\n"]
#![doc = "\n### Compressed\n\n```rust"]
#![doc = include_str!("../examples/compressed.rs")]
#![doc = "```"]

pub use pco::data_types::LatentType;

use seqdb::SeqDB;
#[cfg(feature = "derive")]
pub use vecdb_derive::StoredCompressed;

mod error;
mod exit;
mod stamp;
mod traits;
mod variants;
mod version;

use variants::*;

pub use error::*;
pub use exit::*;
pub use stamp::*;
pub use traits::*;
pub use variants::{
    CompressedVec, Computation, ComputedVec, ComputedVecFrom1, ComputedVecFrom2, ComputedVecFrom3,
    EagerVec, Format, LazyVecFrom1, LazyVecFrom2, LazyVecFrom3, RawVec, StoredVec,
};
pub use version::*;

pub type VecDB = SeqDB;
