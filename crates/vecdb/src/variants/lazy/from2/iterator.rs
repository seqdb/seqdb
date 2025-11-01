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
    pub(crate) index: usize,
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

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
        // let index = I::from(self.index);
        // let opt =
        //     (self.lazy.compute)(index, &mut *self.source1, &mut *self.source2).map(|v| (index, v));
        // if opt.is_some() {
        //     self.index += 1;
        // }
        // opt
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
    fn skip_optimized(self, _: usize) -> Self {
        todo!();
    }

    fn take_optimized(self, _: usize) -> Self {
        todo!();
    }

    // #[inline]
    // fn len(&self) -> usize {
    //     let len1 = if self.source1.index_type_to_string() == I::to_string() {
    //         self.source1.len()
    //     } else {
    //         usize::MAX
    //     };
    //     let len2 = if self.source2.index_type_to_string() == I::to_string() {
    //         self.source2.len()
    //     } else {
    //         usize::MAX
    //     };
    //     len1.min(len2)
    // }
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
