use std::iter::FusedIterator;

use crate::{BoxedVecIterator, LazyVecFrom3, TypedVecIterator, VecIndex, VecIterator, VecValue};

pub struct LazyVecFrom3Iterator<'a, I, T, S1I, S1T, S2I, S2T, S3I, S3T>
where
    S1T: Clone,
    S2T: Clone,
    S3T: Clone,
{
    lazy: &'a LazyVecFrom3<I, T, S1I, S1T, S2I, S2T, S3I, S3T>,
    source1: BoxedVecIterator<'a, S1I, S1T>,
    source2: BoxedVecIterator<'a, S2I, S2T>,
    source3: BoxedVecIterator<'a, S3I, S3T>,
    source1_same_index: bool,
    source2_same_index: bool,
    source3_same_index: bool,
    index: usize,
    end_index: usize,
}

impl<'a, I, T, S1I, S1T, S2I, S2T, S3I, S3T>
    LazyVecFrom3Iterator<'a, I, T, S1I, S1T, S2I, S2T, S3I, S3T>
where
    I: VecIndex,
    T: VecValue + 'a,
    S1I: VecIndex,
    S1T: VecValue,
    S2I: VecIndex,
    S2T: VecValue,
    S3I: VecIndex,
    S3T: VecValue,
{
    #[inline]
    pub fn new(lazy: &'a LazyVecFrom3<I, T, S1I, S1T, S2I, S2T, S3I, S3T>) -> Self {
        let source1_same_index = lazy.source1.index_type_to_string() == I::to_string();
        let source2_same_index = lazy.source2.index_type_to_string() == I::to_string();
        let source3_same_index = lazy.source3.index_type_to_string() == I::to_string();

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
        let len3 = if source3_same_index {
            lazy.source3.len()
        } else {
            usize::MAX
        };
        let end_index = len1.min(len2).min(len3);

        LazyVecFrom3Iterator {
            lazy,
            source1: lazy.source1.iter(),
            source2: lazy.source2.iter(),
            source3: lazy.source3.iter(),
            source1_same_index,
            source2_same_index,
            source3_same_index,
            index: 0,
            end_index,
        }
    }
}

impl<'a, I, T, S1I, S1T, S2I, S2T, S3I, S3T> Iterator
    for LazyVecFrom3Iterator<'a, I, T, S1I, S1T, S2I, S2T, S3I, S3T>
where
    I: VecIndex,
    T: VecValue + 'a,
    S1I: VecIndex,
    S1T: VecValue,
    S2I: VecIndex,
    S2T: VecValue,
    S3I: VecIndex,
    S3T: VecValue,
{
    type Item = T;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.end_index {
            return None;
        }

        let index = I::from(self.index);
        let opt = (self.lazy.compute)(
            index,
            &mut *self.source1,
            &mut *self.source2,
            &mut *self.source3,
        );

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
        if self.source3_same_index {
            self.source3.nth(n - 1);
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

impl<I, T, S1I, S1T, S2I, S2T, S3I, S3T> VecIterator
    for LazyVecFrom3Iterator<'_, I, T, S1I, S1T, S2I, S2T, S3I, S3T>
where
    I: VecIndex,
    T: VecValue,
    S1I: VecIndex,
    S1T: VecValue,
    S2I: VecIndex,
    S2T: VecValue,
    S3I: VecIndex,
    S3T: VecValue,
{
    fn set_position_to(&mut self, i: usize) {
        self.index = i.min(self.end_index);
        if self.source1_same_index {
            self.source1.set_position_to(i);
        }
        if self.source2_same_index {
            self.source2.set_position_to(i);
        }
        if self.source3_same_index {
            self.source3.set_position_to(i);
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
        if self.source3_same_index {
            self.source3.set_end_to(i);
        }
    }

    #[inline]
    fn vec_len(&self) -> usize {
        if self.source1_same_index {
            self.source1.vec_len()
        } else if self.source2_same_index {
            self.source2.vec_len()
        } else if self.source3_same_index {
            self.source3.vec_len()
        } else {
            unreachable!()
        }
    }
}

impl<I, T, S1I, S1T, S2I, S2T, S3I, S3T> TypedVecIterator
    for LazyVecFrom3Iterator<'_, I, T, S1I, S1T, S2I, S2T, S3I, S3T>
where
    I: VecIndex,
    T: VecValue,
    S1I: VecIndex,
    S1T: VecValue,
    S2I: VecIndex,
    S2T: VecValue,
    S3I: VecIndex,
    S3T: VecValue,
{
    type I = I;
    type T = T;
}

impl<'a, I, T, S1I, S1T, S2I, S2T, S3I, S3T> ExactSizeIterator
    for LazyVecFrom3Iterator<'a, I, T, S1I, S1T, S2I, S2T, S3I, S3T>
where
    I: VecIndex,
    T: VecValue + 'a,
    S1I: VecIndex,
    S1T: VecValue,
    S2I: VecIndex,
    S2T: VecValue,
    S3I: VecIndex,
    S3T: VecValue,
{
    #[inline(always)]
    fn len(&self) -> usize {
        self.end_index.saturating_sub(self.index)
    }
}

impl<'a, I, T, S1I, S1T, S2I, S2T, S3I, S3T> FusedIterator
    for LazyVecFrom3Iterator<'a, I, T, S1I, S1T, S2I, S2T, S3I, S3T>
where
    I: VecIndex,
    T: VecValue + 'a,
    S1I: VecIndex,
    S1T: VecValue,
    S2I: VecIndex,
    S2T: VecValue,
    S3I: VecIndex,
    S3T: VecValue,
{
}
