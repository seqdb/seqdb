use std::iter::FusedIterator;

use crate::{
    BoxedVecIterator, LazyVecFrom1, StoredIndex, StoredRaw, VecIterator, VecIteratorExtended,
};

pub struct LazyVecFrom1Iterator<'a, I, T, S1I, S1T>
where
    S1T: Clone,
{
    lazy: &'a LazyVecFrom1<I, T, S1I, S1T>,
    source: BoxedVecIterator<'a, S1I, S1T>,
    index: usize,
    end_index: usize,
}

impl<'a, I, T, S1I, S1T> LazyVecFrom1Iterator<'a, I, T, S1I, S1T>
where
    I: StoredIndex,
    T: StoredRaw + 'a,
    S1I: StoredIndex,
    S1T: StoredRaw,
{
    #[inline]
    pub fn new(lazy: &'a LazyVecFrom1<I, T, S1I, S1T>) -> Self {
        let len = lazy.source.len();
        LazyVecFrom1Iterator {
            lazy,
            source: lazy.source.iter(),
            index: 0,
            end_index: len,
        }
    }
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

    #[inline]
    fn nth(&mut self, n: usize) -> Option<T> {
        if n == 0 {
            return self.next();
        }

        let new_index = self.index.saturating_add(n);
        if new_index >= self.end_index {
            self.index = self.end_index;
            return None;
        }

        self.index = new_index;
        self.source.nth(n - 1);
        self.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.end_index.saturating_sub(self.index);
        (remaining, Some(remaining))
    }

    #[inline]
    fn count(self) -> usize {
        self.len()
    }

    #[inline]
    fn last(mut self) -> Option<T> {
        let last_index = self.end_index.checked_sub(1)?;
        if self.index > last_index {
            return None;
        }

        self.index = last_index;
        self.next()
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

impl<'a, I, T, S1I, S1T> ExactSizeIterator for LazyVecFrom1Iterator<'a, I, T, S1I, S1T>
where
    I: StoredIndex,
    T: StoredRaw + 'a,
    S1I: StoredIndex,
    S1T: StoredRaw,
{
    #[inline(always)]
    fn len(&self) -> usize {
        self.end_index.saturating_sub(self.index)
    }
}

impl<'a, I, T, S1I, S1T> FusedIterator for LazyVecFrom1Iterator<'a, I, T, S1I, S1T>
where
    I: StoredIndex,
    T: StoredRaw + 'a,
    S1I: StoredIndex,
    S1T: StoredRaw,
{
}
