use crate::{IterableVec, TypedVec, i64_to_usize};

use super::{AnyVec, StoredIndex, StoredRaw};

/// Trait for vectors that can be collected into standard Rust collections with range support.
pub trait CollectableVec<I, T>: IterableVec<I, T>
where
    Self: Clone,
    I: StoredIndex,
    T: StoredRaw,
{
    /// Returns an iterator over the specified range.
    fn iter_range(&self, from: Option<usize>, to: Option<usize>) -> impl Iterator<Item = T> {
        let len = self.len();
        let from = from.unwrap_or_default();
        let to = to.map_or(len, |to| to.min(len));
        let mut iter = self.iter();
        iter.set_end_to(to);
        iter.skip(from).take(to - from)
    }

    /// Returns an iterator over the specified range using signed indices (supports negative indexing).
    fn iter_signed_range(&self, from: Option<i64>, to: Option<i64>) -> impl Iterator<Item = T> {
        let from = from.map(|i| self.i64_to_usize(i));
        let to = to.map(|i| self.i64_to_usize(i));
        self.iter_range(from, to)
    }

    /// Collects all values into a Vec.
    fn collect(&self) -> Vec<T> {
        self.collect_range(None, None)
    }

    /// Collects values in the specified range into a Vec.
    fn collect_range(&self, from: Option<usize>, to: Option<usize>) -> Vec<T> {
        self.iter_range(from, to).collect::<Vec<_>>()
    }

    /// Collects values in the specified range into a Vec using signed indices.
    fn collect_signed_range(&self, from: Option<i64>, to: Option<i64>) -> Vec<T> {
        let from = from.map(|i| self.i64_to_usize(i));
        let to = to.map(|i| self.i64_to_usize(i));
        self.collect_range(from, to)
    }

    /// Collects values in the specified range as JSON bytes.
    #[inline]
    fn collect_range_json_bytes(&self, from: Option<usize>, to: Option<usize>) -> Vec<u8> {
        let vec = self.iter_range(from, to).collect::<Vec<_>>();
        let mut bytes = Vec::with_capacity(self.len() * 21);
        sonic_rs::to_writer(&mut bytes, &vec).unwrap();
        bytes
    }

    /// Collects values in the specified range as strings.
    #[inline]
    fn collect_range_string(&self, from: Option<usize>, to: Option<usize>) -> Vec<String> {
        self.iter_range(from, to).map(|v| v.to_string()).collect()
    }

    #[inline]
    fn iter_range_strings(
        &self,
        from: Option<i64>,
        to: Option<i64>,
    ) -> Box<dyn Iterator<Item = String> + '_> {
        let from_usize = from.map(|i| self.i64_to_usize(i));
        let to_usize = to.map(|i| self.i64_to_usize(i));
        Box::new(self.iter_range(from_usize, to_usize).map(|v| v.to_string()))
    }
}

impl<I, T, V> CollectableVec<I, T> for V
where
    V: IterableVec<I, T> + Clone,
    I: StoredIndex,
    T: StoredRaw,
{
}

/// Type-erased trait for collectable vectors.
pub trait AnyCollectableVec: AnyVec {
    fn collect_range_json_bytes(&self, from: Option<usize>, to: Option<usize>) -> Vec<u8>;
    fn collect_range_string(&self, from: Option<usize>, to: Option<usize>) -> Vec<String>;

    fn iter_range_strings(
        &self,
        from: Option<i64>,
        to: Option<i64>,
    ) -> Box<dyn Iterator<Item = String> + '_>;

    /// Returns the number of items in the specified range.
    fn range_count(&self, from: Option<i64>, to: Option<i64>) -> usize {
        let len = self.len();
        let from = from.map(|i| i64_to_usize(i, len));
        let to = to.map(|i| i64_to_usize(i, len));
        (from.unwrap_or_default()..to.unwrap_or(len)).count()
    }

    /// Returns the total size in bytes of items in the specified range.
    fn range_weight(&self, from: Option<i64>, to: Option<i64>) -> usize {
        self.range_count(from, to) * self.value_type_to_size_of()
    }
}

impl<V> AnyCollectableVec for V
where
    V: TypedVec,
    V: CollectableVec<V::I, V::T>,
{
    fn collect_range_json_bytes(&self, from: Option<usize>, to: Option<usize>) -> Vec<u8> {
        <Self as CollectableVec<V::I, V::T>>::collect_range_json_bytes(self, from, to)
    }

    fn collect_range_string(&self, from: Option<usize>, to: Option<usize>) -> Vec<String> {
        <Self as CollectableVec<V::I, V::T>>::collect_range_string(self, from, to)
    }

    fn iter_range_strings(
        &self,
        from: Option<i64>,
        to: Option<i64>,
    ) -> Box<dyn Iterator<Item = String> + '_> {
        <Self as CollectableVec<V::I, V::T>>::iter_range_strings(self, from, to)
    }
}
