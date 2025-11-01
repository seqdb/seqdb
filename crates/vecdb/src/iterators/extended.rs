use crate::{PrintableIndex, StoredIndex, StoredRaw, VecIterator};

pub trait VecIteratorExtended: VecIterator<Item = Self::T> {
    type I: StoredIndex;
    type T: StoredRaw;

    #[inline]
    fn set_position(&mut self, i: Self::I) {
        self.set_position_(i.to_usize());
    }

    #[inline]
    fn set_end(&mut self, i: Self::I) {
        self.set_end_(i.to_usize());
    }

    #[inline]
    fn get(&mut self, i: Self::I) -> Option<Self::Item> {
        self.get_(i.to_usize())
    }

    #[inline]
    fn unsafe_get(&mut self, i: Self::I) -> Self::Item {
        unsafe { self.get(i).unwrap_unchecked() }
    }

    fn index_type_to_string(&self) -> &'static str {
        Self::I::to_string()
    }
}
