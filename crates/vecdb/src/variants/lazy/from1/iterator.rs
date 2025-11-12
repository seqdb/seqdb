use std::iter::FusedIterator;

use crate::{BoxedVecIterator, LazyVecFrom1, TypedVecIterator, VecIndex, VecIterator, VecValue};

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
    I: VecIndex,
    T: VecValue + 'a,
    S1I: VecIndex,
    S1T: VecValue,
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
    I: VecIndex,
    T: VecValue + 'a,
    S1I: VecIndex,
    S1T: VecValue,
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
    I: VecIndex,
    T: VecValue,
    S1I: VecIndex,
    S1T: VecValue,
{
    #[inline]
    fn set_position_to(&mut self, i: usize) {
        self.index = i.min(self.end_index);
        self.source.set_position_to(i);
    }

    #[inline]
    fn set_end_to(&mut self, i: usize) {
        self.end_index = i.min(self.end_index);
        self.source.set_end_to(i);
    }

    #[inline]
    fn vec_len(&self) -> usize {
        self.source.vec_len()
    }
}

impl<I, T, S1I, S1T> TypedVecIterator for LazyVecFrom1Iterator<'_, I, T, S1I, S1T>
where
    I: VecIndex,
    T: VecValue,
    S1I: VecIndex,
    S1T: VecValue,
{
    type I = I;
    type T = T;
}

impl<'a, I, T, S1I, S1T> ExactSizeIterator for LazyVecFrom1Iterator<'a, I, T, S1I, S1T>
where
    I: VecIndex,
    T: VecValue + 'a,
    S1I: VecIndex,
    S1T: VecValue,
{
    #[inline(always)]
    fn len(&self) -> usize {
        self.end_index.saturating_sub(self.index)
    }
}

impl<'a, I, T, S1I, S1T> FusedIterator for LazyVecFrom1Iterator<'a, I, T, S1I, S1T>
where
    I: VecIndex,
    T: VecValue + 'a,
    S1I: VecIndex,
    S1T: VecValue,
{
}
