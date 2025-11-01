use crate::{
    BoxedVecIterator, LazyVecFrom2, StoredIndex, StoredRaw, VecIterator, VecIteratorExtended,
};

pub struct LazyVecFrom2Iterator<'a, I, T, S1I, S1T, S2I, S2T>
where
    S1T: Clone,
    S2T: Clone,
{
    pub(crate) lazy: &'a LazyVecFrom2<I, T, S1I, S1T, S2I, S2T>,
    pub(crate) source1: BoxedVecIterator<'a, S1I, S1T>,
    pub(crate) source2: BoxedVecIterator<'a, S2I, S2T>,
    pub(crate) source1_same_index: bool,
    pub(crate) source2_same_index: bool,
    pub(crate) index: usize,
    pub(crate) end_index: usize,
}

impl<'a, I, T, S1I, S1T, S2I, S2T> Iterator for LazyVecFrom2Iterator<'a, I, T, S1I, S1T, S2I, S2T>
where
    I: StoredIndex,
    T: StoredRaw + 'a,
    S1I: StoredIndex,
    S1T: StoredRaw,
    S2I: StoredIndex,
    S2T: StoredRaw,
{
    type Item = T;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.end_index {
            return None;
        }

        let index = I::from(self.index);
        let opt = (self.lazy.compute)(index, &mut *self.source1, &mut *self.source2);

        if opt.is_some() {
            self.index += 1;
        }

        opt
    }
}

impl<I, T, S1I, S1T, S2I, S2T> VecIterator for LazyVecFrom2Iterator<'_, I, T, S1I, S1T, S2I, S2T>
where
    I: StoredIndex,
    T: StoredRaw,
    S1I: StoredIndex,
    S1T: StoredRaw,
    S2I: StoredIndex,
    S2T: StoredRaw,
{
    fn set_position_(&mut self, i: usize) {
        self.index = i.min(self.end_index);
        if self.source1_same_index {
            self.source1.set_position_(i);
        }
        if self.source2_same_index {
            self.source2.set_position_(i);
        }
    }

    fn set_end_(&mut self, i: usize) {
        self.end_index = i.min(self.end_index);
        if self.source1_same_index {
            self.source1.set_end_(i);
        }
        if self.source2_same_index {
            self.source2.set_end_(i);
        }
    }

    fn skip_optimized(mut self, n: usize) -> Self {
        self.index = self.index.saturating_add(n).min(self.end_index);
        if self.source1_same_index {
            self.source1 = self.source1.skip_optimized(n);
        }
        if self.source2_same_index {
            self.source2 = self.source2.skip_optimized(n);
        }
        self
    }

    fn take_optimized(mut self, n: usize) -> Self {
        let absolute_end = self.index.saturating_add(n);
        self.end_index = absolute_end.min(self.end_index);
        if self.source1_same_index {
            self.source1 = self.source1.take_optimized(n);
        }
        if self.source2_same_index {
            self.source2 = self.source2.take_optimized(n);
        }
        self
    }
}

impl<I, T, S1I, S1T, S2I, S2T> VecIteratorExtended
    for LazyVecFrom2Iterator<'_, I, T, S1I, S1T, S2I, S2T>
where
    I: StoredIndex,
    T: StoredRaw,
    S1I: StoredIndex,
    S1T: StoredRaw,
    S2I: StoredIndex,
    S2T: StoredRaw,
{
    type I = I;
    type T = T;
}
