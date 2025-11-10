use std::marker::PhantomData;

use crate::{AnyCollectableVec, CollectableVec, TypedVec, ValueWriter, VecIteratorWriter};

pub trait AnyWritableVec: AnyCollectableVec {
    /// Create a value writer that can be advanced row by row
    fn create_writer(&self, from: Option<i64>, to: Option<i64>) -> Box<dyn ValueWriter + '_>;
}

impl<V> AnyWritableVec for V
where
    V: TypedVec,
    V: CollectableVec<V::I, V::T>,
{
    fn create_writer(&self, from: Option<i64>, to: Option<i64>) -> Box<dyn ValueWriter + '_> {
        let from_usize = from.map(|i| self.i64_to_usize(i));
        let to_usize = to.map(|i| self.i64_to_usize(i));

        Box::new(VecIteratorWriter {
            iter: Box::new(self.iter_range(from_usize, to_usize)),
            _phantom: PhantomData as PhantomData<V::T>,
        })
    }
}
