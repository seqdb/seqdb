use crate::{
    StoredCompressed, StoredIndex, VecIterator, VecIteratorExtended,
    variants::{CompressedVecIterator, RawVecIterator},
};

pub enum StoredVecIterator<'a, I, T> {
    Raw(RawVecIterator<'a, I, T>),
    Compressed(CompressedVecIterator<'a, I, T>),
}

impl<'a, I, T> Iterator for StoredVecIterator<'a, I, T>
where
    I: StoredIndex,
    T: StoredCompressed,
{
    type Item = T;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Compressed(i) => i.next(),
            Self::Raw(i) => i.next(),
        }
    }
}

impl<I, T> VecIterator for StoredVecIterator<'_, I, T>
where
    I: StoredIndex,
    T: StoredCompressed,
{
    fn skip_optimized(self, n: usize) -> Self {
        match self {
            Self::Compressed(i) => Self::Compressed(i.skip_optimized(n)),
            Self::Raw(i) => Self::Raw(i.skip_optimized(n)),
        }
    }

    fn take_optimized(self, n: usize) -> Self {
        match self {
            Self::Compressed(i) => Self::Compressed(i.take_optimized(n)),
            Self::Raw(i) => Self::Raw(i.take_optimized(n)),
        }
    }
}

impl<I, T> VecIteratorExtended for StoredVecIterator<'_, I, T>
where
    I: StoredIndex,
    T: StoredCompressed,
{
    type I = I;
    type T = T;
}
