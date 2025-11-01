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
}

impl<'a, I, T, S1I, S1T> Iterator for LazyVecFrom1Iterator<'a, I, T, S1I, S1T>
where
    I: StoredIndex,
    T: StoredRaw + 'a,
    S1I: StoredIndex,
    S1T: StoredRaw,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
        // if self.index >= self.len() {
        //     return None;
        // }
        // let index = I::from(self.index);
        // let opt = (self.lazy.compute)(index, &mut *self.source).map(|v| (index, v));
        // if opt.is_some() {
        //     self.index += 1;
        // }
        // opt
    }
}

impl<I, T, S1I, S1T> VecIterator for LazyVecFrom1Iterator<'_, I, T, S1I, S1T>
where
    I: StoredIndex,
    T: StoredRaw,
    S1I: StoredIndex,
    S1T: StoredRaw,
{
    fn skip_optimized(self, _: usize) -> Self {
        todo!();
    }

    fn take_optimized(self, _: usize) -> Self {
        todo!();
    }

    // #[inline]
    // fn len(&self) -> usize {
    //     self.source.len()
    // }
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
