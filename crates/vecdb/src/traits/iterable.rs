use std::collections::VecDeque;

use crate::{AnyStoredVec, AnyVec, BoxedVecIterator, VecIndex, VecValue, lookback::Lookback};

/// Trait for vectors that can be iterated.
pub trait IterableVec<I, T>: AnyVec {
    #[allow(clippy::wrong_self_convention)]
    fn iter(&self) -> BoxedVecIterator<'_, I, T>
    where
        I: VecIndex,
        T: VecValue;

    /// Create a windowed lookback for efficient windowed access.
    /// Uses a ring buffer if many items will be processed, otherwise uses direct access.
    fn create_lookback(&self, skip: usize, window: usize, min_start: usize) -> Lookback<'_, I, T>
    where
        I: VecIndex,
        T: VecValue,
    {
        let items_to_process = self.len().saturating_sub(skip);

        if items_to_process > window {
            // Use ring buffer - beneficial for many items
            let mut buf = VecDeque::with_capacity(window + 1);
            if skip > 0 {
                let start = skip.saturating_sub(window).max(min_start);
                self.iter().skip(start).take(skip - start).for_each(|v| {
                    buf.push_back(v);
                });
            }
            Lookback::Buffer { window, buf }
        } else {
            // Use direct access - beneficial for few items
            Lookback::DirectAccess {
                window,
                iter: self.iter(),
            }
        }
    }
}

/// Trait combining stored and iterable vector capabilities.
pub trait IterableStoredVec<I, T>: IterableVec<I, T> + AnyStoredVec {}

impl<I, T, U> IterableStoredVec<I, T> for U where U: 'static + IterableVec<I, T> + AnyStoredVec {}

/// Trait for iterable vectors that can be cloned as trait objects.
pub trait IterableCloneableVec<I, T>: IterableVec<I, T> {
    fn boxed_clone(&self) -> Box<dyn IterableCloneableVec<I, T>>;
}

impl<I, T, U> IterableCloneableVec<I, T> for U
where
    U: 'static + IterableVec<I, T> + Clone,
{
    fn boxed_clone(&self) -> Box<dyn IterableCloneableVec<I, T>> {
        Box::new(self.clone())
    }
}

impl<I, T> Clone for Box<dyn IterableCloneableVec<I, T>> {
    fn clone(&self) -> Self {
        self.boxed_clone()
    }
}

/// Type alias for boxed cloneable iterable vectors.
pub type IterableBoxedVec<I, T> = Box<dyn IterableCloneableVec<I, T>>;
