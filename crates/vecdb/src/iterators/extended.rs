use crate::{PrintableIndex, StoredIndex, StoredRaw, VecIterator};

pub trait VecIteratorExtended: VecIterator<Item = Self::T> {
    type I: StoredIndex;
    type T: StoredRaw;

    fn index_type_to_string(&self) -> &'static str {
        Self::I::to_string()
    }
}
