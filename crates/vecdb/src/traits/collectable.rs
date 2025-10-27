use crate::i64_to_usize;

use super::{AnyIterableVec, AnyVec, StoredIndex, StoredRaw};

pub trait CollectableVec<I, T>: AnyIterableVec<I, T>
where
    Self: Clone,
    I: StoredIndex,
    T: StoredRaw,
{
    fn iter_range(&self, from: Option<usize>, to: Option<usize>) -> impl Iterator<Item = T> {
        let len = self.len();
        let from = from.unwrap_or_default();
        let to = to.map_or(len, |to| to.min(len));
        self.iter_at_(from).take(to - from).map(|(_, v)| v)
    }

    fn iter_signed_range(&self, from: Option<i64>, to: Option<i64>) -> impl Iterator<Item = T> {
        let from = from.map(|i| self.i64_to_usize(i));
        let to = to.map(|i| self.i64_to_usize(i));
        self.iter_range(from, to)
    }

    fn collect(&self) -> Vec<T> {
        self.collect_range(None, None)
    }

    fn collect_range(&self, from: Option<usize>, to: Option<usize>) -> Vec<T> {
        self.iter_range(from, to).collect::<Vec<_>>()
    }

    fn collect_signed_range(&self, from: Option<i64>, to: Option<i64>) -> Vec<T> {
        let from = from.map(|i| self.i64_to_usize(i));
        let to = to.map(|i| self.i64_to_usize(i));
        self.collect_range(from, to)
    }

    #[inline]
    fn collect_range_json_bytes(&self, from: Option<usize>, to: Option<usize>) -> Vec<u8> {
        let vec = self.iter_range(from, to).collect::<Vec<_>>();
        let mut bytes = Vec::with_capacity(self.len() * 21);
        sonic_rs::to_writer(&mut bytes, &vec).unwrap();
        bytes
    }

    #[inline]
    fn collect_range_string(&self, from: Option<usize>, to: Option<usize>) -> Vec<String> {
        self.iter_range(from, to).map(|v| v.to_string()).collect()
    }

    #[inline]
    fn i64_to_usize_(i: i64, len: usize) -> usize {
        if i >= 0 {
            (i as usize).min(len)
        } else {
            let v = len as i64 + i;
            if v < 0 { 0 } else { v as usize }
        }
    }
}

impl<I, T, V> CollectableVec<I, T> for V
where
    V: AnyVec + AnyIterableVec<I, T> + Clone,
    I: StoredIndex,
    T: StoredRaw + 'static,
{
}

pub trait AnyCollectableVec: AnyVec {
    fn collect_range_json_bytes(&self, from: Option<usize>, to: Option<usize>) -> Vec<u8>;
    fn collect_range_string(&self, from: Option<usize>, to: Option<usize>) -> Vec<String>;

    fn range_count(&self, from: Option<i64>, to: Option<i64>) -> usize {
        let len = self.len();
        let from = from.map(|i| i64_to_usize(i, len));
        let to = to.map(|i| i64_to_usize(i, len));
        (from.unwrap_or_default()..to.unwrap_or(len)).count()
    }

    fn range_weight(&self, from: Option<i64>, to: Option<i64>) -> usize {
        self.range_count(from, to) * self.value_type_to_size_of()
    }
}
