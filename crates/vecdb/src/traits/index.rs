use std::{fmt::Debug, ops::Add};

use zerocopy::{Immutable, IntoBytes, KnownLayout, TryFromBytes};

use crate::PrintableIndex;

pub trait StoredIndex
where
    Self: Debug
        + Default
        + Copy
        + Clone
        + PartialEq
        + Eq
        + PartialOrd
        + Ord
        + From<usize>
        + Into<usize>
        + Add<usize, Output = Self>
        + TryFromBytes
        + IntoBytes
        + Immutable
        + KnownLayout
        + Send
        + Sync
        + PrintableIndex,
{
    #[inline]
    fn to_usize(self) -> usize {
        self.into()
    }

    #[inline]
    fn decremented(self) -> Option<Self> {
        self.to_usize().checked_sub(1).map(Self::from)
    }
}

impl<I> StoredIndex for I where
    I: Debug
        + Default
        + Copy
        + Clone
        + PartialEq
        + Eq
        + PartialOrd
        + Ord
        + From<usize>
        + Into<usize>
        + Add<usize, Output = Self>
        + TryFromBytes
        + IntoBytes
        + Immutable
        + KnownLayout
        + Send
        + Sync
        + PrintableIndex
{
}
