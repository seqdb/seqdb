use std::borrow::Cow;

use allocative::Allocative;

use crate::{
    AnyBoxedIterableVec, AnyCollectableVec, AnyIterableVec, AnyVec, BaseVecIterator,
    BoxedVecIterator, CollectableVec, Result, StoredIndex, StoredRaw, Version,
};

pub type ComputeFrom3<I, T, S1I, S1T, S2I, S2T, S3I, S3T> = for<'a> fn(
    I,
    &mut dyn BaseVecIterator<Item = (S1I, Cow<'a, S1T>)>,
    &mut dyn BaseVecIterator<Item = (S2I, Cow<'a, S2T>)>,
    &mut dyn BaseVecIterator<Item = (S3I, Cow<'a, S3T>)>,
) -> Option<T>;

#[derive(Clone, Allocative)]
pub struct LazyVecFrom3<I, T, S1I, S1T, S2I, S2T, S3I, S3T>
where
    S1T: Clone,
    S2T: Clone,
    S3T: Clone,
{
    name: String,
    version: Version,
    #[allocative(skip)]
    source1: AnyBoxedIterableVec<S1I, S1T>,
    #[allocative(skip)]
    source2: AnyBoxedIterableVec<S2I, S2T>,
    #[allocative(skip)]
    source3: AnyBoxedIterableVec<S3I, S3T>,
    #[allocative(skip)]
    compute: ComputeFrom3<I, T, S1I, S1T, S2I, S2T, S3I, S3T>,
}

impl<I, T, S1I, S1T, S2I, S2T, S3I, S3T> LazyVecFrom3<I, T, S1I, S1T, S2I, S2T, S3I, S3T>
where
    I: StoredIndex,
    T: StoredRaw,
    S1I: StoredIndex,
    S1T: StoredRaw,
    S2I: StoredIndex,
    S2T: StoredRaw,
    S3I: StoredIndex,
    S3T: StoredRaw,
{
    pub fn init(
        name: &str,
        version: Version,
        source1: AnyBoxedIterableVec<S1I, S1T>,
        source2: AnyBoxedIterableVec<S2I, S2T>,
        source3: AnyBoxedIterableVec<S3I, S3T>,
        compute: ComputeFrom3<I, T, S1I, S1T, S2I, S2T, S3I, S3T>,
    ) -> Self {
        if ([
            source1.index_type_to_string(),
            source2.index_type_to_string(),
            source3.index_type_to_string(),
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
            source3,
            compute,
        }
    }

    fn version(&self) -> Version {
        self.version
    }
}

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
    index: usize,
}

impl<'a, I, T, S1I, S1T, S2I, S2T, S3I, S3T> Iterator
    for LazyVecFrom3Iterator<'a, I, T, S1I, S1T, S2I, S2T, S3I, S3T>
where
    I: StoredIndex,
    T: StoredRaw + 'a,
    S1I: StoredIndex,
    S1T: StoredRaw,
    S2I: StoredIndex,
    S2T: StoredRaw,
    S3I: StoredIndex,
    S3T: StoredRaw,
{
    type Item = (I, Cow<'a, T>);

    fn next(&mut self) -> Option<Self::Item> {
        let index = I::from(self.index);
        let opt = (self.lazy.compute)(
            index,
            &mut *self.source1,
            &mut *self.source2,
            &mut *self.source3,
        )
        .map(|v| (index, Cow::Owned(v)));
        if opt.is_some() {
            self.index += 1;
        }
        opt
    }
}

impl<I, T, S1I, S1T, S2I, S2T, S3I, S3T> BaseVecIterator
    for LazyVecFrom3Iterator<'_, I, T, S1I, S1T, S2I, S2T, S3I, S3T>
where
    I: StoredIndex,
    T: StoredRaw,
    S1I: StoredIndex,
    S1T: StoredRaw,
    S2I: StoredIndex,
    S2T: StoredRaw,
    S3I: StoredIndex,
    S3T: StoredRaw,
{
    #[inline]
    fn mut_index(&mut self) -> &mut usize {
        &mut self.index
    }

    #[inline]
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
        let len3 = if self.source3.index_type_to_string() == I::to_string() {
            self.source3.len()
        } else {
            usize::MAX
        };
        len1.min(len2).min(len3)
    }

    #[inline]
    fn name(&self) -> &str {
        self.source1.name()
    }
}

impl<'a, I, T, S1I, S1T, S2I, S2T, S3I, S3T> IntoIterator
    for &'a LazyVecFrom3<I, T, S1I, S1T, S2I, S2T, S3I, S3T>
where
    I: StoredIndex,
    T: StoredRaw + 'a,
    S1I: StoredIndex,
    S1T: StoredRaw,
    S2I: StoredIndex,
    S2T: StoredRaw,
    S3I: StoredIndex,
    S3T: StoredRaw,
{
    type Item = (I, Cow<'a, T>);
    type IntoIter = LazyVecFrom3Iterator<'a, I, T, S1I, S1T, S2I, S2T, S3I, S3T>;

    fn into_iter(self) -> Self::IntoIter {
        LazyVecFrom3Iterator {
            lazy: self,
            source1: self.source1.iter(),
            source2: self.source2.iter(),
            source3: self.source3.iter(),
            index: 0,
        }
    }
}

impl<I, T, S1I, S1T, S2I, S2T, S3I, S3T> AnyVec for LazyVecFrom3<I, T, S1I, S1T, S2I, S2T, S3I, S3T>
where
    I: StoredIndex,
    T: StoredRaw,
    S1I: StoredIndex,
    S1T: StoredRaw,
    S2I: StoredIndex,
    S2T: StoredRaw,
    S3I: StoredIndex,
    S3T: StoredRaw,
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
        let len3 = if self.source3.index_type_to_string() == I::to_string() {
            self.source3.len()
        } else {
            usize::MAX
        };
        len1.min(len2).min(len3)
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

impl<I, T, S1I, S1T, S2I, S2T, S3I, S3T> AnyIterableVec<I, T>
    for LazyVecFrom3<I, T, S1I, S1T, S2I, S2T, S3I, S3T>
where
    I: StoredIndex,
    T: StoredRaw,
    S1I: StoredIndex,
    S1T: StoredRaw,
    S2I: StoredIndex,
    S2T: StoredRaw,
    S3I: StoredIndex,
    S3T: StoredRaw,
{
    fn boxed_iter<'a>(&'a self) -> BoxedVecIterator<'a, I, T>
    where
        T: 'a,
    {
        Box::new(self.into_iter())
    }
}

impl<I, T, S1I, S1T, S2I, S2T, S3I, S3T> AnyCollectableVec
    for LazyVecFrom3<I, T, S1I, S1T, S2I, S2T, S3I, S3T>
where
    I: StoredIndex,
    T: StoredRaw,
    S1I: StoredIndex,
    S1T: StoredRaw,
    S2I: StoredIndex,
    S2T: StoredRaw,
    S3I: StoredIndex,
    S3T: StoredRaw,
{
    fn collect_range_json_bytes(&self, from: Option<usize>, to: Option<usize>) -> Result<Vec<u8>> {
        CollectableVec::collect_range_json_bytes(self, from, to)
    }

    fn collect_range_string(&self, from: Option<usize>, to: Option<usize>) -> Result<Vec<String>> {
        CollectableVec::collect_range_string(self, from, to)
    }
}
