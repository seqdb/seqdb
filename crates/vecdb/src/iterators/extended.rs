use crate::{PrintableIndex, StoredIndex, StoredRaw, VecIterator};

pub trait VecIteratorExtended: VecIterator<Item = Self::T> {
    type I: StoredIndex;
    type T: StoredRaw;

    #[inline]
    fn set_position(&mut self, i: Self::I) {
        self.set_position_to(i.to_usize());
    }

    #[inline]
    fn set_end(&mut self, i: Self::I) {
        self.set_end_to(i.to_usize());
    }

    #[inline]
    fn get(&mut self, i: Self::I) -> Option<Self::Item> {
        self.get_at(i.to_usize())
    }

    #[inline]
    fn get_unwrap(&mut self, i: Self::I) -> Self::Item {
        self.get(i).unwrap()
    }

    fn index_type_to_string(&self) -> &'static str {
        Self::I::to_string()
    }
}
