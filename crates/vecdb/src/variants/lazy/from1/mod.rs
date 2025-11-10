use crate::{
    AnyVec, BoxedVecIterator, IterableBoxedVec, IterableVec, StoredIndex, StoredRaw, TypedVec,
    TypedVecIterator, Version,
};

mod iterator;

pub use iterator::*;

pub type ComputeFrom1<I, T, S1I, S1T> =
    for<'a> fn(I, &mut dyn TypedVecIterator<I = S1I, T = S1T, Item = S1T>) -> Option<T>;

/// Lazily computed vector deriving values from one source vector.
///
/// Values are computed on-the-fly during iteration using a provided function.
/// Nothing is stored on disk - all values are recomputed each time they're accessed.
#[derive(Clone)]
pub struct LazyVecFrom1<I, T, S1I, S1T>
where
    S1T: Clone,
{
    name: String,
    version: Version,
    source: IterableBoxedVec<S1I, S1T>,
    compute: ComputeFrom1<I, T, S1I, S1T>,
}

impl<I, T, S1I, S1T> LazyVecFrom1<I, T, S1I, S1T>
where
    I: StoredIndex,
    T: StoredRaw,
    S1I: StoredIndex,
    S1T: StoredRaw,
{
    pub fn init(
        name: &str,
        version: Version,
        source: IterableBoxedVec<S1I, S1T>,
        compute: ComputeFrom1<I, T, S1I, S1T>,
    ) -> Self {
        if I::to_string() != S1I::to_string() {
            unreachable!()
        }

        Self {
            name: name.to_string(),
            version,
            source,
            compute,
        }
    }

    fn version(&self) -> Version {
        self.version
    }
}

impl<'a, I, T, S1I, S1T> IntoIterator for &'a LazyVecFrom1<I, T, S1I, S1T>
where
    I: StoredIndex,
    T: StoredRaw + 'a,
    S1I: StoredIndex,
    S1T: StoredRaw,
{
    type Item = T;
    type IntoIter = LazyVecFrom1Iterator<'a, I, T, S1I, S1T>;

    fn into_iter(self) -> Self::IntoIter {
        LazyVecFrom1Iterator::new(self)
    }
}

impl<I, T, S1I, S1T> AnyVec for LazyVecFrom1<I, T, S1I, S1T>
where
    I: StoredIndex,
    T: StoredRaw,
    S1I: StoredIndex,
    S1T: StoredRaw,
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
        self.source.len()
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

impl<I, T, S1I, S1T> IterableVec<I, T> for LazyVecFrom1<I, T, S1I, S1T>
where
    I: StoredIndex,
    T: StoredRaw,
    S1I: StoredIndex,
    S1T: StoredRaw,
{
    fn iter(&self) -> BoxedVecIterator<'_, I, T> {
        Box::new(self.into_iter())
    }
}

impl<I, T, S1I, S1T> TypedVec for LazyVecFrom1<I, T, S1I, S1T>
where
    I: StoredIndex,
    T: StoredRaw,
    S1I: StoredIndex,
    S1T: StoredRaw,
{
    type I = I;
    type T = T;
}
