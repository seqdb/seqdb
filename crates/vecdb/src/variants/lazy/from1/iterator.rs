use crate::{
    BoxedVecIterator, LazyVecFrom1, StoredIndex, StoredRaw, VecIterator, VecIteratorExtended,
};

pub struct LazyVecFrom1Iterator<'a, I, T, S1I, S1T>
where
    S1T: Clone,
{
    pub(crate) lazy: &'a LazyVecFrom1<I, T, S1I, S1T>,
    pub(crate) source: BoxedVecIterator<'a, S1I, S1T>,
    pub(crate) index: usize,
    pub(crate) end_index: usize,
}

impl<'a, I, T, S1I, S1T> Iterator for LazyVecFrom1Iterator<'a, I, T, S1I, S1T>
where
    I: StoredIndex,
    T: StoredRaw + 'a,
    S1I: StoredIndex,
    S1T: StoredRaw,
{
    type Item = T;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.end_index {
            return None;
        }

        let index = I::from(self.index);
        let opt = (self.lazy.compute)(index, &mut *self.source);

        if opt.is_some() {
            self.index += 1;
        }

        opt
    }
}

impl<I, T, S1I, S1T> VecIterator for LazyVecFrom1Iterator<'_, I, T, S1I, S1T>
where
    I: StoredIndex,
    T: StoredRaw,
    S1I: StoredIndex,
    S1T: StoredRaw,
{
    fn set_position_(&mut self, i: usize) {
        self.index = i.min(self.end_index);
        self.source.set_position_(i);
    }

    fn set_end_(&mut self, i: usize) {
        self.end_index = i.min(self.end_index);
        self.source.set_end_(i);
    }

    fn skip_optimized(mut self, n: usize) -> Self {
        self.index = self.index.saturating_add(n).min(self.end_index);
        self.source = self.source.skip_optimized(n);
        self
    }

    fn take_optimized(mut self, n: usize) -> Self {
        let absolute_end = self.index.saturating_add(n);
        self.end_index = absolute_end.min(self.end_index);
        self.source = self.source.take_optimized(n);
        self
    }
}

impl<I, T, S1I, S1T> VecIteratorExtended for LazyVecFrom1Iterator<'_, I, T, S1I, S1T>
where
    I: StoredIndex,
    T: StoredRaw,
    S1I: StoredIndex,
    S1T: StoredRaw,
{
    type I = I;
    type T = T;
}
