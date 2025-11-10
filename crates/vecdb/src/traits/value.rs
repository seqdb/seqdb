use std::fmt::Debug;

use serde::Serialize;
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

pub trait VecValue
where
    Self: Sized
        + Debug
        + Clone
        + FromBytes
        + IntoBytes
        + Immutable
        + KnownLayout
        + Serialize
        + Send
        + Sync
        + 'static,
{
}

impl<T> VecValue for T where
    T: Sized
        + Debug
        + Clone
        + FromBytes
        + IntoBytes
        + Immutable
        + KnownLayout
        + Serialize
        + Send
        + Sync
        + 'static
{
}
