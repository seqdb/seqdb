use crate::{AnyVec, StoredIndex, StoredRaw};

pub trait TypedVec: AnyVec {
    type I: StoredIndex;
    type T: StoredRaw;
}
