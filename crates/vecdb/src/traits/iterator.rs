use std::iter::Skip;

use super::{PrintableIndex, StoredIndex, StoredRaw};

// Remove all ????

pub trait BaseVecIterator: Iterator {
    fn mut_index(&mut self) -> &mut usize;

    #[inline]
    fn set_(&mut self, i: usize) {
        *self.mut_index() = i;
    }

    #[inline]
    fn next_at(&mut self, i: usize) -> Option<Self::Item> {
        self.set_(i);
        self.next()
    }

    fn len(&self) -> usize;

    fn name(&self) -> &str;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn skip(self, _: usize) -> Skip<Self>
    where
        Self: Sized,
    {
        todo!("")
    }
}

pub trait VecIterator: BaseVecIterator<Item = (Self::I, Self::T)> {
    type I: StoredIndex;
    type T: StoredRaw;

    #[inline]
    fn set(&mut self, i: Self::I) {
        self.set_(i.to_usize())
    }

    #[inline]
    fn get_(&mut self, i: usize) -> Option<Self::T> {
        self.next_at(i).map(|(_, v)| v)
    }

    #[inline]
    fn get(&mut self, i: Self::I) -> Option<Self::T> {
        self.get_(i.to_usize())
    }

    #[inline]
    fn unwrap_get_inner(&mut self, i: Self::I) -> Self::T {
        self.unwrap_get_inner_(i.to_usize())
    }

    #[inline]
    fn unwrap_get_inner_(&mut self, i: usize) -> Self::T {
        self.get_(i).unwrap_or_else(|| {
            dbg!(self.name(), i, self.len(), std::any::type_name::<Self::I>());
            panic!("unwrap_get_inner_")
        })
    }

    #[inline]
    fn get_inner(&mut self, i: Self::I) -> Option<Self::T> {
        self.get_(i.to_usize())
    }

    fn last(mut self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        let len = self.len();
        if len == 0 {
            return None;
        }
        let i = len - 1;
        self.set_(i);
        self.next()
    }

    fn index_type_to_string(&self) -> &'static str {
        Self::I::to_string()
    }
}

impl<I, T, Iter> VecIterator for Iter
where
    Iter: BaseVecIterator<Item = (I, T)>,
    I: StoredIndex,
    T: StoredRaw,
{
    type I = I;
    type T = T;
}

pub type BoxedVecIterator<'a, I, T> = Box<dyn VecIterator<I = I, T = T, Item = (I, T)> + 'a>;
