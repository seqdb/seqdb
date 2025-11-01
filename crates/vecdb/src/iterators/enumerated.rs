use std::iter::FusedIterator;

use crate::VecIterator;

pub struct Enumerated<Iter> {
    iter: Iter,
    index: usize,
}

impl<Iter> Enumerated<Iter>
where
    Iter: Iterator,
{
    pub fn new(iter: Iter, index: usize) -> Self {
        Self { iter, index }
    }
}

impl<Iter> Iterator for Enumerated<Iter>
where
    Iter: Iterator,
{
    type Item = (usize, Iter::Item);

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        let value = self.iter.next()?;
        let index = self.index;
        self.index += 1;
        Some((index, value))
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.index += n;
        let value = self.iter.nth(n)?;
        let index = self.index;
        self.index += 1;
        Some((index, value))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    #[inline]
    fn count(self) -> usize {
        self.iter.count()
    }

    #[inline]
    fn last(self) -> Option<Self::Item> {
        let (min, Some(max)) = self.size_hint() else {
            panic!("expect size_hint to be valid");
        };
        if min != max {
            panic!("expect size_hint to be valid");
        }
        if min == 0 {
            return None;
        }
        let last_index = self.index + min - 1;
        self.iter.last().map(|item| (last_index, item))
    }
}

impl<Iter> ExactSizeIterator for Enumerated<Iter>
where
    Iter: ExactSizeIterator,
{
    #[inline]
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<Iter> FusedIterator for Enumerated<Iter> where Iter: FusedIterator {}

impl<Iter> VecIterator for Enumerated<Iter>
where
    Iter: VecIterator,
{
    fn skip_optimized(mut self, n: usize) -> Self {
        self.index += n;
        self.iter = self.iter.skip_optimized(n);
        self
    }

    fn take_optimized(mut self, n: usize) -> Self {
        self.iter = self.iter.take_optimized(n);
        self
    }
}
