use std::iter::FusedIterator;

use crate::{
    BoxedVecIterator, LazyVecFrom2, StoredIndex, StoredRaw, VecIterator, VecIteratorExtended,
};

pub struct LazyVecFrom2Iterator<'a, I, T, S1I, S1T, S2I, S2T>
where
    S1T: Clone,
    S2T: Clone,
{
    lazy: &'a LazyVecFrom2<I, T, S1I, S1T, S2I, S2T>,
    source1: BoxedVecIterator<'a, S1I, S1T>,
    source2: BoxedVecIterator<'a, S2I, S2T>,
    source1_same_index: bool,
    source2_same_index: bool,
    index: usize,
    end_index: usize,
}

impl<'a, I, T, S1I, S1T, S2I, S2T> LazyVecFrom2Iterator<'a, I, T, S1I, S1T, S2I, S2T>
where
    I: StoredIndex,
    T: StoredRaw + 'a,
    S1I: StoredIndex,
    S1T: StoredRaw,
    S2I: StoredIndex,
    S2T: StoredRaw,
{
    #[inline]
    pub fn new(lazy: &'a LazyVecFrom2<I, T, S1I, S1T, S2I, S2T>) -> Self {
        let source1_same_index = lazy.source1.index_type_to_string() == I::to_string();
        let source2_same_index = lazy.source2.index_type_to_string() == I::to_string();

        let len1 = if source1_same_index {
            lazy.source1.len()
        } else {
            usize::MAX
        };
        let len2 = if source2_same_index {
            lazy.source2.len()
        } else {
            usize::MAX
        };
        let end_index = len1.min(len2);

        LazyVecFrom2Iterator {
            lazy,
            source1: lazy.source1.iter(),
            source2: lazy.source2.iter(),
            source1_same_index,
            source2_same_index,
            index: 0,
            end_index,
        }
    }
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
        if self.source1_same_index {
            self.source1.nth(n - 1);
        }
        if self.source2_same_index {
            self.source2.nth(n - 1);
        }
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

impl<I, T, S1I, S1T, S2I, S2T> VecIterator for LazyVecFrom2Iterator<'_, I, T, S1I, S1T, S2I, S2T>
where
    I: StoredIndex,
    T: StoredRaw,
    S1I: StoredIndex,
    S1T: StoredRaw,
    S2I: StoredIndex,
    S2T: StoredRaw,
{
    fn set_position_to(&mut self, i: usize) {
        self.index = i.min(self.end_index);
        if self.source1_same_index {
            self.source1.set_position_to(i);
        }
        if self.source2_same_index {
            self.source2.set_position_to(i);
        }
    }

    fn set_end_to(&mut self, i: usize) {
        self.end_index = i.min(self.end_index);
        if self.source1_same_index {
            self.source1.set_end_to(i);
        }
        if self.source2_same_index {
            self.source2.set_end_to(i);
        }
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

impl<'a, I, T, S1I, S1T, S2I, S2T> ExactSizeIterator
    for LazyVecFrom2Iterator<'a, I, T, S1I, S1T, S2I, S2T>
where
    I: StoredIndex,
    T: StoredRaw + 'a,
    S1I: StoredIndex,
    S1T: StoredRaw,
    S2I: StoredIndex,
    S2T: StoredRaw,
{
    #[inline(always)]
    fn len(&self) -> usize {
        self.end_index.saturating_sub(self.index)
    }
}

impl<'a, I, T, S1I, S1T, S2I, S2T> FusedIterator
    for LazyVecFrom2Iterator<'a, I, T, S1I, S1T, S2I, S2T>
where
    I: StoredIndex,
    T: StoredRaw + 'a,
    S1I: StoredIndex,
    S1T: StoredRaw,
    S2I: StoredIndex,
    S2T: StoredRaw,
{
}
