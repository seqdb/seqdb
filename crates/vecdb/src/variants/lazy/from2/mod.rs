use allocative::Allocative;

use crate::{
    AnyBoxedIterableVec, AnyCollectableVec, AnyIterableVec, AnyVec, BoxedVecIterator,
    CollectableVec, StoredIndex, StoredRaw, VecIteratorExtended, Version,
};

mod iterator;

pub use iterator::*;

pub type ComputeFrom2<I, T, S1I, S1T, S2I, S2T> = for<'a> fn(
    I,
    &mut dyn VecIteratorExtended<I = S1I, T = S1T, Item = S1T>,
    &mut dyn VecIteratorExtended<I = S2I, T = S2T, Item = S2T>,
) -> Option<T>;

#[derive(Clone, Allocative)]
pub struct LazyVecFrom2<I, T, S1I, S1T, S2I, S2T>
where
    S1T: Clone,
    S2T: Clone,
{
    name: String,
    version: Version,
    #[allocative(skip)]
    source1: AnyBoxedIterableVec<S1I, S1T>,
    #[allocative(skip)]
    source2: AnyBoxedIterableVec<S2I, S2T>,
    #[allocative(skip)]
    compute: ComputeFrom2<I, T, S1I, S1T, S2I, S2T>,
}

impl<I, T, S1I, S1T, S2I, S2T> LazyVecFrom2<I, T, S1I, S1T, S2I, S2T>
where
    I: StoredIndex,
    T: StoredRaw,
    S1I: StoredIndex,
    S1T: StoredRaw,
    S2I: StoredIndex,
    S2T: StoredRaw,
{
    pub fn init(
        name: &str,
        version: Version,
        source1: AnyBoxedIterableVec<S1I, S1T>,
        source2: AnyBoxedIterableVec<S2I, S2T>,
        compute: ComputeFrom2<I, T, S1I, S1T, S2I, S2T>,
    ) -> Self {
        if ([
            source1.index_type_to_string(),
            source2.index_type_to_string(),
        ])
        .into_iter()
        .filter(|t| *t == I::to_string())
        .count()
            == 0
        {
            panic!("At least one should have same index");
        }

        Self {
            name: name.to_string(),
            version,
            source1,
            source2,
            compute,
        }
    }

    fn version(&self) -> Version {
        self.version
    }
}

impl<'a, I, T, S1I, S1T, S2I, S2T> IntoIterator for &'a LazyVecFrom2<I, T, S1I, S1T, S2I, S2T>
where
    I: StoredIndex,
    T: StoredRaw + 'a,
    S1I: StoredIndex,
    S1T: StoredRaw,
    S2I: StoredIndex,
    S2T: StoredRaw,
{
    type Item = T;
    type IntoIter = LazyVecFrom2Iterator<'a, I, T, S1I, S1T, S2I, S2T>;

    fn into_iter(self) -> Self::IntoIter {
        LazyVecFrom2Iterator::new(self)
    }
}

impl<I, T, S1I, S1T, S2I, S2T> AnyVec for LazyVecFrom2<I, T, S1I, S1T, S2I, S2T>
where
    I: StoredIndex,
    T: StoredRaw,
    S1I: StoredIndex,
    S1T: StoredRaw,
    S2I: StoredIndex,
    S2T: StoredRaw,
{
    fn version(&self) -> Version {
        self.version()
    }

    fn name(&self) -> &str {
        self.name.as_str()
    }

    fn index_type_to_string(&self) -> &'static str {
        I::to_string()
    }

    fn len(&self) -> usize {
        let len1 = if self.source1.index_type_to_string() == I::to_string() {
            self.source1.len()
        } else {
            usize::MAX
        };
        let len2 = if self.source2.index_type_to_string() == I::to_string() {
            self.source2.len()
        } else {
            usize::MAX
        };
        len1.min(len2)
    }

    #[inline]
    fn value_type_to_size_of(&self) -> usize {
        size_of::<T>()
    }

    #[inline]
    fn region_names(&self) -> Vec<String> {
        vec![]
    }
}

impl<I, T, S1I, S1T, S2I, S2T> AnyIterableVec<I, T> for LazyVecFrom2<I, T, S1I, S1T, S2I, S2T>
where
    I: StoredIndex,
    T: StoredRaw,
    S1I: StoredIndex,
    S1T: StoredRaw,
    S2I: StoredIndex,
    S2T: StoredRaw,
{
    fn boxed_iter(&self) -> BoxedVecIterator<'_, I, T> {
        Box::new(self.into_iter())
    }
}

impl<I, T, S1I, S1T, S2I, S2T> AnyCollectableVec for LazyVecFrom2<I, T, S1I, S1T, S2I, S2T>
where
    I: StoredIndex,
    T: StoredRaw,
    S1I: StoredIndex,
    S1T: StoredRaw,
    S2I: StoredIndex,
    S2T: StoredRaw,
{
    fn collect_range_json_bytes(&self, from: Option<usize>, to: Option<usize>) -> Vec<u8> {
        CollectableVec::collect_range_json_bytes(self, from, to)
    }

    fn collect_range_string(&self, from: Option<usize>, to: Option<usize>) -> Vec<String> {
        CollectableVec::collect_range_string(self, from, to)
    }
}
