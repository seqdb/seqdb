use crate::{
    StoredCompressed, StoredIndex, StoredRaw, VecIterator, VecIteratorExtended,
    variants::{
        LazyVecFrom1Iterator, LazyVecFrom2Iterator, LazyVecFrom3Iterator, StoredVecIterator,
    },
};

pub enum ComputedVecIterator<'a, I, T, S1I, S1T, S2I, S2T, S3I, S3T>
where
    S1T: Clone,
    S2T: Clone,
    S3T: Clone,
{
    Eager(StoredVecIterator<'a, I, T>),
    LazyFrom1(LazyVecFrom1Iterator<'a, I, T, S1I, S1T>),
    LazyFrom2(LazyVecFrom2Iterator<'a, I, T, S1I, S1T, S2I, S2T>),
    LazyFrom3(LazyVecFrom3Iterator<'a, I, T, S1I, S1T, S2I, S2T, S3I, S3T>),
}

impl<'a, I, T, S1I, S1T, S2I, S2T, S3I, S3T> Iterator
    for ComputedVecIterator<'a, I, T, S1I, S1T, S2I, S2T, S3I, S3T>
where
    I: StoredIndex,
    T: StoredCompressed,
    S1I: StoredIndex,
    S1T: StoredRaw,
    S2I: StoredIndex,
    S2T: StoredRaw,
    S3I: StoredIndex,
    S3T: StoredRaw,
{
    type Item = T;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        todo!()
        // match self {
        //     Self::Eager(i) => i.next(),
        //     Self::LazyFrom1(i) => i.next(),
        //     Self::LazyFrom2(i) => i.next(),
        //     Self::LazyFrom3(i) => i.next(),
        // }
    }

    #[inline(always)]
    fn nth(&mut self, _: usize) -> Option<Self::Item> {
        todo!()
        // match self {
        //     Self::Eager(i) => i.nth(),
        //     Self::LazyFrom1(i) => i.nth(),
        //     Self::LazyFrom2(i) => i.nth(),
        //     Self::LazyFrom3(i) => i.nth(),
        // }
    }
}

impl<I, T, S1I, S1T, S2I, S2T, S3I, S3T> VecIterator
    for ComputedVecIterator<'_, I, T, S1I, S1T, S2I, S2T, S3I, S3T>
where
    I: StoredIndex,
    T: StoredCompressed,
    S1I: StoredIndex,
    S1T: StoredRaw,
    S2I: StoredIndex,
    S2T: StoredRaw,
    S3I: StoredIndex,
    S3T: StoredRaw,
{
    fn set_position_(&mut self, i: usize) {
        todo!()
    }

    fn set_end_(&mut self, i: usize) {
        todo!()
    }

    fn skip_optimized(self, _: usize) -> Self {
        todo!();
    }

    fn take_optimized(self, _: usize) -> Self {
        todo!();
    }

    // fn len(&self) -> usize {
    //     match self {
    //         Self::Eager(i) => i.len(),
    //         Self::LazyFrom1(i) => i.len(),
    //         Self::LazyFrom2(i) => i.len(),
    //         Self::LazyFrom3(i) => i.len(),
    //     }
    // }
}

impl<I, T, S1I, S1T, S2I, S2T, S3I, S3T> VecIteratorExtended
    for ComputedVecIterator<'_, I, T, S1I, S1T, S2I, S2T, S3I, S3T>
where
    I: StoredIndex,
    T: StoredCompressed,
    S1I: StoredIndex,
    S1T: StoredRaw,
    S2I: StoredIndex,
    S2T: StoredRaw,
    S3I: StoredIndex,
    S3T: StoredRaw,
{
    type I = I;
    type T = T;
}
